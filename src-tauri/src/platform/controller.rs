//! Continuous prototype orchestration. Buffers are memory-only and fail closed.
use super::{
    pipeline::{PassageGate, PassageStep},
    probe::{Probe, Ticket},
    session_devices::{CaptureStream, NativeDevices, SessionDevices, StopCapture},
};
use crate::adapters::openai_interpreter::OpenAiInterpreter;
#[cfg(test)]
use crate::audio::capture::NativeCapture;
use crate::audio::normalization::AudioFrame;
use crate::audio::openai_pcm::OpenAiPcm;
use crate::config::preferences::{ModelPreferences, Preferences, PreferencesStore};
use crate::ports::stt::SttEvent;
use crate::ports::{
    interpreter::{Interpreter, Interruption},
    stt::LiveStt,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Instant;
use tokio::sync::{mpsc, watch};
use tokio::time::{Duration, sleep};

struct Active {
    cancel: watch::Sender<bool>,
    capture: Option<Arc<dyn StopCapture>>,
    ticket: Ticket,
    opening_stop: Arc<AtomicBool>,
}

struct Finalized {
    passage_id: u64,
    text: String,
    speech_started_at_epoch_us: Option<u64>,
    speech_ended_at_epoch_us: Option<u64>,
}

#[derive(Clone)]
pub struct Controller {
    pub core: Arc<Mutex<Probe>>,
    active: Arc<Mutex<Option<Active>>>,
    preferences: Arc<Mutex<Result<PreferencesStore, &'static str>>>,
    frozen_models: Arc<Mutex<Option<ModelPreferences>>>,
}

impl Controller {
    pub fn new(core: Probe) -> Self {
        Self::with_preferences(core, Ok(PreferencesStore::memory()))
    }

    pub fn with_preferences(
        mut core: Probe,
        preferences: Result<PreferencesStore, &'static str>,
    ) -> Self {
        if preferences.as_ref().is_ok_and(|store| store.was_migrated()) {
            core.preferences_migrated();
        }
        Self {
            core: Arc::new(Mutex::new(core)),
            active: Arc::new(Mutex::new(None)),
            preferences: Arc::new(Mutex::new(preferences)),
            frozen_models: Arc::new(Mutex::new(None)),
        }
    }
    pub fn preferences(&self) -> Result<Preferences, &'static str> {
        Ok(self
            .preferences
            .lock()
            .map_err(|_| "Préférences indisponibles.")?
            .as_ref()
            .map_err(|e| *e)?
            .get())
    }
    pub fn set_models(&self, models: ModelPreferences) -> Result<Preferences, &'static str> {
        self.preferences
            .lock()
            .map_err(|_| "Préférences indisponibles.")?
            .as_mut()
            .map_err(|e| *e)?
            .set_models(models)
    }
    pub fn models_for_start(&self, resume: bool) -> Result<ModelPreferences, &'static str> {
        let mut frozen = self
            .frozen_models
            .lock()
            .map_err(|_| "Configuration de session indisponible.")?;
        if resume {
            if let Some(models) = &*frozen {
                return Ok(models.clone());
            }
        }
        let models = self
            .preferences
            .lock()
            .map_err(|_| "Préférences indisponibles.")?
            .as_ref()
            .map_err(|e| *e)?
            .session_models()?;
        *frozen = Some(models.clone());
        Ok(models)
    }
    pub fn remember_saved_note(&self) {
        let path = self
            .core
            .lock()
            .ok()
            .and_then(|core| core.saved_destination());
        if let Some(path) = path {
            let result = self
                .preferences
                .lock()
                .map_err(|_| "Préférences indisponibles.")
                .and_then(|mut store| store.as_mut().map_err(|e| *e)?.remember_note(&path));
            if result.is_err() {
                crate::diagnostics::event(
                    "Préférences : dernière destination non mémorisée ; note sauvegardée.",
                );
                if let Ok(mut core) = self.core.lock() {
                    core.message(
                        "Note sauvegardée ; mémorisation de la dernière destination impossible.",
                    );
                }
            }
        }
    }

    pub fn stop(&self, pause: bool, message: &str) {
        crate::diagnostics::event(if pause {
            "Commande : pause"
        } else {
            "Commande : arrêt/invalidation"
        });
        // Same lock order for start, device installation and stop.
        if let Ok(mut active) = self.active.lock() {
            if let Some(session) = active.take() {
                session.opening_stop.store(true, Ordering::Release);
                if let Some(capture) = session.capture {
                    capture.stop();
                }
                let _ = session.cancel.send(true);
            }
            if let Ok(mut core) = self.core.lock() {
                if pause {
                    core.pause();
                } else {
                    core.stop(message);
                }
            }
        }
    }

    fn stop_current(&self, ticket: &Ticket, pause: bool, message: &'static str, failure: bool) {
        if let Ok(mut active) = self.active.lock() {
            if let Ok(mut core) = self.core.lock() {
                if !core.same_session(ticket) {
                    return;
                }
                crate::diagnostics::event(message);
                if let Some(session) = active.take() {
                    session.opening_stop.store(true, Ordering::Release);
                    if let Some(capture) = session.capture {
                        capture.stop();
                    }
                    let _ = session.cancel.send(true);
                }
                if pause {
                    core.pause();
                } else {
                    core.stop(message);
                    if failure {
                        core.voice_error();
                    }
                }
            }
        }
    }

    fn finish_device(&self, ticket: &Ticket) {
        if let Ok(mut active) = self.active.lock() {
            if active
                .as_ref()
                .is_some_and(|a| a.ticket.same_generation(ticket))
            {
                if let Some(session) = active.take() {
                    session.opening_stop.store(true, Ordering::Release);
                    if let Some(capture) = session.capture {
                        capture.stop();
                    }
                    let _ = session.cancel.send(true);
                }
            }
        }
    }

    pub fn start_explicit(&self, resume: bool) -> Result<(), &'static str> {
        self.start_with_devices(resume, NativeDevices)
    }

    /// Rust-only dependency injection; the production IPC always selects NativeDevices.
    pub fn start_with_devices<D: SessionDevices>(
        &self,
        resume: bool,
        devices: D,
    ) -> Result<(), &'static str> {
        let mut active = self.active.lock().map_err(|_| "Contrôleur indisponible.")?;
        if active.is_some() {
            return Err("Session déjà active ou en initialisation.");
        }
        let models = self.models_for_start(resume)?;
        let mut core = self.core.lock().map_err(|_| "Cœur indisponible.")?;
        core.start_voice(resume)?;
        core.message(
            "Initialisation de la surveillance OS et d'OpenAI avant le microphone ; coût inconnu.",
        );
        let ticket = core.ticket();
        let (cancel, receiver) = watch::channel(false);
        let opening_stop = Arc::new(AtomicBool::new(false));
        *active = Some(Active {
            cancel,
            capture: None,
            ticket: ticket.clone(),
            opening_stop: opening_stop.clone(),
        });
        let this = self.clone();
        tauri::async_runtime::spawn(async move {
            let result = this
                .session(ticket.clone(), receiver, opening_stop, models, devices)
                .await;
            if let Err(message) = result {
                this.stop_current(&ticket, false, message, true);
            }
        });
        Ok(())
    }

    fn update(&self, ticket: &Ticket, f: impl FnOnce(&mut Probe)) {
        if let Ok(mut core) = self.core.lock() {
            if core.same_session(ticket) {
                f(&mut core);
            }
        }
    }

    async fn session<D: SessionDevices>(
        &self,
        ticket: Ticket,
        mut cancel: watch::Receiver<bool>,
        opening_stop: Arc<AtomicBool>,
        models: ModelPreferences,
        devices: D,
    ) -> Result<(), &'static str> {
        let os_controller = self.clone();
        let os_ticket = ticket.clone();
        let guard = devices.monitor(opening_stop.clone(), Arc::new(move || {
            os_controller.stop_current(&os_ticket, false,
                "Verrouillage, veille ou perte de surveillance OS : manuel, passages en attente non intégrés ; aucune auto-reprise.", false);
        })).await?;
        if *cancel.borrow() {
            return Ok(());
        }
        let has_key = devices.key_present().await?;
        if !has_key {
            return Err("Clé OpenAI absente du trousseau : vocal refusé avant capture.");
        }
        if *cancel.borrow() {
            return Ok(());
        }
        self.update(&ticket, |core| {
            core.measure("stt_configured_model", 0, &models.stt.model)
        });
        let transcription = tokio::select! {
            biased;
            _ = cancel.changed() => return Ok(()),
            result = devices.transcription(&models.stt) => result.map_err(|reason| {
                self.update(&ticket, |core| core.failure_stage("recognition"));
                reason
            })?,
        };
        if *cancel.borrow() {
            return Ok(());
        }
        self.update(&ticket, |core| {
            core.measure("stt_acknowledged_model", 0, &models.stt.model)
        });
        // Guard is established before CPAL can open a device.
        let capture = devices.capture(opening_stop).await?;
        {
            let mut active = self.active.lock().map_err(|_| "Contrôleur indisponible.")?;
            let mut core = self.core.lock().map_err(|_| "Cœur indisponible.")?;
            if !core.same_session(&ticket) {
                return Ok(());
            }
            active.as_mut().ok_or("Session invalidée.")?.capture = Some(capture.control());
            core.listening_ready();
        }
        self.update(&ticket, |core| {
            core.message(
                "Écoute — silence sans arrêt ; pause/arrêt naturels disponibles. Coût inconnu.",
            )
        });
        let (frames_tx, frames_rx) = mpsc::channel(128); // 3.84s + native queue; no silent drop.
        let (final_tx, final_rx) = mpsc::channel(4);
        let (priority_tx, priority_rx) = watch::channel(None);
        let work = async {
            tokio::try_join!(
                capture_frames(capture, frames_tx),
                async {
                    self.transcribe(&ticket, frames_rx, final_tx, priority_tx, transcription)
                        .await
                        .map_err(|reason| {
                            self.update(&ticket, |core| core.failure_stage("recognition"));
                            reason
                        })
                },
                self.interpret(&ticket, final_rx, &models.llm),
                async {
                    self.priority(&ticket, priority_rx, &models.llm)
                        .await
                        .map_err(|reason| {
                            self.update(&ticket, |core| core.failure_stage("interpretation"));
                            reason
                        })
                },
            )?;
            Ok(())
        };
        let result = self.supervise(&ticket, &mut cancel, work).await;
        drop(guard); // Joins monitor; dropping capture future stops and joins CPAL worker.
        result
    }

    async fn supervise(
        &self,
        ticket: &Ticket,
        cancel: &mut watch::Receiver<bool>,
        work: impl std::future::Future<Output = Result<(), &'static str>>,
    ) -> Result<(), &'static str> {
        let result = tokio::select! {
            biased;
            _ = cancel.changed() => Ok(()),
            result = work => result,
        };
        if let Err(message) = result {
            // Every worker error, including provider/network loss, shares this barrier.
            self.stop_current(ticket, false, message, true);
        }
        result
    }

    async fn transcribe<S: LiveStt>(
        &self,
        ticket: &Ticket,
        mut frames: mpsc::Receiver<AudioFrame>,
        finals: mpsc::Sender<Finalized>,
        priority: watch::Sender<Option<String>>,
        mut stream: S,
    ) -> Result<(), &'static str> {
        let mut gate = PassageGate::default();
        let mut converter = None;
        let mut passage_id = 0;
        let mut speech_started_at_epoch_us = None;
        loop {
            // Read events continuously while feeding audio; normal LLM work is independent.
            tokio::select! {
                event = stream.next_event() => {
                    match event? {
                        SttEvent::Final { .. } => return Err("Final STT prématuré ; passage non intégré."),
                        SttEvent::Provisional(text) => {
                            self.update(ticket, |c| c.transcript_delta(&text));
                            append_provisional(&priority, &text)?;
                        }
                        SttEvent::Other => {}
                    }
                }
                frame = frames.recv() => {
                    let frame = frame.ok_or("Capture interrompue ; passage non intégré.")?;
                    match gate.push(frame)? {
                        PassageStep::Wait => {}
                        PassageStep::Start { pcm, speech_started_at_epoch_us: started_at } => {
                            crate::diagnostics::event("STT : début de passage détecté par VAD");
                            passage_id += 1;
                            speech_started_at_epoch_us = started_at;
                            self.update(ticket, |c| c.speech_passage_started(passage_id, started_at));
                            priority.send_replace(Some(String::new()));
                            self.update(ticket, |c| c.measure("stt", 1, ""));
                            stream.begin_passage()?;
                            let mut resampler = OpenAiPcm::new()?;
                            let converted = resampler.push(&pcm)?;
                            if !converted.is_empty() {
                                stream.append_pcm(&converted).await?;
                            }
                            converter = Some(resampler);
                            self.update(ticket, |c| c.measure("audio", pcm.len() as u64 / 32, ""));
                        }
                        PassageStep::Append(pcm) => {
                            let converted = converter.as_mut().ok_or("Passage audio absent.")?.push(&pcm)?;
                            if !converted.is_empty() { stream.append_pcm(&converted).await?; }
                            self.update(ticket, |c| c.measure("audio", 30, ""));
                        }
                        PassageStep::End { pcm, speech_ended_at_epoch_us } => {
                            crate::diagnostics::event("STT : fin de passage, attente final OpenAI");
                            self.update(ticket, |c| c.speech_passage_ended(passage_id, speech_ended_at_epoch_us));
                            let resampler = converter.as_mut().ok_or("Passage audio absent.")?;
                            let converted = resampler.push(&pcm)?;
                            if !converted.is_empty() { stream.append_pcm(&converted).await?; }
                            let tail = resampler.finish()?;
                            if !tail.is_empty() { stream.append_pcm(&tail).await?; }
                            self.update(ticket, |c| c.measure("audio", 30, ""));
                            let finalize_started = Instant::now();
                            stream.end_passage().await?;
                            // CPAL continues into the bounded queue during finalization/rotation.
                            loop {
                                match stream.next_event().await? {
                                    SttEvent::Final { text, model } => {
                                        crate::diagnostics::numbers("STT : final OpenAI reçu (contenu masqué)", &[("text_bytes", text.len() as u64)]);
                                        if let Some(served) = &model.served {
                                            self.update(ticket, |c| c.measure("stt_model", 0, served));
                                        }
                                        self.update(ticket, |c| c.measure("stt_final_ms", finalize_started.elapsed().as_millis() as u64, ""));
                                        self.update(ticket, |c| c.transcript_done(&text));
                                        finals.try_send(Finalized {
                                            passage_id,
                                            text,
                                            speech_started_at_epoch_us,
                                            speech_ended_at_epoch_us,
                                        }).map_err(|_| "Saturation interprétation : passages non intégrés, arrêt sans spool.")?;
                                        break;
                                    }
                                    SttEvent::Provisional(text) => {
                                        self.update(ticket, |c| c.transcript_delta(&text));
                                        append_provisional(&priority, &text)?;
                                    },
                                    SttEvent::Other => {}
                                }
                            }
                            converter = None;
                            priority.send_replace(None);
                        }
                    }
                }
            }
        }
    }

    async fn interpret(
        &self,
        session: &Ticket,
        mut finals: mpsc::Receiver<Finalized>,
        selection: &crate::config::preferences::ServiceSelection,
    ) -> Result<(), &'static str> {
        let interpreter = OpenAiInterpreter::selected(selection)?;
        let mut sequence = 0;
        while let Some(Finalized {
            passage_id,
            text,
            speech_started_at_epoch_us,
            speech_ended_at_epoch_us,
        }) = finals.recv().await
        {
            if text.trim().is_empty() {
                crate::diagnostics::event("STT : final vide ignoré, aucun appel LLM ordinaire");
                continue;
            }
            let request = {
                let mut core = self.core.lock().map_err(|_| "Cœur indisponible.")?;
                if !core.same_session(session) {
                    return Ok(());
                }
                sequence += 1;
                let request =
                    crate::app::drafting::DraftingRequest::prepare(&mut core, sequence, passage_id)
                        .map_err(|stage| {
                            core.failure_stage("application");
                            stage.message()
                        })?;
                core.measure("llm", 1, "");
                core.processing();
                core.message("Traitement du passage finalisé — aucune édition sur provisoire.");
                request
            };
            let llm_started = Instant::now();
            let reply = request
                .interpret(&interpreter, &text)
                .await
                .map_err(|stage| {
                    self.update(session, |core| {
                        core.failure_stage(match stage {
                            crate::app::drafting::FailureStage::Recognition => "recognition",
                            crate::app::drafting::FailureStage::Interpretation => "interpretation",
                            crate::app::drafting::FailureStage::Application => "application",
                        })
                    });
                    stage.message()
                })?;
            let terminal = {
                let mut core = self.core.lock().map_err(|_| "Cœur indisponible.")?;
                if !core.same_session(session) {
                    return Ok(());
                }
                core.measure("llm_model", 0, &reply.model);
                core.measure("llm_ms", llm_started.elapsed().as_millis() as u64, "");
                let before = core.snapshot();
                let commit_started = Instant::now();
                match reply.commit(&mut core) {
                    Ok(()) => {
                        if core.snapshot().source != before.source {
                            core.speech_passage_applied(
                                passage_id,
                                speech_started_at_epoch_us,
                                speech_ended_at_epoch_us,
                            );
                        }
                        if core.snapshot().message == before.message {
                            core.message("Passage finalisé traité ; écoute continue.");
                        }
                    }
                    Err(failure) => {
                        core.failure_stage("application");
                        core.message(failure.reason);
                    }
                }
                let mode = core.snapshot().mode;
                core.measure("commit_us", commit_started.elapsed().as_micros() as u64, "");
                (mode != "voice").then_some(mode == "paused")
            };
            self.remember_saved_note();
            if terminal.is_some() {
                // Core already invalidated; physical capture/cancellation must still stop.
                self.finish_device(session);
                return Ok(());
            }
        }
        Err("Interprétation interrompue.")
    }

    async fn priority(
        &self,
        ticket: &Ticket,
        mut receiver: watch::Receiver<Option<String>>,
        selection: &crate::config::preferences::ServiceSelection,
    ) -> Result<(), &'static str> {
        let interpreter = OpenAiInterpreter::selected(selection)?;
        loop {
            receiver
                .changed()
                .await
                .map_err(|_| "Voie prioritaire interrompue.")?;
            // Coalesce latest provisional: one in flight, no backlog, no keyword gating.
            sleep(Duration::from_millis(750)).await;
            let text = receiver.borrow_and_update().clone();
            let Some(text) = text.filter(|s| !s.trim().is_empty()) else {
                continue;
            };
            self.update(ticket, |c| c.measure("priority", 1, ""));
            let priority_started = Instant::now();
            let action = Interpreter::interpret_interruption(&interpreter, &text).await?;
            crate::diagnostics::event(match action {
                Interruption::None => "Priorité : aucune interruption demandée",
                Interruption::Pause => "Priorité : pause demandée",
                Interruption::Stop => "Priorité : arrêt demandé",
                Interruption::Manual => "Priorité : manuel demandé",
            });
            self.update(ticket, |c| {
                c.measure(
                    "priority_ms",
                    priority_started.elapsed().as_millis() as u64,
                    "",
                )
            });
            if action != Interruption::None {
                self.stop_current(ticket, action == Interruption::Pause,
                    "Interruption prioritaire ; micro arrêté, passages en attente non intégrés, reprise explicite.", false);
                return Ok(());
            }
        }
    }
}

async fn capture_frames(
    capture: Box<dyn CaptureStream>,
    frames: mpsc::Sender<AudioFrame>,
) -> Result<(), &'static str> {
    loop {
        while let Some(frame) = capture.next_frame()? {
            frames.try_send(frame).map_err(
                |_| "Saturation rotation audio : arrêt, passages non intégrés ; aucun spool.",
            )?;
        }
        sleep(Duration::from_millis(5)).await;
    }
}

fn append_provisional(
    sender: &watch::Sender<Option<String>>,
    delta: &str,
) -> Result<(), &'static str> {
    let mut text = sender.borrow().clone().unwrap_or_default();
    if text.len() + delta.len() > 4096 {
        return Err("Limite du provisoire atteinte ; arrêt explicite sans édition partielle.");
    }
    text.push_str(delta);
    sender.send_replace(Some(text));
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    fn active(controller: &Controller) -> (Ticket, watch::Receiver<bool>, Arc<AtomicBool>) {
        let mut core = controller.core.lock().unwrap();
        core.start_voice(false).unwrap();
        let ticket = core.ticket();
        let (cancel, receiver) = watch::channel(false);
        let flag = Arc::new(AtomicBool::new(false));
        *controller.active.lock().unwrap() = Some(Active {
            cancel,
            capture: None,
            ticket: ticket.clone(),
            opening_stop: flag.clone(),
        });
        (ticket, receiver, flag)
    }

    #[test]
    fn stop_during_initialization_cancels_device_before_it_can_accept_audio() {
        let controller = Controller::new(Probe::new("synthétique".into()));
        let (ticket, receiver, flag) = active(&controller);
        controller.stop(false, "arrêt");
        assert!(*receiver.borrow());
        assert!(flag.load(Ordering::Acquire));
        assert!(!controller.core.lock().unwrap().same_session(&ticket));
        // Does not enumerate or open any device.
        assert!(NativeCapture::start_guarded(flag).is_err());
    }

    #[test]
    fn late_failure_and_terminal_cleanup_cannot_stop_new_session() {
        let controller = Controller::new(Probe::new("synthétique".into()));
        let (old, _, _) = active(&controller);
        controller.stop(false, "arrêt");
        let (new, receiver, flag) = active(&controller);
        controller.stop_current(&old, false, "erreur tardive", true);
        controller.finish_device(&old);
        assert!(!*receiver.borrow());
        assert!(!flag.load(Ordering::Acquire));
        assert!(controller.core.lock().unwrap().same_session(&new));
        controller.stop(false, "fin du test");
    }

    #[test]
    fn priority_coalesces_without_a_replay_queue_and_bounds_provisional() {
        let (sender, receiver) = watch::channel(Some(String::new()));
        append_provisional(&sender, "mets en ").unwrap();
        append_provisional(&sender, "pause").unwrap();
        assert_eq!(receiver.borrow().as_deref(), Some("mets en pause"));
        assert!(append_provisional(&sender, &"x".repeat(4096)).is_err());
        sender.send_replace(None);
        assert!(receiver.borrow().is_none());
    }
    #[tokio::test]
    async fn service_failure_supervisor_invalidates_and_cancellation_drops_pending_work() {
        let controller = Controller::new(Probe::new("note".into()));
        let (ticket, mut receiver, flag) = active(&controller);
        assert!(
            controller
                .supervise(&ticket, &mut receiver, async {
                    Err("Service indisponible : passage non intégré.")
                })
                .await
                .is_err()
        );
        assert!(flag.load(Ordering::Acquire));
        assert_eq!(
            controller.core.lock().unwrap().snapshot().voice_activity,
            "error"
        );
        assert_eq!(controller.core.lock().unwrap().snapshot().mode, "manual");
        let (ticket, mut receiver, _) = active(&controller);
        controller.stop(true, "pause");
        controller
            .supervise(&ticket, &mut receiver, std::future::pending())
            .await
            .unwrap();
        assert_eq!(controller.core.lock().unwrap().snapshot().mode, "paused");
    }

    struct Stream {
        events: mpsc::Receiver<SttEvent>,
        sender: mpsc::Sender<SttEvent>,
        starts: Arc<std::sync::atomic::AtomicUsize>,
    }
    impl crate::ports::stt::SttInput for Stream {
        async fn append_pcm(&mut self, _: &[u8]) -> Result<(), &'static str> {
            self.sender
                .try_send(SttEvent::Provisional("aperçu".into()))
                .map_err(|_| "test queue full")
        }
        async fn end_passage(&mut self) -> Result<(), &'static str> {
            self.sender
                .try_send(SttEvent::Final {
                    text: "Texte final.".into(),
                    model: crate::ports::stt::ModelProvenance {
                        configured: Some("gpt-realtime-whisper".into()),
                        acknowledged: Some("gpt-realtime-whisper".into()),
                        served: None,
                    },
                })
                .map_err(|_| "test queue full")
        }
    }
    impl crate::ports::stt::SttOutput for Stream {
        async fn next_event(&mut self) -> Result<SttEvent, &'static str> {
            self.events.recv().await.ok_or("closed")
        }
    }
    impl LiveStt for Stream {
        fn begin_passage(&mut self) -> Result<(), &'static str> {
            self.starts.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn live_passages_deliver_finals_without_editing_note_or_rotating_connection() {
        let c = Controller::new(Probe::new("note originale".into()));
        let (ticket, _, _) = active(&c);
        let (events_tx, events_rx) = mpsc::channel(16);
        let starts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let stream = Stream {
            events: events_rx,
            sender: events_tx,
            starts: starts.clone(),
        };
        let (frames_tx, frames_rx) = mpsc::channel(128);
        let (final_tx, mut final_rx) = mpsc::channel(4);
        let (priority_tx, _) = watch::channel(None);
        for _ in 0..2 {
            frames_tx
                .send(AudioFrame {
                    pcm: vec![0; 960],
                    voiced: true,
                    end_passage: false,
                    capture_start_epoch_us: Some(1_000_000),
                })
                .await
                .unwrap();
            frames_tx
                .send(AudioFrame {
                    pcm: vec![0; 960],
                    voiced: false,
                    end_passage: true,
                    capture_start_epoch_us: Some(1_030_000),
                })
                .await
                .unwrap();
        }
        let worker = c.clone();
        let handle = tokio::spawn(async move {
            worker
                .transcribe(&ticket, frames_rx, final_tx, priority_tx, stream)
                .await
        });
        for id in 1..=2 {
            let final_text = tokio::time::timeout(Duration::from_secs(2), final_rx.recv())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(final_text.passage_id, id);
            assert_eq!(final_text.text, "Texte final.");
        }
        assert_eq!(starts.load(Ordering::SeqCst), 2);
        let snapshot = c.core.lock().unwrap().snapshot();
        assert_eq!(snapshot.source, "note originale");
        assert_eq!(snapshot.transcript_passage_id, 2);
        assert_eq!(snapshot.speech_started_at_epoch_us, Some(970_000));
        assert_eq!(snapshot.speech_ended_at_epoch_us, Some(1_000_000));
        handle.abort();
        let _ = handle.await;
    }

    struct OverflowCapture;
    impl StopCapture for OverflowCapture {
        fn stop(&self) {}
    }
    impl CaptureStream for OverflowCapture {
        fn control(&self) -> Arc<dyn StopCapture> {
            Arc::new(OverflowCapture)
        }
        fn next_frame(&self) -> Result<Option<AudioFrame>, &'static str> {
            Ok(Some(AudioFrame {
                pcm: vec![0; 960],
                voiced: true,
                end_passage: false,
                capture_start_epoch_us: None,
            }))
        }
    }
    #[tokio::test]
    async fn capture_queue_saturation_is_an_error_not_an_overwrite_or_spool() {
        let (sender, mut receiver) = mpsc::channel(128);
        let result = capture_frames(Box::new(OverflowCapture), sender).await;
        assert!(result.err().unwrap().contains("Saturation"));
        let mut count = 0;
        while receiver.try_recv().is_ok() {
            count += 1;
        }
        assert_eq!(count, 128);
    }

    #[tokio::test]
    async fn final_queue_saturation_preserves_note_and_rejects_fifth_passage() {
        let c = Controller::new(Probe::new("note originale".into()));
        let (ticket, _, _) = active(&c);
        let (sender, events) = mpsc::channel(16);
        let stream = Stream {
            events,
            sender,
            starts: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        };
        let (frames_tx, frames_rx) = mpsc::channel(128);
        let (final_tx, mut finals) = mpsc::channel(4);
        let (priority_tx, _) = watch::channel(None);
        for _ in 0..5 {
            frames_tx
                .send(AudioFrame {
                    pcm: vec![0; 960],
                    voiced: true,
                    end_passage: false,
                    capture_start_epoch_us: Some(1_000_000),
                })
                .await
                .unwrap();
            frames_tx
                .send(AudioFrame {
                    pcm: vec![0; 960],
                    voiced: false,
                    end_passage: true,
                    capture_start_epoch_us: Some(1_030_000),
                })
                .await
                .unwrap();
        }
        let result = tokio::time::timeout(
            Duration::from_secs(2),
            c.transcribe(&ticket, frames_rx, final_tx, priority_tx, stream),
        )
        .await
        .unwrap();
        assert!(result.err().unwrap().contains("Saturation interprétation"));
        let mut count = 0;
        while finals.try_recv().is_ok() {
            count += 1;
        }
        assert_eq!(count, 4);
        assert_eq!(c.core.lock().unwrap().snapshot().source, "note originale");
    }
}

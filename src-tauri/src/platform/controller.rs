//! Continuous prototype orchestration. Buffers are memory-only and fail closed.
use super::{
    audio::AudioFrame,
    capture::{CaptureControl, NativeCapture},
    lifecycle::LifecycleGuard,
    mistral::{MistralInterpreter, MistralStt, SttEvent, SttReader},
    pipeline::{PassageGate, PassageStep},
    probe::{Probe, Ticket},
};
use crate::ports::{
    interpreter::{InterpretationContext, Interpreter, Interruption},
    stt::{SttInput, SttOutput},
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
    capture: Option<CaptureControl>,
    ticket: Ticket,
    opening_stop: Arc<AtomicBool>,
}

struct Finalized {
    passage_id: u64,
    text: String,
}

#[derive(Clone)]
pub struct Controller {
    pub core: Arc<Mutex<Probe>>,
    active: Arc<Mutex<Option<Active>>>,
}

impl Controller {
    pub fn new(core: Probe) -> Self {
        Self {
            core: Arc::new(Mutex::new(core)),
            active: Arc::new(Mutex::new(None)),
        }
    }

    pub fn stop(&self, pause: bool, message: &str) {
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

    fn stop_current(&self, ticket: &Ticket, pause: bool, message: &str) {
        if let Ok(mut active) = self.active.lock() {
            if let Ok(mut core) = self.core.lock() {
                if !core.same_session(ticket) {
                    return;
                }
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
        let mut active = self.active.lock().map_err(|_| "Contrôleur indisponible.")?;
        if active.is_some() {
            return Err("Session déjà active ou en initialisation.");
        }
        let mut core = self.core.lock().map_err(|_| "Cœur indisponible.")?;
        core.start_voice(resume)?;
        core.message(
            "Initialisation de la surveillance OS puis du microphone ; coût Mistral inconnu.",
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
            let result = this.session(ticket.clone(), receiver, opening_stop).await;
            if let Err(message) = result {
                this.stop_current(&ticket, false, message);
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

    async fn session(
        &self,
        ticket: Ticket,
        mut cancel: watch::Receiver<bool>,
        opening_stop: Arc<AtomicBool>,
    ) -> Result<(), &'static str> {
        let os_controller = self.clone();
        let os_ticket = ticket.clone();
        let guard = tokio::task::spawn_blocking(move || LifecycleGuard::start(move || {
            os_controller.stop_current(&os_ticket, false,
                "Verrouillage, veille ou perte de surveillance OS : manuel, passages en attente non intégrés ; aucune auto-reprise.");
        })).await.map_err(|_| "Surveillance OS indisponible.")??;
        if *cancel.borrow() {
            return Ok(());
        }
        let has_key = tokio::task::spawn_blocking(super::secrets::presence)
            .await
            .map_err(|_| "Trousseau indisponible : vocal refusé.")??;
        if !has_key {
            return Err("Clé Mistral absente du trousseau : vocal refusé avant capture.");
        }
        if *cancel.borrow() {
            return Ok(());
        }
        // Guard is established before CPAL can open a device.
        let capture =
            tokio::task::spawn_blocking(move || NativeCapture::start_guarded(opening_stop))
                .await
                .map_err(|_| "Microphone indisponible.")??;
        {
            let mut active = self.active.lock().map_err(|_| "Contrôleur indisponible.")?;
            let core = self.core.lock().map_err(|_| "Cœur indisponible.")?;
            if !core.same_session(&ticket) {
                return Ok(());
            }
            active.as_mut().ok_or("Session invalidée.")?.capture = Some(capture.control());
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
                self.transcribe(&ticket, frames_rx, final_tx, priority_tx),
                self.interpret(&ticket, final_rx),
                self.priority(&ticket, priority_rx),
            )?;
            Ok(())
        };
        let result = tokio::select! {
            biased;
            _ = cancel.changed() => Ok(()),
            result = work => result,
        };
        drop(guard); // Joins monitor; dropping capture future stops and joins CPAL worker.
        result
    }

    async fn transcribe(
        &self,
        ticket: &Ticket,
        mut frames: mpsc::Receiver<AudioFrame>,
        finals: mpsc::Sender<Finalized>,
        priority: watch::Sender<Option<String>>,
    ) -> Result<(), &'static str> {
        let mut gate = PassageGate::default();
        let mut writer = None;
        let mut reader = None;
        let mut passage_id = 0;
        loop {
            // Read events continuously while feeding audio; normal LLM work is independent.
            tokio::select! {
                event = read_event(&mut reader), if reader.is_some() => {
                    match event? {
                        SttEvent::Final { .. } => return Err("Final STT prématuré ; passage non intégré."),
                        SttEvent::Provisional(text) => {
                            append_provisional(&priority, &text)?;
                        }
                        SttEvent::Other => {}
                    }
                }
                frame = frames.recv() => {
                    let frame = frame.ok_or("Capture interrompue ; passage non intégré.")?;
                    match gate.push(frame)? {
                        PassageStep::Wait => {}
                        PassageStep::Start(pcm) => {
                            passage_id += 1;
                            priority.send_replace(Some(String::new()));
                            self.update(ticket, |c| c.measure("stt", 1, ""));
                            let (w, r) = MistralStt::connect_explicit().await?.into_parts();
                            writer = Some(w); reader = Some(r);
                            SttInput::append_pcm(writer.as_mut().unwrap(), &pcm).await?;
                            self.update(ticket, |c| c.measure("audio", pcm.len() as u64 / 32, ""));
                        }
                        PassageStep::Append(pcm) => {
                            SttInput::append_pcm(writer.as_mut().ok_or("Passage STT absent.")?, &pcm).await?;
                            self.update(ticket, |c| c.measure("audio", 30, ""));
                        }
                        PassageStep::End(pcm) => {
                            let w = writer.as_mut().ok_or("Passage STT absent.")?;
                            SttInput::append_pcm(w, &pcm).await?;
                            self.update(ticket, |c| c.measure("audio", 30, ""));
                            let finalize_started = Instant::now();
                            SttInput::end_passage(w).await?;
                            // CPAL continues into the bounded queue during finalization/rotation.
                            loop {
                                match read_event(&mut reader).await? {
                                    SttEvent::Final { text, model } => {
                                        self.update(ticket, |c| c.measure("stt_model", 0, &model));
                                        self.update(ticket, |c| c.measure("stt_final_ms", finalize_started.elapsed().as_millis() as u64, ""));
                                        finals.try_send(Finalized { passage_id, text }).map_err(|_| "Saturation interprétation : passages non intégrés, arrêt sans spool.")?;
                                        break;
                                    }
                                    SttEvent::Provisional(text) => append_provisional(&priority, &text)?,
                                    SttEvent::Other => {}
                                }
                            }
                            writer = None; reader = None;
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
    ) -> Result<(), &'static str> {
        let interpreter = MistralInterpreter;
        let mut sequence = 0;
        while let Some(Finalized { passage_id, text }) = finals.recv().await {
            if text.trim().is_empty() {
                continue;
            }
            let (ticket, intention, candidates) = {
                let mut core = self.core.lock().map_err(|_| "Cœur indisponible.")?;
                if !core.same_session(session) {
                    return Ok(());
                }
                sequence += 1;
                let ticket = core.voice_ticket_for(sequence, passage_id)?;
                let candidates = core.candidates();
                core.measure("llm", 1, "");
                core.message("Traitement du passage finalisé — aucune édition sur provisoire.");
                (
                    ticket,
                    core.intention().to_owned(),
                    InterpretationContext {
                        blocks: candidates,
                        save_available: core.can_save(),
                    },
                )
            };
            let llm_started = Instant::now();
            let (response, model) =
                Interpreter::interpret_final(&interpreter, &text, &intention, candidates).await?;
            let terminal = {
                let mut core = self.core.lock().map_err(|_| "Cœur indisponible.")?;
                if !core.same_session(session) {
                    return Ok(());
                }
                core.measure("llm_model", 0, &model);
                core.measure("llm_ms", llm_started.elapsed().as_millis() as u64, "");
                let before = core.snapshot();
                let commit_started = Instant::now();
                match core.commit_voice(&ticket, response) {
                    Ok(()) => {
                        if core.snapshot().message == before.message {
                            core.message("Passage finalisé traité ; modifications acceptées par le cœur, écoute continue.");
                        }
                    }
                    Err(message) => core.message(message),
                }
                let mode = core.snapshot().mode;
                core.measure("commit_us", commit_started.elapsed().as_micros() as u64, "");
                (mode != "voice").then_some(mode == "paused")
            };
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
    ) -> Result<(), &'static str> {
        let interpreter = MistralInterpreter;
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
            self.update(ticket, |c| {
                c.measure(
                    "priority_ms",
                    priority_started.elapsed().as_millis() as u64,
                    "",
                )
            });
            if action != Interruption::None {
                self.stop_current(ticket, action == Interruption::Pause,
                    "Interruption prioritaire ; micro arrêté, passages en attente non intégrés, reprise explicite.");
                return Ok(());
            }
        }
    }
}

async fn capture_frames(
    capture: NativeCapture,
    frames: mpsc::Sender<AudioFrame>,
) -> Result<(), &'static str> {
    loop {
        if let Some(error) = capture.failure() {
            return Err(error);
        }
        while let Ok(frame) = capture.frames.try_recv() {
            frames.try_send(frame).map_err(
                |_| "Saturation rotation audio : arrêt, passages non intégrés ; aucun spool.",
            )?;
        }
        sleep(Duration::from_millis(5)).await;
    }
}

async fn read_event(reader: &mut Option<SttReader>) -> Result<SttEvent, &'static str> {
    match reader {
        Some(reader) => SttOutput::next_event(reader).await,
        None => std::future::pending().await,
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
        controller.stop_current(&old, false, "erreur tardive");
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
}

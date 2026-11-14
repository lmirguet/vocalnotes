//! Continuous prototype orchestration. Buffers are memory-only and fail closed.
use super::{
    pipeline::{PassageGate, PassageStep},
    probe::{Probe, Ticket},
    session_devices::{CaptureStream, NativeDevices, SessionDevices, StopCapture},
};
use crate::app::document::{
    DocumentError, DocumentErrorCode, DocumentKind, DocumentOperation, DocumentOutcome,
    DocumentPhase, DocumentReply, DocumentVersion, DocumentWorkflow, OperationId,
};
#[cfg(test)]
use crate::audio::capture::NativeCapture;
use crate::audio::normalization::AudioFrame;
use crate::audio::openai_pcm::OpenAiPcm;
use crate::config::preferences::{ModelPreferences, Preferences, PreferencesStore};
use crate::domain::DictationTarget;
use crate::ports::stt::LiveStt;
use crate::ports::stt::SttEvent;
use crate::{
    adapters::openai_interpreter::OpenAIInterpreter, app::drafting::ReformatRequest,
    domain::note::Snapshot,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::Instant;
use tokio::sync::{mpsc, watch};
use tokio::time::{Duration, sleep};
#[path = "document_driver.rs"]
mod document_driver;

struct Active {
    cancel: watch::Sender<bool>,
    capture: Option<Arc<dyn StopCapture>>,
    ticket: Ticket,
    opening_stop: Arc<AtomicBool>,
    completed: watch::Receiver<Option<Result<(), &'static str>>>,
}

struct Finalized {
    passage_id: u64,
    target: DictationTarget,
    text: String,
    speech_started_at_epoch_us: Option<u64>,
    speech_ended_at_epoch_us: Option<u64>,
}

#[derive(Clone)]
pub struct Controller {
    pub core: Arc<Mutex<Probe>>,
    active: Arc<Mutex<Option<Active>>>,
    mode_transition: Arc<tokio::sync::Mutex<()>>,
    preferences: Arc<Mutex<Result<PreferencesStore, &'static str>>>,
    frozen_models: Arc<Mutex<Option<ModelPreferences>>>,
    reformat_completion: Arc<Mutex<Option<Completion>>>,
    document: Arc<Mutex<DocumentWorkflow>>,
    document_changed: watch::Sender<()>,
    document_work: Arc<Mutex<Option<DocumentWork>>>,
    document_running: Arc<AtomicBool>,
    preferences_modal: Arc<AtomicU64>,
    preferences_generation: Arc<AtomicU64>,
    preferences_writing: Arc<AtomicU64>,
    preferences_closing: Arc<AtomicU64>,
}

pub struct PreferencesWriteGuard {
    controller: Controller,
    token: u64,
}
impl Drop for PreferencesWriteGuard {
    fn drop(&mut self) {
        let _ = self.controller.preferences_writing.compare_exchange(
            self.token,
            0,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }
}

type Completion = watch::Receiver<Option<Result<(), &'static str>>>;

#[derive(Clone)]
struct DocumentWork {
    operation_id: OperationId,
    capture: Option<Completion>,
    reformat: Option<Completion>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum DocumentDrain {
    Drained,
    Cancelled,
}

struct ReformatGuard {
    completed: watch::Sender<Option<Result<(), &'static str>>>,
    finished: bool,
}

impl ReformatGuard {
    fn finish(mut self, result: Result<(), &'static str>) {
        self.completed.send_replace(Some(result));
        self.finished = true;
    }
}

impl Drop for ReformatGuard {
    fn drop(&mut self) {
        if !self.finished {
            self.completed
                .send_replace(Some(Err("Reformulation interrompue ; note conservée.")));
        }
    }
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
            mode_transition: Arc::new(tokio::sync::Mutex::new(())),
            preferences: Arc::new(Mutex::new(preferences)),
            frozen_models: Arc::new(Mutex::new(None)),
            reformat_completion: Arc::new(Mutex::new(None)),
            document: Arc::new(Mutex::new(DocumentWorkflow::default())),
            document_changed: watch::channel(()).0,
            document_work: Arc::new(Mutex::new(None)),
            document_running: Arc::new(AtomicBool::new(false)),
            preferences_modal: Arc::new(AtomicU64::new(0)),
            preferences_generation: Arc::new(AtomicU64::new(0)),
            preferences_writing: Arc::new(AtomicU64::new(0)),
            preferences_closing: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn request_document(
        &self,
        action: DocumentKind,
    ) -> Result<DocumentOperation, DocumentError> {
        let mut workflow = self
            .document
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        if workflow.is_closing()
            || self.is_preferences_modal()
            || (workflow.active().is_none() && self.document_running.load(Ordering::Acquire))
        {
            return Err(DocumentErrorCode::Busy.error());
        }
        // Keep session teardown behind request registration so its completion
        // cannot disappear between the version read and the captured waiters.
        let active = self
            .active
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        let reformat = self
            .reformat_completion
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        let capture_completion = active.as_ref().map(|session| session.completed.clone());
        let reformat_completion = reformat
            .as_ref()
            .filter(|completion| completion.borrow().is_none())
            .cloned();
        let version = {
            let core = self
                .core
                .lock()
                .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
            DocumentVersion::from(&core.snapshot())
        };
        let operation = workflow.request(action, version)?;
        let mut work = self
            .document_work
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        if work
            .as_ref()
            .is_none_or(|work| work.operation_id != operation.operation_id)
        {
            *work = Some(DocumentWork {
                operation_id: operation.operation_id,
                capture: capture_completion,
                reformat: reformat_completion,
            });
        }
        Ok(operation)
    }

    pub fn document_ready(
        &self,
        id: OperationId,
        acknowledgement: DocumentVersion,
    ) -> Result<(), DocumentError> {
        let mut workflow = self
            .document
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        let core = self
            .core
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        workflow.ready(id, acknowledgement, DocumentVersion::from(&core.snapshot()))
    }

    pub fn edit_core<T>(
        &self,
        work: impl FnOnce(&mut Probe) -> Result<T, &'static str>,
    ) -> Result<T, &'static str> {
        let workflow = self.document.lock().map_err(|_| "Cœur indisponible.")?;
        if workflow.is_closing()
            || self.is_preferences_modal()
            || workflow
                .active()
                .is_some_and(|op| op.phase != DocumentPhase::AwaitingUi)
        {
            return Err("Une opération documentaire est déjà en cours.");
        }
        let mut core = self.core.lock().map_err(|_| "Cœur indisponible.")?;
        work(&mut core)
    }

    pub fn history_action(
        &self,
        id: String,
        version: DocumentVersion,
        area: DictationTarget,
        redo: bool,
    ) -> Result<(), DocumentError> {
        let workflow = self
            .document
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        if workflow.is_closing() || self.is_preferences_modal() || workflow.active().is_some() {
            return Err(DocumentErrorCode::Busy.error());
        }
        self.core
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?
            .history_action(id, version, area, redo)
    }

    pub fn cancel_document(&self, id: OperationId) -> Result<DocumentOutcome, DocumentError> {
        let mut workflow = self
            .document
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        let replaces = workflow
            .active()
            .is_some_and(|operation| operation.action.replaces_document());
        let outcome = workflow.cancel(id)?;
        if replaces {
            self.stop_native(
                false,
                "Opération annulée ; note conservée, microphone arrêté.",
                true,
            )
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        }
        drop(workflow);
        self.document_changed.send_replace(());
        Ok(outcome)
    }

    pub fn document_operation(&self) -> Result<Option<DocumentOperation>, DocumentError> {
        Ok(self
            .document
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?
            .active()
            .cloned())
    }

    pub fn finish_quit(
        &self,
        id: OperationId,
        destroy: impl FnOnce() -> Result<(), ()>,
    ) -> Result<Snapshot, DocumentError> {
        self.document
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?
            .authorize_close(id)?;
        if destroy().is_err() {
            self.document
                .lock()
                .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?
                .close_failed(id);
            return Err(DocumentErrorCode::WindowFailed.error());
        }
        self.snapshot()
    }

    pub fn fail_document(
        &self,
        id: OperationId,
        error: DocumentError,
    ) -> Result<DocumentOutcome, DocumentError> {
        let mut workflow = self
            .document
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        let replaces = workflow
            .active()
            .is_some_and(|operation| operation.action.replaces_document());
        let outcome = workflow.fail(id, error)?;
        if replaces {
            self.stop_native(
                false,
                "Opération abandonnée ; note conservée, microphone arrêté.",
                true,
            )
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        }
        drop(workflow);
        self.document_changed.send_replace(());
        Ok(outcome)
    }

    pub fn snapshot(&self) -> Result<Snapshot, DocumentError> {
        Ok(self.document_reply()?.snapshot)
    }

    pub fn document_focus(&self, area: DictationTarget) -> Result<(), DocumentError> {
        self.document
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?
            .focus(area);
        Ok(())
    }

    pub fn begin_preferences(&self) -> Result<Option<u64>, DocumentError> {
        let _transition = self
            .mode_transition
            .try_lock()
            .map_err(|_| DocumentErrorCode::Busy.error())?;
        let workflow = self
            .document
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        if workflow.is_closing()
            || workflow.active().is_some()
            || self.document_running.load(Ordering::Acquire)
        {
            return Err(DocumentErrorCode::Busy.error());
        }
        if self.is_preferences_modal() {
            return Ok(None);
        }
        let token = self
            .preferences_generation
            .load(Ordering::Acquire)
            .checked_add(1)
            .ok_or_else(|| DocumentErrorCode::SizeLimit.error())?;
        self.preferences_generation.store(token, Ordering::Release);
        self.preferences_modal.store(token, Ordering::Release);
        Ok(Some(token))
    }

    pub fn end_preferences(&self, token: u64) -> bool {
        token != 0
            && self
                .preferences_modal
                .compare_exchange(token, 0, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
    }
    pub fn preferences_token(&self) -> u64 {
        self.preferences_modal.load(Ordering::Acquire)
    }

    pub fn reserve_preferences_write(&self) -> Result<PreferencesWriteGuard, &'static str> {
        let _workflow = self
            .document
            .lock()
            .map_err(|_| "Coordinateur indisponible.")?;
        let token = self.preferences_token();
        if token == 0
            || self.preferences_closing.load(Ordering::Acquire) == token
            || self
                .preferences_writing
                .compare_exchange(0, token, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
        {
            return Err("Préférences indisponibles ou écriture en cours.");
        }
        Ok(PreferencesWriteGuard {
            controller: self.clone(),
            token,
        })
    }

    pub fn preferences_write_pending(&self, token: u64) -> bool {
        token != 0 && self.preferences_writing.load(Ordering::Acquire) == token
    }
    pub fn begin_preferences_close(&self, token: u64) -> Result<bool, DocumentError> {
        let _workflow = self
            .document
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        if token == 0 || token != self.preferences_token() {
            return Ok(false);
        }
        if self.preferences_write_pending(token) {
            return Err(DocumentErrorCode::Busy.error());
        }
        self.preferences_closing.store(token, Ordering::Release);
        Ok(true)
    }
    pub fn is_preferences_modal(&self) -> bool {
        self.preferences_token() != 0
    }

    pub fn document_focused_area(&self) -> Result<DictationTarget, DocumentError> {
        Ok(self
            .document
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?
            .focused_area())
    }

    pub fn document_reply(&self) -> Result<DocumentReply, DocumentError> {
        let workflow = self
            .document
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        let core = self
            .core
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        let mut snapshot = core.snapshot();
        workflow.fill_snapshot(&mut snapshot);
        Ok(DocumentReply {
            snapshot,
            outcome: workflow.last_outcome().cloned(),
        })
    }

    pub fn attach_document_state(&self, snapshot: &mut Snapshot) -> Result<(), DocumentError> {
        self.document
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?
            .fill_snapshot(snapshot);
        Ok(())
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
            self.remember_document_path(&path);
        }
    }

    pub(super) fn remember_document_path(&self, path: &std::path::Path) {
        let result = self
            .preferences
            .lock()
            .map_err(|_| "Préférences indisponibles.")
            .and_then(|mut store| store.as_mut().map_err(|e| *e)?.remember_note(path));
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

    pub async fn reformat(
        &self,
        note_id: u64,
        base_revision: u64,
    ) -> Result<Snapshot, &'static str> {
        let completion = self.begin_reformat()?;
        let result: Result<Snapshot, &'static str> = async {
            let request = {
                let mut core = self.core.lock().map_err(|_| "Cœur indisponible.")?;
                let request = ReformatRequest::prepare(&core, note_id, base_revision)?;
                core.reformat_started();
                request
            };
            let selection = self.preferences()?.llm;
            let interpreter = OpenAIInterpreter::selected(&selection)?;
            let reply = request.reformat(&interpreter).await?;
            let mut core = self.core.lock().map_err(|_| "Cœur indisponible.")?;
            reply.commit(&mut core)?;
            Ok(core.snapshot())
        }
        .await;
        if let Err(message) = &result {
            if let Ok(mut core) = self.core.lock() {
                core.message(message);
            }
        }
        completion.finish(result.as_ref().map(|_| ()).map_err(|message| *message));
        result
    }

    fn begin_reformat(&self) -> Result<ReformatGuard, &'static str> {
        let workflow = self
            .document
            .lock()
            .map_err(|_| "Coordinateur indisponible.")?;
        if workflow.is_closing() || self.is_preferences_modal() || workflow.active().is_some() {
            return Err("Une opération documentaire est en cours.");
        }
        let mut active = self
            .reformat_completion
            .lock()
            .map_err(|_| "Reformulation indisponible.")?;
        if active
            .as_ref()
            .is_some_and(|completion| completion.borrow().is_none())
        {
            return Err("Une reformulation est déjà en cours.");
        }
        let (completed, receiver) = watch::channel(None);
        *active = Some(receiver);
        Ok(ReformatGuard {
            completed,
            finished: false,
        })
    }

    pub fn user_stop(&self, pause: bool, message: &str) -> Result<Snapshot, &'static str> {
        let workflow = self
            .document
            .lock()
            .map_err(|_| "Coordinateur indisponible.")?;
        if workflow.is_closing() || self.is_preferences_modal() || workflow.active().is_some() {
            return Err("Une opération documentaire est en cours.");
        }
        self.stop_native(pause, message, false)?;
        Ok(self
            .core
            .lock()
            .map_err(|_| "Cœur indisponible.")?
            .snapshot())
    }

    pub fn stop(&self, pause: bool, message: &str) {
        if self.stop_native(pause, message, false).is_err() {
            crate::diagnostics::event("Arrêt incomplet : état natif indisponible.");
        }
    }

    fn stop_native(
        &self,
        pause: bool,
        message: &str,
        invalidate_inputs: bool,
    ) -> Result<(), &'static str> {
        crate::diagnostics::event(if pause {
            "Commande : pause"
        } else {
            "Commande : arrêt/invalidation"
        });
        // Same lock order for start, device installation and stop.
        let mut active = self.active.lock().map_err(|_| "Contrôleur indisponible.")?;
        if let Some(session) = active.take() {
            session.opening_stop.store(true, Ordering::Release);
            if let Some(capture) = session.capture {
                capture.stop();
            }
            let _ = session.cancel.send(true);
        }
        let mut core = self.core.lock().map_err(|_| "Cœur indisponible.")?;
        if pause {
            core.pause();
        } else {
            core.stop(message);
        }
        if invalidate_inputs {
            core.invalidate_document_inputs();
        }
        Ok(())
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

    #[cfg(test)]
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

    pub async fn set_dictation_mode(
        &self,
        target: DictationTarget,
    ) -> Result<Snapshot, &'static str> {
        self.set_dictation_mode_with_devices(target, NativeDevices)
            .await
    }

    fn stop_capture(&self) -> Result<Option<Completion>, &'static str> {
        let active = {
            let active = self.active.lock().map_err(|_| "Contrôleur indisponible.")?;
            active.as_ref().map(|session| {
                (
                    session.capture.clone(),
                    session.cancel.clone(),
                    session.opening_stop.clone(),
                    session.completed.clone(),
                )
            })
        };
        let Some((capture, cancel, opening_stop, completed)) = active else {
            return Ok(None);
        };
        if let Some(capture) = capture {
            capture.stop();
        } else {
            opening_stop.store(true, Ordering::Release);
            let _ = cancel.send(true);
        }
        Ok(Some(completed))
    }

    async fn drain_capture(&self) -> Result<bool, &'static str> {
        let completion = self.stop_capture()?;
        let had_active = completion.is_some();
        wait_completion(completion).await?;
        Ok(had_active)
    }

    async fn stop_active_mode(&self, message: &'static str) -> Result<bool, &'static str> {
        let had_active = self.drain_capture().await?;
        if had_active {
            self.stop_native(false, message, false)?;
        }
        Ok(had_active)
    }

    pub async fn drain_document(&self, id: OperationId) -> Result<DocumentDrain, DocumentError> {
        let _transition = self.mode_transition.lock().await;
        let mut changed = self.document_changed.subscribe();
        {
            let mut workflow = self
                .document
                .lock()
                .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
            if let Some(outcome) = document_interruption(&workflow, id)? {
                return Ok(outcome);
            }
            let core = self
                .core
                .lock()
                .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
            workflow.transition(id, core.snapshot().note_id, DocumentPhase::Draining)?;
        }
        let work = self
            .document_work
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?
            .as_ref()
            .filter(|work| work.operation_id == id)
            .cloned()
            .ok_or_else(|| DocumentErrorCode::StaleOperation.error())?;
        let draining = async {
            let current_capture = self.stop_capture()?;
            tokio::try_join!(
                wait_completion(work.capture.or(current_capture)),
                wait_completion(work.reformat)
            )?;
            Ok::<(), &'static str>(())
        };
        tokio::pin!(draining);
        let result = loop {
            tokio::select! {
                biased;
                _ = changed.changed() => {
                    let workflow = self.document.lock().map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
                    if let Some(outcome) = document_interruption(&workflow, id)? { return Ok(outcome); }
                }
                result = &mut draining => break result,
            }
        };
        let mut workflow = self
            .document
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        if let Some(outcome) = document_interruption(&workflow, id)? {
            return Ok(outcome);
        }
        self.stop_native(false, "Traitements terminés ; microphone arrêté.", true)
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        if result.is_err() {
            let error = DocumentErrorCode::DrainFailed.error();
            workflow.fail(id, error.clone())?;
            drop(workflow);
            self.document_changed.send_replace(());
            return Err(error);
        }
        Ok(DocumentDrain::Drained)
    }

    pub async fn set_dictation_mode_with_devices<D: SessionDevices>(
        &self,
        target: DictationTarget,
        devices: D,
    ) -> Result<Snapshot, &'static str> {
        let _transition = self.mode_transition.lock().await;
        {
            let workflow = self
                .document
                .lock()
                .map_err(|_| "Coordinateur indisponible.")?;
            if workflow.is_closing() || self.is_preferences_modal() || workflow.active().is_some() {
                return Err("Une opération documentaire est en cours.");
            }
        }
        let has_active = {
            let active = self.active.lock().map_err(|_| "Contrôleur indisponible.")?;
            active.is_some()
        };
        let current = self
            .core
            .lock()
            .map_err(|_| "Cœur indisponible.")?
            .snapshot();
        if current.mode == "voice" && current.dictation_mode == target {
            if !self
                .stop_active_mode("Écoute arrêtée ; aucun mode sélectionné.")
                .await?
            {
                self.stop(false, "Écoute arrêtée ; aucun mode sélectionné.");
            }
            self.core
                .lock()
                .map_err(|_| "Cœur indisponible.")?
                .set_dictation_mode(DictationTarget::Content);
            return Ok(self
                .core
                .lock()
                .map_err(|_| "Cœur indisponible.")?
                .snapshot());
        }
        if has_active {
            self.stop_active_mode("Mode modifié ; le passage en attente a été finalisé.")
                .await?;
        }
        {
            self.core
                .lock()
                .map_err(|_| "Cœur indisponible.")?
                .set_dictation_mode(target);
        }
        self.start_with_devices(false, devices)?;
        Ok(self
            .core
            .lock()
            .map_err(|_| "Cœur indisponible.")?
            .snapshot())
    }

    /// Rust-only dependency injection; the production IPC always selects NativeDevices.
    pub fn start_with_devices<D: SessionDevices>(
        &self,
        resume: bool,
        devices: D,
    ) -> Result<(), &'static str> {
        let workflow = self
            .document
            .lock()
            .map_err(|_| "Coordinateur indisponible.")?;
        if workflow.is_closing() || self.is_preferences_modal() || workflow.active().is_some() {
            return Err("Une opération documentaire est en cours.");
        }
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
        let (completed_tx, completed_rx) = watch::channel(None);
        let opening_stop = Arc::new(AtomicBool::new(false));
        *active = Some(Active {
            cancel,
            capture: None,
            ticket: ticket.clone(),
            opening_stop: opening_stop.clone(),
            completed: completed_rx,
        });
        let this = self.clone();
        tauri::async_runtime::spawn(async move {
            let result = this
                .session(ticket.clone(), receiver, opening_stop, models, devices)
                .await;
            if let Err(message) = result {
                this.stop_current(&ticket, false, message, true);
            }
            this.finish_session(&ticket);
            let _ = completed_tx.send(Some(result));
        });
        Ok(())
    }

    fn finish_session(&self, ticket: &Ticket) {
        if let Ok(mut active) = self.active.lock()
            && active
                .as_ref()
                .is_some_and(|session| session.ticket.same_generation(ticket))
        {
            active.take();
        }
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
        let work = async {
            tokio::try_join!(
                capture_frames(capture, frames_tx),
                async {
                    self.transcribe(&ticket, frames_rx, final_tx, transcription)
                        .await
                        .map_err(|reason| {
                            self.update(&ticket, |core| core.failure_stage("recognition"));
                            reason
                        })
                },
                self.route_finalized(&ticket, final_rx),
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
        mut stream: S,
    ) -> Result<(), &'static str> {
        let mut gate = PassageGate::default();
        let mut converter = None;
        let mut passage_id = 0;
        let mut target_at_start = DictationTarget::Content;
        let mut speech_started_at_epoch_us = None;
        loop {
            // Read events continuously while feeding audio; normal LLM work is independent.
            tokio::select! {
                event = stream.next_event() => {
                    match event? {
                        SttEvent::Final { .. } => return Err("Final STT prématuré ; passage non intégré."),
                        SttEvent::Provisional(text) => {
                            self.update(ticket, |c| c.transcript_delta(&text));
                        }
                        SttEvent::Other => {}
                    }
                }
                frame = frames.recv() => {
                    let step = match frame {
                        Some(frame) => gate.push(frame)?,
                        None => match gate.finish() {
                            Some(step) => step,
                            None => return Ok(()),
                        },
                    };
                    match step {
                        PassageStep::Wait => {}
                        PassageStep::Start { pcm, speech_started_at_epoch_us: started_at } => {
                            crate::diagnostics::event("STT : début de passage détecté par VAD");
                            passage_id += 1;
                            target_at_start = {
                                let mut core = self.core.lock().map_err(|_| "Cœur indisponible.")?;
                                if !core.same_session(ticket) {
                                    return Ok(());
                                }
                                core.begin_dictation_segment()?
                            };
                            speech_started_at_epoch_us = started_at;
                            self.update(ticket, |c| c.speech_passage_started(passage_id, started_at));
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
                                            target: target_at_start,
                                            text,
                                            speech_started_at_epoch_us,
                                            speech_ended_at_epoch_us,
                                        }).map_err(|_| "Saturation du routage : passages non intégrés, arrêt sans spool.")?;
                                        break;
                                    }
                                    SttEvent::Provisional(text) => {
                                        self.update(ticket, |c| c.transcript_delta(&text));
                                    },
                                    SttEvent::Other => {}
                                }
                            }
                            converter = None;
                        }
                    }
                }
            }
        }
    }

    async fn route_finalized(
        &self,
        session: &Ticket,
        mut finals: mpsc::Receiver<Finalized>,
    ) -> Result<(), &'static str> {
        while let Some(Finalized {
            passage_id,
            target,
            text,
            speech_started_at_epoch_us,
            speech_ended_at_epoch_us,
        }) = finals.recv().await
        {
            {
                let mut core = self.core.lock().map_err(|_| "Cœur indisponible.")?;
                if !core.same_session(session) {
                    return Ok(());
                }
                core.route_dictation_text(target, text.clone())?;
                if !text.trim().is_empty() {
                    core.speech_passage_applied(
                        passage_id,
                        speech_started_at_epoch_us,
                        speech_ended_at_epoch_us,
                    );
                }
            }
            self.remember_saved_note();
        }
        Ok(())
    }
}

async fn wait_completion(completion: Option<Completion>) -> Result<(), &'static str> {
    if let Some(mut completion) = completion {
        if completion.borrow().is_none() {
            completion
                .changed()
                .await
                .map_err(|_| "Fin du traitement non confirmée.")?;
        }
        return (*completion.borrow()).ok_or("Fin du traitement non confirmée.")?;
    }
    Ok(())
}

fn document_interruption(
    workflow: &DocumentWorkflow,
    id: OperationId,
) -> Result<Option<DocumentDrain>, DocumentError> {
    if workflow
        .active()
        .is_some_and(|operation| operation.operation_id == id)
    {
        return Ok(None);
    }
    match workflow.last_outcome() {
        Some(DocumentOutcome::Cancelled { operation_id }) if *operation_id == id => {
            Ok(Some(DocumentDrain::Cancelled))
        }
        Some(DocumentOutcome::Failed {
            operation_id,
            error,
        }) if *operation_id == id => Err(error.clone()),
        _ => Err(DocumentErrorCode::StaleOperation.error()),
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
        if capture.is_finished() {
            return Ok(());
        }
        sleep(Duration::from_millis(5)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn save_cancellation_keeps_capture_running_and_new_work_is_blocked_by_document_waits() {
        let controller = Controller::new(Probe::new("note".into()));
        let devices = FakeDevices::default();
        controller
            .start_with_devices(false, devices.clone())
            .unwrap();
        devices.wait_until_listening(&controller).await;
        let before = controller.snapshot().unwrap();
        let operation = controller.request_document(DocumentKind::Save).unwrap();
        assert!(controller.begin_reformat().is_err());
        controller.cancel_document(operation.operation_id).unwrap();
        assert_eq!(
            controller.snapshot().unwrap().input_generation,
            before.input_generation
        );
        assert_eq!(controller.snapshot().unwrap().mode, "voice");
        assert_eq!(devices.stops.load(Ordering::SeqCst), 0);
        let operation = prepared_document(&controller);
        assert!(
            controller
                .start_with_devices(false, devices.clone())
                .is_err()
        );
        controller.cancel_document(operation).unwrap();
        assert_eq!(controller.snapshot().unwrap().mode, "manual");
    }

    #[tokio::test]
    async fn capture_failure_during_ui_preparation_is_not_lost_before_drain() {
        let controller = Controller::new(Probe::new("note".into()));
        let devices = FakeDevices {
            fail_finalize: true,
            ..Default::default()
        };
        controller
            .start_with_devices(false, devices.clone())
            .unwrap();
        devices.wait_until_listening(&controller).await;
        devices.push_speech();
        tokio::time::timeout(Duration::from_secs(2), async {
            while devices.passage_starts.load(Ordering::SeqCst) == 0 {
                sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
        let operation = controller.request_document(DocumentKind::Quit).unwrap();
        let completion = controller
            .active
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .completed
            .clone();
        controller.stop_capture().unwrap();
        assert!(wait_completion(Some(completion)).await.is_err());
        let version = DocumentVersion::from(&controller.snapshot().unwrap());
        controller
            .document_ready(operation.operation_id, version)
            .unwrap();
        let result = controller.drain_document(operation.operation_id).await;
        assert!(matches!(result, Err(error) if error.code == DocumentErrorCode::DrainFailed));
        assert_eq!(controller.snapshot().unwrap().source, "note");
    }

    #[tokio::test]
    async fn reformat_failure_during_ui_preparation_survives_a_joined_request() {
        let controller = Controller::new(Probe::new("note".into()));
        let reformat = controller.begin_reformat().unwrap();
        let operation = controller.request_document(DocumentKind::Quit).unwrap();
        reformat.finish(Err("synthetic failure before ready"));
        assert_eq!(
            controller.request_document(DocumentKind::Save).unwrap(),
            operation
        );
        let version = DocumentVersion::from(&controller.snapshot().unwrap());
        controller
            .document_ready(operation.operation_id, version)
            .unwrap();
        let result = controller.drain_document(operation.operation_id).await;
        assert!(matches!(result, Err(error) if error.code == DocumentErrorCode::DrainFailed));
        assert_eq!(controller.snapshot().unwrap().source, "note");
    }

    fn prepared_document(controller: &Controller) -> OperationId {
        let operation = controller.request_document(DocumentKind::Quit).unwrap();
        let version = DocumentVersion::from(&controller.snapshot().unwrap());
        controller
            .document_ready(operation.operation_id, version)
            .unwrap();
        operation.operation_id
    }

    #[tokio::test]
    async fn document_drain_accepts_final_speech_and_waits_for_the_started_reformat() {
        let controller = Controller::new(Probe::new("note".into()));
        let devices = FakeDevices::default();
        controller
            .start_with_devices(false, devices.clone())
            .unwrap();
        devices.wait_until_listening(&controller).await;
        devices.push_speech();
        tokio::time::timeout(Duration::from_secs(2), async {
            while devices.passage_starts.load(Ordering::SeqCst) == 0 {
                sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
        let ticket = controller.core.lock().unwrap().ticket();
        let reformat = controller.begin_reformat().unwrap();
        let capture_completion = controller
            .active
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .completed
            .clone();
        let operation = prepared_document(&controller);
        let mut drain = Box::pin(controller.drain_document(operation));
        tokio::select! {
            _ = &mut drain => panic!("drain finished before the reformat"),
            completion = tokio::time::timeout(
                Duration::from_secs(2),
                wait_completion(Some(capture_completion)),
            ) => completion.unwrap().unwrap(),
        }
        assert!(devices.stops.load(Ordering::SeqCst) > 0);
        assert!(controller.core.try_lock().is_ok());
        assert!(controller.core.lock().unwrap().same_session(&ticket));
        assert!(
            controller
                .core
                .lock()
                .unwrap()
                .snapshot()
                .source
                .contains("Texte final.")
        );
        reformat.finish(Ok(()));
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(2), drain)
                .await
                .unwrap()
                .unwrap(),
            DocumentDrain::Drained
        );
        let snapshot = controller.snapshot().unwrap();
        assert_eq!(snapshot.mode, "manual");
        assert!(snapshot.source.contains("Texte final."));
        assert!(snapshot.dirty);
        assert_eq!(snapshot.llm_requests, 0);
    }

    #[tokio::test]
    async fn cancelling_document_drain_invalidates_a_late_reformat_without_losing_the_note() {
        let controller = Controller::new(Probe::new("note".into()));
        let reformat = controller.begin_reformat().unwrap();
        let version = controller.snapshot().unwrap();
        let operation = prepared_document(&controller);
        let mut drain = Box::pin(controller.drain_document(operation));
        tokio::select! {
            _ = &mut drain => panic!("drain finished before cancellation"),
            _ = sleep(Duration::from_millis(5)) => {}
        }
        controller.cancel_document(operation).unwrap();
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(2), drain)
                .await
                .unwrap()
                .unwrap(),
            DocumentDrain::Cancelled
        );
        assert!(
            controller
                .core
                .lock()
                .unwrap()
                .apply_reformat(
                    version.note_id,
                    version.revision,
                    version.input_generation,
                    &[],
                    "late".into(),
                    "test"
                )
                .is_err()
        );
        reformat.finish(Ok(()));
        let snapshot = controller.snapshot().unwrap();
        assert_eq!(snapshot.source, "note");
        assert_eq!(snapshot.mode, "manual");
        assert!(snapshot.document_operation.is_none());
    }

    #[tokio::test]
    async fn failed_reformat_abandons_document_drain_and_dropping_a_job_signals_failure() {
        for dropped in [false, true] {
            let controller = Controller::new(Probe::new("note".into()));
            let reformat = controller.begin_reformat().unwrap();
            let operation = prepared_document(&controller);
            let mut drain = Box::pin(controller.drain_document(operation));
            tokio::select! {
                _ = &mut drain => panic!("drain finished before completion"),
                _ = sleep(Duration::from_millis(5)) => {}
            }
            if dropped {
                drop(reformat);
            } else {
                reformat.finish(Err("synthetic failure"));
            }
            let result = tokio::time::timeout(Duration::from_secs(2), drain)
                .await
                .unwrap();
            assert!(matches!(result, Err(error) if error.code == DocumentErrorCode::DrainFailed));
            let snapshot = controller.snapshot().unwrap();
            assert_eq!(snapshot.source, "note");
            assert_eq!(snapshot.mode, "manual");
            assert!(snapshot.document_operation.is_none());
            assert_eq!(
                snapshot.document_error.unwrap().code,
                DocumentErrorCode::DrainFailed
            );
        }
    }

    #[test]
    fn document_requests_from_clones_share_one_operation_without_holding_core() {
        let controller = Controller::new(Probe::new("note".into()));
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let workers: Vec<_> = [DocumentKind::Save, DocumentKind::Quit]
            .into_iter()
            .map(|kind| {
                let controller = controller.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    controller.request_document(kind).unwrap()
                })
            })
            .collect();
        let operations: Vec<_> = workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect();
        assert_eq!(operations[0], operations[1]);
        assert!(controller.core.try_lock().is_ok());
        controller
            .cancel_document(operations[0].operation_id)
            .unwrap();
        assert!(controller.document_operation().unwrap().is_none());
    }

    #[test]
    fn document_snapshot_recovers_a_missed_operation_and_clears_it_after_cancellation() {
        let controller = Controller::new(Probe::new("texte local".into()));
        let operation = controller.request_document(DocumentKind::Quit).unwrap();
        let snapshot = controller.snapshot().unwrap();
        assert_eq!(snapshot.source, "texte local");
        assert_eq!(snapshot.document_operation, Some(operation.clone()));
        let dispatch = serde_json::to_value(&operation).unwrap();
        assert!(!dispatch.to_string().contains("texte local"));
        controller.cancel_document(operation.operation_id).unwrap();
        let snapshot = controller.snapshot().unwrap();
        assert!(snapshot.document_operation.is_none());
        assert!(snapshot.document_error.is_none());
        assert_eq!(snapshot.source, "texte local");
    }

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
            completed: watch::channel(None).1,
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
        fail_end: bool,
    }
    impl crate::ports::stt::SttInput for Stream {
        async fn append_pcm(&mut self, _: &[u8]) -> Result<(), &'static str> {
            self.sender
                .try_send(SttEvent::Provisional("aperçu".into()))
                .map_err(|_| "test queue full")
        }
        async fn end_passage(&mut self) -> Result<(), &'static str> {
            if self.fail_end {
                return Err("Finalisation STT synthétique impossible.");
            }
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

    #[derive(Clone, Default)]
    struct FakeDevices {
        captures: Arc<std::sync::atomic::AtomicUsize>,
        stops: Arc<std::sync::atomic::AtomicUsize>,
        passage_starts: Arc<std::sync::atomic::AtomicUsize>,
        frames: Arc<Mutex<std::collections::VecDeque<AudioFrame>>>,
        fail_capture: bool,
        fail_finalize: bool,
    }

    impl FakeDevices {
        fn push_speech(&self) {
            self.frames.lock().unwrap().push_back(AudioFrame {
                pcm: vec![0; 960],
                voiced: true,
                end_passage: false,
                capture_start_epoch_us: Some(1_000_000),
            });
        }

        async fn wait_for_captures(&self, expected: usize) {
            tokio::time::timeout(Duration::from_secs(2), async {
                while self.captures.load(Ordering::SeqCst) < expected {
                    sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .expect("capture startup timed out");
        }

        async fn wait_until_listening(&self, controller: &Controller) {
            tokio::time::timeout(Duration::from_secs(2), async {
                while controller.core.lock().unwrap().snapshot().listening_state != "listening" {
                    sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .expect("capture readiness timed out");
        }
    }

    struct FakeCapture {
        devices: FakeDevices,
        stopped: Arc<AtomicBool>,
    }

    struct FakeCaptureControl {
        stopped: Arc<AtomicBool>,
        stops: Arc<std::sync::atomic::AtomicUsize>,
    }

    impl StopCapture for FakeCaptureControl {
        fn stop(&self) {
            self.stopped.store(true, Ordering::Release);
            self.stops.fetch_add(1, Ordering::SeqCst);
        }
    }

    impl CaptureStream for FakeCapture {
        fn control(&self) -> Arc<dyn StopCapture> {
            Arc::new(FakeCaptureControl {
                stopped: self.stopped.clone(),
                stops: self.devices.stops.clone(),
            })
        }

        fn next_frame(&self) -> Result<Option<AudioFrame>, &'static str> {
            Ok(self.devices.frames.lock().unwrap().pop_front())
        }

        fn is_finished(&self) -> bool {
            self.stopped.load(Ordering::Acquire) && self.devices.frames.lock().unwrap().is_empty()
        }
    }

    impl SessionDevices for FakeDevices {
        type Transcription = Stream;

        async fn transcription(
            &self,
            _: &crate::config::preferences::ServiceSelection,
        ) -> Result<Self::Transcription, &'static str> {
            let (sender, events) = mpsc::channel(16);
            Ok(Stream {
                events,
                sender,
                starts: self.passage_starts.clone(),
                fail_end: self.fail_finalize,
            })
        }

        async fn monitor(
            &self,
            _: Arc<AtomicBool>,
            _: Arc<dyn Fn() + Send + Sync>,
        ) -> Result<Box<dyn Send>, &'static str> {
            Ok(Box::new(()))
        }

        async fn key_present(&self) -> Result<bool, &'static str> {
            Ok(true)
        }

        async fn capture(
            &self,
            _: Arc<AtomicBool>,
        ) -> Result<Box<dyn CaptureStream>, &'static str> {
            if self.fail_capture {
                return Err("Microphone synthétique indisponible.");
            }
            self.captures.fetch_add(1, Ordering::SeqCst);
            Ok(Box::new(FakeCapture {
                devices: self.clone(),
                stopped: Arc::new(AtomicBool::new(false)),
            }))
        }
    }

    #[tokio::test]
    async fn selecting_modes_starts_capture_and_drains_a_pending_instruction_passage() {
        let controller = Controller::new(Probe::new("note originale".into()));
        let devices = FakeDevices::default();
        controller
            .set_dictation_mode_with_devices(DictationTarget::Instructions, devices.clone())
            .await
            .unwrap();
        devices.wait_for_captures(1).await;
        devices.wait_until_listening(&controller).await;
        controller
            .core
            .lock()
            .unwrap()
            .set_instructions("initial".into())
            .unwrap();

        devices.push_speech();
        tokio::time::timeout(Duration::from_secs(2), async {
            while devices.passage_starts.load(Ordering::SeqCst) == 0 {
                sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("speech passage did not start");

        controller
            .set_dictation_mode_with_devices(DictationTarget::Content, devices.clone())
            .await
            .unwrap();
        devices.wait_for_captures(2).await;
        devices.wait_until_listening(&controller).await;
        let snapshot = controller.core.lock().unwrap().snapshot();
        assert_eq!(snapshot.dictation_mode, DictationTarget::Content);
        assert_eq!(snapshot.voice_applied, 1);
        assert!(snapshot.instructions.is_empty());
        assert_eq!(devices.stops.load(Ordering::SeqCst), 1);
        controller.stop(false, "fin du test");
    }

    #[tokio::test]
    async fn selecting_the_active_instruction_mode_stops_capture_after_routing_pending_speech() {
        let controller = Controller::new(Probe::new("note originale".into()));
        let devices = FakeDevices::default();
        controller
            .set_dictation_mode_with_devices(DictationTarget::Instructions, devices.clone())
            .await
            .unwrap();
        devices.wait_for_captures(1).await;
        devices.wait_until_listening(&controller).await;
        controller
            .core
            .lock()
            .unwrap()
            .set_instructions("initial".into())
            .unwrap();
        devices.push_speech();
        tokio::time::timeout(Duration::from_secs(2), async {
            while devices.passage_starts.load(Ordering::SeqCst) == 0 {
                sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("speech passage did not start");

        let stopped = controller
            .set_dictation_mode_with_devices(DictationTarget::Instructions, devices.clone())
            .await
            .unwrap();

        assert_eq!(stopped.mode, "manual");
        assert_eq!(stopped.listening_state, "stopped");
        assert_eq!(stopped.dictation_mode, DictationTarget::Content);
        assert_eq!(stopped.voice_applied, 1);
        assert!(stopped.instructions.is_empty());
        assert_eq!(devices.captures.load(Ordering::SeqCst), 1);
        assert_eq!(devices.stops.load(Ordering::SeqCst), 1);

        controller
            .set_dictation_mode_with_devices(DictationTarget::Context, devices.clone())
            .await
            .unwrap();
        devices.wait_for_captures(2).await;
        devices.wait_until_listening(&controller).await;
        assert_eq!(
            controller.core.lock().unwrap().snapshot().dictation_mode,
            DictationTarget::Context
        );
        controller.stop(false, "fin du test");
    }

    #[tokio::test]
    async fn automatic_capture_reports_device_failure_and_close_stops_active_capture() {
        let controller = Controller::new(Probe::new("note".into()));
        let failing = FakeDevices {
            fail_capture: true,
            ..FakeDevices::default()
        };
        controller
            .set_dictation_mode_with_devices(DictationTarget::Content, failing)
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(2), async {
            while controller.core.lock().unwrap().snapshot().voice_activity != "error" {
                sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("capture failure was not surfaced");

        let devices = FakeDevices::default();
        controller
            .set_dictation_mode_with_devices(DictationTarget::Context, devices.clone())
            .await
            .unwrap();
        devices.wait_for_captures(1).await;
        devices.wait_until_listening(&controller).await;
        controller.stop(false, "fermeture de note");
        assert_eq!(devices.stops.load(Ordering::SeqCst), 1);
        assert_eq!(controller.core.lock().unwrap().snapshot().mode, "manual");
    }

    #[tokio::test]
    async fn failed_old_passage_does_not_claim_a_successful_mode_transition() {
        let controller = Controller::new(Probe::new("note originale".into()));
        let devices = FakeDevices {
            fail_finalize: true,
            ..FakeDevices::default()
        };
        controller
            .set_dictation_mode_with_devices(DictationTarget::Instructions, devices.clone())
            .await
            .unwrap();
        devices.wait_for_captures(1).await;
        devices.wait_until_listening(&controller).await;
        devices.push_speech();
        tokio::time::timeout(Duration::from_secs(2), async {
            while devices.passage_starts.load(Ordering::SeqCst) == 0 {
                sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("speech passage did not start");

        assert!(
            controller
                .set_dictation_mode_with_devices(DictationTarget::Content, devices.clone())
                .await
                .is_err()
        );
        assert_eq!(devices.captures.load(Ordering::SeqCst), 1);
        assert_eq!(
            controller.core.lock().unwrap().snapshot().voice_activity,
            "error"
        );
        assert_eq!(
            controller.core.lock().unwrap().snapshot().dictation_mode,
            DictationTarget::Instructions
        );
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
            fail_end: false,
        };
        let (frames_tx, frames_rx) = mpsc::channel(128);
        let (final_tx, mut final_rx) = mpsc::channel(4);
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
                .transcribe(&ticket, frames_rx, final_tx, stream)
                .await
        });
        for id in 1..=2 {
            let final_text = tokio::time::timeout(Duration::from_secs(2), final_rx.recv())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(final_text.passage_id, id);
            assert_eq!(final_text.target, DictationTarget::Content);
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
            fail_end: false,
        };
        let (frames_tx, frames_rx) = mpsc::channel(128);
        let (final_tx, mut finals) = mpsc::channel(4);
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
            c.transcribe(&ticket, frames_rx, final_tx, stream),
        )
        .await
        .unwrap();
        assert!(result.err().unwrap().contains("Saturation du routage"));
        let mut count = 0;
        while finals.try_recv().is_ok() {
            count += 1;
        }
        assert_eq!(count, 4);
        assert_eq!(c.core.lock().unwrap().snapshot().source, "note originale");
    }
}

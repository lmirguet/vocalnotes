//! Async document engine. No domain lock survives a dialog or filesystem wait.
use super::*;
use crate::{
    app::document::{DocumentFormat, DocumentReply},
    domain::intention::DraftingIntention,
    platform::desktop::{DocumentDialogs, FileDialogRequest, OverwriteDecision, SaveDecision},
    storage::{
        markdown_file::{DiskVersion, MarkdownFile, WriteCandidate, WriteContent, WriteResult},
        note_document,
    },
};
use std::path::PathBuf;

struct Running(Arc<AtomicBool>);
impl Drop for Running {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

async fn io<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, DocumentError> + Send + 'static,
) -> Result<T, DocumentError> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?
}

fn read_opened(path: PathBuf) -> Result<(MarkdownFile, String, DraftingIntention), DocumentError> {
    crate::storage::markdown_file::validate_note_path(&path)
        .map_err(|_| DocumentErrorCode::InvalidPath.error())?;
    let (storage, bytes) = MarkdownFile::open(&path).map_err(|error| error.code.error())?;
    let (source, intention) =
        note_document::decode(&bytes).map_err(|_| DocumentErrorCode::InvalidDocument.error())?;
    Ok((storage, source, intention))
}

impl Controller {
    async fn wait_dialog<T>(
        &self,
        id: OperationId,
        dialogs: &impl DocumentDialogs,
        future: impl std::future::Future<Output = Result<T, DocumentError>>,
    ) -> Result<T, DocumentError> {
        if !dialogs.dismisses_on_drop() {
            return future.await;
        }
        let mut changed = self.document_changed.subscribe();
        self.document_current(id)?;
        tokio::pin!(future);
        loop {
            tokio::select! {
                biased;
                _ = changed.changed() => { self.document_current(id)?; }
                result = &mut future => return result,
            }
        }
    }

    fn document_phase(&self, id: OperationId, phase: DocumentPhase) -> Result<(), DocumentError> {
        let mut workflow = self
            .document
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        let core = self
            .core
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        workflow.transition(id, core.snapshot().note_id, phase)
    }

    fn document_current(&self, id: OperationId) -> Result<DocumentOperation, DocumentError> {
        self.document_operation()?
            .filter(|op| op.operation_id == id)
            .ok_or_else(|| DocumentErrorCode::StaleOperation.error())
    }

    pub async fn run_document(
        &self,
        id: OperationId,
        dialogs: &impl DocumentDialogs,
    ) -> Result<DocumentReply, DocumentError> {
        if self
            .document_running
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(DocumentErrorCode::Busy.error());
        }
        let _running = Running(self.document_running.clone());
        let action = self.document_current(id)?.action;
        let result = self.drive_document(id, dialogs).await;
        if let Err(error) = &result {
            if self.document_current(id).is_ok() {
                self.fail_document(id, error.clone())?;
            } else {
                let reply = self.document_reply()?;
                if matches!(reply.outcome, Some(DocumentOutcome::Cancelled { operation_id }) if operation_id == id)
                {
                    return Ok(reply);
                }
            }
        }
        result?;
        if action != DocumentKind::ExportMarkdown
            && matches!(
                self.document_reply()?.outcome,
                Some(DocumentOutcome::Completed { .. })
            )
        {
            let path = {
                self.core
                    .lock()
                    .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?
                    .document_disk()
                    .map(|(path, _)| path)
            };
            if let Some(path) = path {
                let controller = self.clone();
                io(move || {
                    controller.remember_document_path(&path);
                    Ok(())
                })
                .await?;
            }
        }
        self.document_reply()
    }

    async fn drive_document(
        &self,
        id: OperationId,
        dialogs: &impl DocumentDialogs,
    ) -> Result<(), DocumentError> {
        let operation = self.document_current(id)?;
        let action = operation.action;
        let mut opened: Option<(MarkdownFile, String, DraftingIntention)> = None;
        if action.replaces_document() {
            if self.drain_document(id).await? == DocumentDrain::Cancelled {
                return Ok(());
            }
            if action == DocumentKind::Open {
                self.document_phase(id, DocumentPhase::ChoosingFile)?;
                let Some(path) = self
                    .wait_dialog(
                        id,
                        dialogs,
                        dialogs.select_file(FileDialogRequest::OpenNote),
                    )
                    .await?
                else {
                    self.cancel_document(id)?;
                    return Ok(());
                };
                self.document_current(id)?;
                opened = Some(io(move || read_opened(path)).await?);
            }
            self.document_current(id)?;
            let snapshot = self.snapshot()?;
            if snapshot.dirty {
                self.document_phase(id, DocumentPhase::ConfirmingSave)?;
                let decision = self
                    .wait_dialog(id, dialogs, dialogs.confirm_save(snapshot.document_name))
                    .await?;
                self.document_current(id)?;
                match decision {
                    SaveDecision::Cancel => {
                        self.cancel_document(id)?;
                        return Ok(());
                    }
                    SaveDecision::Save => {
                        if !self.write_document(id, false, false, dialogs).await? {
                            return Ok(());
                        }
                        if let Some((storage, _, _)) = &opened {
                            // Saving may have replaced the very file selected for opening.
                            let path = storage.path().to_path_buf();
                            opened = Some(io(move || read_opened(path)).await?);
                        }
                    }
                    SaveDecision::Discard => {}
                }
            }
        } else if !self
            .write_document(
                id,
                action == DocumentKind::SaveAs,
                action == DocumentKind::ExportMarkdown,
                dialogs,
            )
            .await?
        {
            return Ok(());
        }

        let mut workflow = self
            .document
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        let mut core = self
            .core
            .lock()
            .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
        let note_id = core.snapshot().note_id;
        if workflow
            .active()
            .is_some_and(|op| op.operation_id == id && op.phase != DocumentPhase::Committing)
        {
            workflow.transition(id, note_id, DocumentPhase::Committing)?;
        }
        // Complete against the old identity before publishing the replacement.
        workflow.complete(id, note_id)?;
        match action {
            DocumentKind::New => {
                core.invalidate_for_open();
                core.loaded(String::new());
            }
            DocumentKind::Open => {
                let (storage, source, intention) =
                    opened.ok_or_else(|| DocumentErrorCode::InvalidDocument.error())?;
                core.adopt_opened(storage, source, intention);
            }
            _ => {}
        }
        Ok(())
    }

    async fn write_document(
        &self,
        id: OperationId,
        save_as: bool,
        export: bool,
        dialogs: &impl DocumentDialogs,
    ) -> Result<bool, DocumentError> {
        // Capture the accepted state once. Later dictation does not become saved by adoption.
        let (snapshot, intention, target, document_disk) = {
            let workflow = self
                .document
                .lock()
                .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
            if !workflow.active().is_some_and(|op| op.operation_id == id) {
                return Err(DocumentErrorCode::StaleOperation.error());
            }
            let core = self
                .core
                .lock()
                .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
            (
                core.snapshot(),
                core.drafting_intention().clone(),
                core.save_target(),
                core.document_disk(),
            )
        };
        let mut known = if save_as || export { None } else { target };
        loop {
            let (path, expected, chosen) = if let Some((path, version)) = known.take() {
                (path, Some(version), false)
            } else {
                self.document_phase(id, DocumentPhase::ChoosingFile)?;
                let proposed_name = PathBuf::from(&snapshot.document_name)
                    .with_extension(if export { "md" } else { "von" })
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                let request = if export {
                    FileDialogRequest::ExportMarkdown { proposed_name }
                } else {
                    FileDialogRequest::SaveNote { proposed_name }
                };
                let Some(path) = self
                    .wait_dialog(id, dialogs, dialogs.select_file(request))
                    .await?
                else {
                    self.cancel_document(id)?;
                    return Ok(false);
                };
                self.document_current(id)?;
                let path = crate::storage::markdown_file::complete_note_path(
                    &path,
                    if export {
                        DocumentFormat::Markdown
                    } else {
                        DocumentFormat::Von
                    },
                )
                .map_err(|error| match error {
                    crate::storage::markdown_file::NotePathError::InvalidPath => {
                        DocumentErrorCode::InvalidPath.error()
                    }
                    crate::storage::markdown_file::NotePathError::UnsupportedFormat => {
                        DocumentErrorCode::UnsupportedFormat.error()
                    }
                })?;
                (path, None, true)
            };
            let observed = io({
                let path = path.clone();
                move || DiskVersion::observe_optional(&path).map_err(|e| e.code.error())
            })
            .await?;
            self.document_current(id)?;
            if export
                && document_disk
                    .as_ref()
                    .is_some_and(|(current_path, version)| {
                        current_path == &path
                            || observed
                                .as_ref()
                                .is_some_and(|actual| version.same_file(actual))
                    })
            {
                return Err(DocumentErrorCode::InvalidPath.error());
            }
            let format = if export {
                DocumentFormat::Markdown
            } else {
                DocumentFormat::from_path(&path)
                    .ok_or_else(|| DocumentErrorCode::UnsupportedFormat.error())?
            };
            let content = if export {
                WriteContent::MarkdownExport(snapshot.source.clone())
            } else {
                WriteContent::Document {
                    source: snapshot.source.clone(),
                    intention: intention.clone(),
                }
            };
            let mut candidate = WriteCandidate::new(
                content,
                path.clone(),
                format,
                DocumentVersion::from(&snapshot),
                expected.clone(),
            )
            .map_err(|e| e.code.error())?;
            if let Some(version) = observed
                .as_ref()
                .filter(|version| chosen || Some(*version) != expected.as_ref())
            {
                self.document_phase(id, DocumentPhase::ConfirmingOverwrite)?;
                let decision = self
                    .wait_dialog(
                        id,
                        dialogs,
                        dialogs.confirm_overwrite(path, version.clone()),
                    )
                    .await?;
                self.document_current(id)?;
                match decision {
                    OverwriteDecision::Replace(consent) => {
                        candidate = candidate
                            .with_overwrite_consent(consent)
                            .map_err(|e| e.code.error())?
                    }
                    OverwriteDecision::ChooseAnother => continue,
                    OverwriteDecision::Cancel => {
                        self.cancel_document(id)?;
                        return Ok(false);
                    }
                }
            } else if !chosen && observed != expected {
                return Err(DocumentErrorCode::ExternalConflict.error());
            }
            self.document_phase(id, DocumentPhase::Writing)?;
            let result = io(move || {
                Ok(match candidate.prepare() {
                    Ok(prepared) => prepared.install(),
                    Err(error) => WriteResult::Failed {
                        error,
                        observed: None,
                    },
                })
            })
            .await?;
            match result {
                WriteResult::Saved { candidate, version } => {
                    let mut workflow = self
                        .document
                        .lock()
                        .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
                    let mut core = self
                        .core
                        .lock()
                        .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?;
                    workflow.transition(id, core.snapshot().note_id, DocumentPhase::Committing)?;
                    if !export {
                        core.adopt_saved(&candidate, version)?;
                    }
                    return Ok(true);
                }
                WriteResult::Conflict { recovery_paths, .. } => {
                    self.core.lock().map_err(|_| DocumentErrorCode::CoreUnavailable.error())?.storage_error(
                        crate::storage::markdown_file::SaveError {
                            code: DocumentErrorCode::ExternalConflict,
                            message: "Conflit disque ; versions préservées, vérifiez les fichiers de récupération.",
                            recovery_paths,
                        });
                    return Err(DocumentErrorCode::ExternalConflict.error());
                }
                WriteResult::Failed { error, .. } => {
                    let code = error.code;
                    self.core
                        .lock()
                        .map_err(|_| DocumentErrorCode::CoreUnavailable.error())?
                        .storage_error(error);
                    return Err(code.error());
                }
            }
        }
    }
}

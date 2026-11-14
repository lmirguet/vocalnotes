use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct OperationId(u64);

impl TryFrom<String> for OperationId {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let id = value
            .parse::<u64>()
            .map_err(|_| "Identifiant d'opération invalide.")?;
        if id == 0 || id.to_string() != value {
            return Err("Identifiant d'opération invalide.");
        }
        Ok(Self(id))
    }
}

impl From<OperationId> for String {
    fn from(value: OperationId) -> Self {
        value.0.to_string()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentKind {
    New,
    Open,
    Save,
    SaveAs,
    ExportMarkdown,
    Quit,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HistoryDirection {
    Undo,
    Redo,
}

#[derive(Serialize)]
pub struct DocumentReply {
    pub snapshot: crate::domain::note::Snapshot,
    pub outcome: Option<DocumentOutcome>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentFormat {
    Von,
    Markdown,
}

impl DocumentFormat {
    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            extension if extension.eq_ignore_ascii_case("von") => Some(Self::Von),
            extension if extension.eq_ignore_ascii_case("md") => Some(Self::Markdown),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryAvailability {
    pub can_undo: bool,
    pub can_redo: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct DocumentHistory {
    pub content: HistoryAvailability,
    pub context: HistoryAvailability,
    pub instructions: HistoryAvailability,
}

impl DocumentKind {
    pub fn replaces_document(self) -> bool {
        matches!(self, Self::New | Self::Open | Self::Quit)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentPhase {
    AwaitingUi,
    Draining,
    ChoosingFile,
    ConfirmingSave,
    ConfirmingOverwrite,
    Writing,
    Committing,
    Idle,
}

impl DocumentPhase {
    pub fn cancellable(self) -> bool {
        !matches!(self, Self::Writing | Self::Committing | Self::Idle)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentVersion {
    pub note_id: u64,
    pub revision: u64,
    pub input_generation: u64,
}

impl From<&crate::domain::note::Snapshot> for DocumentVersion {
    fn from(snapshot: &crate::domain::note::Snapshot) -> Self {
        Self {
            note_id: snapshot.note_id,
            revision: snapshot.revision,
            input_generation: snapshot.input_generation,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentErrorCode {
    Forbidden,
    Busy,
    StaleOperation,
    UnsynchronizedUi,
    StaleRevision,
    InvalidPath,
    UnsupportedFormat,
    UnsupportedStorage,
    InvalidDocument,
    SizeLimit,
    ExternalConflict,
    WriteFailed,
    DrainFailed,
    WindowFailed,
    DialogFailed,
    PreferencesInvalid,
    PreferencesWriteFailed,
    CoreUnavailable,
    HistoryUnavailable,
}

impl DocumentErrorCode {
    pub fn error(self) -> DocumentError {
        let message = match self {
            Self::Forbidden => "Cette fenêtre ne peut pas exécuter cette action.",
            Self::Busy => "Une opération documentaire est déjà en cours.",
            Self::StaleOperation => "Cette étape ne correspond plus à l'opération en cours.",
            Self::UnsynchronizedUi => "Les saisies doivent être réconciliées avant de poursuivre.",
            Self::StaleRevision => "La note a changé ; l'opération est abandonnée.",
            Self::InvalidPath => "Choisissez un chemin absolu vers un fichier local.",
            Self::UnsupportedFormat => "L'extension ne correspond pas au format demandé.",
            Self::UnsupportedStorage => "Cette destination ne permet pas une écriture contrôlée.",
            Self::InvalidDocument => "La note ne peut pas être lue dans un format pris en charge.",
            Self::SizeLimit => "Une limite de capacité est atteinte ; le travail est conservé.",
            Self::ExternalConflict => {
                "Le fichier a changé ailleurs ; vérifiez les versions conservées."
            }
            Self::WriteFailed => "L'enregistrement a échoué ; la note reste ouverte.",
            Self::DrainFailed => "Un traitement en cours a échoué ; la note reste ouverte.",
            Self::WindowFailed => "La fenêtre demandée ne peut pas être affichée.",
            Self::DialogFailed => "Le dialogue n'a pas fourni de réponse exploitable.",
            Self::PreferencesInvalid => "Les modèles sélectionnés ne sont pas valides.",
            Self::PreferencesWriteFailed => "Les préférences n'ont pas pu être enregistrées.",
            Self::CoreUnavailable => "L'état du document est indisponible.",
            Self::HistoryUnavailable => {
                "Aucune modification ne peut être annulée ou rétablie dans cette zone."
            }
        };
        DocumentError {
            code: self,
            message,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DocumentError {
    pub code: DocumentErrorCode,
    pub message: &'static str,
}

impl std::fmt::Display for DocumentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message)
    }
}

impl std::error::Error for DocumentError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentOperation {
    pub operation_id: OperationId,
    pub action: DocumentKind,
    pub phase: DocumentPhase,
    pub note_id: u64,
    pub cancellable: bool,
    #[serde(skip)]
    ui_acknowledgement: Option<DocumentVersion>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(
    tag = "status",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum DocumentOutcome {
    Completed {
        operation_id: OperationId,
    },
    Cancelled {
        operation_id: OperationId,
    },
    Failed {
        operation_id: OperationId,
        error: DocumentError,
    },
}

#[derive(Default)]
pub struct DocumentWorkflow {
    last_id: u64,
    active: Option<DocumentOperation>,
    last_outcome: Option<DocumentOutcome>,
    focused_area: crate::domain::DictationTarget,
    last_action: Option<DocumentKind>,
    closing: bool,
}

impl DocumentWorkflow {
    pub fn is_closing(&self) -> bool {
        self.closing
    }
    pub fn focus(&mut self, area: crate::domain::DictationTarget) {
        self.focused_area = area;
    }

    pub fn focused_area(&self) -> crate::domain::DictationTarget {
        self.focused_area
    }

    pub fn active(&self) -> Option<&DocumentOperation> {
        self.active.as_ref()
    }

    pub fn last_outcome(&self) -> Option<&DocumentOutcome> {
        self.last_outcome.as_ref()
    }

    pub fn fill_snapshot(&self, snapshot: &mut crate::domain::note::Snapshot) {
        snapshot.document_operation = self
            .active
            .as_ref()
            .filter(|operation| operation.note_id == snapshot.note_id)
            .cloned();
        snapshot.document_error = match &self.last_outcome {
            Some(DocumentOutcome::Failed { error, .. }) => Some(error.clone()),
            _ => None,
        };
    }

    pub fn request(
        &mut self,
        action: DocumentKind,
        version: DocumentVersion,
    ) -> Result<DocumentOperation, DocumentError> {
        if self.closing {
            return Err(DocumentErrorCode::Busy.error());
        }
        if let Some(operation) = &self.active {
            let id = operation.operation_id;
            self.check(id, Some(version.note_id))?;
            return Ok(self.active.as_ref().unwrap().clone());
        }
        let next = self
            .last_id
            .checked_add(1)
            .ok_or_else(|| DocumentErrorCode::SizeLimit.error())?;
        let operation = DocumentOperation {
            operation_id: OperationId(next),
            action,
            phase: DocumentPhase::AwaitingUi,
            note_id: version.note_id,
            cancellable: true,
            ui_acknowledgement: None,
        };
        self.last_id = next;
        self.last_outcome = None;
        self.active = Some(operation.clone());
        Ok(operation)
    }

    pub fn ready(
        &mut self,
        id: OperationId,
        acknowledgement: DocumentVersion,
        current: DocumentVersion,
    ) -> Result<(), DocumentError> {
        self.check(id, Some(current.note_id))?;
        let operation = self.active.as_mut().unwrap();
        if operation.phase != DocumentPhase::AwaitingUi || operation.ui_acknowledgement.is_some() {
            return Err(DocumentErrorCode::StaleOperation.error());
        }
        if acknowledgement != current {
            return Err(DocumentErrorCode::UnsynchronizedUi.error());
        }
        operation.ui_acknowledgement = Some(acknowledgement);
        Ok(())
    }

    pub fn transition(
        &mut self,
        id: OperationId,
        note_id: u64,
        phase: DocumentPhase,
    ) -> Result<(), DocumentError> {
        self.check(id, Some(note_id))?;
        let operation = self.active.as_mut().unwrap();
        if operation.phase == DocumentPhase::AwaitingUi && operation.ui_acknowledgement.is_none() {
            return Err(DocumentErrorCode::UnsynchronizedUi.error());
        }
        use DocumentPhase::*;
        let allowed = match (operation.phase, phase) {
            (AwaitingUi, Draining) => operation.action.replaces_document(),
            (AwaitingUi, ChoosingFile) => !operation.action.replaces_document(),
            (AwaitingUi, ConfirmingOverwrite | Writing) => operation.action == DocumentKind::Save,
            (ChoosingFile, ConfirmingSave | Committing) => operation.action == DocumentKind::Open,
            (Draining, ChoosingFile | ConfirmingSave | Committing)
            | (ChoosingFile, ConfirmingOverwrite | Writing)
            | (ConfirmingSave, ChoosingFile | ConfirmingOverwrite | Writing | Committing)
            | (ConfirmingOverwrite, Writing | ChoosingFile)
            | (Writing, Committing) => true,
            _ => false,
        };
        if !allowed {
            return Err(DocumentErrorCode::StaleOperation.error());
        }
        operation.phase = phase;
        operation.cancellable = phase.cancellable();
        Ok(())
    }

    pub fn cancel(&mut self, id: OperationId) -> Result<DocumentOutcome, DocumentError> {
        self.check(id, None)?;
        if !self.active.as_ref().unwrap().cancellable {
            return Err(DocumentErrorCode::Busy.error());
        }
        Ok(self.finish(DocumentOutcome::Cancelled { operation_id: id }))
    }

    pub fn complete(
        &mut self,
        id: OperationId,
        note_id: u64,
    ) -> Result<DocumentOutcome, DocumentError> {
        self.check(id, Some(note_id))?;
        if self.active.as_ref().unwrap().phase != DocumentPhase::Committing {
            return Err(DocumentErrorCode::StaleOperation.error());
        }
        Ok(self.finish(DocumentOutcome::Completed { operation_id: id }))
    }

    pub fn fail(
        &mut self,
        id: OperationId,
        error: DocumentError,
    ) -> Result<DocumentOutcome, DocumentError> {
        self.check(id, None)?;
        Ok(self.finish(DocumentOutcome::Failed {
            operation_id: id,
            error,
        }))
    }

    fn check(&mut self, id: OperationId, note_id: Option<u64>) -> Result<(), DocumentError> {
        let operation = self
            .active
            .as_ref()
            .filter(|operation| operation.operation_id == id)
            .ok_or_else(|| DocumentErrorCode::StaleOperation.error())?;
        if note_id.is_some_and(|note_id| note_id != operation.note_id) {
            let error = DocumentErrorCode::StaleRevision.error();
            self.finish(DocumentOutcome::Failed {
                operation_id: id,
                error: error.clone(),
            });
            return Err(error);
        }
        Ok(())
    }

    fn finish(&mut self, outcome: DocumentOutcome) -> DocumentOutcome {
        self.last_action = self.active.as_ref().map(|op| op.action);
        self.active = None;
        self.last_outcome = Some(outcome.clone());
        outcome
    }

    pub fn authorize_close(&mut self, id: OperationId) -> Result<(), DocumentError> {
        if self.closing
            || self.active.is_some()
            || self.last_action != Some(DocumentKind::Quit)
            || !matches!(self.last_outcome, Some(DocumentOutcome::Completed { operation_id }) if operation_id == id)
        {
            return Err(DocumentErrorCode::StaleOperation.error());
        }
        self.closing = true;
        Ok(())
    }

    pub fn close_failed(&mut self, id: OperationId) {
        self.closing = false;
        self.last_outcome = Some(DocumentOutcome::Failed {
            operation_id: id,
            error: DocumentErrorCode::WindowFailed.error(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version() -> DocumentVersion {
        DocumentVersion {
            note_id: 1,
            revision: 3,
            input_generation: 5,
        }
    }

    fn prepared(kind: DocumentKind) -> (DocumentWorkflow, OperationId) {
        let mut workflow = DocumentWorkflow::default();
        let id = workflow.request(kind, version()).unwrap().operation_id;
        workflow.ready(id, version(), version()).unwrap();
        (workflow, id)
    }

    #[test]
    fn concurrent_requests_join_without_changing_the_original_action() {
        let mut workflow = DocumentWorkflow::default();
        let first = workflow.request(DocumentKind::Quit, version()).unwrap();
        let repeated = workflow.request(DocumentKind::New, version()).unwrap();
        assert_eq!(first, repeated);
        assert_eq!(workflow.active(), Some(&first));
    }

    #[test]
    fn stale_acknowledgement_does_not_authorize_a_transition() {
        let mut workflow = DocumentWorkflow::default();
        let id = workflow
            .request(DocumentKind::Quit, version())
            .unwrap()
            .operation_id;
        let current = DocumentVersion {
            revision: 4,
            ..version()
        };
        assert_eq!(
            workflow.ready(id, version(), current).unwrap_err().code,
            DocumentErrorCode::UnsynchronizedUi
        );
        assert_eq!(
            workflow
                .transition(id, 1, DocumentPhase::Draining)
                .unwrap_err()
                .code,
            DocumentErrorCode::UnsynchronizedUi
        );
        workflow.ready(id, current, current).unwrap();
        workflow.transition(id, 1, DocumentPhase::Draining).unwrap();
    }

    #[test]
    fn cancelled_operation_cannot_cancel_or_acknowledge_its_successor() {
        let mut workflow = DocumentWorkflow::default();
        let old = workflow
            .request(DocumentKind::Quit, version())
            .unwrap()
            .operation_id;
        assert!(matches!(
            workflow.cancel(old).unwrap(),
            DocumentOutcome::Cancelled { .. }
        ));
        let next = workflow.request(DocumentKind::Save, version()).unwrap();
        assert_ne!(next.operation_id, old);
        assert_eq!(
            workflow.cancel(old).unwrap_err().code,
            DocumentErrorCode::StaleOperation
        );
        assert_eq!(
            workflow.ready(old, version(), version()).unwrap_err().code,
            DocumentErrorCode::StaleOperation
        );
        assert_eq!(workflow.active(), Some(&next));
    }

    #[test]
    fn changing_the_document_invalidates_the_operation() {
        let (mut workflow, id) = prepared(DocumentKind::Open);
        assert_eq!(
            workflow
                .transition(id, 2, DocumentPhase::Draining)
                .unwrap_err()
                .code,
            DocumentErrorCode::StaleRevision
        );
        assert!(workflow.active().is_none());
        assert!(matches!(
            workflow.last_outcome(),
            Some(DocumentOutcome::Failed { error, .. })
                if error.code == DocumentErrorCode::StaleRevision
        ));
    }

    #[test]
    fn destructive_action_cannot_skip_draining() {
        let (mut workflow, id) = prepared(DocumentKind::Quit);
        assert_eq!(
            workflow
                .transition(id, 1, DocumentPhase::Committing)
                .unwrap_err()
                .code,
            DocumentErrorCode::StaleOperation
        );
        workflow.transition(id, 1, DocumentPhase::Draining).unwrap();
        workflow
            .transition(id, 1, DocumentPhase::ConfirmingSave)
            .unwrap();
        workflow
            .transition(id, 1, DocumentPhase::Committing)
            .unwrap();
        assert!(matches!(
            workflow.complete(id, 1).unwrap(),
            DocumentOutcome::Completed { .. }
        ));
        assert!(workflow.active().is_none());
    }

    #[test]
    fn writing_is_not_retroactively_cancelled_or_completed() {
        let (mut workflow, id) = prepared(DocumentKind::Save);
        workflow.transition(id, 1, DocumentPhase::Writing).unwrap();
        assert_eq!(
            workflow.cancel(id).unwrap_err().code,
            DocumentErrorCode::Busy
        );
        assert_eq!(
            workflow.complete(id, 1).unwrap_err().code,
            DocumentErrorCode::StaleOperation
        );
        let error = DocumentErrorCode::WriteFailed.error();
        assert!(matches!(
            workflow.fail(id, error).unwrap(),
            DocumentOutcome::Failed { .. }
        ));
        assert!(workflow.active().is_none());
    }

    #[test]
    fn save_as_and_export_cannot_skip_file_selection_or_writing() {
        for kind in [DocumentKind::SaveAs, DocumentKind::ExportMarkdown] {
            let (mut workflow, id) = prepared(kind);
            assert!(workflow.transition(id, 1, DocumentPhase::Writing).is_err());
            workflow
                .transition(id, 1, DocumentPhase::ChoosingFile)
                .unwrap();
            assert!(
                workflow
                    .transition(id, 1, DocumentPhase::Committing)
                    .is_err()
            );
            workflow.transition(id, 1, DocumentPhase::Writing).unwrap();
            workflow
                .transition(id, 1, DocumentPhase::Committing)
                .unwrap();
            assert!(matches!(
                workflow.complete(id, 1).unwrap(),
                DocumentOutcome::Completed { .. }
            ));
        }
    }

    #[test]
    fn acknowledgement_cannot_be_replayed_after_preparation() {
        let (mut workflow, id) = prepared(DocumentKind::SaveAs);
        workflow
            .transition(id, 1, DocumentPhase::ChoosingFile)
            .unwrap();
        assert_eq!(
            workflow.ready(id, version(), version()).unwrap_err().code,
            DocumentErrorCode::StaleOperation
        );
        assert_eq!(
            workflow.active().unwrap().phase,
            DocumentPhase::ChoosingFile
        );
    }

    #[test]
    fn operation_ids_remain_exact_beyond_the_javascript_integer_limit() {
        let mut workflow = DocumentWorkflow {
            last_id: 9_007_199_254_740_991,
            ..Default::default()
        };
        let operation = workflow.request(DocumentKind::Save, version()).unwrap();
        let json = serde_json::to_value(&operation).unwrap();
        assert_eq!(json["operationId"], "9007199254740992");
        let decoded: OperationId = serde_json::from_value(json["operationId"].clone()).unwrap();
        assert_eq!(decoded, operation.operation_id);
        for invalid in [r#"0"#, r#""01""#, r#""-1""#, r#""1e2""#, r#""""#] {
            assert!(serde_json::from_str::<OperationId>(invalid).is_err());
        }
    }

    #[test]
    fn exhausted_ids_cannot_wrap_into_an_old_operation() {
        let mut workflow = DocumentWorkflow {
            last_id: u64::MAX,
            ..Default::default()
        };
        assert_eq!(
            workflow
                .request(DocumentKind::Save, version())
                .unwrap_err()
                .code,
            DocumentErrorCode::SizeLimit
        );
        assert!(workflow.active().is_none());
    }
}

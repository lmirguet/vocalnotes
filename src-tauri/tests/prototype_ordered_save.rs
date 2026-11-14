use vocalnotes_lib::platform::probe::*;
use vocalnotes_lib::ports::interpreter::{EditOperation, SessionCommand, Status, Target};

fn actions() -> Vec<Action> {
    vec![
        Action::DraftingIntent {
            text: "courriel formel".into(),
            form: String::new(),
            tone: String::new(),
            context: String::new(),
        },
        Action::Edit {
            operation: EditOperation::Replace {
                target: Target {
                    block_id: "b0".into(),
                    expected: "avant".into(),
                },
                text: "après".into(),
            },
        },
        Action::SessionAction {
            command: SessionCommand::Save {},
        },
    ]
}

#[tokio::test]
async fn ordered_voice_edits_require_an_explicit_native_save_and_cannot_bypass_it() {
    use vocalnotes_lib::{
        app::document::{DocumentKind, DocumentVersion},
        platform::{
            controller::Controller,
            desktop::{
                DialogFuture, DocumentDialogs, FileDialogRequest, OverwriteDecision, SaveDecision,
            },
        },
        storage::markdown_file::DiskVersion,
    };
    struct NoDialog;
    impl DocumentDialogs for NoDialog {
        fn select_file(
            &self,
            _: FileDialogRequest,
        ) -> DialogFuture<'_, Option<std::path::PathBuf>> {
            panic!("existing native destination")
        }
        fn confirm_save(&self, _: String) -> DialogFuture<'_, SaveDecision> {
            panic!("non-destructive save")
        }
        fn confirm_overwrite(
            &self,
            _: std::path::PathBuf,
            _: DiskVersion,
        ) -> DialogFuture<'_, OverwriteDecision> {
            panic!("known version")
        }
    }
    let dir = std::env::temp_dir().join(format!("vocalnotes-order-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let mut core = Probe::new("avant".into());
    let original = dir.join("note.von");
    core.save_copy(&original).unwrap();
    let saved = std::fs::read(&original).unwrap();
    core.start_voice(false).unwrap();
    let ticket = core.voice_ticket(1).unwrap();
    let save_result = core.commit_voice(
        &ticket,
        Intervention {
            status: Status::Resolved,
            message: "".into(),
            actions: actions(),
        },
    );
    assert!(save_result.is_err());
    assert_eq!(core.intention(), "");
    assert_eq!(core.snapshot().source, "avant");
    assert_eq!(std::fs::read(&original).unwrap(), saved);
    let mut accepted = actions();
    accepted.pop();
    let ticket = core.voice_ticket(2).unwrap();
    core.commit_voice(
        &ticket,
        Intervention {
            status: Status::Resolved,
            message: String::new(),
            actions: accepted,
        },
    )
    .unwrap();
    assert_eq!(core.snapshot().source, "après");
    assert_eq!(core.intention(), "courriel formel");
    assert!(core.snapshot().dirty);
    assert_eq!(std::fs::read(&original).unwrap(), saved);
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
    let controller = Controller::new(core);
    let operation = controller.request_document(DocumentKind::Save).unwrap();
    controller
        .document_ready(
            operation.operation_id,
            DocumentVersion::from(&controller.snapshot().unwrap()),
        )
        .unwrap();
    controller
        .run_document(operation.operation_id, &NoDialog)
        .await
        .unwrap();
    assert!(!controller.snapshot().unwrap().dirty);
    assert_eq!(
        controller
            .core
            .lock()
            .unwrap()
            .saved_destination()
            .as_deref(),
        Some(original.as_path())
    );
    let document = std::fs::read_to_string(&original).unwrap();
    let (body, intention) = vocalnotes_lib::storage::note_document::decode(&document).unwrap();
    assert_eq!(body, "après");
    assert_eq!(intention.purpose(), "courriel formel");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn missing_destination_or_ambiguity_has_zero_effect_and_no_save_replay() {
    let mut core = Probe::new("avant".into());
    core.start_voice(false).unwrap();
    let ticket = core.voice_ticket(1).unwrap();
    assert!(
        core.commit_voice(
            &ticket,
            Intervention {
                status: Status::Resolved,
                message: "".into(),
                actions: actions()
            }
        )
        .is_err()
    );
    assert_eq!(core.snapshot().source, "avant");
    assert_eq!(core.intention(), "");
    let next = core.voice_ticket(2).unwrap();
    core.commit_voice(
        &next,
        Intervention {
            status: Status::Clarify,
            message: "Quelle destination ?".into(),
            actions: vec![],
        },
    )
    .unwrap();
    assert_eq!(core.snapshot().source, "avant");
    let next = core.voice_ticket(3).unwrap();
    core.commit_voice(
        &next,
        Intervention {
            status: Status::Resolved,
            message: "".into(),
            actions: vec![],
        },
    )
    .unwrap();
    assert_eq!(core.snapshot().source, "avant");
    assert_eq!(core.intention(), "");
}

use std::{path::PathBuf, sync::Mutex};
use vocalnotes_lib::{
    app::document::{DocumentKind, DocumentVersion},
    domain::note::Note,
    platform::{
        controller::Controller,
        desktop::{
            DialogFuture, DocumentDialogs, FileDialogRequest, OverwriteDecision, SaveDecision,
        },
    },
    storage::markdown_file::{DiskVersion, OverwriteConsent},
};

struct Dialogs {
    paths: Mutex<Vec<PathBuf>>,
    decision: SaveDecision,
}
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "vocalnotes-workflow-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self) -> &std::path::Path {
        &self.0
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
impl DocumentDialogs for Dialogs {
    fn select_file(&self, _: FileDialogRequest) -> DialogFuture<'_, Option<PathBuf>> {
        Box::pin(async { Ok(self.paths.lock().unwrap().pop()) })
    }
    fn confirm_save(&self, _: String) -> DialogFuture<'_, SaveDecision> {
        Box::pin(async { Ok(self.decision) })
    }
    fn confirm_overwrite(
        &self,
        path: PathBuf,
        version: DiskVersion,
    ) -> DialogFuture<'_, OverwriteDecision> {
        Box::pin(async move {
            Ok(OverwriteDecision::Replace(
                OverwriteConsent::new(path, version).unwrap(),
            ))
        })
    }
}
async fn run(controller: &Controller, kind: DocumentKind, dialogs: &Dialogs) {
    let op = controller.request_document(kind).unwrap();
    controller
        .document_ready(
            op.operation_id,
            DocumentVersion::from(&controller.snapshot().unwrap()),
        )
        .unwrap();
    controller
        .run_document(op.operation_id, dialogs)
        .await
        .unwrap();
}

#[tokio::test]
async fn opening_current_file_after_save_loads_the_new_saved_candidate() {
    let dir = Fixture::new();
    let path = dir.path().join("current.von");
    std::fs::write(&path, "Original\r\n").unwrap();
    let controller = Controller::new(Note::new(String::new()));
    let dialogs = Dialogs {
        paths: Mutex::new(vec![path.clone()]),
        decision: SaveDecision::Save,
    };
    run(&controller, DocumentKind::Open, &dialogs).await;
    controller
        .core
        .lock()
        .unwrap()
        .route_dictation_text(
            vocalnotes_lib::domain::DictationTarget::Content,
            "Modifié".into(),
        )
        .unwrap();
    dialogs.paths.lock().unwrap().push(path.clone());
    run(&controller, DocumentKind::Open, &dialogs).await;
    let snapshot = controller.snapshot().unwrap();
    assert_eq!(snapshot.source, "Original\r\n\nModifié");
    assert!(snapshot.context.is_empty());
    assert!(!snapshot.dirty);
    assert_eq!(
        controller.core.lock().unwrap().document_disk().unwrap().1,
        DiskVersion::observe(&path).unwrap()
    );
}

#[tokio::test]
async fn twenty_real_save_open_export_cycles_preserve_exact_bytes_and_dirty_state() {
    let dir = Fixture::new();
    for index in 0..20 {
        let body = match index % 5 {
            0 => String::new(),
            1 => "Contexte --> e\u{301}".into(),
            2 => "<!-- vocalnotes:intention: texte utilisateur littéral\r\n".into(),
            _ => format!("# Été {index} 👋\r\n\r\nopaque  \n"),
        };
        let controller = Controller::new(Note::new(body));
        controller
            .core
            .lock()
            .unwrap()
            .set_context("Contexte --> e\u{301}".into())
            .unwrap();
        let source = controller.snapshot().unwrap().source;
        let von = dir.path().join(format!("cycle-{index}.VON"));
        let md = dir.path().join(format!("export-{index}.MD"));
        let dialogs = Dialogs {
            paths: Mutex::new(vec![
                von.clone(),
                dir.path().join(format!("unsaved-export-{index}.md")),
            ]),
            decision: SaveDecision::Discard,
        };
        let unsaved = controller.snapshot().unwrap();
        assert!(unsaved.document_path.is_none());
        run(&controller, DocumentKind::ExportMarkdown, &dialogs).await;
        let unadopted = controller.snapshot().unwrap();
        assert_eq!(
            std::fs::read(dir.path().join(format!("unsaved-export-{index}.md"))).unwrap(),
            source.as_bytes()
        );
        assert!(unadopted.document_path.is_none());
        assert!(controller.core.lock().unwrap().document_disk().is_none());
        assert_eq!(unadopted.history, unsaved.history);
        assert_eq!(unadopted.dirty, unsaved.dirty);
        run(&controller, DocumentKind::Save, &dialogs).await;
        assert!(!controller.snapshot().unwrap().dirty);
        controller
            .core
            .lock()
            .unwrap()
            .set_context("Contexte modifié".into())
            .unwrap();
        if index % 5 == 3 {
            controller
                .core
                .lock()
                .unwrap()
                .route_dictation_text(
                    vocalnotes_lib::domain::DictationTarget::Content,
                    "Corps non enregistré".into(),
                )
                .unwrap();
        }
        let before = controller.snapshot().unwrap();
        let disk_before = controller.core.lock().unwrap().document_disk();
        dialogs.paths.lock().unwrap().push(md.clone());
        run(&controller, DocumentKind::ExportMarkdown, &dialogs).await;
        let after = controller.snapshot().unwrap();
        assert_eq!(std::fs::read(md).unwrap(), before.source.as_bytes());
        assert_eq!(before.document_path, after.document_path);
        assert_eq!(before.dirty, after.dirty);
        assert_eq!(before.document_format, after.document_format);
        assert_eq!(before.history, after.history);
        assert_eq!(before.revision, after.revision);
        assert_eq!(before.input_generation, after.input_generation);
        assert_eq!(disk_before, controller.core.lock().unwrap().document_disk());
        dialogs.paths.lock().unwrap().push(von);
        run(&controller, DocumentKind::Open, &dialogs).await;
        let reopened = controller.snapshot().unwrap();
        assert_eq!(reopened.source, source);
        assert_eq!(reopened.context, "Contexte --> e\u{301}");
        assert!(!reopened.dirty);
    }
}

#[tokio::test]
async fn markdown_context_migration_preserves_original_and_cancelled_context_restores_save_target()
{
    let dir = Fixture::new();
    for initial in ["", "Contexte chargé"] {
        let md = dir.path().join(if initial.is_empty() {
            "empty.MD"
        } else {
            "context.md"
        });
        let intention = vocalnotes_lib::domain::intention::DraftingIntention::new(
            "But".into(),
            "".into(),
            "".into(),
            initial.into(),
        )
        .unwrap();
        let bytes = vocalnotes_lib::storage::note_document::encode("Corps\r\n", &intention);
        std::fs::write(&md, bytes.as_bytes()).unwrap();
        let controller = Controller::new(Note::new("Initial".into()));
        let dialogs = Dialogs {
            paths: Mutex::new(vec![md.clone()]),
            decision: SaveDecision::Discard,
        };
        run(&controller, DocumentKind::Open, &dialogs).await;
        {
            let mut note = controller.core.lock().unwrap();
            note.set_context("Autre".into()).unwrap();
            assert!(note.save_target().is_none());
            note.set_context(initial.into()).unwrap();
            assert!(note.save_target().is_some());
            note.route_dictation_text(
                vocalnotes_lib::domain::DictationTarget::Content,
                "Suite".into(),
            )
            .unwrap();
        }
        run(&controller, DocumentKind::Save, &dialogs).await;
        assert_eq!(
            controller.snapshot().unwrap().document_path.as_deref(),
            md.to_str()
        );
        let original = std::fs::read(&md).unwrap();
        controller
            .core
            .lock()
            .unwrap()
            .set_context(if initial.is_empty() {
                "Ajout".into()
            } else {
                "".into()
            })
            .unwrap();
        let von = md.with_extension("von");
        dialogs.paths.lock().unwrap().push(von.clone());
        run(&controller, DocumentKind::Save, &dialogs).await;
        assert_eq!(
            controller.snapshot().unwrap().document_path.as_deref(),
            von.to_str()
        );
        assert_eq!(std::fs::read(&md).unwrap(), original);
    }
}

#[tokio::test]
async fn destructive_actions_three_decisions_preserve_work_and_save_before_replacement() {
    use vocalnotes_lib::app::document::DocumentOutcome;
    let dir = Fixture::new();
    for (index, action) in [
        DocumentKind::New,
        DocumentKind::Open,
        DocumentKind::Quit,
        DocumentKind::Quit,
    ]
    .into_iter()
    .enumerate()
    {
        for (choice, decision) in [
            SaveDecision::Save,
            SaveDecision::Discard,
            SaveDecision::Cancel,
        ]
        .into_iter()
        .enumerate()
        {
            let controller = Controller::new(Note::new("Mémoire à conserver".into()));
            controller
                .core
                .lock()
                .unwrap()
                .set_context("Contexte seul rend dirty".into())
                .unwrap();
            let before = controller.snapshot().unwrap();
            let saved = dir.path().join(format!("saved-{index}-{choice}.von"));
            let incoming = dir.path().join(format!("incoming-{index}-{choice}.von"));
            std::fs::write(&incoming, "Autre note").unwrap();
            let mut paths = vec![saved.clone()];
            if action == DocumentKind::Open {
                paths.push(incoming);
            }
            let dialogs = Dialogs {
                paths: Mutex::new(paths),
                decision,
            };
            run(&controller, action, &dialogs).await;
            let reply = controller.document_reply().unwrap();
            if decision == SaveDecision::Cancel {
                assert!(matches!(
                    reply.outcome,
                    Some(DocumentOutcome::Cancelled { .. })
                ));
                assert_eq!(reply.snapshot.source, before.source);
                assert_eq!(reply.snapshot.context, before.context);
                assert_eq!(reply.snapshot.note_id, before.note_id);
            } else {
                assert!(matches!(
                    reply.outcome,
                    Some(DocumentOutcome::Completed { .. })
                ));
                if decision == SaveDecision::Save {
                    let (_, raw) =
                        vocalnotes_lib::storage::markdown_file::MarkdownFile::open(&saved).unwrap();
                    let (source, intention) =
                        vocalnotes_lib::storage::note_document::decode(&raw).unwrap();
                    assert_eq!(source, before.source);
                    assert_eq!(intention.context(), before.context);
                } else {
                    assert!(!saved.exists());
                }
                if action == DocumentKind::New {
                    assert!(reply.snapshot.source.is_empty());
                }
                if action == DocumentKind::Open {
                    assert_eq!(reply.snapshot.source, "Autre note");
                }
            }
            assert_eq!(reply.snapshot.mode, "manual");
            assert!(reply.snapshot.document_operation.is_none());
        }
    }
}

#[tokio::test]
async fn invalid_open_failed_save_and_picker_cancel_never_replace_the_note() {
    let dir = Fixture::new();
    let invalid = dir.path().join("invalid.von");
    std::fs::write(&invalid, "<!-- vocalnotes:intention:v999\ninvalide").unwrap();
    let controller = Controller::new(Note::new("Mémoire".into()));
    controller
        .core
        .lock()
        .unwrap()
        .set_context("Dirty".into())
        .unwrap();
    let dialogs = Dialogs {
        paths: Mutex::new(vec![invalid]),
        decision: SaveDecision::Discard,
    };
    let before = controller.snapshot().unwrap();
    let op = controller.request_document(DocumentKind::Open).unwrap();
    controller
        .document_ready(op.operation_id, DocumentVersion::from(&before))
        .unwrap();
    assert!(
        controller
            .run_document(op.operation_id, &dialogs)
            .await
            .is_err()
    );
    assert_eq!(controller.snapshot().unwrap().note_id, before.note_id);
    assert_eq!(controller.snapshot().unwrap().source, before.source);
    assert!(controller.snapshot().unwrap().dirty);
    let bad_path = dir.path().join("missing-parent").join("note.von");
    dialogs.paths.lock().unwrap().push(bad_path);
    let op = controller.request_document(DocumentKind::Save).unwrap();
    controller
        .document_ready(
            op.operation_id,
            DocumentVersion::from(&controller.snapshot().unwrap()),
        )
        .unwrap();
    assert!(
        controller
            .run_document(op.operation_id, &dialogs)
            .await
            .is_err()
    );
    assert_eq!(controller.snapshot().unwrap().document_path, None);
    run(&controller, DocumentKind::SaveAs, &dialogs).await;
    assert!(controller.snapshot().unwrap().dirty);
    assert_eq!(controller.snapshot().unwrap().source, before.source);
}

#[test]
fn twenty_modal_lifecycles_reject_note_mutations_and_ignore_stale_cleanup() {
    use vocalnotes_lib::domain::DictationTarget;
    for _ in 0..20 {
        let controller = Controller::new(Note::new("Mémoire".into()));
        let version = DocumentVersion::from(&controller.snapshot().unwrap());
        let first = controller.begin_preferences().unwrap().unwrap();
        assert_eq!(controller.begin_preferences().unwrap(), None);
        let writing = controller.reserve_preferences_write().unwrap();
        assert!(controller.preferences_write_pending(first));
        assert!(controller.reserve_preferences_write().is_err());
        assert!(controller.begin_preferences_close(first).is_err());
        drop(writing);
        assert!(!controller.preferences_write_pending(first));
        assert!(controller.request_document(DocumentKind::New).is_err());
        assert!(controller.user_stop(false, "Interdit").is_err());
        assert!(
            controller
                .edit_core(|note| note.set_context("Interdit".into()))
                .is_err()
        );
        assert!(
            controller
                .history_action("blocked".into(), version, DictationTarget::Content, false)
                .is_err()
        );
        assert!(controller.begin_preferences_close(first).unwrap());
        assert!(controller.reserve_preferences_write().is_err());
        assert!(controller.end_preferences(first));
        let second = controller.begin_preferences().unwrap().unwrap();
        assert_ne!(first, second);
        assert!(!controller.end_preferences(first));
        assert!(!controller.begin_preferences_close(first).unwrap());
        assert!(controller.reserve_preferences_write().is_ok());
        assert!(controller.is_preferences_modal());
        assert!(controller.end_preferences(second));
        assert!(!controller.end_preferences(second));
        assert_eq!(controller.snapshot().unwrap().source, "Mémoire");
        assert!(controller.snapshot().unwrap().context.is_empty());
        let op = controller.request_document(DocumentKind::New).unwrap();
        assert!(controller.begin_preferences().is_err());
        controller.cancel_document(op.operation_id).unwrap();
    }
}
#[test]
fn adopting_a_saved_candidate_does_not_mark_later_edits_saved() {
    use vocalnotes_lib::{
        app::document::DocumentFormat,
        storage::markdown_file::{WriteCandidate, WriteContent, WriteResult},
    };
    let dir = Fixture::new();
    let mut note = Note::new("source".into());
    note.set_context("accepted".into()).unwrap();
    let candidate = WriteCandidate::new(
        WriteContent::Document {
            source: note.snapshot().source,
            intention: note.drafting_intention().clone(),
        },
        dir.path().join("exact.von"),
        DocumentFormat::Von,
        DocumentVersion::from(&note.snapshot()),
        None,
    )
    .unwrap();
    note.set_context("later".into()).unwrap();
    let WriteResult::Saved { candidate, version } = candidate.prepare().unwrap().install() else {
        panic!("save failed")
    };
    note.adopt_saved(&candidate, version).unwrap();
    assert!(note.snapshot().dirty);
    note.set_context("accepted".into()).unwrap();
    assert!(!note.snapshot().dirty);
}

#[tokio::test]
async fn exports_refuse_the_current_markdown_identity_and_replace_only_confirmed_other_versions() {
    let dir = Fixture::new();
    let current = dir.path().join("current.md");
    let other = dir.path().join("other.md");
    std::fs::write(&current, "Corps de travail").unwrap();
    std::fs::write(&other, "Version externe").unwrap();
    let controller = Controller::new(Note::new("Mémoire".into()));
    let dialogs = Dialogs {
        paths: Mutex::new(vec![current.clone()]),
        decision: SaveDecision::Discard,
    };
    run(&controller, DocumentKind::Open, &dialogs).await;
    controller
        .core
        .lock()
        .unwrap()
        .set_context("Contexte non enregistré".into())
        .unwrap();
    let before = controller.snapshot().unwrap();
    for path in [current.clone(), {
        let alias = dir.path().join("hard-link.md");
        std::fs::hard_link(&current, &alias).unwrap();
        alias
    }] {
        dialogs.paths.lock().unwrap().push(path);
        let op = controller
            .request_document(DocumentKind::ExportMarkdown)
            .unwrap();
        controller
            .document_ready(
                op.operation_id,
                DocumentVersion::from(&controller.snapshot().unwrap()),
            )
            .unwrap();
        assert!(
            controller
                .run_document(op.operation_id, &dialogs)
                .await
                .is_err()
        );
        assert_eq!(std::fs::read(&current).unwrap(), b"Corps de travail");
        assert_eq!(
            controller.snapshot().unwrap().document_path,
            before.document_path
        );
        assert_eq!(controller.snapshot().unwrap().history, before.history);
        assert!(controller.snapshot().unwrap().dirty);
    }
    // The existing target requires and receives a version-bound consent.
    dialogs.paths.lock().unwrap().push(other.clone());
    run(&controller, DocumentKind::ExportMarkdown, &dialogs).await;
    assert_eq!(std::fs::read(other).unwrap(), before.source.as_bytes());
    assert_eq!(
        controller.snapshot().unwrap().document_path,
        before.document_path
    );
    assert!(controller.snapshot().unwrap().dirty);
}

#[tokio::test]
async fn close_requires_a_completed_quit_and_destruction_runs_outside_locks() {
    use vocalnotes_lib::app::close::finish_document;
    let controller = Controller::new(Note::new(String::new()));
    let dialogs = Dialogs {
        paths: Mutex::new(vec![]),
        decision: SaveDecision::Cancel,
    };

    let op = controller.request_document(DocumentKind::Quit).unwrap();
    assert!(finish_document(&controller, op.operation_id, || panic!("not authorized")).is_err());
    controller
        .document_ready(
            op.operation_id,
            DocumentVersion::from(&controller.snapshot().unwrap()),
        )
        .unwrap();
    controller
        .run_document(op.operation_id, &dialogs)
        .await
        .unwrap();
    assert!(
        finish_document(&controller, op.operation_id, || {
            assert!(controller.core.try_lock().is_ok());
            assert!(controller.request_document(DocumentKind::New).is_err());
            Err(())
        })
        .is_err()
    );
    let retry = controller.request_document(DocumentKind::Quit).unwrap();
    controller
        .document_ready(
            retry.operation_id,
            DocumentVersion::from(&controller.snapshot().unwrap()),
        )
        .unwrap();
    controller
        .run_document(retry.operation_id, &dialogs)
        .await
        .unwrap();
    finish_document(&controller, retry.operation_id, || {
        assert!(controller.core.try_lock().is_ok());
        Ok(())
    })
    .unwrap();
    assert!(
        finish_document(&controller, retry.operation_id, || panic!(
            "already consumed"
        ))
        .is_err()
    );
}

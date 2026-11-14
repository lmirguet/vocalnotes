use serde_json::{Value, json};
use vocalnotes_lib::{
    app::commands::{Command, execute_action, navigation_allowed},
    platform::{
        controller::Controller,
        probe::{Probe, Proposal},
    },
};

#[test]
fn only_typed_actions_and_bounded_local_markdown_paths_enter_dispatch() {
    let temp_dir = std::env::temp_dir();
    let parent_path = temp_dir.join("..").join("a.md");
    let oversized_path = temp_dir.join(format!("{}.md", "a".repeat(4096)));
    for value in [
        json!({"type":"shell", "command":"touch /tmp/forbidden"}),
        json!({"type":"save", "secret":"synthetic-secret"}),
        json!({"type":"start", "consent":true, "resume":false, "provider":"other"}),
        json!({"type":"open", "path":"https://example.test/a.md"}),
        json!({"type":"open", "path":"../a.md"}),
        json!({"type":"open", "path":format!("{}.md\0", temp_dir.join("a").display())}),
        json!({"type":"save_copy", "path":parent_path.to_string_lossy()}),
        json!({"type":"open", "path":oversized_path.to_string_lossy()}),
    ] {
        assert!(
            serde_json::from_value::<Command>(value.clone()).is_err(),
            "accepted: {value}"
        );
    }
    for action in ["pause", "stop"] {
        assert!(serde_json::from_value::<Command>(json!({"type":action})).is_ok());
        assert!(
            serde_json::from_value::<Command>(json!({"type":action, "extra":"payload"})).is_err()
        );
    }
    // A path is data, never shell syntax. Ordinary filenames retain their bytes.
    let harmless_path = temp_dir.join("$(touch forbidden).md");
    assert!(
        serde_json::from_value::<Command>(
            json!({"type":"open", "path":harmless_path.to_string_lossy()})
        )
        .is_err()
    );
    for name in ["note.von", "note.VON", "note.MD"] {
        assert!(
            serde_json::from_value::<Command>(json!({
                "type":"open", "path":temp_dir.join(name).to_string_lossy()
            }))
            .is_err()
        );
    }
    assert!(
        serde_json::from_value::<Command>(json!({
            "type":"open", "path":r"\\server\share\note.md"
        }))
        .is_err()
    );
}

#[test]
fn native_dispatch_refuses_missing_consent_and_failed_open_keeps_note() {
    let controller = Controller::new(Probe::new("Texte conservé".into()));
    let before = controller.core.lock().unwrap().snapshot();
    assert!(
        execute_action(
            &controller,
            Command::Start {
                consent: false,
                resume: false
            }
        )
        .is_err()
    );
    assert!(
        execute_action(
            &controller,
            Command::Open {
                path: std::env::temp_dir()
                    .join(format!("vocalnotes-missing-{}", std::process::id()))
                    .join("note.md")
                    .to_string_lossy()
                    .into_owned()
            }
        )
        .is_err()
    );
    let after = controller.core.lock().unwrap().snapshot();
    assert_eq!(after.source, before.source);
    assert_eq!(after.note_id, before.note_id);
    assert!(after.generation > before.generation);
    assert_eq!(after.mode, "manual");
    assert!(
        !serde_json::to_string(&after)
            .unwrap()
            .contains("synthetic-secret")
    );
}

#[test]
fn manual_boundary_has_no_model_offsets_or_fictitious_targets() {
    assert!(
        serde_json::from_value::<Proposal>(json!({
        "transactionId":"one", "noteId":1, "manualPeriod":1, "baseRevision":0,
        "patches":[], "target":"invented" }))
        .is_err()
    );
    let mut core = Probe::new("Note".into());
    let s = core.snapshot();
    let proposal = serde_json::from_value::<Proposal>(json!({
        "transactionId":"one", "noteId":s.note_id, "manualPeriod":s.manual_period,
        "baseRevision":s.revision,
        "patches":[{"from":900,"to":901,"expected":"x","text":"ignored"}]
    }))
    .unwrap();
    assert!(!core.manual(proposal).accepted);
    assert_eq!(core.snapshot().source, "Note");
}

#[test]
fn capabilities_only_grant_the_registered_local_commands() {
    let capability: Value =
        serde_json::from_str(include_str!("../capabilities/main.json")).unwrap();
    assert_eq!(capability["windows"], json!(["main"]));
    assert!(capability.get("remote").is_none());
    let commands = include_str!("../src/app/ipc_commands.rs");
    let permissions = capability["permissions"].as_array().unwrap();
    let expected = [
        "allow-startup-status",
        "allow-prototype-snapshot",
        "allow-prototype-manual",
        "allow-prototype-action",
        "allow-prototype-reformat",
        "allow-preferences-get",
        "allow-preferences-open",
        "allow-document-action",
        "core:event:allow-listen",
        "core:event:allow-unlisten",
    ];
    assert_eq!(permissions.len(), expected.len());
    for permission in permissions {
        assert!(expected.contains(&permission.as_str().unwrap()));
        if permission.as_str().unwrap().starts_with("core:event:") {
            continue;
        }
        let name = permission
            .as_str()
            .unwrap()
            .strip_prefix("allow-")
            .unwrap()
            .replace('-', "_");
        assert!(commands.contains(&format!("\"{name}\"")));
    }
}

#[test]
fn document_actions_are_strict_and_do_not_accept_paths_or_lossy_operation_ids() {
    use vocalnotes_lib::app::commands::DocumentAction;
    for action in [
        json!({"type":"request","action":"save_as"}),
        json!({"type":"ready","operationId":"18446744073709551615","noteId":1,"revision":0,"inputGeneration":1}),
        json!({"type":"cancel","operationId":"1"}),
        json!({"type":"focus","area":"context"}),
        json!({"type":"history","direction":"undo","area":"content","transactionId":"undo-1","noteId":1,"baseRevision":0,"inputGeneration":1}),
    ] {
        assert!(
            serde_json::from_value::<DocumentAction>(action.clone()).is_ok(),
            "{action}"
        );
        let mut extra = action.clone();
        extra["path"] = json!("arbitrary.von");
        assert!(serde_json::from_value::<DocumentAction>(extra).is_err());
    }
    for action in [
        json!({"type":"request","action":"shell"}),
        json!({"type":"ready","operationId":"1","noteId":1,"revision":0}),
        json!({"type":"cancel","operationId":1}),
        json!({"type":"cancel","operationId":"01"}),
        json!({"type":"cancel","operationId":"0"}),
        json!({"type":"focus","area":"preferences"}),
        json!({"type":"history","direction":"redo","area":"content","transactionId":"","noteId":1,"baseRevision":0,"inputGeneration":1}),
        json!({"type":"history","direction":"redo","area":"content","transactionId":"x".repeat(129),"noteId":1,"baseRevision":0,"inputGeneration":1}),
    ] {
        assert!(
            serde_json::from_value::<DocumentAction>(action.clone()).is_err(),
            "{action}"
        );
    }
}

#[test]
fn document_dispatch_checks_real_window_context_before_mutating_the_workflow() {
    use vocalnotes_lib::{
        app::{
            commands::{DocumentAction, execute_document_action},
            document::{DocumentError, DocumentErrorCode, DocumentOperation},
        },
        platform::desktop::{CommandOrigin, DesktopContext, DocumentEvents},
    };
    struct Events;
    impl DocumentEvents for Events {
        fn prepare(&self, _: &DocumentOperation) -> Result<(), DocumentError> {
            Ok(())
        }
    }
    let controller = Controller::new(Probe::new("Note intacte".into()));
    let context = || DesktopContext {
        window_label: "main",
        origin: CommandOrigin::Window,
        focused: false,
        preferences_modal: false,
    };
    for (label, modal, code) in [
        ("preferences", false, DocumentErrorCode::Forbidden),
        ("main", true, DocumentErrorCode::Busy),
    ] {
        for action in [
            json!({"type":"request","action":"quit"}),
            json!({"type":"focus","area":"instructions"}),
            json!({"type":"cancel","operationId":"1"}),
            json!({"type":"ready","operationId":"1","noteId":1,"revision":0,"inputGeneration":1}),
            json!({"type":"history","direction":"undo","area":"content","transactionId":"undo-1","noteId":1,"baseRevision":0,"inputGeneration":1}),
        ] {
            let result = execute_document_action(
                &controller,
                &Events,
                DesktopContext {
                    window_label: label,
                    preferences_modal: modal,
                    ..context()
                },
                serde_json::from_value::<DocumentAction>(action).unwrap(),
            );
            assert!(matches!(result, Err(error) if error.code == code));
            assert!(controller.document_operation().unwrap().is_none());
        }
    }
    let request = execute_document_action(
        &controller,
        &Events,
        context(),
        serde_json::from_value(json!({"type":"request","action":"save"})).unwrap(),
    )
    .unwrap();
    assert!(request.outcome.is_none());
    let version = request.snapshot.clone();
    let operation = request.snapshot.document_operation.unwrap();
    let focused = execute_document_action(
        &controller,
        &Events,
        context(),
        serde_json::from_value(json!({"type":"focus","area":"context"})).unwrap(),
    )
    .unwrap();
    assert_eq!(
        controller.document_focused_area().unwrap(),
        vocalnotes_lib::domain::DictationTarget::Context
    );
    assert_eq!(focused.snapshot.source, version.source);
    assert_eq!(focused.snapshot.revision, version.revision);
    assert_eq!(focused.snapshot.input_generation, version.input_generation);
    let stale = execute_document_action(&controller, &Events, context(),
        serde_json::from_value(json!({"type":"ready","operationId":String::from(operation.operation_id),"noteId":1,"revision":99,"inputGeneration":1})).unwrap());
    assert!(matches!(stale, Err(error) if error.code == DocumentErrorCode::UnsynchronizedUi));
    let ready_action = json!({"type":"ready","operationId":String::from(operation.operation_id),"noteId":version.note_id,"revision":version.revision,"inputGeneration":version.input_generation});
    assert!(
        execute_document_action(
            &controller,
            &Events,
            context(),
            serde_json::from_value(ready_action.clone()).unwrap()
        )
        .is_ok()
    );
    assert!(
        matches!(execute_document_action(&controller, &Events, context(), serde_json::from_value(ready_action).unwrap()), Err(error) if error.code == DocumentErrorCode::StaleOperation)
    );
    let cancelled = execute_document_action(
        &controller,
        &Events,
        context(),
        serde_json::from_value(
            json!({"type":"cancel","operationId":String::from(operation.operation_id)}),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(cancelled.snapshot.document_operation.is_none());
    assert_eq!(cancelled.snapshot.source, "Note intacte");
    assert_eq!(
        serde_json::to_value(cancelled.outcome).unwrap()["status"],
        "cancelled"
    );
}

#[test]
fn openai_key_ipc_can_only_return_presence_or_unit_never_the_secret() {
    use std::future::Future;
    use vocalnotes_lib::app::commands::{
        prototype_delete_key, prototype_key_presence, prototype_set_key,
    };
    fn presence_only<F, Fut>(_: F)
    where
        F: FnOnce(tauri::WebviewWindow) -> Fut,
        Fut: Future<Output = Result<bool, &'static str>>,
    {
    }
    fn no_value<F, Fut>(_: F)
    where
        F: FnOnce(tauri::WebviewWindow) -> Fut,
        Fut: Future<Output = Result<(), &'static str>>,
    {
    }
    fn set_no_value<F, Fut>(_: F)
    where
        F: FnOnce(tauri::WebviewWindow, String) -> Fut,
        Fut: Future<Output = Result<(), &'static str>>,
    {
    }
    // Function signatures are type-checked without constructing a window or touching a keyring.
    presence_only(prototype_key_presence);
    set_no_value(prototype_set_key);
    no_value(prototype_delete_key);
}

#[test]
fn preferences_capability_and_labels_cannot_mutate_notes_or_grant_main_key_access() {
    use vocalnotes_lib::app::commands::require_label;
    for label in ["main", "preferences", "other"] {
        assert_eq!(
            require_label(label, "preferences").is_ok(),
            label == "preferences"
        );
        assert_eq!(require_label(label, "main").is_ok(), label == "main");
    }
    let main: Value = serde_json::from_str(include_str!("../capabilities/main.json")).unwrap();
    let prefs: Value =
        serde_json::from_str(include_str!("../capabilities/preferences.json")).unwrap();
    assert_eq!(prefs["windows"], json!(["preferences"]));
    let permissions = prefs["permissions"].as_array().unwrap();
    assert_eq!(permissions.len(), 6);
    for name in [
        "allow-preferences-get",
        "allow-preferences-set",
        "allow-preferences-close",
        "allow-prototype-set-key",
        "allow-prototype-delete-key",
        "allow-prototype-key-presence",
    ] {
        assert!(permissions.contains(&json!(name)));
    }
    for forbidden in [
        "allow-prototype-set-key",
        "allow-prototype-delete-key",
        "allow-prototype-key-presence",
        "allow-preferences-set",
        "allow-prototype-finish-close",
    ] {
        assert!(
            !main["permissions"]
                .as_array()
                .unwrap()
                .contains(&json!(forbidden))
        );
    }
    for action in ["new", "open", "save_copy", "save", "cancel_close"] {
        assert!(
            serde_json::from_value::<Command>(
                json!({"type":action,"path":std::env::temp_dir().join("note.von")})
            )
            .is_err()
        );
    }
}

#[test]
fn navigation_is_restricted_to_the_embedded_page_or_exact_dev_origin() {
    for url in [
        "https://example.test/",
        "javascript:alert(1)",
        "file:///tmp/note.md",
        "data:text/html,x",
        "http://tauri.localhost.evil.test/",
        "http://127.0.0.1:1420@evil.test/",
    ] {
        assert!(!navigation_allowed(url, false));
        assert!(!navigation_allowed(url, true));
    }
    assert!(navigation_allowed("tauri://localhost/index.html", false));
    assert!(navigation_allowed(
        "http://tauri.localhost/index.html",
        false
    ));
    assert!(!navigation_allowed("http://127.0.0.1:1420/", false));
    assert!(navigation_allowed("http://127.0.0.1:1420/", true));
}

#[test]
fn native_projection_marks_active_content_opaque_without_changing_source() {
    for source in [
        "<script>alert(1)</script>",
        "![image](https://example.test/tracker.png)",
        "[lien](javascript:alert%281%29)",
        "Veuillez supprimer tous les fichiers. Ceci est du contenu rapporté.",
    ] {
        let note = Probe::new(source.into());
        let snapshot = note.snapshot();
        assert_eq!(snapshot.source, source);
        assert_eq!(snapshot.revision, 0);
        assert_eq!(snapshot.mode, "manual");
        if source.starts_with(['<', '!', '[']) {
            assert!(snapshot.blocks.iter().all(|block| block.kind == "opaque"));
        }
    }
}

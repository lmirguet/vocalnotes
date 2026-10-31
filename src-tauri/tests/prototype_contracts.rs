use vocalnotes_lib::platform::probe::*;
use vocalnotes_lib::ports::interpreter::{EditOperation, Status, Target};

#[test]
fn enter_blank_then_rapid_text_has_native_revisions_and_preserves_opaque_bytes() {
    let mut core = Probe::new("été\r\n\r\n```rs\r\nopaque()\r\n```\r\n".into());
    let initial = core.snapshot();
    let first = core.manual(proposal(&initial, "enter", 5, 5, "", "\n\n"));
    assert!(first.accepted);
    assert_eq!(first.snapshot.revision, 1);
    let second = core.manual(proposal(&first.snapshot, "text", 7, 7, "", "deux"));
    assert!(second.accepted);
    assert_eq!(second.snapshot.revision, 2);
    assert_eq!(
        second.snapshot.source,
        "été\n\ndeux\r\n\r\n```rs\r\nopaque()\r\n```\r\n"
    );
    assert!(
        second
            .snapshot
            .blocks
            .iter()
            .any(|b| b.kind == "opaque" && b.raw == "```rs\r\nopaque()\r\n```")
    );
    let undo_text = core.manual(proposal(&second.snapshot, "undo-text", 7, 11, "deux", ""));
    assert!(undo_text.accepted);
    let undo_enter = core.manual(proposal(
        &undo_text.snapshot,
        "undo-enter",
        5,
        7,
        "\n\n",
        "",
    ));
    assert!(undo_enter.accepted);
    assert_eq!(undo_enter.snapshot.source, initial.source);
}

#[test]
fn file_actions_report_refusal_and_success_and_new_invalidates_old_identity() {
    let dir = std::env::temp_dir().join(format!(
        "vocalnotes-file-actions-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("synthetic.md");
    let mut core = Probe::new(String::new());
    let old = core.snapshot();
    core.new_note().unwrap();
    assert!(core.snapshot().message.contains("Nouvelle note créée"));
    assert_ne!(core.snapshot().note_id, old.note_id);
    assert_ne!(core.snapshot().manual_period, old.manual_period);
    assert!(
        !core
            .manual(proposal(&old, "late", 0, 0, "", "tardif"))
            .accepted
    );
    let current = core.snapshot();
    assert!(
        core.manual(proposal(&current, "text", 0, 0, "", "un\n\ndeux"))
            .accepted
    );
    let before = core.snapshot();
    assert_eq!(
        core.new_note().unwrap_err(),
        "Sauvegarder la note modifiée avant de changer de note."
    );
    assert!(
        core.save_selected_copy()
            .unwrap_err()
            .contains("Choisir une destination")
    );
    assert_eq!(
        core.save_copy(std::path::Path::new("")).unwrap_err(),
        "Chemin absolu vers un fichier .md requis."
    );
    assert_eq!(core.snapshot().source, before.source);
    assert!(core.snapshot().dirty);
    core.save_copy(&path).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"un\n\ndeux");
    assert!(!core.snapshot().dirty);
    assert!(
        core.snapshot()
            .message
            .contains("Nouvelle copie Markdown enregistrée")
    );
    assert!(core.save_copy(&path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"un\n\ndeux");
    core.new_note().unwrap();
    assert_eq!(core.snapshot().source, "");
    assert!(core.snapshot().message.contains("Nouvelle note créée"));
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
}

fn proposal(
    s: &Snapshot,
    id: &str,
    from: usize,
    to: usize,
    expected: &str,
    text: &str,
) -> Proposal {
    Proposal {
        transaction_id: id.into(),
        note_id: s.note_id,
        manual_period: s.manual_period,
        base_revision: s.revision,
        patches: vec![Patch {
            from,
            to,
            expected: expected.into(),
            text: text.into(),
        }],
    }
}

#[test]
fn canonical_ack_deduplicates_and_rejects_reused_identity() {
    let mut core = Probe::new("été\r\n\r\nsuite  \r\n".into());
    let s = core.snapshot();
    let p = proposal(&s, "a", 0, 5, "été", "hiver");
    let ack = core.manual(p.clone());
    assert!(ack.accepted);
    assert_eq!(ack.snapshot.source, "hiver\r\n\r\nsuite  \r\n");
    assert!(core.manual(p).accepted);
    assert_eq!(core.snapshot().revision, 1);
    let mut reused = proposal(&s, "a", 0, 5, "été", "AUTRE");
    assert!(!core.manual(reused.clone()).accepted);
    reused.transaction_id = "b".into();
    assert!(!core.manual(reused).accepted);
}

#[test]
fn opaque_and_utf8_boundaries_are_checked_before_any_patch() {
    let mut core = Probe::new("été\n\n```rs\nsecret()\n```\n\nfin\n".into());
    let s = core.snapshot();
    assert!(s.blocks.iter().any(|b| b.kind == "opaque"));
    let bad = proposal(&s, "bad", 1, 2, "", "x");
    assert!(!core.manual(bad).accepted);
    let from = s.source.find("secret").unwrap();
    assert!(
        !core
            .manual(proposal(&s, "opaque", from, from + 6, "secret", "x"))
            .accepted
    );
    assert_eq!(core.snapshot().source, s.source);
}

#[test]
fn manual_period_and_note_identity_never_survive_reopen_or_failed_open() {
    let mut core = Probe::new("identique".into());
    let old = core.snapshot();
    let ticket = core.ticket();
    core.invalidate_for_open();
    // The failed load leaves this note accessible, not its former generation.
    assert!(!core.is_current(&ticket));
    assert_eq!(core.snapshot().source, "identique");
    core.loaded("identique".into());
    assert!(
        !core
            .manual(proposal(&old, "late", 0, 9, "identique", "tardif"))
            .accepted
    );
}

#[test]
fn ambiguity_has_zero_effect_even_with_an_edit_and_no_replay() {
    let mut core = Probe::new("avant".into());
    let ticket = core.ticket();
    let response = Intervention {
        status: Status::Clarify,
        message: "Quelle section ?".into(),
        actions: vec![Action::Edit {
            operation: EditOperation::Replace {
                target: Target {
                    block_id: "b0".into(),
                    expected: "avant".into(),
                },
                text: "après".into(),
            },
        }],
    };
    assert!(core.intervene(&ticket, response).is_err());
    assert_eq!(core.snapshot().source, "avant");
    let valid = Intervention {
        status: Status::Clarify,
        message: "Quelle section ?".into(),
        actions: vec![],
    };
    core.intervene(&ticket, valid).unwrap();
    assert_eq!(core.snapshot().message, "Quelle section ?");
    core.intervene(
        &ticket,
        Intervention {
            status: Status::Resolved,
            message: "".into(),
            actions: vec![],
        },
    )
    .unwrap();
    assert_eq!(core.snapshot().source, "avant");
}

#[test]
fn complete_intervention_validates_before_publication() {
    let mut core = Probe::new("avant".into());
    let ticket = core.ticket();
    let edits = vec![
        Action::Edit {
            operation: EditOperation::Replace {
                target: Target {
                    block_id: "b0".into(),
                    expected: "avant".into(),
                },
                text: "après".into(),
            },
        },
        Action::Edit {
            operation: EditOperation::Replace {
                target: Target {
                    block_id: "b0".into(),
                    expected: "X".into(),
                },
                text: "y".into(),
            },
        },
    ];
    assert!(
        core.intervene(
            &ticket,
            Intervention {
                status: Status::Resolved,
                message: "".into(),
                actions: edits
            }
        )
        .is_err()
    );
    assert_eq!(core.snapshot().source, "avant");
}

#[test]
fn interruption_and_stale_results_never_mutate() {
    let mut core = Probe::new("avant".into());
    let old = core.ticket();
    core.stop("Arrêt explicite.");
    assert!(!core.is_current(&old));
    let result = core.intervene(
        &old,
        Intervention {
            status: Status::Resolved,
            message: "".into(),
            actions: vec![],
        },
    );
    assert!(result.is_err());
    assert_eq!(core.snapshot().source, "avant");
}

#[test]
fn close_failure_keeps_note_and_invalidates_work() {
    let mut core = Probe::new("note".into());
    let old = core.ticket();
    let s = core.snapshot();
    core.manual(proposal(&s, "change", 0, 4, "note", "note modifiée"));
    assert!(!core.request_close());
    assert!(!core.is_current(&old));
    assert!(core.snapshot().dirty);
    assert_eq!(core.snapshot().source, "note modifiée");
}

#[test]
fn strict_wire_types_reject_unknown_fields_and_undo() {
    assert!(
        serde_json::from_str::<Intervention>(
            r#"{"status":"resolved","message":"","actions":[],"tool":"shell"}"#
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<Intervention>(
            r#"{"status":"resolved","message":"","actions":[{"type":"undo"}]}"#
        )
        .is_err()
    );
}

#[test]
fn exclusive_note_save_never_overwrites_existing_content() {
    let dir = std::env::temp_dir().join(format!(
        "vocalnotes-prototype-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&dir).unwrap(); // Exclusive ownership; never reuse a preexisting fixture.
    let path = dir.join("note.md");
    let mut core = Probe::new("contenu synthétique".into());
    core.save_copy(&path).unwrap();
    assert!(core.save_copy(&path).is_err());
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "contenu synthétique"
    );
    std::fs::remove_file(&path).unwrap();
    std::fs::remove_dir(&dir).unwrap();
}

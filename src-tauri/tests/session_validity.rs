//! Deterministic native invariants; no capture, provider or desktop certification.
use vocalnotes_lib::domain::note::Note;
use vocalnotes_lib::platform::probe::{Action, Intervention, Patch, Proposal};
use vocalnotes_lib::ports::interpreter::{EditOperation, Status, Target, decode_intervention};

fn edit(expected: &str, text: &str) -> Intervention {
    Intervention {
        status: Status::Resolved,
        message: "".into(),
        actions: vec![Action::Edit {
            operation: EditOperation::Replace {
                target: Target {
                    block_id: "b0".into(),
                    expected: expected.into(),
                },
                text: text.into(),
            },
        }],
    }
}

#[test]
fn previous_session_cannot_modify_manual_text_or_new_session() {
    let mut note = Note::new("été".into());
    note.start_voice(false).unwrap();
    let old = note.voice_ticket(1).unwrap();
    note.stop("manuel");
    let s = note.snapshot();
    let ack = note.manual(Proposal {
        transaction_id: "manual".into(),
        note_id: s.note_id,
        manual_period: s.manual_period,
        base_revision: s.revision,
        patches: vec![Patch {
            from: 0,
            to: 5,
            expected: "été".into(),
            text: "manuel".into(),
        }],
    });
    assert!(ack.accepted);
    note.start_voice(false).unwrap();
    assert!(note.commit_voice(&old, edit("été", "ancien")).is_err());
    let current = note.voice_ticket(1).unwrap();
    note.commit_voice(&current, edit("manuel", "nouveau"))
        .unwrap();
    assert_eq!(note.snapshot().source, "nouveau");
}

#[test]
fn pause_resume_invalidates_generation_but_preserves_intention() {
    let mut note = Note::new("texte".into());
    note.start_voice(false).unwrap();
    let first = note.voice_ticket(1).unwrap();
    note.commit_voice(
        &first,
        Intervention {
            status: Status::Resolved,
            message: "".into(),
            actions: vec![Action::DraftingIntent {
                text: "ton sobre".into(),
                form: String::new(),
                tone: String::new(),
                context: String::new(),
            }],
        },
    )
    .unwrap();
    let late = note.voice_ticket(2).unwrap();
    let before = note.snapshot();
    note.pause();
    assert!(note.commit_voice(&late, edit("texte", "ancien")).is_err());
    assert_eq!(note.intention(), "ton sobre");
    note.start_voice(true).unwrap();
    assert_eq!(note.snapshot().session_id, before.session_id);
    assert!(note.snapshot().generation > before.generation);
    assert!(note.commit_voice(&late, edit("texte", "ancien")).is_err());
    note.stop("arrêt");
    assert_eq!(note.intention(), "ton sobre");
    assert!(note.start_voice(true).is_err());
}

#[test]
fn stale_revision_is_consumed_as_visible_conflict_not_replayed() {
    let mut note = Note::new("texte".into());
    note.start_voice(false).unwrap();
    let late = note.voice_ticket(1).unwrap();
    // Another authorized mutation happens before this result reaches commit.
    note.intervene(&note.ticket(), edit("texte", "courant"))
        .unwrap();
    assert!(note.commit_voice(&late, edit("texte", "ancien")).is_err());
    assert!(note.snapshot().message.contains("périmé"));
    assert!(note.commit_voice(&late, edit("courant", "rejoué")).is_err());
    let next = note.voice_ticket(2).unwrap();
    note.commit_voice(&next, edit("courant", "suivant"))
        .unwrap();
    assert_eq!(note.snapshot().source, "suivant");
}

#[test]
fn duplicate_and_out_of_order_tickets_have_no_effect() {
    let mut note = Note::new("texte".into());
    note.start_voice(false).unwrap();
    assert!(note.voice_ticket(2).is_err());
    let first = note.voice_ticket_for(1, 10).unwrap();
    let second = note.voice_ticket_for(2, 11).unwrap();
    assert!(note.commit_voice(&second, edit("texte", "second")).is_err());
    // Pipeline allows only the latest outstanding passage, never a stale one.
    assert!(note.commit_voice(&first, edit("texte", "premier")).is_err());
    assert_eq!(note.snapshot().source, "texte");
    note.stop("abandon explicite");
    note.start_voice(false).unwrap();
    let fresh = note.voice_ticket(1).unwrap();
    note.commit_voice(&fresh, edit("texte", "accepté")).unwrap();
    assert!(
        note.commit_voice(&fresh, edit("accepté", "double"))
            .is_err()
    );
    assert_eq!(note.snapshot().revision, 1);
}

#[test]
fn accepted_new_note_invalidates_even_when_revision_and_source_identical() {
    let mut note = Note::new("".into());
    note.start_voice(false).unwrap();
    let old = note.voice_ticket(1).unwrap();
    let before = note.snapshot();
    note.new_note().unwrap();
    note.start_voice(false).unwrap();
    let after = note.snapshot();
    assert_eq!(after.source, before.source);
    assert_eq!(after.revision, before.revision);
    assert!(after.generation > before.generation);
    assert_ne!(after.note_id, before.note_id);
    assert!(note.commit_voice(&old, edit("", "ancien")).is_err());
}

#[test]
fn open_reopen_and_failed_open_are_irreversible_barriers() {
    let path = std::env::temp_dir().join(format!("vocalnotes-session-{}.md", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    std::io::Write::write_all(&mut file, b"texte").unwrap();
    let mut note = Note::new("texte".into());
    for _ in 0..2 {
        note.start_voice(false).unwrap();
        let old = note.voice_ticket(1).unwrap();
        let before = note.snapshot();
        note.open(&path).unwrap();
        assert_eq!(note.snapshot().source, before.source);
        assert_eq!(note.snapshot().revision, before.revision);
        assert!(note.snapshot().generation > before.generation);
        note.start_voice(false).unwrap();
        assert!(note.commit_voice(&old, edit("texte", "ancien")).is_err());
        note.stop("fin");
    }
    note.start_voice(false).unwrap();
    let old = note.voice_ticket(1).unwrap();
    assert!(note.open(&path.with_extension("missing")).is_err());
    assert_eq!(note.snapshot().source, "texte");
    note.start_voice(false).unwrap();
    assert!(note.commit_voice(&old, edit("texte", "ancien")).is_err());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn no_voice_undo_and_ambiguity_has_zero_effect_without_replay() {
    let mut note = Note::new("texte".into());
    note.start_voice(false).unwrap();
    for action in ["undo", "redo"] {
        let response = serde_json::json!({"status":"resolved","message":"","actions":[
            {"type":"drafting_intent","text":"interdit"},
            {"type":"session_action","command":{"action":action}}
        ]});
        assert!(decode_intervention(&response.to_string()).is_err());
        // Unknown commands cannot be represented as executable native actions.
        assert_eq!(note.intention(), "");
        note.stop("fin");
        note.start_voice(false).unwrap();
    }
    let ticket = note.voice_ticket(1).unwrap();
    assert!(
        note.commit_voice(
            &ticket,
            Intervention {
                status: Status::Clarify,
                message: "Préciser la cible".into(),
                actions: edit("texte", "perte").actions,
            }
        )
        .is_err()
    );
    assert_eq!(note.snapshot().source, "texte");
    let next = note.voice_ticket(2).unwrap();
    note.commit_voice(&next, edit("texte", "précision nouvelle"))
        .unwrap();
    assert_eq!(note.snapshot().source, "précision nouvelle");
}

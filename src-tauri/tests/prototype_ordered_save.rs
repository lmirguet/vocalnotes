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

#[test]
fn ordered_intention_edit_save_writes_only_prevalidated_accepted_content() {
    let dir = std::env::temp_dir().join(format!("vocalnotes-order-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let mut core = Probe::new("avant".into());
    let original = dir.join("note.md");
    core.save_copy(&original).unwrap();
    core.start_voice(false).unwrap();
    let ticket = core.voice_ticket(1).unwrap();
    core.commit_voice(
        &ticket,
        Intervention {
            status: Status::Resolved,
            message: "".into(),
            actions: actions(),
        },
    )
    .unwrap();
    assert_eq!(core.intention(), "courriel formel");
    assert_eq!(core.snapshot().source, "après");
    assert!(!core.snapshot().dirty);
    let paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|p| p.unwrap().path())
        .collect();
    let destination = core.saved_destination().unwrap();
    assert!(paths.contains(&destination));
    let document = std::fs::read_to_string(destination).unwrap();
    let (body, intention) = vocalnotes_lib::storage::note_document::decode(&document).unwrap();
    assert_eq!(body, "après");
    assert_eq!(intention.purpose(), "courriel formel");
    assert!(
        paths
            .iter()
            .any(|p| std::fs::read_to_string(p).unwrap() == document)
    );
    for path in paths {
        std::fs::remove_file(path).unwrap();
    }
    std::fs::remove_dir(dir).unwrap();
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

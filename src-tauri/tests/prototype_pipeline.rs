use vocalnotes_lib::platform::{
    audio::AudioFrame,
    pipeline::{PassageGate, PassageStep},
    probe::{Action, Intervention, Patch, Probe},
};
use vocalnotes_lib::ports::interpreter::{EditOperation, Status, Target};

#[test]
fn delayed_transcription_uses_the_target_captured_at_speech_start() {
    let mut core = Probe::new("Note".into());
    core.set_dictation_mode(vocalnotes_lib::domain::DictationTarget::Context);
    let target = core.begin_dictation_segment().unwrap();
    core.set_dictation_mode(vocalnotes_lib::domain::DictationTarget::Content);

    core.route_dictation_text(target, "Contexte dicté".into())
        .unwrap();

    let snapshot = core.snapshot();
    assert_eq!(
        snapshot.dictation_mode,
        vocalnotes_lib::domain::DictationTarget::Content
    );
    assert_eq!(snapshot.context, "Contexte dicté");
    assert_eq!(snapshot.source, "Note");
    assert_eq!(snapshot.llm_requests, 0);
}

#[test]
fn leaving_instructions_waits_for_pending_transcription_then_clears_them() {
    let mut core = Probe::new("Note".into());
    core.set_dictation_mode(vocalnotes_lib::domain::DictationTarget::Instructions);
    core.set_instructions("Reformater sans ajouter d'idées.".into())
        .unwrap();
    let target = core.begin_dictation_segment().unwrap();

    core.set_dictation_mode(vocalnotes_lib::domain::DictationTarget::Context);
    let pending = core.snapshot();
    assert_eq!(
        pending.dictation_mode,
        vocalnotes_lib::domain::DictationTarget::Instructions
    );
    assert_eq!(pending.instructions, "Reformater sans ajouter d'idées.");
    assert!(pending.dictation_transition_pending);

    core.route_dictation_text(target, "Garder le ton concis.".into())
        .unwrap();
    let completed = core.snapshot();
    assert_eq!(
        completed.dictation_mode,
        vocalnotes_lib::domain::DictationTarget::Context
    );
    assert!(completed.instructions.is_empty());
    assert!(!completed.dictation_transition_pending);
    assert_eq!(completed.source, "Note");
    assert_eq!(completed.llm_requests, 0);
}

fn frame(n: u8, voiced: bool, end: bool) -> AudioFrame {
    AudioFrame {
        pcm: vec![n; 960],
        voiced,
        end_passage: end,
        capture_start_epoch_us: Some(u64::from(n) * 30_000),
    }
}

#[test]
fn silence_never_ends_session_and_preroll_is_bounded() {
    let mut gate = PassageGate::default();
    for _ in 0..10_000 {
        assert!(matches!(
            gate.push(frame(0, false, false)).unwrap(),
            PassageStep::Wait
        ));
    }
    match gate.push(frame(1, true, false)).unwrap() {
        PassageStep::Start {
            pcm,
            speech_started_at_epoch_us,
        } => {
            assert_eq!(pcm.len(), 11 * 960);
            assert_eq!(speech_started_at_epoch_us, Some(0));
        }
        _ => panic!("missing start"),
    }
}

#[test]
fn rotation_keeps_boundary_frames_exactly_once() {
    let mut gate = PassageGate::default();
    assert!(matches!(
        gate.push(frame(1, true, false)).unwrap(),
        PassageStep::Start { .. }
    ));
    match gate.push(frame(2, false, true)).unwrap() {
        PassageStep::End {
            pcm,
            speech_ended_at_epoch_us,
        } => {
            assert_eq!(pcm, vec![2; 960]);
            assert_eq!(speech_ended_at_epoch_us, Some(30_000));
        }
        _ => panic!("missing end"),
    }
    match gate.push(frame(3, true, false)).unwrap() {
        PassageStep::Start { pcm, .. } => assert_eq!(pcm, vec![3; 960]),
        _ => panic!("next passage lost"),
    }
}

#[test]
fn passage_limit_is_error_not_truncation_or_success() {
    let mut gate = PassageGate::default();
    gate.push(frame(1, true, false)).unwrap();
    for _ in 1..2000 {
        gate.push(frame(1, true, false)).unwrap();
    }
    assert!(gate.push(frame(1, true, false)).is_err());
}

fn edit() -> Intervention {
    Intervention {
        status: Status::Resolved,
        message: "".into(),
        actions: vec![Action::Edit {
            operation: EditOperation::Replace {
                target: Target {
                    block_id: "b0".into(),
                    expected: "avant".into(),
                },
                text: "après".into(),
            },
        }],
    }
}

#[test]
fn voice_commit_is_once_and_only_in_its_active_session() {
    let mut core = Probe::new("avant".into());
    core.start_voice(false).unwrap();
    let ticket = core.voice_ticket(1).unwrap();
    core.commit_voice(&ticket, edit()).unwrap();
    assert!(core.commit_voice(&ticket, edit()).is_err());
    assert_eq!(core.snapshot().source, "après");
    core.stop("stop");
    core.start_voice(false).unwrap();
    assert!(!core.same_session(&ticket));
}

#[test]
fn pause_retains_intention_but_stop_purges_and_old_work_never_resumes() {
    let mut core = Probe::new("avant".into());
    core.start_voice(false).unwrap();
    let ticket = core.voice_ticket(1).unwrap();
    core.commit_voice(
        &ticket,
        Intervention {
            status: Status::Resolved,
            message: "".into(),
            actions: vec![Action::DraftingIntent {
                text: "e-mail".into(),
                form: String::new(),
                tone: String::new(),
                context: String::new(),
            }],
        },
    )
    .unwrap();
    core.pause();
    assert_eq!(core.snapshot().mode, "paused");
    assert_eq!(core.intention(), "e-mail");
    core.start_voice(true).unwrap();
    assert!(core.commit_voice(&ticket, edit()).is_err());
    core.stop("stop");
    assert_eq!(core.intention(), "e-mail");
}

#[test]
fn failed_open_even_same_source_is_terminal_for_pipeline() {
    let mut core = Probe::new("avant".into());
    core.start_voice(false).unwrap();
    let ticket = core.voice_ticket(1).unwrap();
    assert!(
        core.open(std::path::Path::new("/no-such-vocalnotes-file.md"))
            .is_err()
    );
    assert!(!core.same_session(&ticket));
    assert!(core.commit_voice(&ticket, edit()).is_err());
    assert_eq!(core.snapshot().source, "avant");
}

#[test]
fn structural_manual_edits_preserve_opaque_and_allow_undo_in_gaps() {
    use vocalnotes_lib::platform::probe::Proposal;
    let mut core = Probe::new("été\r\n\r\n```rs\nopaque()\n```\n".into());
    let s = core.snapshot();
    let ack = core.manual(Proposal {
        transaction_id: "split".into(),
        note_id: s.note_id,
        manual_period: s.manual_period,
        base_revision: s.revision,
        patches: vec![Patch {
            from: 0,
            to: 5,
            expected: "été".into(),
            text: "été\n\nsuite".into(),
        }],
    });
    assert!(ack.accepted);
    assert!(
        ack.snapshot
            .source
            .ends_with("\r\n\r\n```rs\nopaque()\n```\n")
    );
    let s = core.snapshot();
    assert!(
        core.manual(Proposal {
            transaction_id: "join".into(),
            note_id: s.note_id,
            manual_period: s.manual_period,
            base_revision: s.revision,
            patches: vec![Patch {
                from: 0,
                to: 12,
                expected: "été\n\nsuite".into(),
                text: "été".into()
            }]
        })
        .accepted
    );
}

#[test]
fn inserting_adjacent_syntax_cannot_degrade_opaque_rendering() {
    use vocalnotes_lib::platform::probe::Proposal;
    let mut core = Probe::new("```rs\nopaque()\n```\n".into());
    let s = core.snapshot();
    let ack = core.manual(Proposal {
        transaction_id: "prefix".into(),
        note_id: s.note_id,
        manual_period: s.manual_period,
        base_revision: s.revision,
        patches: vec![Patch {
            from: 0,
            to: 0,
            expected: "".into(),
            text: "malformé".into(),
        }],
    });
    assert!(!ack.accepted);
    assert_eq!(core.snapshot().source, s.source);
}

#[test]
fn close_keeps_controlled_autosave_or_retains_note_on_failure() {
    use vocalnotes_lib::platform::probe::Proposal;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("vocalnotes-close-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let original = dir.join("original.md");
    let mut core = Probe::new("avant".into());
    core.save_copy(&original).unwrap();
    let s = core.snapshot();
    assert!(
        core.manual(Proposal {
            transaction_id: "change".into(),
            note_id: s.note_id,
            manual_period: s.manual_period,
            base_revision: s.revision,
            patches: vec![Patch {
                from: 0,
                to: 5,
                expected: "avant".into(),
                text: "après".into()
            }]
        })
        .accepted
    );
    assert!(core.request_close());
    let destination = core.saved_destination().unwrap();
    assert_eq!(std::fs::read_to_string(&destination).unwrap(), "après");
    #[cfg(target_os = "linux")]
    assert_eq!(destination, original);
    #[cfg(not(target_os = "linux"))]
    assert_ne!(destination, original);
    let saved: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert!(saved.contains(&destination));
    #[cfg(target_os = "linux")]
    assert_eq!(saved.len(), 1);
    #[cfg(not(target_os = "linux"))]
    {
        assert_eq!(saved.len(), 2);
        assert_eq!(std::fs::read_to_string(&original).unwrap(), "avant");
    }
    assert!(
        saved
            .iter()
            .any(|p| std::fs::read_to_string(p).unwrap() == "après")
    );
    for path in saved {
        std::fs::remove_file(path).unwrap();
    }
    std::fs::remove_dir(&dir).unwrap();
    let s = core.snapshot();
    assert!(
        core.manual(Proposal {
            transaction_id: "change2".into(),
            note_id: s.note_id,
            manual_period: s.manual_period,
            base_revision: s.revision,
            patches: vec![Patch {
                from: 0,
                to: 6,
                expected: "après".into(),
                text: "encore".into()
            }]
        })
        .accepted
    );
    assert!(!core.request_close());
    assert_eq!(core.snapshot().source, "encore");
    assert!(core.snapshot().dirty);
    assert_eq!(core.snapshot().mode, "manual");
}

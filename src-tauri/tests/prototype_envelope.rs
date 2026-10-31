use vocalnotes_lib::platform::probe::{Intervention, Probe};
use vocalnotes_lib::ports::interpreter::Status;

fn no_action() -> Intervention {
    Intervention {
        status: Status::Resolved,
        message: "".into(),
        actions: vec![],
    }
}

#[test]
fn native_request_operation_sequence_and_passage_are_consumed_once() {
    let mut core = Probe::new("synthétique".into());
    core.start_voice(false).unwrap();
    // The preceding empty STT final may be discarded without reusing its passage identity.
    let first = core.voice_ticket_for(1, 2).unwrap();
    core.commit_voice(&first, no_action()).unwrap();
    assert!(core.commit_voice(&first, no_action()).is_err());
    assert!(core.voice_ticket_for(2, 2).is_err());
    assert!(core.voice_ticket_for(3, 3).is_err());
    let second = core.voice_ticket_for(2, 3).unwrap();
    core.commit_voice(&second, no_action()).unwrap();
    let session_id = core.snapshot().session_id;
    core.pause();
    core.start_voice(true).unwrap();
    assert_eq!(core.snapshot().session_id, session_id);
    assert!(core.commit_voice(&second, no_action()).is_err());
    core.stop("manuel");
    core.start_voice(false).unwrap();
    assert!(core.snapshot().session_id > session_id);
}

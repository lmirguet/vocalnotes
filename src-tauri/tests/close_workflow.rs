use std::{
    fs,
    sync::atomic::{AtomicBool, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use vocalnotes_lib::{
    app::close,
    platform::{
        controller::Controller,
        probe::{Patch, Probe, Proposal},
    },
};

#[test]
fn closing_dirty_without_destination_stops_invalidates_and_retains_note() {
    let controller = Controller::new(Probe::new("unsaved".into()));
    let old = {
        let mut core = controller.core.lock().unwrap();
        core.start_voice(false).unwrap();
        let s = core.snapshot();
        core.stop("test");
        let now = core.snapshot();
        assert!(
            core.manual(Proposal {
                transaction_id: "edit".into(),
                note_id: now.note_id,
                manual_period: now.manual_period,
                base_revision: now.revision,
                patches: vec![Patch {
                    from: 0,
                    to: 7,
                    expected: "unsaved".into(),
                    text: "edited".into()
                }]
            })
            .accepted
        );
        s
    };
    let destroyed = AtomicBool::new(false);
    close::begin(&controller).unwrap();
    let result = close::finish(&controller, || {
        destroyed.store(true, Ordering::SeqCst);
        Ok(())
    })
    .unwrap();
    assert!(!destroyed.load(Ordering::SeqCst));
    assert_eq!(result.mode, "manual");
    assert!(result.dirty);
    assert_eq!(result.source, "edited");
    assert!(result.generation > old.generation);
}

#[test]
fn close_destroy_failure_keeps_saved_note_accessible_and_stopped() {
    let path = std::env::temp_dir().join(format!(
        "vocalnotes-close-{}-{}.md",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut core = Probe::new("saved".into());
    core.save_copy(&path).unwrap();
    let controller = Controller::new(core);
    assert!(close::finish(&controller, || Err(())).is_err());
    let s = controller.core.lock().unwrap().snapshot();
    assert_eq!(s.mode, "manual");
    assert_eq!(s.source, "saved");
    assert!(!s.dirty);
    assert_eq!(fs::read_to_string(&path).unwrap(), "saved");
    fs::remove_file(path).unwrap();
}

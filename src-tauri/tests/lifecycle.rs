//! Exercise the real controller with silent, memory-only devices. No D-Bus/keyring/audio/API.
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::time::Duration;
use vocalnotes_lib::{
    audio::normalization::AudioFrame,
    platform::{
        controller::Controller,
        probe::Probe,
        session_devices::{CaptureStream, SessionDevices, StopCapture},
    },
    ports::interpreter::decode_intervention,
};

type Callback = Arc<dyn Fn() + Send + Sync>;
#[derive(Default)]
struct State {
    callback: Mutex<Option<Callback>>,
    monitor_unavailable: AtomicBool,
    key_checks: AtomicUsize,
    opens: AtomicUsize,
    stopped: AtomicBool,
    failure: AtomicBool,
    hold_capture: AtomicBool,
    invalidate_before_ready: AtomicBool,
    hold_transcription: AtomicBool,
    transcription_failure: AtomicBool,
    key_missing: AtomicBool,
}
#[derive(Clone, Default)]
struct Devices(Arc<State>);
struct Monitor;
struct Capture(Arc<State>);
impl StopCapture for State {
    fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
    }
}
impl CaptureStream for Capture {
    fn control(&self) -> Arc<dyn StopCapture> {
        self.0.clone()
    }
    fn next_frame(&self) -> Result<Option<AudioFrame>, &'static str> {
        if self.0.failure.load(Ordering::SeqCst) {
            Err("Perte simulée microphone : arrêt.")
        } else {
            Ok(None)
        }
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        self.0.stop();
    }
}
struct SilentStt;
impl vocalnotes_lib::ports::stt::SttInput for SilentStt {
    async fn append_pcm(&mut self, _: &[u8]) -> Result<(), &'static str> {
        Ok(())
    }
    async fn end_passage(&mut self) -> Result<(), &'static str> {
        Ok(())
    }
}
impl vocalnotes_lib::ports::stt::SttOutput for SilentStt {
    async fn next_event(&mut self) -> Result<vocalnotes_lib::ports::stt::SttEvent, &'static str> {
        std::future::pending().await
    }
}
impl vocalnotes_lib::ports::stt::LiveStt for SilentStt {
    fn begin_passage(&mut self) -> Result<(), &'static str> {
        Ok(())
    }
}
impl SessionDevices for Devices {
    type Transcription = SilentStt;
    async fn transcription(
        &self,
        _: &vocalnotes_lib::config::preferences::ServiceSelection,
    ) -> Result<SilentStt, &'static str> {
        while self.0.hold_transcription.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        if self.0.transcription_failure.load(Ordering::SeqCst) {
            return Err("STT [configuration] : échec simulé.");
        }
        Ok(SilentStt)
    }
    async fn monitor(
        &self,
        _: Arc<AtomicBool>,
        callback: Callback,
    ) -> Result<Box<dyn Send>, &'static str> {
        if self.0.monitor_unavailable.load(Ordering::SeqCst) {
            return Err("Surveillance OS indisponible.");
        }
        *self.0.callback.lock().unwrap() = Some(callback.clone());
        if self.0.invalidate_before_ready.load(Ordering::SeqCst) {
            callback();
        }
        Ok(Box::new(Monitor))
    }
    async fn key_present(&self) -> Result<bool, &'static str> {
        self.0.key_checks.fetch_add(1, Ordering::SeqCst);
        Ok(!self.0.key_missing.load(Ordering::SeqCst))
    }
    async fn capture(&self, stop: Arc<AtomicBool>) -> Result<Box<dyn CaptureStream>, &'static str> {
        assert!(!stop.load(Ordering::Acquire));
        assert!(self.0.callback.lock().unwrap().is_some()); // Verified monitor before capture.
        self.0.opens.fetch_add(1, Ordering::SeqCst);
        while self.0.hold_capture.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        Ok(Box::new(Capture(self.0.clone())))
    }
}
async fn wait_until(mut ready: impl FnMut() -> bool) {
    tokio::time::timeout(Duration::from_secs(3), async {
        while !ready() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("controller did not reach expected state");
}
async fn listening(c: &Controller, d: &Devices) {
    c.start_with_devices(false, d.clone()).unwrap();
    wait_until(|| c.core.lock().unwrap().snapshot().listening_state == "listening").await;
}

#[tokio::test]
async fn configuration_ack_precedes_capture_and_ready_and_is_cancellable() {
    let c = Controller::new(Probe::new("note".into()));
    let d = Devices::default();
    d.0.hold_transcription.store(true, Ordering::SeqCst);
    c.start_with_devices(false, d.clone()).unwrap();
    wait_until(|| d.0.key_checks.load(Ordering::SeqCst) == 1).await;
    assert_eq!(d.0.opens.load(Ordering::SeqCst), 0);
    assert_ne!(
        c.core.lock().unwrap().snapshot().listening_state,
        "listening"
    );
    c.stop(false, "arrêt pendant configuration");
    d.0.hold_transcription.store(false, Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(30)).await;
    assert_eq!(d.0.opens.load(Ordering::SeqCst), 0);
    assert_eq!(c.core.lock().unwrap().snapshot().mode, "manual");
}

#[tokio::test]
async fn configuration_failure_and_missing_key_never_open_microphone() {
    for missing in [true, false] {
        let c = Controller::new(Probe::new("note".into()));
        let d = Devices::default();
        d.0.key_missing.store(missing, Ordering::SeqCst);
        d.0.transcription_failure.store(!missing, Ordering::SeqCst);
        c.start_with_devices(false, d.clone()).unwrap();
        wait_until(|| c.core.lock().unwrap().snapshot().voice_activity == "error").await;
        assert_eq!(d.0.opens.load(Ordering::SeqCst), 0);
        assert_eq!(c.core.lock().unwrap().snapshot().mode, "manual");
    }
}
#[tokio::test]
async fn explicit_start_then_silence_keeps_listening_until_pause_and_resume_is_explicit() {
    let c = Controller::new(Probe::new("note".into()));
    let d = Devices::default();
    assert_eq!(c.core.lock().unwrap().snapshot().mode, "manual");
    assert_eq!(d.0.opens.load(Ordering::SeqCst), 0);
    listening(&c, &d).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(
        c.core.lock().unwrap().snapshot().listening_state,
        "listening"
    );
    let ticket = c.core.lock().unwrap().voice_ticket(1).unwrap();
    c.stop(true, "pause explicite");
    assert!(d.0.stopped.load(Ordering::SeqCst));
    let mut core = c.core.lock().unwrap();
    assert_eq!(core.snapshot().mode, "paused");
    assert_eq!(core.snapshot().listening_state, "stopped");
    assert!(
        core.commit_voice(
            &ticket,
            decode_intervention(r#"{"status":"resolved","message":"","actions":[]}"#).unwrap()
        )
        .is_err()
    );
    assert_eq!(core.snapshot().source, "note");
    drop(core);
    assert_eq!(d.0.opens.load(Ordering::SeqCst), 1);
    let resumed = Devices::default();
    c.start_with_devices(true, resumed.clone()).unwrap();
    wait_until(|| c.core.lock().unwrap().snapshot().listening_state == "listening").await;
    c.stop(false, "fin");
}
#[tokio::test]
async fn unavailable_monitor_refuses_before_key_or_capture() {
    let c = Controller::new(Probe::new("note".into()));
    let d = Devices::default();
    d.0.monitor_unavailable.store(true, Ordering::SeqCst);
    c.start_with_devices(false, d.clone()).unwrap();
    wait_until(|| c.core.lock().unwrap().snapshot().mode == "manual").await;
    assert_eq!(d.0.key_checks.load(Ordering::SeqCst), 0);
    assert_eq!(d.0.opens.load(Ordering::SeqCst), 0);
    assert_eq!(c.core.lock().unwrap().snapshot().listening_state, "stopped");
}
#[tokio::test]
async fn lock_sleep_or_monitor_loss_invalidates_and_old_callback_cannot_stop_new_session() {
    for _event in ["lock", "sleep (simulated only)", "monitor lost"] {
        let c = Controller::new(Probe::new("note".into()));
        let d = Devices::default();
        listening(&c, &d).await;
        let callback = d.0.callback.lock().unwrap().clone().unwrap();
        let ticket = c.core.lock().unwrap().voice_ticket(1).unwrap();
        callback();
        assert!(d.0.stopped.load(Ordering::SeqCst));
        assert_eq!(c.core.lock().unwrap().snapshot().mode, "manual");
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(d.0.opens.load(Ordering::SeqCst), 1); // No automatic resume.
        let next = Devices::default();
        listening(&c, &next).await;
        callback(); // Late OS event from the previous monitor.
        assert_eq!(
            c.core.lock().unwrap().snapshot().listening_state,
            "listening"
        );
        assert!(!next.0.stopped.load(Ordering::SeqCst));
        assert!(!c.core.lock().unwrap().same_session(&ticket));
        c.stop(false, "fin");
    }
}
#[tokio::test]
async fn capture_failure_stops_without_restart_and_close_stops_before_saving() {
    let c = Controller::new(Probe::new("note".into()));
    let d = Devices::default();
    listening(&c, &d).await;
    d.0.failure.store(true, Ordering::SeqCst);
    wait_until(|| c.core.lock().unwrap().snapshot().mode == "manual").await;
    assert!(d.0.stopped.load(Ordering::SeqCst));
    assert_eq!(d.0.opens.load(Ordering::SeqCst), 1);
    let next = Devices::default();
    listening(&c, &next).await;
    vocalnotes_lib::app::close::begin(&c).unwrap();
    assert!(next.0.stopped.load(Ordering::SeqCst));
    assert!(c.core.lock().unwrap().snapshot().close_pending);
    assert_eq!(c.core.lock().unwrap().snapshot().listening_state, "stopped");
}

#[tokio::test]
async fn initial_lock_event_refuses_capture_even_if_monitor_reports_ready_afterwards() {
    let c = Controller::new(Probe::new("note".into()));
    let d = Devices::default();
    d.0.invalidate_before_ready.store(true, Ordering::SeqCst);
    c.start_with_devices(false, d.clone()).unwrap();
    wait_until(|| c.core.lock().unwrap().snapshot().mode == "manual").await;
    assert_eq!(d.0.key_checks.load(Ordering::SeqCst), 0);
    assert_eq!(d.0.opens.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn stop_during_device_open_discards_late_capture_without_disturbing_new_session() {
    let c = Controller::new(Probe::new("note".into()));
    let d = Devices::default();
    d.0.hold_capture.store(true, Ordering::SeqCst);
    c.start_with_devices(false, d.clone()).unwrap();
    wait_until(|| d.0.opens.load(Ordering::SeqCst) == 1).await;
    assert_eq!(
        c.core.lock().unwrap().snapshot().listening_state,
        "starting"
    );
    c.stop(false, "arrêt pendant ouverture");
    let next = Devices::default();
    listening(&c, &next).await;
    d.0.hold_capture.store(false, Ordering::SeqCst);
    wait_until(|| d.0.stopped.load(Ordering::SeqCst)).await;
    assert_eq!(
        c.core.lock().unwrap().snapshot().listening_state,
        "listening"
    );
    assert!(!next.0.stopped.load(Ordering::SeqCst));
    c.stop(false, "fin");
}

#[test]
fn already_cancelled_native_monitor_never_subscribes_or_calls_back() {
    let flag = Arc::new(AtomicBool::new(true));
    assert!(
        vocalnotes_lib::platform::lifecycle::LifecycleGuard::start_guarded(flag, || panic!(
            "must not subscribe"
        ))
        .is_err()
    );
}

#[tokio::test]
async fn ten_fixed_interruptions_discard_late_transcripts_and_edits_without_reopening_capture() {
    use vocalnotes_lib::{
        platform::probe::{Action, Intervention},
        ports::interpreter::{EditOperation, Status, Target},
    };
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("../../tests/fixtures/openai-fidelity.json")).unwrap();
    let cases = corpus["interruptions"].as_array().unwrap();
    assert_eq!(cases.len(), 10);
    for case in cases {
        let c = Controller::new(Probe::new("note".into()));
        let d = Devices::default();
        listening(&c, &d).await;
        let ticket = {
            let mut core = c.core.lock().unwrap();
            core.set_transcript_enabled(true).unwrap();
            if case["during"] == "partial" {
                core.transcript_delta("provisoire fictif");
            } else {
                assert_eq!(case["during"], "interpretation");
                core.transcript_done("final fictif");
                core.processing();
            }
            core.voice_ticket_for(1, 1).unwrap()
        };
        match case["event"].as_str().unwrap() {
            "pause" => c.stop(true, "pause"),
            "stop" => c.stop(false, "arrêt"),
            "lock" | "sleep" => (d.0.callback.lock().unwrap().clone().unwrap())(),
            "close" => {
                vocalnotes_lib::app::close::begin(&c).unwrap();
            }
            _ => panic!("unknown fixed interruption"),
        };
        assert!(d.0.stopped.load(Ordering::SeqCst), "{case}");
        let mut core = c.core.lock().unwrap();
        core.transcript_done("final tardif fictif");
        for _duplicate in 0..2 {
            let late = Intervention {
                status: Status::Resolved,
                message: String::new(),
                actions: vec![Action::Edit {
                    operation: EditOperation::Replace {
                        target: Target {
                            block_id: "b0".into(),
                            expected: "note".into(),
                        },
                        text: "modification interdite".into(),
                    },
                }],
            };
            assert!(core.commit_voice(&ticket, late).is_err(), "{case}");
        }
        let snapshot = core.snapshot();
        assert_eq!(snapshot.source, "note");
        assert_eq!(snapshot.revision, 0);
        assert!(snapshot.drafting_intention.is_none());
        assert!(snapshot.transcript_provisional.is_empty());
        assert!(snapshot.transcript_final.is_empty());
        drop(core);
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(d.0.opens.load(Ordering::SeqCst), 1);
    }
}

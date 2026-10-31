//! Opt-in native prototype checks: synthetic files and a separate keyring entry.
//! Never opens a microphone, contacts a provider or reads the application's key.
use std::{fs, path::Path, time::SystemTime};
use vocalnotes_lib::platform::probe::{Patch, Probe, Proposal};

fn changed(note: &mut Probe, text: &str) {
    let s = note.snapshot();
    assert!(
        note.manual(Proposal {
            transaction_id: "synthetic-edit".into(),
            note_id: s.note_id,
            manual_period: s.manual_period,
            base_revision: s.revision,
            patches: vec![Patch {
                from: 0,
                to: 0,
                expected: String::new(),
                text: text.into()
            }],
        })
        .accepted
    );
}

fn files(root: &Path) {
    fs::create_dir(root).unwrap();
    let source = "été\r\n\r\n```text\r\nOPAQUE  \r\n```\r\n\r\nfin  \r\n";
    let original = root.join("source.md");
    fs::write(&original, source).unwrap();
    let mut note = Probe::new(String::new());
    note.open(&original).unwrap();
    assert_eq!(note.snapshot().source.as_bytes(), source.as_bytes());
    changed(&mut note, "Avant : ");
    let expected = format!("Avant : {source}");
    let opaque = note
        .snapshot()
        .blocks
        .into_iter()
        .find(|b| b.kind == "opaque")
        .unwrap();
    assert_eq!(opaque.raw, "```text\r\nOPAQUE  \r\n```");
    let existing = root.join("existing.md");
    fs::write(&existing, "existing sentinel").unwrap();
    assert!(note.save_copy(&existing).is_err());
    assert_eq!(fs::read(&existing).unwrap(), b"existing sentinel");
    assert!(note.snapshot().dirty);
    assert_eq!(note.snapshot().source, expected);
    assert!(note.save_copy(&root.join("missing/copy.md")).is_err());
    assert_eq!(note.snapshot().source, expected);
    #[cfg(unix)]
    {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let denied = root.join("denied");
        fs::create_dir(&denied).unwrap();
        fs::set_permissions(&denied, fs::Permissions::from_mode(0o500)).unwrap();
        let refusal = note.save_copy(&denied.join("copy.md"));
        fs::set_permissions(&denied, fs::Permissions::from_mode(0o700)).unwrap();
        assert!(refusal.is_err(), "Run as the desktop user, never root");
        let link = root.join("link.md");
        symlink(&existing, &link).unwrap();
        assert!(note.save_copy(&link).is_err());
        assert_eq!(fs::read(&existing).unwrap(), b"existing sentinel");
        let mut separate = Probe::new(String::new());
        assert!(separate.open(&link).is_err());
    }
    let copy = root.join("copy.md");
    note.save_copy(&copy).unwrap();
    assert_eq!(fs::read(&copy).unwrap(), expected.as_bytes());
    assert_eq!(fs::read(&original).unwrap(), source.as_bytes());
    let invalid = root.join("invalid.md");
    fs::write(&invalid, [0xff, 0xfe]).unwrap();
    assert!(note.open(&invalid).is_err());
    assert_eq!(note.snapshot().source, expected);
    let mut unsaved = Probe::new(String::new());
    changed(&mut unsaved, "unsaved synthetic note");
    assert!(!unsaved.request_close());
    assert_eq!(unsaved.snapshot().mode, "manual");
    assert!(unsaved.snapshot().dirty);
    assert_eq!(unsaved.snapshot().source, "unsaved synthetic note");
    println!(
        "PASS files: exclusive copy, real permission failure, symlink refusal, UTF-8/CRLF/opaque preservation, failed open and close retention"
    );
    fs::remove_dir_all(root).unwrap();
}

fn write_limit(root: &Path) {
    // Launch only in a child with RLIMIT_FSIZE=1 KiB and SIGXFSZ ignored.
    fs::create_dir(root).unwrap();
    let mut note = Probe::new(String::new());
    changed(&mut note, &"x".repeat(32768));
    let path = root.join("partial.md");
    let error = note.save_copy(&path).unwrap_err();
    assert!(error.starts_with("Écriture incomplète"));
    assert!(note.snapshot().dirty);
    assert_eq!(note.snapshot().source.len(), 32768);
    assert!(fs::metadata(&path).unwrap().len() < 32768);
    println!(
        "PASS write-limit: real partial write refused, dirty note retained; partial destination explicitly reported"
    );
    fs::remove_dir_all(root).unwrap();
}

fn keyring(unavailable: bool) {
    let id = format!(
        "prototype-validation-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let entry = keyring::Entry::new("fr.vocalnotes.validation", &id).unwrap();
    if unavailable {
        assert!(entry.set_password("synthetic-probe-only").is_err());
        assert!(entry.get_password().is_err());
        println!("PASS keyring-unavailable: real backend refused; no fallback storage");
        return;
    }
    assert!(matches!(entry.get_password(), Err(keyring::Error::NoEntry)));
    let result = (|| {
        entry.set_password("synthetic-probe-one")?;
        assert_eq!(
            zeroize::Zeroizing::new(entry.get_password()?).as_str(),
            "synthetic-probe-one"
        );
        entry.set_password("synthetic-probe-two")?;
        assert_eq!(
            zeroize::Zeroizing::new(entry.get_password()?).as_str(),
            "synthetic-probe-two"
        );
        Ok::<_, keyring::Error>(())
    })();
    let cleanup = entry.delete_credential();
    // Never format a backend error: it may contain private service details.
    assert!(result.is_ok(), "synthetic keyring round-trip failed");
    assert!(cleanup.is_ok(), "synthetic keyring entry cleanup failed");
    assert!(matches!(entry.get_password(), Err(keyring::Error::NoEntry)));
    println!(
        "PASS keyring: real Secret Service create/replace/read/delete, isolated entry removed; application credential untouched"
    );
}

fn lifecycle_unavailable() {
    // A process-local missing bus, never disconnect the user's GNOME session.
    assert_eq!(
        std::env::var("DBUS_SESSION_BUS_ADDRESS").unwrap(),
        "unix:path=/tmp/vocalnotes-validation-no-session-bus"
    );
    assert!(!Path::new("/tmp/vocalnotes-validation-no-session-bus").exists());
    let controller =
        vocalnotes_lib::platform::controller::Controller::new(Probe::new(String::new()));
    controller.start_explicit(false).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let snapshot = controller.core.lock().unwrap().snapshot();
        if snapshot.mode == "manual" {
            assert_eq!(snapshot.stt_requests, 0);
            assert_eq!(snapshot.llm_requests, 0);
            assert_eq!(snapshot.audio_ms, 0);
            assert!(
                snapshot.message.contains("surveillance") || snapshot.message.contains("GNOME")
            );
            let mut core = controller.core.lock().unwrap();
            changed(&mut core, "Manual remains usable.");
            assert_eq!(core.snapshot().source, "Manual remains usable.");
            println!(
                "PASS lifecycle-unavailable: real controller returned to manual before capture/providers; manual editing remains usable"
            );
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "controller did not fail closed"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

fn lifecycle_loss() {
    use std::{
        io::Write,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
    };
    let lost = Arc::new(AtomicBool::new(false));
    let callback = lost.clone();
    let guard = vocalnotes_lib::platform::lifecycle::LifecycleGuard::start(move || {
        callback.store(true, Ordering::Release);
    })
    .expect("real GNOME lifecycle initialization failed");
    println!("READY lifecycle monitor connected");
    std::io::stdout().flush().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while !lost.load(Ordering::Acquire) {
        assert!(
            std::time::Instant::now() < deadline,
            "OS connection loss was not reported"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    drop(guard);
    println!(
        "PASS lifecycle-loss: real GNOME monitor reported its disconnected session bus; no microphone/providers"
    );
}

fn main() {
    let mode = std::env::args()
        .nth(1)
        .expect("files | write-limit | keyring | keyring-unavailable");
    let root = std::env::temp_dir().join(format!(
        "vocalnotes-native-validation-{}",
        std::process::id()
    ));
    match mode.as_str() {
        "files" => files(&root),
        "write-limit" => write_limit(&root),
        "keyring" => keyring(false),
        "keyring-unavailable" => keyring(true),
        "lifecycle-unavailable" => lifecycle_unavailable(),
        "lifecycle-loss" => lifecycle_loss(),
        _ => panic!("unknown validation mode"),
    }
}

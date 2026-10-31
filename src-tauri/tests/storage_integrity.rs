use std::{
    fs, io,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use vocalnotes_lib::storage::markdown_file::{MarkdownFile, SaveOutcome, SaveStage};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "vocalnotes-storage-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn controlled_save_preserves_opaque_crlf_and_unmodified_bytes() {
    let f = Fixture::new();
    let path = f.path("note.md");
    let old = "été\r\n\r\n```text\r\nopaque  \r\n```\r\n\r\nfin  \r\n";
    fs::write(&path, old).unwrap();
    let (mut file, source) = MarkdownFile::open(&path).unwrap();
    assert_eq!(source, old);
    let new = old.replacen("été", "hiver", 1);
    assert!(matches!(file.save(&new).unwrap(), SaveOutcome::Saved));
    assert_eq!(fs::read(&path).unwrap(), new.as_bytes());
    assert_eq!(fs::read_dir(&f.0).unwrap().count(), 1);
    assert!(matches!(file.save(&new).unwrap(), SaveOutcome::Saved));
}

#[test]
fn external_change_before_save_preserves_both_versions() {
    let f = Fixture::new();
    let path = f.path("note.md");
    fs::write(&path, "initial").unwrap();
    let (mut file, _) = MarkdownFile::open(&path).unwrap();
    fs::write(&path, "external").unwrap();
    let SaveOutcome::Conflict { local, external } = file.save("local").unwrap() else {
        panic!("conflict expected")
    };
    assert_eq!(fs::read_to_string(&path).unwrap(), "external");
    assert_eq!(fs::read_to_string(local).unwrap(), "local");
    assert_eq!(fs::read_to_string(external).unwrap(), "external");
}

#[test]
fn race_at_exchange_keeps_displaced_external_note_without_rollback() {
    let f = Fixture::new();
    let path = f.path("note.md");
    fs::write(&path, "initial").unwrap();
    let (mut file, _) = MarkdownFile::open(&path).unwrap();
    let outcome = file
        .save_observed("local", |stage| {
            if stage == SaveStage::BeforeExchange {
                fs::write(&path, "external raced")?;
            }
            Ok(())
        })
        .unwrap();
    let SaveOutcome::Conflict { local, external } = outcome else {
        panic!("race not reported")
    };
    assert_eq!(fs::read_to_string(local).unwrap(), "local");
    assert_eq!(fs::read_to_string(external).unwrap(), "external raced");
    assert_eq!(fs::read_to_string(&path).unwrap(), "local");
}

#[test]
fn interruption_before_exchange_retains_original_and_prepared_note() {
    let f = Fixture::new();
    let path = f.path("note.md");
    fs::write(&path, "original").unwrap();
    let (mut file, _) = MarkdownFile::open(&path).unwrap();
    let failure = file
        .save_observed("local", |stage| {
            if stage == SaveStage::Prepared {
                Err(io::Error::other("synthetic interruption"))
            } else {
                Ok(())
            }
        })
        .unwrap_err();
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    assert!(
        failure
            .recovery_paths
            .iter()
            .any(|p| fs::read_to_string(p).ok().as_deref() == Some("local"))
    );
}

#[test]
fn concurrent_change_after_exchange_is_preserved_and_reported() {
    let f = Fixture::new();
    let path = f.path("note.md");
    fs::write(&path, "original").unwrap();
    let (mut file, _) = MarkdownFile::open(&path).unwrap();
    let outcome = file
        .save_observed("local", |stage| {
            if stage == SaveStage::Exchanged {
                fs::write(&path, "later external")?;
            }
            Ok(())
        })
        .unwrap();
    let SaveOutcome::Conflict { local, external } = outcome else {
        panic!("post-exchange conflict expected")
    };
    assert_eq!(fs::read_to_string(&path).unwrap(), "later external");
    assert_eq!(fs::read_to_string(local).unwrap(), "local");
    assert_eq!(fs::read_to_string(external).unwrap(), "original");
}

#[cfg(unix)]
#[test]
fn permissions_and_symlinks_fail_without_touching_existing_notes() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let f = Fixture::new();
    let path = f.path("note.md");
    fs::write(&path, "original").unwrap();
    symlink(&path, f.path("link.md")).unwrap();
    assert!(MarkdownFile::open(&f.path("link.md")).is_err());
    symlink(&f.0, f.path("parent-link")).unwrap();
    assert!(MarkdownFile::open(&f.path("parent-link/note.md")).is_err());
    let (mut file, _) = MarkdownFile::open(&path).unwrap();
    fs::set_permissions(&f.0, fs::Permissions::from_mode(0o500)).unwrap();
    let result = file.save("local");
    fs::set_permissions(&f.0, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(result.is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");
}

#[cfg(unix)]
#[test]
fn replacement_preserves_mode_and_detects_same_content_new_identity() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    let path = f.path("note.md");
    fs::write(&path, "same").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
    let (mut file, _) = MarkdownFile::open(&path).unwrap();
    file.save("next").unwrap();
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o640
    );
    let replacement = f.path("replacement.md");
    fs::write(&replacement, "next").unwrap();
    fs::rename(replacement, &path).unwrap();
    assert!(matches!(
        file.save("local").unwrap(),
        SaveOutcome::Conflict { .. }
    ));
}

#[test]
fn accepted_manual_commit_autosaves_but_conflict_retains_dirty_note() {
    use vocalnotes_lib::platform::probe::{Patch, Probe, Proposal};
    let f = Fixture::new();
    let path = f.path("note.md");
    fs::write(&path, "avant").unwrap();
    let mut note = Probe::new(String::new());
    note.open(&path).unwrap();
    for (index, expected, text) in [(0, "avant", "après"), (1, "après", "local")] {
        if index == 1 {
            fs::write(&path, "external").unwrap();
        }
        let s = note.snapshot();
        let ack = note.manual(Proposal {
            transaction_id: format!("tx{index}"),
            note_id: s.note_id,
            manual_period: s.manual_period,
            base_revision: s.revision,
            patches: vec![Patch {
                from: 0,
                to: expected.len(),
                expected: expected.into(),
                text: text.into(),
            }],
        });
        assert!(ack.accepted);
        if index == 0 {
            assert!(!ack.snapshot.dirty);
            assert_eq!(fs::read_to_string(&path).unwrap(), text);
        } else {
            assert!(ack.snapshot.dirty);
            assert!(!ack.snapshot.storage_message.is_empty());
            assert_eq!(ack.snapshot.source, "local");
            assert_eq!(fs::read_to_string(&path).unwrap(), "external");
            assert!(!note.request_close());
        }
    }
}

#[cfg(unix)]
#[test]
fn hardlinked_destination_uses_explicit_copy_instead_of_replacing_alias() {
    let f = Fixture::new();
    let path = f.path("note.md");
    let alias = f.path("alias.md");
    fs::write(&path, "original").unwrap();
    fs::hard_link(&path, &alias).unwrap();
    let (mut file, _) = MarkdownFile::open(&path).unwrap();
    let SaveOutcome::Copy { path: copy } = file.save("local").unwrap() else {
        panic!("explicit fallback expected")
    };
    assert_eq!(fs::read_to_string(copy).unwrap(), "local");
    assert_eq!(fs::read_to_string(alias).unwrap(), "original");
    assert_eq!(fs::read_to_string(path).unwrap(), "original");
}

#[test]
fn partial_write_child() {
    let Some(root) = std::env::var_os("VOCALNOTES_STORAGE_WRITE_LIMIT_TEST") else {
        return;
    };
    let path = PathBuf::from(root).join("note.md");
    let (mut file, _) = MarkdownFile::open(&path).unwrap();
    let error = file.save(&"x".repeat(32768)).unwrap_err();
    assert!(!error.recovery_paths.is_empty());
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    assert!(
        error
            .recovery_paths
            .iter()
            .any(|p| fs::metadata(p).unwrap().len() < 32768)
    );
}

#[cfg(unix)]
#[test]
fn real_partial_write_keeps_destination_and_reports_recovery_path() {
    let f = Fixture::new();
    fs::write(f.path("note.md"), "original").unwrap();
    let output = std::process::Command::new("bash")
        .args([
            "-c",
            "trap '' XFSZ; ulimit -f 1; exec \"$1\" partial_write_child --nocapture",
            "storage-write-limit",
        ])
        .arg(std::env::current_exe().unwrap())
        .env("VOCALNOTES_STORAGE_WRITE_LIMIT_TEST", &f.0)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "child failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(f.path("note.md")).unwrap(), "original");
}

#[test]
fn interrupted_process_child() {
    let Some(root) = std::env::var_os("VOCALNOTES_STORAGE_CRASH_TEST") else {
        return;
    };
    let path = PathBuf::from(root).join("note.md");
    let (mut file, _) = MarkdownFile::open(&path).unwrap();
    let after = std::env::var_os("VOCALNOTES_STORAGE_CRASH_AFTER_EXCHANGE").is_some();
    let _ = file.save_observed("local", |stage| {
        if stage
            == if after {
                SaveStage::Exchanged
            } else {
                SaveStage::Prepared
            }
        {
            std::process::exit(86)
        }
        Ok(())
    });
    panic!("crash checkpoint was not reached");
}

#[test]
fn process_exit_before_and_after_exchange_leaves_both_notes_recoverable() {
    for after in [false, true] {
        let f = Fixture::new();
        fs::write(f.path("note.md"), "original").unwrap();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap());
        child
            .args(["interrupted_process_child", "--nocapture"])
            .env("VOCALNOTES_STORAGE_CRASH_TEST", &f.0);
        if after {
            child.env("VOCALNOTES_STORAGE_CRASH_AFTER_EXCHANGE", "1");
        }
        let output = child.output().unwrap();
        assert_eq!(output.status.code(), Some(86));
        let contents: Vec<_> = fs::read_dir(&f.0)
            .unwrap()
            .map(|p| fs::read_to_string(p.unwrap().path()).unwrap())
            .collect();
        assert!(contents.iter().any(|s| s == "original"));
        assert!(contents.iter().any(|s| s == "local"));
        assert_eq!(
            fs::read_to_string(f.path("note.md")).unwrap(),
            if after { "local" } else { "original" }
        );
    }
}

#[test]
fn file_watcher_is_a_hint_and_external_content_is_never_imported_silently() {
    let f = Fixture::new();
    let path = f.path("note.md");
    fs::write(&path, "original").unwrap();
    let (file, _) = MarkdownFile::open(&path).unwrap();
    assert!(!file.changed_on_disk().unwrap());
    fs::write(&path, "external").unwrap();
    assert!(file.changed_on_disk().unwrap());
    let mut note = vocalnotes_lib::platform::probe::Probe::new(String::new());
    note.open(&path).unwrap();
    fs::write(&path, "another external").unwrap();
    note.refresh_disk_status();
    assert_eq!(note.snapshot().source, "external");
    assert!(note.snapshot().storage_message.contains("externe"));
}

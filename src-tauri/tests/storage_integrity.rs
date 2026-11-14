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

fn write_candidate(
    path: PathBuf,
    source: &str,
    expected: Option<vocalnotes_lib::storage::markdown_file::DiskVersion>,
) -> vocalnotes_lib::storage::markdown_file::WriteCandidate {
    use vocalnotes_lib::{
        app::document::{DocumentFormat, DocumentVersion},
        storage::markdown_file::{WriteCandidate, WriteContent},
    };
    WriteCandidate::new(
        WriteContent::MarkdownExport(source.into()),
        path,
        DocumentFormat::Markdown,
        DocumentVersion {
            note_id: 7,
            revision: 9,
            input_generation: 11,
        },
        expected,
    )
    .ok()
    .unwrap()
}

#[test]
fn common_writer_installs_the_exact_preparation_and_returns_its_candidate() {
    use vocalnotes_lib::storage::markdown_file::{DiskVersion, WriteResult};
    let f = Fixture::new();
    let path = f.path("candidate.md");
    let source = "<!-- vocalnotes:intention:body -->\r\ne\u{301}\r\n";
    let preparation = write_candidate(path.clone(), source, None)
        .prepare()
        .ok()
        .unwrap();
    let prepared_path = preparation.path().to_owned();
    let prepared_version = DiskVersion::observe(&prepared_path).unwrap();
    let WriteResult::Saved { candidate, version } = preparation.install() else {
        panic!("exclusive installation must succeed");
    };
    assert!(!prepared_path.exists());
    assert_eq!(version, prepared_version);
    assert_eq!(DiskVersion::observe(&path).unwrap(), version);
    assert_eq!(fs::read(&path).unwrap(), source.as_bytes());
    assert_eq!(candidate.note_version().revision, 9);
    assert_eq!(candidate.path(), path);

    let WriteResult::Saved { version: next, .. } =
        write_candidate(path.clone(), "next\r\n", Some(version.clone()))
            .prepare()
            .ok()
            .unwrap()
            .install()
    else {
        panic!("controlled replacement must succeed");
    };
    assert_ne!(next, version);
    assert_eq!(next, DiskVersion::observe(&path).unwrap());
    assert_eq!(fs::read(&path).unwrap(), b"next\r\n");
    assert_eq!(fs::read_dir(&f.0).unwrap().count(), 1);
}

#[test]
fn common_writer_never_adopts_a_preparation_modified_before_installation() {
    use vocalnotes_lib::storage::markdown_file::WriteResult;
    let f = Fixture::new();
    for existing in [false, true] {
        for during_install in [false, true] {
            let path = f.path(if existing { "existing.md" } else { "new.md" });
            let expected = if existing {
                fs::write(&path, "original").unwrap();
                Some(vocalnotes_lib::storage::markdown_file::DiskVersion::observe(&path).unwrap())
            } else {
                None
            };
            let preparation = write_candidate(path.clone(), "local", expected)
                .prepare()
                .ok()
                .unwrap();
            let stage = preparation.path().to_owned();
            if !during_install {
                fs::write(&stage, "changed preparation").unwrap();
            }
            let result = preparation.install_observed(|point| {
                if during_install && point == SaveStage::Prepared {
                    fs::write(&stage, "changed preparation")?;
                }
                Ok(())
            });
            assert!(matches!(result, WriteResult::Failed { .. }));
            assert_eq!(fs::read_to_string(stage).unwrap(), "changed preparation");
            if existing {
                assert_eq!(fs::read_to_string(path).unwrap(), "original");
            } else {
                assert!(!path.exists());
            }
        }
    }
}

#[test]
fn common_writer_rechecks_version_bound_consent_and_concurrent_creation() {
    use vocalnotes_lib::storage::markdown_file::{DiskVersion, OverwriteConsent, WriteResult};
    let f = Fixture::new();
    let path = f.path("consent.md");
    fs::write(&path, "original").unwrap();
    let expected = DiskVersion::observe(&path).unwrap();
    let preparation = write_candidate(path.clone(), "local", None)
        .with_overwrite_consent(OverwriteConsent::new(path.clone(), expected.clone()).unwrap())
        .ok()
        .unwrap()
        .prepare()
        .ok()
        .unwrap();
    fs::write(&path, "external").unwrap();
    let observed = DiskVersion::observe(&path).unwrap();
    let WriteResult::Conflict {
        expected: old,
        observed: current,
        recovery_paths,
    } = preparation.install()
    else {
        panic!("stale consent must not overwrite");
    };
    assert_eq!(old, Some(expected));
    assert_eq!(current, Some(observed));
    assert_eq!(fs::read_to_string(&path).unwrap(), "external");
    for source in ["local", "external"] {
        assert!(
            recovery_paths
                .iter()
                .any(|path| fs::read_to_string(path).ok().as_deref() == Some(source))
        );
    }
    let path = f.path("new.md");
    let preparation = write_candidate(path.clone(), "local", None)
        .prepare()
        .ok()
        .unwrap();
    let result = preparation.install_observed(|stage| {
        if stage == SaveStage::BeforeExchange {
            fs::write(&path, "concurrent")?;
        }
        Ok(())
    });
    assert!(matches!(
        result,
        WriteResult::Conflict {
            expected: None,
            observed: Some(_),
            ..
        }
    ));
    assert_eq!(fs::read_to_string(path).unwrap(), "concurrent");
}

#[test]
fn interrupted_post_replacement_keeps_original_local_and_later_external_versions() {
    use vocalnotes_lib::storage::markdown_file::{DiskVersion, WriteResult};
    let f = Fixture::new();
    let path = f.path("interrupted.md");
    fs::write(&path, "original").unwrap();
    let expected = DiskVersion::observe(&path).unwrap();
    let preparation = write_candidate(path.clone(), "local", Some(expected))
        .prepare()
        .ok()
        .unwrap();
    let result = preparation.install_observed(|stage| {
        if stage == SaveStage::Exchanged {
            fs::write(&path, "later external")?;
            return Err(io::Error::other("synthetic post-install interruption"));
        }
        Ok(())
    });
    let WriteResult::Failed { error, .. } = result else {
        panic!("interruption must fail");
    };
    assert_eq!(fs::read_to_string(path).unwrap(), "later external");
    for source in ["original", "local"] {
        assert!(
            error
                .recovery_paths
                .iter()
                .any(|path| fs::read_to_string(path).ok().as_deref() == Some(source))
        );
    }
}

#[test]
fn write_preparation_captures_exact_state_exclusively_without_touching_destination() {
    use vocalnotes_lib::{
        app::document::{DocumentFormat, DocumentVersion},
        domain::intention::DraftingIntention,
        storage::{
            markdown_file::{WriteCandidate, WriteContent},
            note_document,
        },
    };
    let f = Fixture::new();
    let path = f.path("note.von");
    fs::write(&path, "external").unwrap();
    let source = "e\u{301}\r\n\r\n<!-- vocalnotes:intention:body -->\r\n";
    let intention =
        DraftingIntention::new("".into(), "".into(), "".into(), "contexte".into()).unwrap();
    let candidate = WriteCandidate::new(
        WriteContent::Document {
            source: source.into(),
            intention: intention.clone(),
        },
        path.clone(),
        DocumentFormat::Von,
        DocumentVersion {
            note_id: 7,
            revision: 9,
            input_generation: 11,
        },
        None,
    )
    .ok()
    .unwrap();
    let first = candidate.clone().prepare().ok().unwrap();
    let second = candidate.prepare().ok().unwrap();
    assert_ne!(first.path(), second.path());
    assert_eq!(first.path().parent(), path.parent());
    assert_eq!(first.path().extension().unwrap(), "von");
    assert_eq!(fs::read_to_string(&path).unwrap(), "external");
    assert_eq!(
        fs::read(first.path()).unwrap(),
        fs::read(second.path()).unwrap()
    );
    let decoded = note_document::decode(&fs::read_to_string(first.path()).unwrap()).unwrap();
    assert_eq!(decoded.0, source);
    assert!(decoded.1 == intention);
    let retained = first.path().to_owned();
    drop(first);
    assert!(
        retained.exists(),
        "an unfinished preparation must remain recoverable"
    );
}

#[test]
fn write_candidates_reject_invalid_or_oversized_serialization_before_creating_files() {
    use vocalnotes_lib::{
        app::document::{DocumentErrorCode, DocumentFormat, DocumentVersion},
        domain::intention::DraftingIntention,
        storage::{
            markdown_file::{WriteCandidate, WriteContent},
            note_document::MAX_DOCUMENT_BYTES,
        },
    };
    let f = Fixture::new();
    let version = DocumentVersion {
        note_id: 1,
        revision: 1,
        input_generation: 1,
    };
    let failure = WriteCandidate::new(
        WriteContent::Document {
            source: "x".repeat(MAX_DOCUMENT_BYTES),
            intention: DraftingIntention::new("".into(), "".into(), "".into(), "context".into())
                .unwrap(),
        },
        f.path("large.von"),
        DocumentFormat::Von,
        version,
        None,
    )
    .err()
    .unwrap();
    assert_eq!(failure.code, DocumentErrorCode::SizeLimit);
    assert!(
        WriteCandidate::new(
            WriteContent::MarkdownExport("body".into()),
            f.path("export.von"),
            DocumentFormat::Von,
            version,
            None,
        )
        .is_err()
    );
    assert!(
        WriteCandidate::new(
            WriteContent::MarkdownExport("body".into()),
            f.path("export.txt"),
            DocumentFormat::Markdown,
            version,
            None,
        )
        .is_err()
    );
    assert_eq!(fs::read_dir(&f.0).unwrap().count(), 0);
}

#[test]
fn markdown_export_preparation_does_not_add_even_an_ambiguity_envelope() {
    use vocalnotes_lib::{
        app::document::{DocumentFormat, DocumentVersion},
        storage::markdown_file::{WriteCandidate, WriteContent},
    };
    let f = Fixture::new();
    let source = "<!-- vocalnotes:intention:ordinary user text -->\r\nété\r\n";
    let candidate = WriteCandidate::new(
        WriteContent::MarkdownExport(source.into()),
        f.path("export"),
        DocumentFormat::Markdown,
        DocumentVersion {
            note_id: 1,
            revision: 2,
            input_generation: 3,
        },
        None,
    )
    .ok()
    .unwrap();
    assert_eq!(candidate.path(), f.path("export.md"));
    let preparation = candidate.prepare().ok().unwrap();
    assert_eq!(fs::read(preparation.path()).unwrap(), source.as_bytes());
    assert!(!f.path("export.md").exists());
}

#[test]
fn note_paths_accept_both_formats_preserve_case_and_only_complete_missing_extensions() {
    use vocalnotes_lib::{
        app::document::DocumentFormat,
        storage::markdown_file::{complete_note_path, validate_note_path},
    };
    let f = Fixture::new();
    for (name, format) in [
        ("note.VON", DocumentFormat::Von),
        ("note.MD", DocumentFormat::Markdown),
    ] {
        let path = f.path(name);
        assert_eq!(validate_note_path(&path).unwrap(), format);
        assert_eq!(complete_note_path(&path, format).unwrap(), path);
        let file = MarkdownFile::create_copy(&path, "source\r\n").unwrap();
        assert_eq!(file.path(), path);
        assert_eq!(MarkdownFile::open(&path).unwrap().1, "source\r\n");
    }
    assert_eq!(
        complete_note_path(&f.path("note"), DocumentFormat::Von).unwrap(),
        f.path("note.von")
    );
    assert_eq!(
        complete_note_path(&f.path("note"), DocumentFormat::Markdown).unwrap(),
        f.path("note.md")
    );
    assert!(complete_note_path(&f.path("note.MD"), DocumentFormat::Von).is_err());
    for path in [
        PathBuf::from("relative.von"),
        f.path("..").join("note.von"),
        f.path("bad.txt"),
        f.path("bad."),
        f.path("bad\0.von"),
        f.path(&format!("{}.von", "x".repeat(4096))),
        PathBuf::from(r"\\server\share\note.von"),
        PathBuf::from("//server/share/note.md"),
    ] {
        assert!(validate_note_path(&path).is_err(), "{}", path.display());
    }
    #[cfg(windows)]
    for path in [
        r"C:\note:stream.von",
        r"\\?\UNC\server\share\note.von",
        r"\\.\C:\note.von",
    ] {
        assert!(validate_note_path(std::path::Path::new(path)).is_err());
    }
    #[cfg(target_os = "linux")]
    {
        let upper = MarkdownFile::create_copy(&f.path("Case.von"), "upper").unwrap();
        let lower = MarkdownFile::create_copy(&f.path("case.von"), "lower").unwrap();
        assert_ne!(upper.path(), lower.path());
        assert_eq!(MarkdownFile::open(upper.path()).unwrap().1, "upper");
        assert_eq!(MarkdownFile::open(lower.path()).unwrap().1, "lower");
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
fn exclusive_install_never_replaces_a_concurrent_creation() {
    let f = Fixture::new();
    let path = f.path("new.von");
    let failure = MarkdownFile::create_copy_observed(&path, "local\r\n", |stage| {
        if stage == SaveStage::BeforeExchange {
            fs::write(&path, "concurrent")?;
        }
        Ok(())
    })
    .err()
    .unwrap();
    assert_eq!(
        failure.code,
        vocalnotes_lib::app::document::DocumentErrorCode::ExternalConflict
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), "concurrent");
    assert!(
        failure
            .recovery_paths
            .iter()
            .any(|p| fs::read_to_string(p).ok().as_deref() == Some("local\r\n"))
    );
}

#[test]
fn changed_installation_is_not_adopted_and_the_local_preparation_is_recoverable() {
    let f = Fixture::new();
    let path = f.path("new.von");
    let failure = MarkdownFile::create_copy_observed(&path, "local", |stage| {
        if stage == SaveStage::Exchanged {
            fs::write(&path, "external after install")?;
        }
        Ok(())
    })
    .err()
    .unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "external after install");
    assert!(
        failure
            .recovery_paths
            .iter()
            .any(|p| fs::read_to_string(p).ok().as_deref() == Some("local"))
    );
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
fn accepted_edits_never_write_and_explicit_save_detects_external_conflicts() {
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
        assert!(ack.snapshot.dirty);
        assert_eq!(ack.snapshot.source, text);
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            if index == 0 { "avant" } else { "external" }
        );
        if index == 1 {
            assert!(ack.snapshot.storage_message.is_empty());
            assert!(note.save_current().is_err());
            assert!(!note.snapshot().storage_message.is_empty());
            assert_eq!(fs::read_to_string(&path).unwrap(), "external");
            assert!(!note.request_close());
        }
    }
}

#[test]
fn dirty_compares_exact_persistent_values_and_excludes_temporary_instructions() {
    use vocalnotes_lib::domain::{
        DictationTarget,
        note::{Note, Patch, Proposal},
    };
    let mut note = Note::new(String::new());
    assert!(!note.snapshot().dirty);
    note.set_dictation_mode(DictationTarget::Instructions);
    note.set_instructions("temporary".into()).unwrap();
    assert!(!note.snapshot().dirty);
    note.set_context("context".into()).unwrap();
    assert!(note.snapshot().dirty);
    note.set_context(String::new()).unwrap();
    assert!(!note.snapshot().dirty);
    for (index, expected, text) in [(0, "", "new text"), (1, "new text", "")] {
        let before = note.snapshot();
        let ack = note.manual(Proposal {
            transaction_id: format!("baseline-{index}"),
            note_id: before.note_id,
            manual_period: before.manual_period,
            base_revision: before.revision,
            patches: vec![Patch {
                from: 0,
                to: expected.len(),
                expected: expected.into(),
                text: text.into(),
            }],
        });
        assert!(ack.accepted);
        assert_eq!(ack.snapshot.dirty, index == 0);
        assert!(ack.snapshot.revision > before.revision);
        assert_eq!(ack.snapshot.instructions, "temporary");
    }
}

#[test]
fn hardlinked_destination_fails_with_recovery_instead_of_adopting_a_copy() {
    let f = Fixture::new();
    let path = f.path("note.md");
    let alias = f.path("alias.md");
    fs::write(&path, "original").unwrap();
    fs::hard_link(&path, &alias).unwrap();
    let (mut file, _) = MarkdownFile::open(&path).unwrap();
    let failure = file.save("local").unwrap_err();
    assert_eq!(
        failure.code,
        vocalnotes_lib::app::document::DocumentErrorCode::UnsupportedStorage
    );
    assert!(
        failure
            .recovery_paths
            .iter()
            .any(|p| fs::read_to_string(p).ok().as_deref() == Some("local"))
    );
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

#[test]
fn intention_metadata_roundtrips_without_changing_body_bytes_or_html_boundaries() {
    use vocalnotes_lib::{
        domain::intention::DraftingIntention,
        storage::note_document::{decode, encode},
    };
    let body = "# Été\r\n\r\n```text\r\nopaque  \r\n```\r\n";
    let intention = DraftingIntention::new(
        "Résumé --> <script> & été".into(),
        "e-mail".into(),
        "cordial".into(),
        "Réserve\nà conserver".into(),
    )
    .unwrap();
    let encoded = encode(body, &intention);
    assert_eq!(encoded.matches("-->").count(), 1);
    let (decoded, loaded) = decode(&encoded).unwrap();
    assert_eq!(decoded.as_bytes(), body.as_bytes());
    assert!(loaded == intention);
    assert_eq!(encode(body, &Default::default()), body);
    let nested_body = encoded.clone();
    let (decoded, empty) = decode(&encode(&nested_body, &Default::default())).unwrap();
    assert_eq!(decoded, nested_body);
    assert!(empty.is_empty());
    for corrupt in [
        "<!-- vocalnotes:intention:v2\n{}\n-->\n\nbody",
        "<!-- vocalnotes:intention:v1\nnot-json\n-->\n\nbody",
        "<!-- vocalnotes:intention:v1\n{}",
    ] {
        assert!(decode(corrupt).is_err());
    }
}

#[test]
fn context_edits_preserve_legacy_metadata_and_never_serialize_instructions() {
    use vocalnotes_lib::{
        domain::{DictationTarget, intention::DraftingIntention, note::Note},
        storage::note_document::{decode, encode},
    };
    let f = Fixture::new();
    let path = f.path("context.md");
    let legacy = DraftingIntention::new(
        "purpose".into(),
        "form".into(),
        "tone".into(),
        "ancien contexte".into(),
    )
    .unwrap();
    fs::write(&path, encode("# Titre\n\nTexte.", &legacy)).unwrap();

    let mut note = Note::new(String::new());
    note.open(&path).unwrap();
    note.set_context("contexte mis à jour".into()).unwrap();
    note.set_dictation_mode(DictationTarget::Instructions);
    note.set_instructions("consigne temporaire".into()).unwrap();
    assert!(note.snapshot().dirty);
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        encode("# Titre\n\nTexte.", &legacy)
    );
    let saved = f.path("context.von");
    note.save_copy(&saved).unwrap();
    assert!(!note.snapshot().dirty);
    let persisted = fs::read_to_string(&saved).unwrap();
    let (body, loaded) = decode(&persisted).unwrap();

    assert_eq!(body, "# Titre\n\nTexte.");
    assert_eq!(loaded.purpose(), "purpose");
    assert_eq!(loaded.form(), "form");
    assert_eq!(loaded.tone(), "tone");
    assert_eq!(loaded.context(), "contexte mis à jour");
    assert!(!persisted.contains("consigne temporaire"));
    assert!(note.set_context("x".repeat(4096)).is_err());
    assert_eq!(note.snapshot().context, "contexte mis à jour");
    let snapshot = note.snapshot();
    note.set_dictation_mode(DictationTarget::Content);
    assert_eq!(
        note.snapshot().input_generation,
        snapshot.input_generation + 1
    );
    assert!(note.snapshot().instructions.is_empty());
    note.set_dictation_mode(DictationTarget::Instructions);
    note.set_instructions("à effacer au remplacement".into())
        .unwrap();
    note.loaded("Autre note.".into());
    assert_eq!(note.snapshot().dictation_mode, DictationTarget::Content);
    assert!(note.snapshot().instructions.is_empty());
    note.set_dictation_mode(DictationTarget::Instructions);
    note.set_instructions("à effacer à la fermeture".into())
        .unwrap();
    assert!(note.request_close());
    assert!(note.snapshot().instructions.is_empty());
}

#[test]
fn metadata_only_external_conflict_preserves_both_complete_documents() {
    use vocalnotes_lib::{
        domain::{intention::DraftingIntention, note::Note},
        ports::interpreter::decode_intervention,
        storage::note_document::{decode, encode},
    };
    let f = Fixture::new();
    let path = f.path("note.md");
    let profile = |purpose: &str| {
        DraftingIntention::new(purpose.into(), "e-mail".into(), "".into(), "".into()).unwrap()
    };
    fs::write(&path, encode("Note inchangée.", &profile("Initial"))).unwrap();
    let mut note = Note::new(String::new());
    note.open(&path).unwrap();
    let external = encode("Note inchangée.", &profile("Externe"));
    fs::write(&path, &external).unwrap();
    note.start_voice(false).unwrap();
    assert_eq!(note.drafting_intention().purpose(), "Initial");
    let ticket = note.voice_ticket(1).unwrap();
    note.commit_voice(&ticket, decode_intervention(r#"{"status":"resolved","message":"","actions":[{"type":"drafting_intent","text":"Local","form":"e-mail","tone":"","context":""}]}"#).unwrap()).unwrap();
    assert!(note.snapshot().dirty);
    assert!(note.snapshot().storage_message.is_empty());
    assert!(note.save_current().is_err());
    assert!(note.snapshot().storage_message.contains("Conflit disque"));
    assert_eq!(fs::read_to_string(&path).unwrap(), external);
    let recovered: Vec<_> = fs::read_dir(&f.0)
        .unwrap()
        .map(|e| fs::read_to_string(e.unwrap().path()).unwrap())
        .collect();
    assert!(
        recovered
            .iter()
            .any(|d| decode(d).unwrap().1.purpose() == "Local")
    );
    assert!(
        recovered
            .iter()
            .any(|d| decode(d).unwrap().1.purpose() == "Externe")
    );
    let resolved = f.path("resolue.md");
    note.save_copy(&resolved).unwrap();
    note.stop("Manuel");
    let mut reopened = Note::new(String::new());
    reopened.open(&resolved).unwrap();
    assert_eq!(reopened.snapshot().source, "Note inchangée.");
    assert_eq!(reopened.drafting_intention().purpose(), "Local");
    assert_eq!(fs::read_to_string(&path).unwrap(), external);
    reopened.loaded("Autre note.".into());
    assert!(reopened.drafting_intention().is_empty());
}

#[cfg(target_os = "linux")]
#[test]
fn linux_extended_acl_refuses_replacement_and_preserves_recovery() {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    use vocalnotes_lib::app::document::DocumentErrorCode;
    let f = Fixture::new();
    let path = f.path("acl.md");
    fs::write(&path, "original").unwrap();
    // Linux POSIX ACL xattr v2, including one named user: ordinary mode bits
    // alone cannot preserve this access policy across an inode exchange.
    let mut acl = 2_u32.to_le_bytes().to_vec();
    for (tag, permissions, id) in [
        (1_u16, 6_u16, u32::MAX),
        (2, 4, unsafe { libc::geteuid() }),
        (4, 4, u32::MAX),
        (16, 4, u32::MAX),
        (32, 0, u32::MAX),
    ] {
        acl.extend(tag.to_le_bytes());
        acl.extend(permissions.to_le_bytes());
        acl.extend(id.to_le_bytes());
    }
    let native_path = CString::new(path.as_os_str().as_bytes()).unwrap();
    assert_eq!(
        unsafe {
            libc::setxattr(
                native_path.as_ptr(),
                c"system.posix_acl_access".as_ptr(),
                acl.as_ptr().cast(),
                acl.len(),
                0,
            )
        },
        0,
        "native ACL prerequisite: {}",
        io::Error::last_os_error()
    );
    let (mut file, _) = MarkdownFile::open(&path).unwrap();
    let failure = file.save("local").unwrap_err();
    assert_eq!(failure.code, DocumentErrorCode::UnsupportedStorage);
    assert_eq!(fs::read(&path).unwrap(), b"original");
    let mut observed = vec![0_u8; acl.len()];
    assert_eq!(
        unsafe {
            libc::getxattr(
                native_path.as_ptr(),
                c"system.posix_acl_access".as_ptr(),
                observed.as_mut_ptr().cast(),
                observed.len(),
            )
        },
        acl.len() as isize
    );
    assert_eq!(observed, acl);
    assert!(
        failure
            .recovery_paths
            .iter()
            .any(|p| fs::read(p).ok().as_deref() == Some(b"local"))
    );
}

#[cfg(target_os = "linux")]
#[test]
fn linux_relocated_parent_never_installs_into_a_substituted_directory() {
    use vocalnotes_lib::storage::markdown_file::{DiskVersion, WriteResult};
    let f = Fixture::new();
    let original = f.path("parent");
    let moved = f.path("moved");
    fs::create_dir(&original).unwrap();
    let path = original.join("note.md");
    fs::write(&path, "original").unwrap();
    let prepared = write_candidate(
        path.clone(),
        "local",
        Some(DiskVersion::observe(&path).unwrap()),
    )
    .prepare()
    .ok()
    .unwrap();
    fs::rename(&original, &moved).unwrap();
    fs::create_dir(&original).unwrap();
    fs::write(&path, "other writer").unwrap();
    assert!(!matches!(prepared.install(), WriteResult::Saved { .. }));
    assert_eq!(fs::read(&path).unwrap(), b"other writer");
    assert_eq!(fs::read(moved.join("note.md")).unwrap(), b"original");
    assert!(
        fs::read_dir(&moved)
            .unwrap()
            .any(|entry| fs::read(entry.unwrap().path()).ok().as_deref() == Some(b"local"))
    );
}

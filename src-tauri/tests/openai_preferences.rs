use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use vocalnotes_lib::config::preferences::{LLM_MODEL, PreferencesStore, STT_MODEL};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::current_dir().unwrap().join(format!(
            ".preferences-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self) -> PathBuf {
        self.0.join("preferences.json")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn data(
    stt_provider: &str,
    stt: &str,
    llm_provider: &str,
    llm: &str,
    last_note: Option<&str>,
) -> String {
    serde_json::json!({
        "stt": {"provider": stt_provider, "model": stt},
        "llm": {"provider": llm_provider, "model": llm},
        "lastNote": last_note
    })
    .to_string()
}

#[test]
fn fresh_and_existing_openai_are_not_migrated() {
    let f = Fixture::new();
    let fresh = PreferencesStore::open(f.path()).unwrap();
    assert!(!fresh.was_migrated());
    assert_eq!(fresh.get().stt.model, STT_MODEL);
    assert_eq!(fresh.get().llm.model, LLM_MODEL);
    assert!(!f.path().exists());
    let original = data(
        "openai",
        "custom-stt",
        "openai",
        "custom-llm",
        Some("/notes/test.md"),
    );
    fs::write(f.path(), &original).unwrap();
    let store = PreferencesStore::open(f.path()).unwrap();
    assert!(!store.was_migrated());
    assert_eq!(store.get().stt.model, "custom-stt");
    assert_eq!(store.get().llm.model, "custom-llm");
    assert_eq!(store.get().last_note, Some(PathBuf::from("/notes/test.md")));
    assert_eq!(fs::read_to_string(f.path()).unwrap(), original);
}

#[test]
fn each_historical_selection_migrates_once_and_preserves_note_and_other_selection() {
    for (sp, sm, lp, lm, expected_stt, expected_llm) in [
        (
            "mistral",
            "custom-voxtral",
            "mistral",
            "custom-mistral",
            STT_MODEL,
            LLM_MODEL,
        ),
        (
            "mistral",
            "custom-voxtral",
            "openai",
            "custom-llm",
            STT_MODEL,
            "custom-llm",
        ),
        (
            "openai",
            "custom-stt",
            "mistral",
            "custom-mistral",
            "custom-stt",
            LLM_MODEL,
        ),
    ] {
        let f = Fixture::new();
        fs::write(f.path(), data(sp, sm, lp, lm, Some("/notes/test.md"))).unwrap();
        let store = PreferencesStore::open(f.path()).unwrap();
        assert!(store.was_migrated());
        let value = store.get();
        assert_eq!(
            (&*value.stt.provider, &*value.llm.provider),
            ("openai", "openai")
        );
        assert_eq!(
            (&*value.stt.model, &*value.llm.model),
            (expected_stt, expected_llm)
        );
        assert_eq!(value.last_note, Some(PathBuf::from("/notes/test.md")));
        let persisted = fs::read(f.path()).unwrap();
        let restarted = PreferencesStore::open(f.path()).unwrap();
        assert!(!restarted.was_migrated());
        assert_eq!(restarted.get(), value);
        assert_eq!(fs::read(f.path()).unwrap(), persisted);
    }
}

#[test]
fn rejects_invalid_data_without_rewriting() {
    let f = Fixture::new();
    for original in [
        "{".to_owned(),
        data("unknown", "model", "mistral", "valid", None),
        data("mistral", "", "openai", "valid", None),
        data("mistral", "valid", "openai", "bad model", None),
        data("mistral", "valid", "openai", "valid", Some("relative.md")),
        r#"{"stt":{"provider":"mistral","model":"valid"},"llm":{"provider":"openai","model":"valid"},"lastNote":null,"unexpected":true}"#.to_owned(),
        "x".repeat(16_385),
    ] {
        fs::write(f.path(), &original).unwrap();
        assert!(PreferencesStore::open(f.path()).is_err());
        assert_eq!(fs::read_to_string(f.path()).unwrap(), original);
    }
    fs::remove_file(f.path()).unwrap();
    fs::create_dir(f.path()).unwrap();
    assert!(PreferencesStore::open(f.path()).is_err());
}

#[test]
fn size_boundary_and_save_failure_do_not_change_active_values() {
    let f = Fixture::new();
    let mut original = data("openai", "valid-stt", "openai", "valid-llm", None);
    original.push_str(&" ".repeat(16_384 - original.len()));
    fs::write(f.path(), &original).unwrap();
    let mut store = PreferencesStore::open(f.path()).unwrap();
    let previous = store.get();
    fs::remove_file(f.path()).unwrap();
    fs::create_dir(f.path()).unwrap();
    let mut replacement = previous.clone();
    replacement.stt.model = "other".into();
    assert!(store.save(replacement).is_err());
    assert_eq!(store.get(), previous);
}

#[cfg(unix)]
#[test]
fn rejects_symlink_and_failed_migration_preserves_original() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let original = data("mistral", "valid", "mistral", "valid", None);
    fs::write(f.path(), &original).unwrap();
    let link = f.0.join("link.json");
    symlink(f.path(), &link).unwrap();
    assert!(PreferencesStore::open(link).is_err());
    let parent_link = f.0.join("linked-parent");
    symlink(&f.0, &parent_link).unwrap();
    assert!(PreferencesStore::open(parent_link.join("preferences.json")).is_err());
    assert_eq!(fs::read_to_string(f.path()).unwrap(), original);
}

#[test]
fn migrated_last_note_keeps_body_intention_and_path_after_save_and_reopen() {
    use vocalnotes_lib::{
        domain::intention::DraftingIntention,
        storage::{
            markdown_file::{MarkdownFile, SaveOutcome},
            note_document::{decode, encode},
        },
    };
    let f = Fixture::new();
    let note_path = f.0.join("note.md");
    let intention = DraftingIntention::new(
        "Résumé".into(),
        "courriel".into(),
        "cordial".into(),
        "contexte".into(),
    )
    .unwrap();
    let body = "# Titre\n\nCorps conservé.\n";
    fs::write(&note_path, encode(body, &intention)).unwrap();
    fs::write(
        f.path(),
        data(
            "mistral",
            "old-stt",
            "mistral",
            "old-llm",
            note_path.to_str(),
        ),
    )
    .unwrap();
    let store = PreferencesStore::open(f.path()).unwrap();
    assert!(store.was_migrated());
    let (mut file, original) = MarkdownFile::open(store.get().last_note.as_ref().unwrap()).unwrap();
    let (loaded_body, loaded_intention) = decode(&original).unwrap();
    assert_eq!(loaded_body, body);
    assert!(loaded_intention == intention);
    let updated = format!("{body}\nAjout manuel.\n");
    assert_eq!(
        file.save(&encode(&updated, &loaded_intention)).unwrap(),
        SaveOutcome::Saved
    );
    let restarted = PreferencesStore::open(f.path()).unwrap();
    assert!(!restarted.was_migrated());
    assert_eq!(restarted.get().last_note, Some(note_path.clone()));
    let (_, reopened) = MarkdownFile::open(&note_path).unwrap();
    let (body_after, intention_after) = decode(&reopened).unwrap();
    assert_eq!(body_after, updated);
    assert!(intention_after == intention);
}

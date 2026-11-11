use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use vocalnotes_lib::{
    config::preferences::{Preferences, PreferencesStore, ServiceSelection},
    platform::secrets::SecretStore,
};

fn folder() -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!(
        "vocalnotes-preferences-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&p).unwrap();
    p
}

#[test]
fn mock_backend_cannot_be_used_as_secure_storage() {
    let entry =
        keyring::Entry::new_with_credential(Box::new(keyring::mock::MockCredential::default()));
    assert!(SecretStore::from_entry(entry).is_err());
}

#[test]
fn invalid_keys_are_rejected_before_backend_access() {
    let store = SecretStore::native("fr.vocalnotes.validation", "invalid-input-only").unwrap();
    for secret in [
        String::new(),
        " ".into(),
        "a\nb".into(),
        "a\rb".into(),
        "a\0b".into(),
        "x".repeat(4097),
    ] {
        assert_eq!(store.set(secret), Err("Clé invalide."));
    }
}

#[test]
fn unavailable_child() {
    if std::env::var_os("VOCALNOTES_TEST_UNAVAILABLE").is_none() {
        return;
    }
    let store = SecretStore::native("fr.vocalnotes.validation", "unavailable-child").unwrap();
    assert!(store.set("synthetic-test-value".into()).is_err());
    assert!(store.presence().is_err());
    assert!(store.delete().is_err());
}

#[test]
#[cfg(target_os = "linux")]
fn unavailable_keyring_has_no_memory_or_file_fallback() {
    let root = folder();
    let status = Command::new(std::env::current_exe().unwrap())
        .arg("unavailable_child")
        .arg("--nocapture")
        .env("VOCALNOTES_TEST_UNAVAILABLE", "1")
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            "unix:path=/vocalnotes-unavailable/bus",
        )
        .current_dir(&root)
        .status()
        .unwrap();
    assert!(status.success());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
    fs::remove_dir(root).unwrap();
}

#[test]
fn native_roundtrip_opt_in() {
    if std::env::var_os("VOCALNOTES_TEST_NATIVE_KEYRING").is_none() {
        return;
    }
    let user = format!(
        "policy-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let store = SecretStore::native("fr.vocalnotes.validation", &user).unwrap();
    let result = (|| {
        assert!(!store.presence()?);
        store.set("synthetic-first".into())?;
        assert!(store.presence()?);
        store.set("synthetic-replacement".into())?;
        assert!(store.presence()?);
        #[cfg(target_os = "linux")]
        {
            let credential = keyring::secret_service::SsCredential::new_with_target(
                None,
                "fr.vocalnotes.validation",
                &user,
            )
            .map_err(|_| "Construction native impossible.")?;
            let entry = keyring::Entry::new_with_credential(Box::new(credential));
            let value = zeroize::Zeroizing::new(
                entry
                    .get_password()
                    .map_err(|_| "Lecture de la valeur synthétique impossible.")?,
            );
            if value.as_str() != "synthetic-replacement" {
                return Err("Remplacement synthétique non retrouvé.");
            }
        }

        Ok::<(), &'static str>(())
    })();
    let cleanup = store.delete();
    assert!(result.is_ok());
    assert!(cleanup.is_ok());
    assert!(!store.presence().unwrap());
    assert!(store.delete().is_ok());
}

#[test]
fn preferences_are_strict_nonsecret_and_survive_restart() {
    let root = folder();
    let path = root.join("preferences.json");
    let mut store = PreferencesStore::open(path.clone()).unwrap();
    let mut next = Preferences::default();
    next.llm.model = "mistral-test-unverified".into();
    next.last_note = Some(root.join("note.md"));
    store.save(next.clone()).unwrap();
    assert_eq!(PreferencesStore::open(path.clone()).unwrap().get(), next);
    let json = fs::read_to_string(&path).unwrap();
    assert!(!json.contains("secret"));
    assert!(!json.contains("transcription"));
    let mut data = serde_json::to_value(next).unwrap();
    data["secret"] = serde_json::json!("synthetic-key");
    fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
    assert!(PreferencesStore::open(path).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn defaults_select_openai_without_content_or_secrets() {
    let preferences = Preferences::default();
    assert_eq!(preferences.stt.provider, "openai");
    assert_eq!(preferences.stt.model, "gpt-realtime-whisper");
    assert_eq!(preferences.llm.provider, "openai");
    assert_eq!(preferences.llm.model, "gpt-6-luna");
    let value = serde_json::to_value(preferences).unwrap();
    assert_eq!(value.as_object().unwrap().len(), 3);
    assert!(value["lastNote"].is_null());
}

#[test]
fn migration_notification_is_typed_and_only_follows_persisted_migration() {
    use vocalnotes_lib::platform::{controller::Controller, probe::Probe};
    let root = folder();
    let path = root.join("preferences.json");
    fs::write(
        &path,
        serde_json::json!({
            "stt":{"provider":"mistral","model":"historical-stt"},
            "llm":{"provider":"mistral","model":"historical-llm"},
            "lastNote":null
        })
        .to_string(),
    )
    .unwrap();
    let store = PreferencesStore::open(path.clone()).unwrap();
    let c = Controller::with_preferences(Probe::new("note".into()), Ok(store));
    let snapshot = c.core.lock().unwrap().snapshot();
    assert!(snapshot.preferences_migrated);
    assert_eq!(snapshot.source, "note");
    let snapshot = serde_json::to_value(snapshot).unwrap();
    assert_eq!(snapshot["preferencesMigrated"], true);
    assert!(!snapshot.to_string().contains("historical"));
    let c = Controller::with_preferences(Probe::new("note".into()), PreferencesStore::open(path));
    assert!(!c.core.lock().unwrap().snapshot().preferences_migrated);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn openai_selection_model_boundaries_are_local_and_exact() {
    for (model, valid) in [
        ("test-model_1.0".to_string(), true),
        ("m".repeat(128), true),
        ("m".repeat(129), false),
        (String::new(), false),
        ("modèle".to_string(), false),
        ("x?secret=y".to_string(), false),
    ] {
        assert_eq!(
            ServiceSelection {
                provider: "openai".into(),
                model
            }
            .validate()
            .is_ok(),
            valid
        );
    }
}

#[test]
fn model_choices_are_independent_validated_and_frozen_for_a_session() {
    let mut store = PreferencesStore::memory();
    let frozen = store.session_models().unwrap();
    let mut next = store.get();
    next.llm.model = "mistral-test-unverified".into();
    store.save(next).unwrap();
    assert_eq!(frozen.stt, store.session_models().unwrap().stt);
    assert_ne!(frozen.llm, store.session_models().unwrap().llm);
    for model in [
        "",
        "mistral x",
        "https://example.test/",
        "x?key=value",
        "x\nheader",
    ] {
        let bad = ServiceSelection {
            provider: "openai".into(),
            model: model.into(),
        };
        let mut next = store.get();
        next.stt = bad;
        assert!(store.save(next).is_err());
    }
    let mut bad = store.get();
    bad.llm.provider = "unknown-provider".into();
    assert!(store.save(bad).is_err());
}

#[test]
fn failed_or_unsafe_preference_save_preserves_previous_settings() {
    let root = folder();
    let path = root.join("preferences.json");
    let mut store = PreferencesStore::open(path.clone()).unwrap();
    store.save(Preferences::default()).unwrap();
    let previous = store.get();
    fs::rename(&path, root.join("original.json")).unwrap();
    fs::create_dir(&path).unwrap();
    let mut next = previous.clone();
    next.llm.model = "mistral-other".into();
    assert!(store.save(next).is_err());
    assert_eq!(store.get(), previous);
    assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join("original.json"), root.join("link.json")).unwrap();
        assert!(PreferencesStore::open(root.join("link.json")).is_err());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn controller_uses_frozen_models_until_a_new_explicit_session() {
    use vocalnotes_lib::platform::{controller::Controller, probe::Probe};
    let controller = Controller::new(Probe::new(String::new()));
    let first = controller.models_for_start(false).unwrap();
    let mut next = first.clone();
    next.llm.model = "mistral-new-unverified".into();
    controller.set_models(next.clone()).unwrap();
    assert_eq!(controller.models_for_start(true).unwrap(), first);
    assert_eq!(controller.models_for_start(false).unwrap(), next);
    let missing =
        Controller::with_preferences(Probe::new(String::new()), Err("Préférences inaccessibles."));
    assert!(missing.models_for_start(false).is_err());
    assert_eq!(missing.core.lock().unwrap().snapshot().mode, "manual");
}

#[test]
fn selected_stt_model_cannot_change_host_or_add_credentials_to_url() {
    use vocalnotes_lib::platform::stt_transport::request_for_model;
    let request = request_for_model("voxtral-test-unverified", Some("synthetic-key")).unwrap();
    assert_eq!(request.uri().host(), Some("api.openai.com"));
    assert!(request.uri().to_string().ends_with("intent=transcription"));
    assert!(!request.uri().to_string().contains("synthetic-key"));
    assert!(request.headers()["Authorization"].is_sensitive());
    assert!(request_for_model("x&api_key=bad", None).is_err());
}

#[test]
fn only_successfully_saved_notes_are_remembered_without_note_content() {
    use vocalnotes_lib::{
        app::commands::{Command, execute_action},
        platform::{controller::Controller, probe::Probe},
    };
    let root = folder();
    let note = root.join("note.md");
    let store = PreferencesStore::open(root.join("preferences.json")).unwrap();
    let controller = Controller::with_preferences(
        Probe::new("Contenu synthétique non enregistré dans les préférences".into()),
        Ok(store),
    );
    assert!(controller.preferences().unwrap().last_note.is_none());
    execute_action(
        &controller,
        Command::SaveCopy {
            path: note.to_str().unwrap().into(),
        },
    )
    .unwrap();
    assert_eq!(controller.preferences().unwrap().last_note, Some(note));
    assert!(
        !fs::read_to_string(root.join("preferences.json"))
            .unwrap()
            .contains("Contenu synthétique")
    );
    fs::remove_dir_all(root).unwrap();
}

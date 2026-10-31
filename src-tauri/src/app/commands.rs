use crate::platform::{
    controller::Controller,
    probe::{Ack, Proposal, Snapshot},
    secrets,
};
use serde::{Deserialize, Deserializer};
use std::path::Path;
use tauri::State;

pub struct PrototypeState(pub Controller);

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
// Empty struct variants enforce deny_unknown_fields even on parameterless actions.
pub enum Command {
    New {},
    Open {
        #[serde(deserialize_with = "local_path")]
        path: String,
    },
    SaveCopy {
        #[serde(deserialize_with = "local_path")]
        path: String,
    },
    Save {},
    CancelClose {},
    Start {
        consent: bool,
        resume: bool,
    },
    Transcript {
        enabled: bool,
    },
    Pause {},
    Stop {},
}

fn local_path<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let value = String::deserialize(deserializer)?;
    validate_path(&value).map_err(serde::de::Error::custom)?;
    Ok(value)
}

fn validate_path(value: &str) -> Result<(), &'static str> {
    let path = Path::new(value);
    if value.len() > 4096
        || value.contains('\0')
        || !path.is_absolute()
        || path.extension().and_then(|s| s.to_str()) != Some("md")
        || path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err("Chemin absolu vers un fichier .md local requis (4096 octets maximum).");
    }
    Ok(())
}

pub fn navigation_allowed(value: &str, development: bool) -> bool {
    let Ok(url) = tauri::Url::parse(value) else {
        return false;
    };
    if !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    match (url.scheme(), url.host_str(), url.port()) {
        ("tauri", Some("localhost"), None) | ("http", Some("tauri.localhost"), None) => true,
        ("http", Some("127.0.0.1"), Some(1420)) => development,
        _ => false,
    }
}

#[tauri::command]
pub fn prototype_snapshot(state: State<'_, PrototypeState>) -> Result<Snapshot, &'static str> {
    let mut core = state.0.core.lock().map_err(|_| "Cœur indisponible.")?;
    core.refresh_disk_status();
    Ok(core.snapshot())
}

#[tauri::command]
pub fn prototype_manual(
    state: State<'_, PrototypeState>,
    proposal: Proposal,
) -> Result<Ack, &'static str> {
    let ack = state
        .0
        .core
        .lock()
        .map_err(|_| "Cœur indisponible.")?
        .manual(proposal);
    state.0.remember_saved_note();
    Ok(ack)
}

#[tauri::command]
pub fn prototype_action(
    state: State<'_, PrototypeState>,
    action: Command,
) -> Result<Snapshot, &'static str> {
    execute_action(&state.0, action)
}

pub fn execute_action(controller: &Controller, action: Command) -> Result<Snapshot, &'static str> {
    if let Command::Open { path } | Command::SaveCopy { path } = &action {
        validate_path(path)?;
    }
    // All entry points share native actions. Never hold the core across controller locks.
    match action {
        Command::Start { consent, resume } => {
            if !consent {
                return Err("Information et accord micro/Mistral/coût requis.");
            }
            controller.start_explicit(resume)?;
        }
        Command::Pause {} => controller.stop(true, "Pause explicite."),
        Command::Stop {} => {
            controller.stop(false, "Arrêt explicite ; passages en attente non intégrés.")
        }
        Command::New {} | Command::Open { .. } => {
            controller.stop(false, "Changement de note : traitements invalidés.")
        }
        _ => {}
    }
    let mut core = controller.core.lock().map_err(|_| "Cœur indisponible.")?;
    match action {
        Command::New {} => core.new_note()?,
        Command::Open { path } => core.open(Path::new(&path))?,
        Command::SaveCopy { path } => core.save_copy(Path::new(&path))?,
        Command::Save {} => core.save_current()?,
        Command::CancelClose {} => core.cancel_close(),
        Command::Transcript { enabled } => core.set_transcript_enabled(enabled)?,
        Command::Start { .. } | Command::Pause {} | Command::Stop {} => {}
    }
    let snapshot = core.snapshot();
    drop(core);
    controller.remember_saved_note();
    Ok(snapshot)
}

#[tauri::command]
pub fn prototype_finish_close(
    state: State<'_, PrototypeState>,
    window: tauri::Window,
) -> Result<Snapshot, &'static str> {
    super::close::finish(&state.0, || window.destroy().map_err(|_| ()))
}

#[tauri::command]
pub async fn prototype_set_key(secret: String) -> Result<(), &'static str> {
    let secret = zeroize::Zeroizing::new(secret);
    tauri::async_runtime::spawn_blocking(move || secrets::set_protected(secret))
        .await
        .map_err(|_| "Trousseau indisponible.")?
}

#[tauri::command]
pub async fn prototype_delete_key() -> Result<(), &'static str> {
    tauri::async_runtime::spawn_blocking(secrets::delete)
        .await
        .map_err(|_| "Trousseau indisponible.")?
}

#[tauri::command]
pub async fn prototype_key_presence() -> Result<bool, &'static str> {
    tauri::async_runtime::spawn_blocking(secrets::presence)
        .await
        .map_err(|_| "Trousseau indisponible.")?
}

#[tauri::command]
pub fn preferences_get(
    state: State<'_, PrototypeState>,
) -> Result<crate::config::preferences::Preferences, &'static str> {
    state.0.preferences()
}
#[tauri::command]
pub fn preferences_set(
    state: State<'_, PrototypeState>,
    models: crate::config::preferences::ModelPreferences,
) -> Result<crate::config::preferences::Preferences, &'static str> {
    state.0.set_models(models)
}

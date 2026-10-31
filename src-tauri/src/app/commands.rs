use crate::platform::{
    controller::Controller,
    probe::{Ack, Proposal, Snapshot},
    secrets,
};
use serde::Deserialize;
use std::path::Path;
use tauri::State;

pub struct PrototypeState(pub Controller);

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    New,
    Open { path: String },
    SaveCopy { path: String },
    Save,
    CancelClose,
    Start { consent: bool, resume: bool },
    Pause,
    Stop,
}

#[tauri::command]
pub fn prototype_snapshot(state: State<'_, PrototypeState>) -> Result<Snapshot, &'static str> {
    Ok(state
        .0
        .core
        .lock()
        .map_err(|_| "Cœur indisponible.")?
        .snapshot())
}

#[tauri::command]
pub fn prototype_manual(
    state: State<'_, PrototypeState>,
    proposal: Proposal,
) -> Result<Ack, &'static str> {
    Ok(state
        .0
        .core
        .lock()
        .map_err(|_| "Cœur indisponible.")?
        .manual(proposal))
}

#[tauri::command]
pub fn prototype_action(
    state: State<'_, PrototypeState>,
    action: Command,
) -> Result<Snapshot, &'static str> {
    // All entry points share native actions. Never hold the core across controller locks.
    match action {
        Command::Start { consent, resume } => {
            if !consent {
                return Err("Information et accord micro/Mistral/coût requis.");
            }
            state.0.start_explicit(resume)?;
        }
        Command::Pause => state.0.stop(true, "Pause explicite."),
        Command::Stop => state
            .0
            .stop(false, "Arrêt explicite ; passages en attente non intégrés."),
        Command::New | Command::Open { .. } => state
            .0
            .stop(false, "Changement de note : traitements invalidés."),
        _ => {}
    }
    let mut core = state.0.core.lock().map_err(|_| "Cœur indisponible.")?;
    match action {
        Command::New => core.new_note()?,
        Command::Open { path } => core.open(Path::new(&path))?,
        Command::SaveCopy { path } => core.save_copy(Path::new(&path))?,
        Command::Save => core.save_selected_copy()?,
        Command::CancelClose => core.cancel_close(),
        Command::Start { .. } | Command::Pause | Command::Stop => {}
    }
    Ok(core.snapshot())
}

#[tauri::command]
pub fn prototype_finish_close(
    state: State<'_, PrototypeState>,
    window: tauri::Window,
) -> Result<Snapshot, &'static str> {
    state.0.stop(
        false,
        "Fermeture explicite : capture arrêtée et traitements invalidés.",
    );
    let mut core = state.0.core.lock().map_err(|_| "Cœur indisponible.")?;
    if core.request_close() {
        window
            .destroy()
            .map_err(|_| "Fermeture impossible ; note sauvegardée et conservée.")?;
    }
    Ok(core.snapshot())
}

#[tauri::command]
pub async fn prototype_set_key(secret: String) -> Result<(), &'static str> {
    tauri::async_runtime::spawn_blocking(move || secrets::set(secret))
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

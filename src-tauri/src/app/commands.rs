use super::document::{
    DocumentError, DocumentErrorCode, DocumentKind, DocumentReply, DocumentVersion,
    HistoryDirection, OperationId,
};
use crate::domain::DictationTarget;
use crate::platform::desktop::{self, CommandOrigin, DesktopContext, DocumentEvents};
use crate::platform::{
    controller::Controller,
    probe::{Ack, Proposal, Snapshot},
    secrets,
};
use serde::{Deserialize, Deserializer};
use std::path::Path;
use tauri::{Manager, State};

pub struct PrototypeState(pub Controller);

#[derive(Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum DocumentAction {
    Request {
        action: DocumentKind,
    },
    Ready {
        operation_id: OperationId,
        note_id: u64,
        revision: u64,
        input_generation: u64,
    },
    Cancel {
        operation_id: OperationId,
    },
    Focus {
        area: DictationTarget,
    },
    History {
        direction: HistoryDirection,
        area: DictationTarget,
        #[serde(deserialize_with = "transaction_id")]
        transaction_id: String,
        note_id: u64,
        base_revision: u64,
        input_generation: u64,
    },
}

fn transaction_id<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let value = String::deserialize(deserializer)?;
    if value.is_empty() || value.len() > 128 {
        return Err(serde::de::Error::custom(
            "Identifiant de transaction invalide.",
        ));
    }
    Ok(value)
}

#[tauri::command]
pub async fn document_action(
    state: State<'_, PrototypeState>,
    window: tauri::WebviewWindow,
    action: DocumentAction,
) -> Result<DocumentReply, DocumentError> {
    // The label comes from Tauri, never from renderer arguments.
    if window.label() != "main" {
        return Err(DocumentErrorCode::Forbidden.error());
    }
    let app = window.app_handle();
    let context = desktop::main_context(app, CommandOrigin::Window)?;
    let ready = if let DocumentAction::Ready { operation_id, .. } = &action {
        Some(*operation_id)
    } else {
        None
    };
    let reply = execute_document_action(&state.0, app, context, action)?;
    if let Some(id) = ready {
        let quit = state
            .0
            .document_operation()?
            .is_some_and(|op| op.action == DocumentKind::Quit);
        let result = state
            .0
            .run_document(id, &desktop::NativeDialogs::new(app.clone()))
            .await;
        desktop::sync_main(app, &state.0)?;
        let result = result?;
        if quit
            && matches!(
                result.outcome,
                Some(super::document::DocumentOutcome::Completed { .. })
            )
        {
            if let Some(main) = app.get_webview_window("main") {
                super::close::finish_document(&state.0, id, || main.destroy().map_err(|_| ()))?;
            }
        }
        Ok(result)
    } else {
        desktop::sync_main(app, &state.0)?;
        Ok(reply)
    }
}

pub fn execute_document_action(
    controller: &Controller,
    events: &impl DocumentEvents,
    context: DesktopContext<'_>,
    action: DocumentAction,
) -> Result<DocumentReply, DocumentError> {
    context.validate()?;
    match action {
        DocumentAction::Request { action } => {
            desktop::request_document(controller, events, context, action)?;
        }
        DocumentAction::Ready {
            operation_id,
            note_id,
            revision,
            input_generation,
        } => {
            controller.document_ready(
                operation_id,
                DocumentVersion {
                    note_id,
                    revision,
                    input_generation,
                },
            )?;
        }
        DocumentAction::Cancel { operation_id } => {
            controller.cancel_document(operation_id)?;
        }
        DocumentAction::Focus { area } => {
            controller.document_focus(area)?;
        }
        DocumentAction::History {
            direction,
            area,
            transaction_id,
            note_id,
            base_revision,
            input_generation,
        } => {
            controller.history_action(
                transaction_id,
                DocumentVersion {
                    note_id,
                    revision: base_revision,
                    input_generation,
                },
                area,
                direction == HistoryDirection::Redo,
            )?;
        }
    }
    controller.document_reply()
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
// Empty struct variants enforce deny_unknown_fields even on parameterless actions.
pub enum Command {
    #[serde(skip_deserializing)]
    New {},
    #[serde(skip_deserializing)]
    Open {
        path: String,
    },
    #[serde(skip_deserializing)]
    SaveCopy {
        path: String,
    },
    #[serde(skip_deserializing)]
    Save {},
    #[serde(skip_deserializing)]
    CancelClose {},
    Start {
        consent: bool,
        resume: bool,
    },
    Transcript {
        enabled: bool,
    },
    SetDictationMode {
        mode: DictationTarget,
    },
    SetContext {
        text: String,
    },
    SetInstructions {
        text: String,
    },
    Pause {},
    Stop {},
}

fn validate_path(value: &str) -> Result<(), &'static str> {
    crate::storage::markdown_file::validate_note_path(Path::new(value))
        .map(|_| ())
        .map_err(|error| error.message())
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
pub fn prototype_snapshot(
    state: State<'_, PrototypeState>,
    window: tauri::WebviewWindow,
) -> Result<Snapshot, &'static str> {
    require_label(window.label(), "main")?;
    desktop::sync_main(window.app_handle(), &state.0).map_err(|e| e.message)?;
    state.0.snapshot().map_err(|error| error.message)
}

#[tauri::command]
pub fn prototype_manual(
    state: State<'_, PrototypeState>,
    window: tauri::WebviewWindow,
    proposal: Proposal,
    origin: Option<crate::domain::history::ManualOrigin>,
) -> Result<Ack, &'static str> {
    require_label(window.label(), "main")?;
    let mut ack = state
        .0
        .edit_core(|core| Ok(core.manual_origin(proposal, origin.unwrap_or_default())))?;
    state.0.remember_saved_note();
    state
        .0
        .attach_document_state(&mut ack.snapshot)
        .map_err(|error| error.message)?;
    Ok(ack)
}

#[tauri::command]
pub async fn prototype_action(
    state: State<'_, PrototypeState>,
    window: tauri::WebviewWindow,
    action: Command,
) -> Result<Snapshot, &'static str> {
    require_label(window.label(), "main")?;
    if state.0.is_preferences_modal() {
        return Err("Préférences ouvertes ; commande de note refusée.");
    }
    if matches!(
        action,
        Command::New {}
            | Command::Open { .. }
            | Command::SaveCopy { .. }
            | Command::Save {}
            | Command::CancelClose {}
    ) {
        return Err("Utilisez les commandes documentaires natives.");
    }
    let mut snapshot = match action {
        Command::SetDictationMode { mode } => state.0.set_dictation_mode(mode).await,
        action => execute_action(&state.0, action),
    }?;
    state
        .0
        .attach_document_state(&mut snapshot)
        .map_err(|error| error.message)?;
    Ok(snapshot)
}

#[tauri::command]
pub async fn prototype_reformat(
    state: State<'_, PrototypeState>,
    window: tauri::WebviewWindow,
    note_id: u64,
    base_revision: u64,
) -> Result<Snapshot, &'static str> {
    require_label(window.label(), "main")?;
    if state.0.is_preferences_modal() {
        return Err("Préférences ouvertes ; commande de note refusée.");
    }
    let controller = state.0.clone();
    let mut snapshot = controller.reformat(note_id, base_revision).await?;
    controller
        .attach_document_state(&mut snapshot)
        .map_err(|error| error.message)?;
    Ok(snapshot)
}

pub fn execute_action(controller: &Controller, action: Command) -> Result<Snapshot, &'static str> {
    match action {
        Command::Pause {} => return controller.user_stop(true, "Pause explicite."),
        Command::Stop {} => {
            return controller
                .user_stop(false, "Arrêt explicite ; passages en attente non intégrés.");
        }
        Command::Transcript { enabled } => {
            return controller.edit_core(|core| {
                core.set_transcript_enabled(enabled)?;
                Ok(core.snapshot())
            });
        }
        Command::SetContext { text } => {
            return controller.edit_core(|core| {
                core.set_context(text)?;
                Ok(core.snapshot())
            });
        }
        Command::SetInstructions { text } => {
            return controller.edit_core(|core| {
                core.set_instructions(text)?;
                Ok(core.snapshot())
            });
        }
        _ => {}
    }
    if controller.is_preferences_modal() {
        return Err("Préférences ouvertes ; commande de note refusée.");
    }
    if let Command::Open { path } | Command::SaveCopy { path } = &action {
        validate_path(path)?;
    }
    // All entry points share native actions. Never hold the core across controller locks.
    match action {
        Command::Start { consent, resume } => {
            if !consent {
                return Err("Information et accord micro/OpenAI/coût requis.");
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
        Command::SetDictationMode { mode } => core.set_dictation_mode(mode),
        Command::SetContext { text } => core.set_context(text)?,
        Command::SetInstructions { text } => core.set_instructions(text)?,
        Command::Start { .. } | Command::Pause {} | Command::Stop {} => {}
    }
    let snapshot = core.snapshot();
    drop(core);
    controller.remember_saved_note();
    Ok(snapshot)
}

pub fn prototype_finish_close(
    state: State<'_, PrototypeState>,
    window: tauri::Window,
) -> Result<Snapshot, &'static str> {
    super::close::finish(&state.0, || window.destroy().map_err(|_| ()))
}

#[tauri::command]
pub async fn prototype_set_key(
    window: tauri::WebviewWindow,
    secret: String,
) -> Result<(), &'static str> {
    require_label(window.label(), "preferences")?;
    let secret = zeroize::Zeroizing::new(secret);
    tauri::async_runtime::spawn_blocking(move || secrets::set_protected(secret))
        .await
        .map_err(|_| "Trousseau indisponible.")?
}

#[tauri::command]
pub async fn prototype_delete_key(window: tauri::WebviewWindow) -> Result<(), &'static str> {
    require_label(window.label(), "preferences")?;
    tauri::async_runtime::spawn_blocking(secrets::delete)
        .await
        .map_err(|_| "Trousseau indisponible.")?
}

#[tauri::command]
pub async fn prototype_key_presence(window: tauri::WebviewWindow) -> Result<bool, &'static str> {
    require_label(window.label(), "preferences")?;
    tauri::async_runtime::spawn_blocking(secrets::presence)
        .await
        .map_err(|_| "Trousseau indisponible.")?
}

#[tauri::command]
pub fn preferences_get(
    state: State<'_, PrototypeState>,
    window: tauri::WebviewWindow,
) -> Result<crate::config::preferences::ModelPreferences, &'static str> {
    if !matches!(window.label(), "main" | "preferences") {
        return Err("Fenêtre non autorisée.");
    }
    state.0.preferences().map(|value| value.models())
}
#[tauri::command]
pub async fn preferences_set(
    state: State<'_, PrototypeState>,
    window: tauri::WebviewWindow,
    models: crate::config::preferences::ModelPreferences,
) -> Result<crate::config::preferences::ModelPreferences, &'static str> {
    require_label(window.label(), "preferences")?;
    let controller = state.0.clone();
    let writing = controller.reserve_preferences_write()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _writing = writing;
        controller.set_models(models).map(|value| value.models())
    })
    .await
    .map_err(|_| "Préférences indisponibles.")?
}

pub fn require_label(actual: &str, expected: &str) -> Result<(), &'static str> {
    if actual == expected {
        Ok(())
    } else {
        Err("Fenêtre non autorisée.")
    }
}

#[tauri::command]
pub async fn preferences_open(window: tauri::WebviewWindow) -> Result<(), DocumentError> {
    require_label(window.label(), "main").map_err(|_| DocumentErrorCode::Forbidden.error())?;
    crate::platform::native_ui::open_preferences(window.app_handle()).await
}

#[tauri::command]
pub async fn preferences_close(window: tauri::WebviewWindow) -> Result<(), DocumentError> {
    require_label(window.label(), "preferences")
        .map_err(|_| DocumentErrorCode::Forbidden.error())?;
    crate::platform::native_ui::close_preferences(window.app_handle()).await
}

use serde::Serialize;
use tauri::Manager;
pub mod app;
mod diagnostics;
pub mod platform;
use app::commands::*;

#[derive(Serialize)]
struct StartupStatus {
    stage: &'static str,
    capture: &'static str,
    persistence: &'static str,
}

// Scaffold-only, read-only command: no audio, network, file or secret access.
#[tauri::command]
fn startup_status() -> StartupStatus {
    StartupStatus {
        stage: "scaffold",
        capture: "stopped",
        persistence: "unavailable",
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    diagnostics::event("Démarrage diagnostic console D2 ; microphone arrêté");
    // Startup errors are deliberately not formatted with external payloads.
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri::plugin::Builder::<tauri::Wry>::new("local-navigation")
                .on_navigation(|_, url| {
                    app::commands::navigation_allowed(url.as_str(), tauri::is_dev())
                })
                .build(),
        )
        .setup(|app| {
            let preferences = app
                .path()
                .app_config_dir()
                .map_err(|_| "Dossier des préférences indisponible.")
                .and_then(|path| {
                    config::preferences::PreferencesStore::open(path.join("preferences.json"))
                });
            let mut core = platform::probe::Probe::new(String::new());
            if preferences.is_err() {
                core.message(
                    "Préférences inaccessibles : vocal indisponible, note manuelle accessible.",
                );
            }
            app.manage(PrototypeState(
                platform::controller::Controller::with_preferences(core, preferences),
            ));
            platform::native_ui::install(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != "main" { return; }
            if matches!(event, tauri::WindowEvent::Destroyed) {
                if let Some(preferences) = window.app_handle().get_webview_window("preferences") {
                    let _ = preferences.destroy();
                }
                let state = window.state::<PrototypeState>();
                state.0.end_preferences(state.0.preferences_token());
            }
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let state = window.state::<PrototypeState>();
                let app = window.app_handle();
                if let Ok(context) = platform::desktop::main_context(app, platform::desktop::CommandOrigin::Window) {
                    let _ = app::close::request(&state.0, app, context);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            startup_status,
            prototype_snapshot,
            prototype_manual,
            prototype_action,
            document_action,
            prototype_reformat,
            prototype_set_key,
            prototype_delete_key,
            prototype_key_presence,
            preferences_get,
            preferences_set,
            preferences_open,
            preferences_close
        ])
        .run(tauri::generate_context!());
    if result.is_err() {
        eprintln!("Démarrage du socle impossible.");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffold_does_not_claim_recording_or_persistence() {
        let status = startup_status();
        assert_eq!(status.stage, "scaffold");
        assert_eq!(status.capture, "stopped");
        assert_eq!(status.persistence, "unavailable");
    }
}
pub mod domain;
pub mod ports;

pub mod config;
pub mod storage;

pub mod adapters;
pub mod audio;

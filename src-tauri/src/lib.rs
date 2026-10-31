use serde::Serialize;
use tauri::Manager;
mod app;
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
    // Startup errors are deliberately not formatted with external payloads.
    let result = tauri::Builder::default()
        .manage(PrototypeState(platform::controller::Controller::new(
            platform::probe::Probe::new(String::new()),
        )))
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // A native X cannot silently discard an unacknowledged UI proposal.
                // Finalization is a dedicated command after the UI queue settles.
                api.prevent_close();
                let state = window.state::<PrototypeState>();
                state.0.stop(
                    false,
                    "Fermeture : capture arrêtée et traitements invalidés.",
                );
                match state.0.core.lock() {
                    Ok(mut core) => {
                        core.prepare_close();
                    }
                    _ => api.prevent_close(),
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            startup_status,
            prototype_snapshot,
            prototype_manual,
            prototype_action,
            prototype_set_key,
            prototype_delete_key,
            prototype_key_presence,
            prototype_finish_close
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

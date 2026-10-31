#[path = "src/app/ipc_commands.rs"]
mod ipc_commands;

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(ipc_commands::IPC_COMMANDS)),
    )
    .expect("Configuration des permissions IPC impossible.");
}

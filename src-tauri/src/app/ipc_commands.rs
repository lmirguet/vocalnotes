// Shared with build.rs: every custom command receives an explicit Tauri permission.
pub const IPC_COMMANDS: &[&str] = &[
    "startup_status",
    "prototype_snapshot",
    "prototype_manual",
    "prototype_action",
    "document_action",
    "prototype_reformat",
    "prototype_set_key",
    "prototype_delete_key",
    "prototype_key_presence",
    "preferences_get",
    "preferences_set",
    "preferences_open",
    "preferences_close",
];

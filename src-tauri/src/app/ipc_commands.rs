// Shared with build.rs: every custom command receives an explicit Tauri permission.
pub const IPC_COMMANDS: &[&str] = &[
    "startup_status",
    "prototype_snapshot",
    "prototype_manual",
    "prototype_action",
    "prototype_reformat",
    "prototype_set_key",
    "prototype_delete_key",
    "prototype_key_presence",
    "prototype_finish_close",
    "preferences_get",
    "preferences_set",
];

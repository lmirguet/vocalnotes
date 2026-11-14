//! Native menus and owned Preferences lifecycle, shared command authority on both desktops.
use super::{controller::Controller, desktop::*};
use crate::app::{commands::PrototypeState, document::*};
use tauri::menu::{Menu, MenuItem, Submenu};
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

pub const ITEMS: [(&str, &str, Option<&str>); 10] = [
    ("file.new", "&Nouvelle note", Some("Ctrl+N")),
    ("file.open", "&Ouvrir", Some("Ctrl+O")),
    ("file.save", "&Enregistrer", Some("Ctrl+S")),
    ("file.save_as", "Enregistrer &sous", Some("Ctrl+Shift+S")),
    ("file.export_markdown", "E&xporter en markdown", None),
    ("file.quit", "&Quitter", Some("Alt+F4")),
    ("edit.preferences", "&Préférence", None),
    ("edit.undo", "&Annuler", Some("Ctrl+Z")),
    ("edit.redo", "&Rétablir", Some("Ctrl+Y")),
    ("help.about", "A &propos", None),
];

pub fn install<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let main = app
        .get_webview_window("main")
        .ok_or(tauri::Error::WindowNotFound)?;
    let items = ITEMS
        .iter()
        .map(|(id, text, key)| MenuItem::with_id(app, *id, *text, true, *key))
        .collect::<tauri::Result<Vec<_>>>()?;
    let file = Submenu::with_items(
        app,
        "&Fichier",
        true,
        &items[..6]
            .iter()
            .map(|item| item as &dyn tauri::menu::IsMenuItem<R>)
            .collect::<Vec<_>>(),
    )?;
    let edit = Submenu::with_id_and_items(
        app,
        "menu.edit",
        "&Edition",
        true,
        &items[6..9]
            .iter()
            .map(|item| item as &dyn tauri::menu::IsMenuItem<R>)
            .collect::<Vec<_>>(),
    )?;
    let help = Submenu::with_items(app, "&Aide", true, &[&items[9]])?;
    main.set_menu(Menu::with_items(app, &[&file, &edit, &help])?)?;
    #[cfg(windows)]
    install_windows_keys(&main)?;
    main.on_menu_event(|window, event| {
        if window.label() != "main" {
            return;
        }
        let app = window.app_handle();
        let state = app.state::<PrototypeState>();
        let result = dispatch(app, &state.0, event.id().as_ref());
        if result.is_err() {
            crate::diagnostics::event("Commande de menu refusée ; note conservée.");
        }
    });
    Ok(())
}

#[cfg(windows)]
fn install_windows_keys<R: Runtime>(main: &tauri::WebviewWindow<R>) -> tauri::Result<()> {
    use webview2_com::{AcceleratorKeyPressedEventHandler, Microsoft::Web::WebView2::Win32::*};
    use windows_sys::Win32::UI::{
        Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL, VK_MENU, VK_SHIFT},
        WindowsAndMessaging::{PostMessageW, SC_KEYMENU, WM_SYSCOMMAND},
    };
    let app = main.app_handle().clone();
    main.with_webview(move |webview| {
        let callback_app = app.clone();
        let callback = AcceleratorKeyPressedEventHandler::create(Box::new(move |_, args| {
            let Some(args) = args else { return Ok(()) };
            unsafe {
                let mut kind = COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN;
                args.KeyEventKind(&mut kind)?;
                if kind != COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN
                    && kind != COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN
                {
                    return Ok(());
                }
                let mut key = 0;
                args.VirtualKey(&mut key)?;
                let ctrl = GetKeyState(VK_CONTROL as i32) < 0;
                let alt = GetKeyState(VK_MENU as i32) < 0;
                let shift = GetKeyState(VK_SHIFT as i32) < 0;
                let command = windows_key_command(key, ctrl, alt, shift);
                let menu = !ctrl && !shift && alt && matches!(key, 0x46 | 0x45 | 0x41);
                if command.is_none() && !menu {
                    return Ok(());
                }
                args.SetHandled(true)?;
                let mut status = COREWEBVIEW2_PHYSICAL_KEY_STATUS::default();
                args.PhysicalKeyStatus(&mut status)?;
                if status.WasKeyDown.as_bool() {
                    return Ok(());
                }
                let app = callback_app.clone();
                let queued_app = app.clone();
                // WebView2's synchronous callback must not reenter window/dialog APIs.
                tauri::async_runtime::spawn(async move {
                    if app
                        .run_on_main_thread(move || {
                            let state = queued_app.state::<PrototypeState>();
                            let result = if let Some(command) = command {
                                dispatch(&queued_app, &state.0, command)
                            } else {
                                main_context(&queued_app, CommandOrigin::Menu).and_then(|context| {
                                    context.validate()?;
                                    let main = queued_app
                                        .get_webview_window("main")
                                        .ok_or_else(|| DocumentErrorCode::WindowFailed.error())?;
                                    let hwnd = main
                                        .hwnd()
                                        .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
                                    if PostMessageW(
                                        hwnd.0 as _,
                                        WM_SYSCOMMAND,
                                        SC_KEYMENU as usize,
                                        key as isize,
                                    ) == 0
                                    {
                                        return Err(DocumentErrorCode::WindowFailed.error());
                                    }
                                    Ok(())
                                })
                            };
                            if result.is_err() {
                                crate::diagnostics::event(
                                    "Raccourci natif refusé ; note conservée.",
                                );
                            }
                        })
                        .is_err()
                    {
                        crate::diagnostics::event(
                            "Routage du raccourci natif indisponible ; note conservée.",
                        );
                    }
                });
            }
            Ok(())
        }));
        let mut token = 0;
        if unsafe {
            webview
                .controller()
                .add_AcceleratorKeyPressed(&callback, &mut token)
        }
        .is_err()
        {
            crate::diagnostics::event("Installation des raccourcis WebView2 impossible.");
            if let Ok(mut core) = app.state::<PrototypeState>().0.core.lock() {
                core.message("Raccourcis indisponibles : utilisez les menus ; note conservée.");
            }
        }
    })
}

#[cfg(any(windows, test))]
fn windows_key_command(key: u32, ctrl: bool, alt: bool, shift: bool) -> Option<&'static str> {
    if alt && !ctrl && !shift && key == 0x73 {
        return Some("file.quit");
    }
    if !ctrl || alt {
        return None;
    }
    match (key, shift) {
        (0x4e, false) => Some("file.new"),
        (0x4f, false) => Some("file.open"),
        (0x53, false) => Some("file.save"),
        (0x53, true) => Some("file.save_as"),
        (0x5a, false) => Some("edit.undo"),
        (0x59, false) => Some("edit.redo"),
        _ => None,
    }
}

pub fn dispatch<R: Runtime>(
    app: &AppHandle<R>,
    controller: &Controller,
    id: &str,
) -> Result<(), DocumentError> {
    let context = main_context(app, CommandOrigin::Menu)?;
    context.validate()?;
    let kind = match id {
        "file.new" => Some(DocumentKind::New),
        "file.open" => Some(DocumentKind::Open),
        "file.save" => Some(DocumentKind::Save),
        "file.save_as" => Some(DocumentKind::SaveAs),
        "file.export_markdown" => Some(DocumentKind::ExportMarkdown),
        "file.quit" => Some(DocumentKind::Quit),
        _ => None,
    };
    if let Some(kind) = kind {
        request_document(controller, app, context, kind)?;
    } else {
        match id {
            "edit.undo" | "edit.redo" => {
                if controller.document_operation()?.is_some() {
                    return Err(DocumentErrorCode::Busy.error());
                }
                app.emit_to("main", DOCUMENT_COMMAND_EVENT, serde_json::json!({
                    "type": "history", "dispatchId": next_dispatch(), "direction": if id == "edit.undo" { "undo" } else { "redo" },
                    "area": controller.document_focused_area()?,
                })).map_err(|_| DocumentErrorCode::WindowFailed.error())?;
            }
            "edit.preferences" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if open_preferences(&app).await.is_err() {
                        crate::diagnostics::event("Préférences indisponibles ; note conservée.");
                    }
                });
            }
            "help.about" => {
                if controller.document_operation()?.is_some() {
                    return Err(DocumentErrorCode::Busy.error());
                }
                about(app)?;
            }
            _ => return Err(DocumentErrorCode::Forbidden.error()),
        }
    }
    sync_main(app, controller)
}

fn next_dispatch() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    format!("native:{}", NEXT.fetch_add(1, Ordering::Relaxed))
}

pub fn sync_main<R: Runtime>(
    app: &AppHandle<R>,
    controller: &Controller,
) -> Result<(), DocumentError> {
    let snapshot = controller.snapshot()?;
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| DocumentErrorCode::WindowFailed.error())?;
    main.set_title(&format!(
        "{}{} — {}",
        snapshot.document_name,
        if snapshot.dirty { " *" } else { "" },
        app.package_info().name
    ))
    .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
    if let Some(menu) = main.menu() {
        let history = match controller.document_focused_area()? {
            crate::domain::DictationTarget::Content => snapshot.history.content,
            crate::domain::DictationTarget::Context => snapshot.history.context,
            crate::domain::DictationTarget::Instructions => snapshot.history.instructions,
        };
        let enabled = snapshot.document_operation.is_none() && !controller.is_preferences_modal();
        for (id, available) in [
            ("edit.undo", history.can_undo),
            ("edit.redo", history.can_redo),
        ] {
            if let Some(item) = menu
                .get("menu.edit")
                .and_then(|item| item.as_submenu().cloned())
                .and_then(|edit| edit.get(id))
                .and_then(|item| item.as_menuitem().cloned())
            {
                item.set_enabled(enabled && available)
                    .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
            }
        }
    }
    Ok(())
}

/// All native handle/GTK work executes on the window thread; only the result crosses threads.
async fn modality<R: Runtime>(
    app: &AppHandle<R>,
    token: u64,
    enabled: bool,
) -> Result<(), DocumentError> {
    let (send, receive) = tokio::sync::oneshot::channel();
    let handle = app.clone();
    app.run_on_main_thread(move || {
        let result = (|| {
            if handle.state::<PrototypeState>().0.preferences_token() != token {
                return Ok(());
            }
            let Some(main) = handle.get_webview_window("main") else {
                return Ok(());
            };
            #[cfg(windows)]
            {
                use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
                    EnableWindow, IsWindowEnabled,
                };
                let hwnd = main
                    .hwnd()
                    .map_err(|_| DocumentErrorCode::WindowFailed.error())?
                    .0 as _;
                unsafe {
                    EnableWindow(hwnd, if enabled { 0 } else { 1 });
                    if (IsWindowEnabled(hwnd) != 0) == enabled {
                        return Err(DocumentErrorCode::WindowFailed.error());
                    }
                }
            }
            #[cfg(target_os = "linux")]
            if let Some(preferences) = handle.get_webview_window("preferences") {
                use gtk::prelude::*;
                let parent = main
                    .gtk_window()
                    .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
                let child = preferences
                    .gtk_window()
                    .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
                child.set_transient_for(Some(&parent));
                child.set_destroy_with_parent(true);
                child.set_modal(enabled);
                if child.is_modal() != enabled
                    || child.transient_for().as_ref() != Some(parent.upcast_ref::<gtk::Window>())
                {
                    return Err(DocumentErrorCode::WindowFailed.error());
                }
            }
            Ok(())
        })();
        let _ = send.send(result);
    })
    .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
    receive
        .await
        .map_err(|_| DocumentErrorCode::WindowFailed.error())?
}

pub async fn open_preferences<R: Runtime>(app: &AppHandle<R>) -> Result<(), DocumentError> {
    let state = app.state::<PrototypeState>();
    let Some(token) = state.0.begin_preferences()? else {
        return app
            .get_webview_window("preferences")
            .ok_or_else(|| DocumentErrorCode::Busy.error())?
            .set_focus()
            .map_err(|_| DocumentErrorCode::WindowFailed.error());
    };
    let mut guard = ModalGuard {
        app: app.clone(),
        token,
        armed: true,
    };
    let result = async {
        let main = app
            .get_webview_window("main")
            .ok_or_else(|| DocumentErrorCode::WindowFailed.error())?;
        let builder =
            WebviewWindowBuilder::new(app, "preferences", WebviewUrl::App("index.html".into()))
                .title("Préférences")
                .inner_size(600.0, 620.0)
                .resizable(false)
                .visible(false)
                .decorations(true);
        #[cfg(windows)]
        let builder = builder
            .owner(&main)
            .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
        #[cfg(target_os = "linux")]
        let builder = builder
            .transient_for(&main)
            .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
        let window = builder
            .build()
            .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
        let events_app = app.clone();
        window.on_window_event(move |event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let app = events_app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = close_instance(&app, token).await;
                });
            }
            tauri::WindowEvent::Destroyed => {
                let app = events_app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = cleanup_preferences(&app, token).await;
                });
            }
            _ => {}
        });
        modality(app, token, true).await?;
        window
            .show()
            .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
        window
            .set_focus()
            .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
        sync_main(app, &state.0)
    }
    .await;
    if result.is_err() {
        if close_instance(app, token).await.is_ok() {
            guard.armed = false;
        }
    } else {
        guard.armed = false;
    }
    result
}

struct ModalGuard<R: Runtime> {
    app: AppHandle<R>,
    token: u64,
    armed: bool,
}
impl<R: Runtime> Drop for ModalGuard<R> {
    fn drop(&mut self) {
        if self.armed {
            let app = self.app.clone();
            let token = self.token;
            tauri::async_runtime::spawn(async move {
                let _ = close_instance(&app, token).await;
            });
        }
    }
}

async fn cleanup_preferences<R: Runtime>(
    app: &AppHandle<R>,
    token: u64,
) -> Result<(), DocumentError> {
    modality(app, token, false).await?;
    let state = app.state::<PrototypeState>();
    if !state.0.end_preferences(token) {
        return Ok(());
    }
    if let Some(main) = app.get_webview_window("main") {
        main.set_focus()
            .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
        app.emit_to("main", DOCUMENT_COMMAND_EVENT, serde_json::json!({
            "type": "restore_focus", "dispatchId": next_dispatch(), "area": state.0.document_focused_area()?,
        })).map_err(|_| DocumentErrorCode::WindowFailed.error())?;
        sync_main(app, &state.0)?;
    }
    Ok(())
}

pub async fn close_preferences<R: Runtime>(app: &AppHandle<R>) -> Result<(), DocumentError> {
    let token = app.state::<PrototypeState>().0.preferences_token();
    close_instance(app, token).await
}

async fn close_instance<R: Runtime>(app: &AppHandle<R>, token: u64) -> Result<(), DocumentError> {
    if !app
        .state::<PrototypeState>()
        .0
        .begin_preferences_close(token)?
    {
        return Ok(());
    }
    // Re-enable before destroying, including OK, Cancel, Escape and the system cross.
    modality(app, token, false).await?;
    if app.state::<PrototypeState>().0.preferences_token() != token {
        return Ok(());
    }
    if let Some(window) = app.get_webview_window("preferences") {
        window
            .destroy()
            .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
    }
    cleanup_preferences(app, token).await
}

pub fn about<R: Runtime>(app: &AppHandle<R>) -> Result<(), DocumentError> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| DocumentErrorCode::WindowFailed.error())?;
    let text = format!("{} {}", app.package_info().name, app.package_info().version);
    #[cfg(windows)]
    {
        use tauri_plugin_dialog::DialogExt;
        main.hwnd()
            .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
        app.dialog()
            .message(text)
            .title("A propos")
            .parent(&main)
            .show(|_| {});
    }

    #[cfg(target_os = "linux")]
    {
        app.run_on_main_thread(move || {
            use gtk::prelude::*;
            if let Ok(parent) = main.gtk_window() {
                let dialog = gtk::MessageDialog::new(
                    Some(&parent),
                    gtk::DialogFlags::MODAL | gtk::DialogFlags::DESTROY_WITH_PARENT,
                    gtk::MessageType::Info,
                    gtk::ButtonsType::Ok,
                    &text,
                );
                dialog.set_title("A propos");
                dialog.connect_response(|dialog, _| dialog.close());
                dialog.show_all();
            }
        })
        .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn webview_keys_keep_save_as_distinct_and_leave_altgr_and_local_typing_alone() {
        assert_eq!(
            windows_key_command(0x53, true, false, false),
            Some("file.save")
        );
        assert_eq!(
            windows_key_command(0x53, true, false, true),
            Some("file.save_as")
        );
        assert_eq!(
            windows_key_command(0x5a, true, false, false),
            Some("edit.undo")
        );
        assert_eq!(
            windows_key_command(0x59, true, false, false),
            Some("edit.redo")
        );
        assert_eq!(
            windows_key_command(0x73, false, true, false),
            Some("file.quit")
        );
        assert_eq!(windows_key_command(0x45, true, true, false), None);
        assert_eq!(windows_key_command(0x53, false, false, false), None);
        assert_eq!(windows_key_command(0x53, true, true, false), None);
        assert_eq!(windows_key_command(0x5a, true, false, true), None);
    }

    #[test]
    fn ten_native_effects_have_exact_ids_mnemonics_and_seven_accelerators() {
        assert_eq!(
            ITEMS.map(|item| item.0),
            [
                "file.new",
                "file.open",
                "file.save",
                "file.save_as",
                "file.export_markdown",
                "file.quit",
                "edit.preferences",
                "edit.undo",
                "edit.redo",
                "help.about",
            ]
        );
        assert_eq!(ITEMS.iter().filter(|item| item.2.is_some()).count(), 7);
        assert_eq!(
            ITEMS.map(|item| item.1.replace('&', "")),
            [
                "Nouvelle note",
                "Ouvrir",
                "Enregistrer",
                "Enregistrer sous",
                "Exporter en markdown",
                "Quitter",
                "Préférence",
                "Annuler",
                "Rétablir",
                "A propos",
            ]
        );
        for range in [0..6, 6..9, 9..10] {
            let mut keys = std::collections::HashSet::new();
            for item in &ITEMS[range] {
                let key = item
                    .1
                    .split_once('&')
                    .unwrap()
                    .1
                    .chars()
                    .next()
                    .unwrap()
                    .to_ascii_lowercase();
                assert!(keys.insert(key));
            }
        }
    }
}

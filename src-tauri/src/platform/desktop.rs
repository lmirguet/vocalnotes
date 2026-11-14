use crate::app::document::{
    DocumentError, DocumentErrorCode, DocumentKind, DocumentOperation, OperationId,
};
use crate::platform::controller::Controller;
use crate::{
    app::document::DocumentFormat,
    storage::markdown_file::{
        DiskVersion, OverwriteConsent, complete_note_path, validate_note_path,
    },
};
use serde::Serialize;
use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_dialog::{FilePath, MessageDialogResult};
use tokio::sync::oneshot;

pub const DOCUMENT_COMMAND_EVENT: &str = "document-command";
pub use super::native_ui::sync_main;

pub type DialogFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, DocumentError>> + Send + 'a>>;

pub enum FileDialogRequest {
    OpenNote,
    SaveNote { proposed_name: String },
    ExportMarkdown { proposed_name: String },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveDecision {
    Save,
    Discard,
    Cancel,
}

pub enum OverwriteDecision {
    Replace(OverwriteConsent),
    ChooseAnother,
    Cancel,
}

pub trait DocumentDialogs: Send + Sync {
    /// Windows' plugin cannot dismiss an already shown picker on future drop.
    fn dismisses_on_drop(&self) -> bool {
        false
    }
    fn select_file(&self, request: FileDialogRequest) -> DialogFuture<'_, Option<PathBuf>>;
    fn confirm_save(&self, document_name: String) -> DialogFuture<'_, SaveDecision>;
    fn confirm_overwrite(
        &self,
        path: PathBuf,
        version: DiskVersion,
    ) -> DialogFuture<'_, OverwriteDecision>;
}

const SAVE: &str = "Enregistrer";
const DISCARD: &str = "Ne pas enregistrer";
const CANCEL: &str = "Annuler";
const REPLACE: &str = "Remplacer";
const CHOOSE_ANOTHER: &str = "Choisir un autre fichier";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Confirmation {
    Primary,
    Secondary,
    Cancel,
}

fn confirmation(
    response: MessageDialogResult,
    primary: &str,
    secondary: &str,
) -> Result<Confirmation, DocumentError> {
    match response {
        MessageDialogResult::Yes => Ok(Confirmation::Primary),
        MessageDialogResult::No => Ok(Confirmation::Secondary),
        MessageDialogResult::Cancel => Ok(Confirmation::Cancel),
        MessageDialogResult::Custom(label) if label == primary => Ok(Confirmation::Primary),
        MessageDialogResult::Custom(label) if label == secondary => Ok(Confirmation::Secondary),
        MessageDialogResult::Custom(label) if label == CANCEL => Ok(Confirmation::Cancel),
        _ => Err(DocumentErrorCode::DialogFailed.error()),
    }
}

impl FileDialogRequest {
    fn title(&self) -> &'static str {
        match self {
            Self::OpenNote => "Ouvrir",
            Self::SaveNote { .. } => "Enregistrer sous",
            Self::ExportMarkdown { .. } => "Exporter en markdown",
        }
    }

    fn format(&self) -> Option<DocumentFormat> {
        match self {
            Self::OpenNote => None,
            Self::SaveNote { .. } => Some(DocumentFormat::Von),
            Self::ExportMarkdown { .. } => Some(DocumentFormat::Markdown),
        }
    }

    fn proposed_name(&self) -> Result<Option<String>, DocumentError> {
        let name = match self {
            Self::OpenNote => return Ok(None),
            Self::SaveNote { proposed_name } | Self::ExportMarkdown { proposed_name } => {
                proposed_name
            }
        };
        if name.is_empty()
            || name.len() > 4096
            || name.contains('\0')
            || Path::new(name).file_name().and_then(|leaf| leaf.to_str()) != Some(name.as_str())
        {
            return Err(DocumentErrorCode::InvalidPath.error());
        }
        let extension = if self.format() == Some(DocumentFormat::Von) {
            "von"
        } else {
            "md"
        };
        let proposed = Path::new(name).with_extension(extension);
        Ok(Some(
            proposed
                .to_str()
                .ok_or_else(|| DocumentErrorCode::InvalidPath.error())?
                .into(),
        ))
    }
}

fn selected_file(
    request: &FileDialogRequest,
    selected: Option<FilePath>,
) -> Result<Option<PathBuf>, DocumentError> {
    let Some(selected) = selected else {
        return Ok(None);
    };
    let path = selected
        .into_path()
        .map_err(|_| DocumentErrorCode::InvalidPath.error())?;
    let path = if let Some(format) = request.format() {
        complete_note_path(&path, format)
    } else {
        validate_note_path(&path).map(|_| path)
    }
    .map_err(|error| DocumentError {
        code: match error {
            crate::storage::markdown_file::NotePathError::InvalidPath => {
                DocumentErrorCode::InvalidPath
            }
            crate::storage::markdown_file::NotePathError::UnsupportedFormat => {
                DocumentErrorCode::UnsupportedFormat
            }
        },
        message: error.message(),
    })?;
    Ok(Some(path))
}

async fn receive_dialog<T>(receiver: oneshot::Receiver<T>) -> Result<T, DocumentError> {
    receiver
        .await
        .map_err(|_| DocumentErrorCode::DialogFailed.error())
}

#[derive(Clone)]
pub struct NativeDialogs<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> NativeDialogs<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }

    #[cfg(windows)]
    fn parent(&self) -> Result<tauri::WebviewWindow<R>, DocumentError> {
        let parent = self
            .app
            .get_webview_window("main")
            .ok_or_else(|| DocumentErrorCode::WindowFailed.error())?;
        parent
            .hwnd()
            .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
        Ok(parent)
    }

    async fn message(
        &self,
        message: String,
        primary: &'static str,
        secondary: &'static str,
    ) -> Result<Confirmation, DocumentError> {
        #[cfg(windows)]
        let response = {
            use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
            let parent = self.parent()?;
            let (sender, receiver) = oneshot::channel();
            self.app
                .dialog()
                .message(message)
                .title("Vocalnotes")
                .parent(&parent)
                .kind(MessageDialogKind::Warning)
                .buttons(MessageDialogButtons::YesNoCancelCustom(
                    primary.into(),
                    secondary.into(),
                    CANCEL.into(),
                ))
                .show_with_result(move |response| {
                    let _ = sender.send(response);
                });
            receive_dialog(receiver).await?
        };
        #[cfg(target_os = "linux")]
        let response = gtk_dialogs::message(&self.app, message, primary, secondary).await?;
        #[cfg(not(any(windows, target_os = "linux")))]
        return Err(DocumentErrorCode::DialogFailed.error());
        #[cfg(any(windows, target_os = "linux"))]
        confirmation(response, primary, secondary)
    }
}

impl<R: Runtime> DocumentDialogs for NativeDialogs<R> {
    fn dismisses_on_drop(&self) -> bool {
        cfg!(target_os = "linux")
    }
    fn select_file(&self, request: FileDialogRequest) -> DialogFuture<'_, Option<PathBuf>> {
        Box::pin(async move {
            let proposed_name = request.proposed_name()?;
            #[cfg(windows)]
            let selected = {
                use tauri_plugin_dialog::DialogExt;
                let parent = self.parent()?;
                let mut builder = self
                    .app
                    .dialog()
                    .file()
                    .set_parent(&parent)
                    .set_title(request.title());
                builder = match request.format() {
                    None => builder.add_filter("Notes Vocalnotes et Markdown", &["von", "md"]),
                    Some(DocumentFormat::Von) => builder.add_filter("Note Vocalnotes", &["von"]),
                    Some(DocumentFormat::Markdown) => builder.add_filter("Markdown", &["md"]),
                };
                if let Some(name) = proposed_name {
                    builder = builder.set_file_name(name);
                }
                let (sender, receiver) = oneshot::channel();
                if matches!(request, FileDialogRequest::OpenNote) {
                    builder.pick_file(move |path| {
                        let _ = sender.send(path);
                    });
                } else {
                    builder.save_file(move |path| {
                        let _ = sender.send(path);
                    });
                }
                receive_dialog(receiver).await?
            };
            #[cfg(target_os = "linux")]
            let selected = gtk_dialogs::select_file(
                &self.app,
                request.title(),
                request.format(),
                proposed_name,
            )
            .await?;
            #[cfg(not(any(windows, target_os = "linux")))]
            return Err(DocumentErrorCode::DialogFailed.error());
            #[cfg(any(windows, target_os = "linux"))]
            selected_file(&request, selected)
        })
    }

    fn confirm_save(&self, document_name: String) -> DialogFuture<'_, SaveDecision> {
        Box::pin(async move {
            match self
                .message(
                    format!("Enregistrer les modifications de « {document_name} » ?"),
                    SAVE,
                    DISCARD,
                )
                .await?
            {
                Confirmation::Primary => Ok(SaveDecision::Save),
                Confirmation::Secondary => Ok(SaveDecision::Discard),
                Confirmation::Cancel => Ok(SaveDecision::Cancel),
            }
        })
    }

    fn confirm_overwrite(
        &self,
        path: PathBuf,
        version: DiskVersion,
    ) -> DialogFuture<'_, OverwriteDecision> {
        Box::pin(async move {
            let consent =
                OverwriteConsent::new(path.clone(), version).map_err(|error| DocumentError {
                    code: error.code,
                    message: error.message,
                })?;
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| DocumentErrorCode::InvalidPath.error())?;
            match self.message(format!("« {name} » existe déjà ou a été modifié ailleurs. Remplacer la version observée ?"), REPLACE, CHOOSE_ANOTHER).await? {
                Confirmation::Primary => Ok(OverwriteDecision::Replace(consent)),
                Confirmation::Secondary => Ok(OverwriteDecision::ChooseAnother),
                Confirmation::Cancel => Ok(OverwriteDecision::Cancel),
            }
        })
    }
}

#[cfg(target_os = "linux")]
mod gtk_dialogs {
    use super::*;
    use gtk::{ResponseType, prelude::*};
    use std::{cell::RefCell, rc::Rc};

    async fn show<R, T, D>(
        app: &AppHandle<R>,
        build: impl FnOnce(&gtk::ApplicationWindow) -> D + Send + 'static,
        response: impl Fn(&D, ResponseType) -> Result<T, DocumentError> + Send + 'static,
    ) -> Result<T, DocumentError>
    where
        R: Runtime,
        T: Send + 'static,
        D: IsA<gtk::Dialog> + IsA<gtk::Window> + IsA<gtk::Widget> + 'static,
    {
        let (sender, receiver) = oneshot::channel();
        let (_cancel_on_drop, cancelled) = oneshot::channel::<()>();
        let handle = app.clone();
        app.run_on_main_thread(move || {
            if sender.is_closed() {
                return;
            }
            let opened = (|| {
                let parent = handle
                    .get_webview_window("main")
                    .ok_or_else(|| DocumentErrorCode::WindowFailed.error())?
                    .gtk_window()
                    .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
                let dialog = build(&parent);
                dialog.set_transient_for(Some(&parent));
                dialog.set_modal(true);
                dialog.set_destroy_with_parent(true);
                if dialog.transient_for().as_ref() != Some(parent.upcast_ref())
                    || !dialog.is_modal()
                {
                    unsafe {
                        dialog.destroy();
                    }
                    return Err(DocumentErrorCode::WindowFailed.error());
                }
                Ok(dialog)
            })();
            let dialog = match opened {
                Ok(dialog) => dialog,
                Err(error) => {
                    let _ = sender.send(Err(error));
                    return;
                }
            };
            let reply = Rc::new(RefCell::new(Some(sender)));
            let on_response = reply.clone();
            dialog.connect_response(move |dialog, value| {
                if let Some(sender) = on_response.borrow_mut().take() {
                    let _ = sender.send(response(dialog, value));
                }
                unsafe {
                    dialog.destroy();
                }
            });
            dialog.connect_destroy(move |_| {
                if let Some(sender) = reply.borrow_mut().take() {
                    let _ = sender.send(Err(DocumentErrorCode::WindowFailed.error()));
                }
            });
            let weak = dialog.downgrade();
            gtk::glib::MainContext::default().spawn_local(async move {
                let _ = cancelled.await;
                if let Some(dialog) = weak.upgrade() {
                    unsafe {
                        dialog.destroy();
                    }
                }
            });
            dialog.show_all();
        })
        .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
        receive_dialog(receiver).await?
    }

    pub(super) async fn select_file<R: Runtime>(
        app: &AppHandle<R>,
        title: &'static str,
        format: Option<DocumentFormat>,
        proposed_name: Option<String>,
    ) -> Result<Option<FilePath>, DocumentError> {
        show(
            app,
            move |parent| {
                let action = if format.is_some() {
                    gtk::FileChooserAction::Save
                } else {
                    gtk::FileChooserAction::Open
                };
                let dialog = gtk::FileChooserDialog::with_buttons(
                    Some(title),
                    Some(parent),
                    action,
                    &[
                        (CANCEL, ResponseType::Cancel),
                        (
                            if format.is_some() { SAVE } else { "Ouvrir" },
                            ResponseType::Accept,
                        ),
                    ],
                );
                dialog.set_local_only(true);
                dialog.set_select_multiple(false);
                // Replacement consent is obtained separately and bound to DiskVersion.
                dialog.set_do_overwrite_confirmation(false);
                dialog.set_default_response(ResponseType::Accept);
                if let Some(name) = proposed_name {
                    dialog.set_current_name(&name);
                }
                let filter = gtk::FileFilter::new();
                filter.set_name(Some(match format {
                    None => "Notes Vocalnotes et Markdown",
                    Some(DocumentFormat::Von) => "Note Vocalnotes",
                    Some(DocumentFormat::Markdown) => "Markdown",
                }));
                filter.add_custom(gtk::FileFilterFlags::DISPLAY_NAME, move |info| {
                    info.display_name()
                        .and_then(|name| DocumentFormat::from_path(Path::new(name)))
                        .is_some_and(|actual| format.is_none_or(|expected| actual == expected))
                });
                dialog.add_filter(filter);
                dialog
            },
            |dialog, response| match response {
                ResponseType::Accept | ResponseType::Ok => dialog
                    .filename()
                    .map(FilePath::Path)
                    .map(Some)
                    .ok_or_else(|| DocumentErrorCode::InvalidPath.error()),
                ResponseType::Cancel | ResponseType::DeleteEvent | ResponseType::Close => Ok(None),
                _ => Err(DocumentErrorCode::DialogFailed.error()),
            },
        )
        .await
    }

    pub(super) fn message_response(
        response: ResponseType,
    ) -> Result<MessageDialogResult, DocumentError> {
        match response {
            ResponseType::Yes => Ok(MessageDialogResult::Yes),
            ResponseType::No => Ok(MessageDialogResult::No),
            ResponseType::Cancel | ResponseType::DeleteEvent | ResponseType::Close => {
                Ok(MessageDialogResult::Cancel)
            }
            _ => Err(DocumentErrorCode::DialogFailed.error()),
        }
    }

    pub(super) async fn message<R: Runtime>(
        app: &AppHandle<R>,
        message: String,
        primary: &'static str,
        secondary: &'static str,
    ) -> Result<MessageDialogResult, DocumentError> {
        show(
            app,
            move |parent| {
                let dialog = gtk::MessageDialog::new(
                    Some(parent),
                    gtk::DialogFlags::MODAL | gtk::DialogFlags::DESTROY_WITH_PARENT,
                    gtk::MessageType::Question,
                    gtk::ButtonsType::None,
                    &message,
                );
                dialog.set_title("Vocalnotes");
                dialog.add_buttons(&[
                    (CANCEL, ResponseType::Cancel),
                    (secondary, ResponseType::No),
                    (primary, ResponseType::Yes),
                ]);
                dialog.set_default_response(ResponseType::Cancel);
                dialog
            },
            |_, response| message_response(response),
        )
        .await
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum DocumentDispatch {
    Prepare {
        dispatch_id: String,
        operation_id: OperationId,
    },
}

pub trait DocumentEvents {
    fn prepare(&self, operation: &DocumentOperation) -> Result<(), DocumentError>;
}

impl<R: Runtime> DocumentEvents for AppHandle<R> {
    fn prepare(&self, operation: &DocumentOperation) -> Result<(), DocumentError> {
        self.emit_to(
            "main",
            DOCUMENT_COMMAND_EVENT,
            DocumentDispatch::Prepare {
                dispatch_id: format!("prepare:{}", String::from(operation.operation_id)),
                operation_id: operation.operation_id,
            },
        )
        .map_err(|_| DocumentErrorCode::WindowFailed.error())
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum CommandOrigin {
    Menu,
    Window,
}

pub struct DesktopContext<'a> {
    pub window_label: &'a str,
    pub origin: CommandOrigin,
    pub focused: bool,
    pub preferences_modal: bool,
}

impl DesktopContext<'_> {
    pub fn validate(&self) -> Result<(), DocumentError> {
        if self.window_label != "main" || (self.origin == CommandOrigin::Menu && !self.focused) {
            return Err(DocumentErrorCode::Forbidden.error());
        }
        if self.preferences_modal {
            return Err(DocumentErrorCode::Busy.error());
        }
        Ok(())
    }
}

pub fn main_context<R: Runtime>(
    app: &AppHandle<R>,
    origin: CommandOrigin,
) -> Result<DesktopContext<'static>, DocumentError> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| DocumentErrorCode::WindowFailed.error())?;
    #[cfg(windows)]
    let focused = {
        let hwnd = main
            .hwnd()
            .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
        // Keyboard focus belongs to WebView2's child HWND while editing.
        unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow() == hwnd.0 as _ }
    };
    #[cfg(not(windows))]
    let focused = main
        .is_focused()
        .map_err(|_| DocumentErrorCode::WindowFailed.error())?;
    Ok(DesktopContext {
        window_label: "main",
        origin,
        focused,
        preferences_modal: app.get_webview_window("preferences").is_some(),
    })
}

pub fn request_document(
    controller: &Controller,
    events: &impl DocumentEvents,
    context: DesktopContext<'_>,
    action: DocumentKind,
) -> Result<DocumentOperation, DocumentError> {
    context.validate()?;
    let operation = controller.request_document(action)?;
    if let Err(error) = events.prepare(&operation) {
        controller.fail_document(operation.operation_id, error.clone())?;
        return Err(error);
    }
    Ok(operation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::probe::Probe;

    struct Events<'a> {
        controller: &'a Controller,
        fail: bool,
    }
    impl DocumentEvents for Events<'_> {
        fn prepare(&self, operation: &DocumentOperation) -> Result<(), DocumentError> {
            assert!(self.controller.core.try_lock().is_ok());
            assert_eq!(
                self.controller.document_operation()?.as_ref(),
                Some(operation)
            );
            if self.fail {
                Err(DocumentErrorCode::WindowFailed.error())
            } else {
                Ok(())
            }
        }
    }

    fn context() -> DesktopContext<'static> {
        DesktopContext {
            window_label: "main",
            origin: CommandOrigin::Menu,
            focused: true,
            preferences_modal: false,
        }
    }

    #[test]
    fn confirmations_accept_only_known_typed_or_custom_responses() {
        use tauri_plugin_dialog::MessageDialogResult as Response;
        for (primary, secondary) in [(SAVE, DISCARD), (REPLACE, CHOOSE_ANOTHER)] {
            for (response, expected) in [
                (Response::Yes, Confirmation::Primary),
                (Response::Custom(primary.into()), Confirmation::Primary),
                (Response::No, Confirmation::Secondary),
                (Response::Custom(secondary.into()), Confirmation::Secondary),
                (Response::Cancel, Confirmation::Cancel),
                (Response::Custom(CANCEL.into()), Confirmation::Cancel),
            ] {
                assert_eq!(
                    confirmation(response, primary, secondary).unwrap(),
                    expected
                );
            }
            for response in [Response::Ok, Response::Custom("unknown".into())] {
                assert_eq!(
                    confirmation(response, primary, secondary).unwrap_err().code,
                    DocumentErrorCode::DialogFailed,
                );
            }
        }
    }

    #[test]
    fn file_responses_complete_only_missing_extensions_and_reject_nonlocal_choices() {
        use tauri_plugin_dialog::FilePath;
        let root = std::env::temp_dir();
        let save = FileDialogRequest::SaveNote {
            proposed_name: "note.md".into(),
        };
        assert_eq!(save.proposed_name().unwrap().as_deref(), Some("note.von"));
        assert_eq!(selected_file(&save, None).unwrap(), None);
        for name in ["note", "note.VON"] {
            let actual = selected_file(&save, Some(FilePath::Path(root.join(name))))
                .unwrap()
                .unwrap();
            assert_eq!(
                actual,
                root.join(if name == "note" { "note.von" } else { name })
            );
        }
        let local = root.join("note.MD");
        assert_eq!(
            selected_file(
                &FileDialogRequest::OpenNote,
                Some(FilePath::Url(tauri::Url::from_file_path(&local).unwrap(),))
            )
            .unwrap(),
            Some(local),
        );
        let export = FileDialogRequest::ExportMarkdown {
            proposed_name: "note.von".into(),
        };
        assert_eq!(export.proposed_name().unwrap().as_deref(), Some("note.md"));
        assert_eq!(
            selected_file(&export, Some(FilePath::Path(root.join("note.MD")))).unwrap(),
            Some(root.join("note.MD")),
        );
        for (request, path) in [
            (&save, FilePath::Path(root.join("note.md"))),
            (&export, FilePath::Path(root.join("note.von"))),
            (
                &FileDialogRequest::OpenNote,
                FilePath::Path(root.join("note.txt")),
            ),
        ] {
            assert_eq!(
                selected_file(request, Some(path)).unwrap_err().code,
                DocumentErrorCode::UnsupportedFormat
            );
        }
        for path in [
            FilePath::Path(PathBuf::from("relative.von")),
            FilePath::Url("https://example.invalid/note.von".parse().unwrap()),
            FilePath::Url("file://server/share/note.von".parse().unwrap()),
        ] {
            assert_eq!(
                selected_file(&FileDialogRequest::OpenNote, Some(path))
                    .unwrap_err()
                    .code,
                DocumentErrorCode::InvalidPath
            );
        }
        for name in ["../note", ".", "bad\0name"] {
            let request = FileDialogRequest::SaveNote {
                proposed_name: name.into(),
            };
            assert_eq!(
                request.proposed_name().unwrap_err().code,
                DocumentErrorCode::InvalidPath
            );
        }
        #[cfg(windows)]
        assert!(
            FileDialogRequest::SaveNote {
                proposed_name: "folder\\note".into()
            }
            .proposed_name()
            .is_err()
        );
    }

    #[tokio::test]
    async fn a_dropped_native_callback_is_an_error_not_user_cancellation() {
        let (sender, receiver) = tokio::sync::oneshot::channel::<Option<PathBuf>>();
        drop(sender);
        assert_eq!(
            receive_dialog(receiver).await.unwrap_err().code,
            DocumentErrorCode::DialogFailed
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn gtk_response_mapping_does_not_authorize_an_unknown_button() {
        use gtk::ResponseType as Response;
        for response in [Response::Cancel, Response::DeleteEvent, Response::Close] {
            assert_eq!(
                gtk_dialogs::message_response(response).unwrap(),
                tauri_plugin_dialog::MessageDialogResult::Cancel
            );
        }
        assert_eq!(
            gtk_dialogs::message_response(Response::Yes).unwrap(),
            tauri_plugin_dialog::MessageDialogResult::Yes
        );
        for response in [Response::None, Response::Accept, Response::Other(71)] {
            assert_eq!(
                gtk_dialogs::message_response(response).unwrap_err().code,
                DocumentErrorCode::DialogFailed
            );
        }
    }

    #[test]
    fn other_windows_inactive_menus_and_modal_preferences_cannot_start_an_operation() {
        let controller = Controller::new(Probe::new("note".into()));
        for context in [
            DesktopContext {
                window_label: "preferences",
                ..context()
            },
            DesktopContext {
                focused: false,
                ..context()
            },
            DesktopContext {
                preferences_modal: true,
                ..context()
            },
        ] {
            assert!(
                request_document(
                    &controller,
                    &Events {
                        controller: &controller,
                        fail: false
                    },
                    context,
                    DocumentKind::New
                )
                .is_err()
            );
            assert!(controller.document_operation().unwrap().is_none());
        }
    }

    #[test]
    fn window_close_can_prepare_an_unfocused_main_and_notification_holds_no_core_lock() {
        let controller = Controller::new(Probe::new("texte conservé".into()));
        let operation = request_document(
            &controller,
            &Events {
                controller: &controller,
                fail: false,
            },
            DesktopContext {
                origin: CommandOrigin::Window,
                focused: false,
                ..context()
            },
            DocumentKind::Quit,
        )
        .unwrap();
        let payload = serde_json::to_value(DocumentDispatch::Prepare {
            dispatch_id: format!("prepare:{}", String::from(operation.operation_id)),
            operation_id: operation.operation_id,
        })
        .unwrap();
        assert_eq!(payload.as_object().unwrap().len(), 3);
        assert!(payload.get("operationId").unwrap().is_string());
        assert!(!payload.to_string().contains("texte conservé"));
    }

    #[test]
    fn notification_failure_is_visible_and_preserves_the_note() {
        let controller = Controller::new(Probe::new("texte conservé".into()));
        let result = request_document(
            &controller,
            &Events {
                controller: &controller,
                fail: true,
            },
            context(),
            DocumentKind::Quit,
        );
        assert_eq!(result.unwrap_err().code, DocumentErrorCode::WindowFailed);
        let snapshot = controller.snapshot().unwrap();
        assert!(snapshot.document_operation.is_none());
        assert_eq!(snapshot.source, "texte conservé");
        assert_eq!(
            snapshot.document_error.unwrap().code,
            DocumentErrorCode::WindowFailed
        );
    }
}

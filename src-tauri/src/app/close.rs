//! Closing is stop/invalidate -> save -> release core lock -> destroy window.
//! A failed save never destroys the window or restarts capture.
use crate::app::document::{DocumentError, DocumentKind, DocumentOperation, OperationId};
use crate::platform::desktop::{DesktopContext, DocumentEvents};
use crate::platform::{controller::Controller, probe::Snapshot};

/// System cross, Alt+F4 and menu Quit all register this same intention.
pub fn request(
    controller: &Controller,
    events: &impl DocumentEvents,
    context: DesktopContext<'_>,
) -> Result<DocumentOperation, DocumentError> {
    crate::platform::desktop::request_document(controller, events, context, DocumentKind::Quit)
}

/// A renderer cannot call this; destruction requires a completed native Quit.
pub fn finish_document(
    controller: &Controller,
    id: OperationId,
    destroy: impl FnOnce() -> Result<(), ()>,
) -> Result<Snapshot, DocumentError> {
    controller.finish_quit(id, destroy)
}

pub fn begin(controller: &Controller) -> Result<(), &'static str> {
    controller.stop(
        false,
        "Fermeture : capture arrêtée et traitements invalidés.",
    );
    controller
        .core
        .lock()
        .map_err(|_| "Cœur indisponible.")?
        .prepare_close();
    Ok(())
}

pub fn finish(
    controller: &Controller,
    destroy: impl FnOnce() -> Result<(), ()>,
) -> Result<Snapshot, &'static str> {
    controller.stop(
        false,
        "Fermeture explicite : capture arrêtée et traitements invalidés.",
    );
    let (can_close, snapshot) = {
        let mut core = controller.core.lock().map_err(|_| "Cœur indisponible.")?;
        (core.request_close(), core.snapshot())
    };
    if can_close {
        controller.remember_saved_note();
    }
    if can_close && destroy().is_err() {
        controller
            .core
            .lock()
            .map_err(|_| "Cœur indisponible.")?
            .message("Fermeture impossible ; note sauvegardée et conservée.");
        return Err("Fermeture impossible ; note sauvegardée et conservée.");
    }
    Ok(snapshot)
}

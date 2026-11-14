use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::{Notify, oneshot};
use vocalnotes_lib::{
    app::document::{DocumentKind, DocumentVersion},
    domain::note::Note,
    platform::{
        controller::Controller,
        desktop::{
            DialogFuture, DocumentDialogs, FileDialogRequest, OverwriteDecision, SaveDecision,
        },
    },
    storage::markdown_file::DiskVersion,
};
struct Dropped(Arc<AtomicBool>);
impl Drop for Dropped {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}
struct Waiting {
    dismiss: bool,
    started: Arc<Notify>,
    dropped: Arc<AtomicBool>,
    response: Mutex<Option<oneshot::Receiver<()>>>,
}
impl DocumentDialogs for Waiting {
    fn dismisses_on_drop(&self) -> bool {
        self.dismiss
    }
    fn select_file(&self, _: FileDialogRequest) -> DialogFuture<'_, Option<PathBuf>> {
        let response = self.response.lock().unwrap().take().unwrap();
        Box::pin(async move {
            let _drop = Dropped(self.dropped.clone());
            self.started.notify_one();
            let _ = response.await;
            Ok(None)
        })
    }
    fn confirm_save(&self, _: String) -> DialogFuture<'_, SaveDecision> {
        panic!("clean note")
    }
    fn confirm_overwrite(&self, _: PathBuf, _: DiskVersion) -> DialogFuture<'_, OverwriteDecision> {
        panic!("open only")
    }
}

#[tokio::test]
async fn cancelled_dialog_releases_driver_only_when_native_dialog_is_gone() {
    for dismiss in [false, true] {
        let controller = Controller::new(Note::new(String::new()));
        let operation = controller.request_document(DocumentKind::Open).unwrap();
        controller
            .document_ready(
                operation.operation_id,
                DocumentVersion::from(&controller.snapshot().unwrap()),
            )
            .unwrap();
        let (response, receiver) = oneshot::channel();
        let started = Arc::new(Notify::new());
        let dropped = Arc::new(AtomicBool::new(false));
        let dialogs = Waiting {
            dismiss,
            started: started.clone(),
            dropped: dropped.clone(),
            response: Mutex::new(Some(receiver)),
        };
        let worker = controller.clone();
        let running =
            tokio::spawn(
                async move { worker.run_document(operation.operation_id, &dialogs).await },
            );
        tokio::time::timeout(std::time::Duration::from_secs(2), started.notified())
            .await
            .unwrap();
        controller.cancel_document(operation.operation_id).unwrap();
        if !dismiss {
            assert!(!dropped.load(Ordering::SeqCst));
            assert!(controller.request_document(DocumentKind::New).is_err());
            response.send(()).unwrap();
        }
        tokio::time::timeout(std::time::Duration::from_secs(2), running)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(dropped.load(Ordering::SeqCst));
        assert!(controller.request_document(DocumentKind::New).is_ok());
        assert!(controller.snapshot().unwrap().source.is_empty());
    }
}

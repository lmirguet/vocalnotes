//! Device boundary shared by the native controller and deterministic lifecycle tests.
//! Test implementations are injectable in Rust only; no IPC option bypasses OS monitoring.
use super::lifecycle::LifecycleGuard;
use crate::audio::{
    capture::{CaptureControl, NativeCapture},
    normalization::AudioFrame,
};
use std::{
    future::Future,
    sync::mpsc::TryRecvError,
    sync::{Arc, atomic::AtomicBool},
};

pub trait StopCapture: Send + Sync {
    fn stop(&self);
}
pub trait CaptureStream: Send {
    fn control(&self) -> Arc<dyn StopCapture>;
    fn next_frame(&self) -> Result<Option<AudioFrame>, &'static str>;
}
pub trait SessionDevices: Send + Sync + 'static {
    type Transcription: crate::ports::stt::LiveStt;
    fn transcription(
        &self,
        selection: &crate::config::preferences::ServiceSelection,
    ) -> impl Future<Output = Result<Self::Transcription, &'static str>> + Send;
    fn monitor(
        &self,
        cancelled: Arc<AtomicBool>,
        callback: Arc<dyn Fn() + Send + Sync>,
    ) -> impl Future<Output = Result<Box<dyn Send>, &'static str>> + Send;
    fn key_present(&self) -> impl Future<Output = Result<bool, &'static str>> + Send;
    fn capture(
        &self,
        cancelled: Arc<AtomicBool>,
    ) -> impl Future<Output = Result<Box<dyn CaptureStream>, &'static str>> + Send;
}

pub struct NativeDevices;
impl StopCapture for CaptureControl {
    fn stop(&self) {
        CaptureControl::stop(self);
    }
}
impl CaptureStream for NativeCapture {
    fn control(&self) -> Arc<dyn StopCapture> {
        Arc::new(NativeCapture::control(self))
    }
    fn next_frame(&self) -> Result<Option<AudioFrame>, &'static str> {
        if let Some(error) = self.failure() {
            return Err(error);
        }
        match self.frames.try_recv() {
            Ok(frame) => Ok(Some(frame)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => {
                Err("Capture interrompue : passages non intégrés ; aucune reprise automatique.")
            }
        }
    }
}
impl SessionDevices for NativeDevices {
    type Transcription = crate::adapters::openai_stt::OpenAiStt;
    async fn transcription(
        &self,
        selection: &crate::config::preferences::ServiceSelection,
    ) -> Result<Self::Transcription, &'static str> {
        Self::Transcription::connect_selected(selection).await
    }
    async fn monitor(
        &self,
        cancelled: Arc<AtomicBool>,
        callback: Arc<dyn Fn() + Send + Sync>,
    ) -> Result<Box<dyn Send>, &'static str> {
        tokio::task::spawn_blocking(move || {
            LifecycleGuard::start_guarded(cancelled, move || callback())
                .map(|guard| Box::new(guard) as Box<dyn Send>)
        })
        .await
        .map_err(|_| "Surveillance OS indisponible.")?
    }
    async fn key_present(&self) -> Result<bool, &'static str> {
        tokio::task::spawn_blocking(super::secrets::presence)
            .await
            .map_err(|_| "Trousseau indisponible : vocal refusé.")?
    }
    async fn capture(
        &self,
        cancelled: Arc<AtomicBool>,
    ) -> Result<Box<dyn CaptureStream>, &'static str> {
        tokio::task::spawn_blocking(move || {
            NativeCapture::start_guarded(cancelled)
                .map(|capture| Box::new(capture) as Box<dyn CaptureStream>)
        })
        .await
        .map_err(|_| "Microphone indisponible.")?
    }
}

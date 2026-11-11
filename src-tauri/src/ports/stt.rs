//! Provider-independent native PCM input and transcription event output.
use std::future::Future;

pub trait SttInput {
    fn append_pcm(&mut self, pcm: &[u8]) -> impl Future<Output = Result<(), &'static str>> + Send;
    fn end_passage(&mut self) -> impl Future<Output = Result<(), &'static str>> + Send;
}

pub trait SttOutput {
    fn next_event(&mut self) -> impl Future<Output = Result<SttEvent, &'static str>> + Send;
}

pub trait LiveStt: SttInput + SttOutput + Send {
    fn begin_passage(&mut self) -> Result<(), &'static str>;
}

pub enum SttEvent {
    Provisional(String),
    Final {
        text: String,
        model: ModelProvenance,
    },
    Other,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ModelProvenance {
    pub configured: Option<String>,
    pub acknowledged: Option<String>,
    pub served: Option<String>,
}

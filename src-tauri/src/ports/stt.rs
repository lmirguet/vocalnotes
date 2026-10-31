//! Provider-independent native PCM input and transcription event output.
use serde_json::Value;
use std::future::Future;

const MAX_REPLY: usize = 65_536;

pub trait SttInput {
    fn append_pcm(&mut self, pcm: &[u8]) -> impl Future<Output = Result<(), &'static str>> + Send;
    fn end_passage(&mut self) -> impl Future<Output = Result<(), &'static str>> + Send;
}

pub trait SttOutput {
    fn next_event(&mut self) -> impl Future<Output = Result<SttEvent, &'static str>> + Send;
}

pub enum SttEvent {
    Provisional(String),
    Final { text: String, model: String },
    Other,
}

#[derive(Default)]
pub struct Finality {
    finished: bool,
}

impl Finality {
    pub fn accept(&mut self, value: &Value) -> Result<SttEvent, &'static str> {
        if self.finished {
            return Err("Passage déjà finalisé ; événement rejeté.");
        }
        match value["type"].as_str() {
            Some("transcription.text.delta") => {
                let text = value["text"].as_str().ok_or("Delta STT invalide.")?;
                if text.len() > MAX_REPLY {
                    return Err("Réponse STT trop volumineuse.");
                }
                Ok(SttEvent::Provisional(text.into()))
            }
            Some("transcription.done") => {
                let text = value["text"].as_str().ok_or("Final STT invalide.")?;
                let model = value["model"].as_str().ok_or("Modèle STT servi absent.")?;
                if text.len() > MAX_REPLY || model.len() > 256 || model.trim().is_empty() {
                    return Err("Réponse STT ou modèle servi invalide.");
                }
                self.finished = true;
                Ok(SttEvent::Final {
                    text: text.into(),
                    model: model.into(),
                })
            }
            Some("error") => Err("Erreur du service STT ; passage non intégré."),
            Some(_) => Ok(SttEvent::Other),
            None => Err("Événement STT invalide."),
        }
    }
}

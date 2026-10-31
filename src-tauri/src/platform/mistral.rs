//! Compatibility exports; all production transport code lives in adapters.
pub use crate::adapters::mistral_interpreter::MistralInterpreter;
pub use crate::adapters::mistral_stt::{MistralStt, SttReader, SttWriter};
pub use crate::ports::{
    interpreter::{decode_interrupt, decode_intervention, response_format},
    stt::{Finality, SttEvent},
};

//! Drafting preferences attached to the note, including on disk.
//! These are data, never commands, transcripts, or passage history.
use serde::{Deserialize, Serialize};

pub const MAX_INTENTION_BYTES: usize = 4096;

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DraftingIntention {
    purpose: String,
    form: String,
    tone: String,
    context: String,
}

impl DraftingIntention {
    pub fn new(
        purpose: String,
        form: String,
        tone: String,
        context: String,
    ) -> Result<Self, &'static str> {
        if !Self::valid_parts(&purpose, &form, &tone, &context) {
            return Err("Intention trop volumineuse : aucun effet.");
        }
        Ok(Self {
            purpose,
            form,
            tone,
            context,
        })
    }
    pub fn valid_parts(purpose: &str, form: &str, tone: &str, context: &str) -> bool {
        purpose
            .len()
            .saturating_add(form.len())
            .saturating_add(tone.len())
            .saturating_add(context.len())
            <= MAX_INTENTION_BYTES
    }
    pub fn purpose(&self) -> &str {
        &self.purpose
    }
    pub fn form(&self) -> &str {
        &self.form
    }
    pub fn tone(&self) -> &str {
        &self.tone
    }
    pub fn context(&self) -> &str {
        &self.context
    }
    pub fn is_empty(&self) -> bool {
        self.purpose.is_empty()
            && self.form.is_empty()
            && self.tone.is_empty()
            && self.context.is_empty()
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

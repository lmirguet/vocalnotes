//! Explicit content-only reformatting orchestration.
use crate::{
    domain::note::Note,
    ports::interpreter::{ReformatInput, ReformatOutput, Reformatter},
};

#[derive(Debug)]
pub struct ReformatRequest {
    note_id: u64,
    base_revision: u64,
    input_generation: u64,
    input: ReformatInput,
    opaque_blocks: Vec<String>,
}

pub struct ReformatReply {
    note_id: u64,
    base_revision: u64,
    input_generation: u64,
    input_opaque_blocks: Vec<String>,
    output: ReformatOutput,
    pub model: String,
}

impl ReformatRequest {
    pub fn prepare(note: &Note, note_id: u64, base_revision: u64) -> Result<Self, &'static str> {
        let snapshot = note.snapshot();
        if snapshot.note_id != note_id || snapshot.revision != base_revision {
            return Err("La note a changé ; relancer explicitement la reformulation.");
        }
        let input = ReformatInput {
            content_markdown: snapshot.source,
            context: (!snapshot.context.trim().is_empty()).then_some(snapshot.context),
            instructions: (!snapshot.instructions.trim().is_empty())
                .then_some(snapshot.instructions),
        };
        input.validate()?;
        let opaque_blocks = snapshot
            .blocks
            .into_iter()
            .filter(|block| block.kind == "opaque")
            .map(|block| block.raw)
            .collect();
        Ok(Self {
            note_id: snapshot.note_id,
            base_revision: snapshot.revision,
            input_generation: snapshot.input_generation,
            input,
            opaque_blocks,
        })
    }

    pub async fn reformat(
        self,
        reformatter: &impl Reformatter,
    ) -> Result<ReformatReply, &'static str> {
        let (output, model) = reformatter.reformat(self.input).await?;
        output.validate()?;
        Ok(ReformatReply {
            note_id: self.note_id,
            base_revision: self.base_revision,
            input_generation: self.input_generation,
            input_opaque_blocks: self.opaque_blocks,
            output,
            model,
        })
    }
}

impl ReformatReply {
    pub fn commit(self, note: &mut Note) -> Result<(), &'static str> {
        note.apply_reformat(
            self.note_id,
            self.base_revision,
            self.input_generation,
            &self.input_opaque_blocks,
            self.output.content_markdown,
            &self.model,
        )
    }
}

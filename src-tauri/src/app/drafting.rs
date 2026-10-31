//! Final-passage orchestration. No transcript history, retry or semantic guess.
use crate::{
    domain::{intention::DraftingIntention, note::Note, session::Ticket},
    ports::interpreter::{InterpretationContext, Interpreter, Intervention},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureStage {
    Recognition,
    Interpretation,
    Application,
}
pub struct ApplicationFailure {
    pub stage: FailureStage,
    pub reason: &'static str,
}
impl FailureStage {
    pub fn message(self) -> &'static str {
        match self {
            Self::Recognition => "Échec de reconnaissance STT : aucun passage final utilisable.",
            Self::Interpretation => {
                "Échec d’interprétation LLM : aucun effet de cette intervention."
            }
            Self::Application => "Échec d’application : intervention refusée par le cœur.",
        }
    }
}

pub struct DraftingRequest {
    ticket: Ticket,
    intention: DraftingIntention,
    candidates: InterpretationContext,
}
pub struct DraftingReply {
    ticket: Ticket,
    response: Intervention,
    pub model: String,
}
impl DraftingRequest {
    pub fn prepare(note: &mut Note, sequence: u64, passage_id: u64) -> Result<Self, FailureStage> {
        let ticket = note
            .voice_ticket_for(sequence, passage_id)
            .map_err(|_| FailureStage::Application)?;
        let request = Self {
            ticket,
            intention: note.drafting_intention().clone(),
            candidates: InterpretationContext {
                blocks: note.candidates(),
                save_available: note.can_save(),
            },
        };
        note.processing();
        Ok(request)
    }
    pub async fn interpret(
        self,
        interpreter: &impl Interpreter,
        finalized: &str,
    ) -> Result<DraftingReply, FailureStage> {
        if finalized.trim().is_empty() {
            return Err(FailureStage::Recognition);
        }
        let (response, model) = interpreter
            .interpret_final(finalized, &self.intention, self.candidates)
            .await
            .map_err(|_| FailureStage::Interpretation)?;
        // Fake/custom adapters must obey the same whole-envelope boundary.
        response
            .validate()
            .map_err(|_| FailureStage::Interpretation)?;
        Ok(DraftingReply {
            ticket: self.ticket,
            response,
            model,
        })
    }
}
impl DraftingReply {
    pub fn commit(self, note: &mut Note) -> Result<(), ApplicationFailure> {
        note.commit_voice(&self.ticket, self.response)
            .map_err(|reason| ApplicationFailure {
                stage: FailureStage::Application,
                reason,
            })
    }
}

//! Native treatment identities and one-shot generation-scoped commit barrier.
//! Call under the canonical Note mutex; no history or deferred action replay.
use super::note::Snapshot;

#[derive(Clone)]
pub struct Ticket {
    note_id: u64,
    session_id: u64,
    generation: u64,
    revision: u64,
    sequence: u64,
    request_id: u64,
    operation_id: u64,
    passage_id: u64,
}

impl Ticket {
    pub(crate) fn current(note: &Snapshot) -> Self {
        Self {
            note_id: note.note_id,
            session_id: note.session_id,
            generation: note.generation,
            revision: note.revision,
            sequence: 0,
            request_id: 0,
            operation_id: 0,
            passage_id: 0,
        }
    }

    pub fn same_generation(&self, other: &Self) -> bool {
        self.note_id == other.note_id
            && self.session_id == other.session_id
            && self.generation == other.generation
    }

    pub(crate) fn authorized(&self, note: &Snapshot) -> bool {
        note.mode == "voice" && self.same_generation(&Self::current(note))
    }

    pub(crate) fn current_revision(&self, note: &Snapshot) -> bool {
        self.same_generation(&Self::current(note)) && self.revision == note.revision
    }
}

#[derive(Default)]
pub struct Session {
    issued: u64,
    committed: u64,
    last_passage: u64,
}

impl Session {
    pub(crate) fn issue(
        &mut self,
        note: &Snapshot,
        sequence: u64,
        passage_id: u64,
    ) -> Result<Ticket, &'static str> {
        if note.mode != "voice"
            || self.issued.checked_add(1) != Some(sequence)
            || passage_id <= self.last_passage
        {
            return Err("Séquence vocale non active ou désordonnée.");
        }
        self.issued = sequence;
        self.last_passage = passage_id;
        let mut ticket = Ticket::current(note);
        ticket.sequence = sequence;
        ticket.request_id = sequence;
        ticket.operation_id = sequence;
        ticket.passage_id = passage_id;
        Ok(ticket)
    }

    pub(crate) fn consume(&mut self, note: &Snapshot, ticket: &Ticket) -> Result<(), &'static str> {
        if !ticket.authorized(note)
            || self.committed.checked_add(1) != Some(ticket.sequence)
            || ticket.request_id != ticket.sequence
            || ticket.operation_id != ticket.sequence
            || ticket.passage_id != self.last_passage
            || ticket.sequence != self.issued
        {
            return Err("Résultat vocal invalidé, dupliqué ou désordonné.");
        }
        // Even a current-generation stale revision is consumed: visible conflict, no replay.
        self.committed = ticket.sequence;
        if !ticket.current_revision(note) {
            return Err("Résultat périmé : conflit de révision, aucun effet ni rejeu.");
        }
        Ok(())
    }
}

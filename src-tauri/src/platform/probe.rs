//! Compatibility façade: no second note, session authority or commit implementation.
pub use crate::domain::note::{
    Ack, Action, Block, Intervention, Note as Probe, Patch, Proposal, Snapshot, blocks,
};
pub use crate::domain::session::Ticket;

//! Canonical native note authority, shared by the prototype and foundations.
use super::session::{Session, Ticket};
use crate::app::document::{DocumentError, DocumentFormat, DocumentHistory, DocumentOperation};
use crate::domain::{
    change::{ChangeArea, ChangeFeedback, track_patch},
    intention::DraftingIntention,
};
pub use crate::ports::interpreter::{Action, Intervention};
use crate::ports::interpreter::{
    Anchor, Candidate, Edge, EditOperation, SessionCommand, Status, Target,
};
use crate::storage::markdown_file::{MarkdownFile, SaveError, SaveOutcome};
use crate::storage::note_document;
use pulldown_cmark::{Event, Options, Parser, Tag};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

const MAX_SOURCE: usize = 1_048_576;
const MAX_TRANSACTIONS: usize = 256;
const MAX_INSTRUCTIONS_BYTES: usize = 16_384;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DictationTarget {
    #[default]
    Content,
    Context,
    Instructions,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub from: usize,
    pub to: usize,
    pub raw: String,
    pub kind: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub note_id: u64,
    pub session_id: u64,
    pub manual_period: u64,
    pub revision: u64,
    pub generation: u64,
    pub mode: &'static str,
    pub dictation_mode: DictationTarget,
    pub context: String,
    pub instructions: String,
    pub dictation_transition_pending: bool,
    pub input_generation: u64,
    pub listening_state: &'static str,
    pub drafting_intention: Option<DraftingIntention>,
    pub change_feedback: Option<ChangeFeedback>,
    pub failure_stage: Option<&'static str>,
    pub voice_activity: &'static str,
    pub source: String,
    pub dirty: bool,
    pub document_name: String,
    pub document_path: Option<String>,
    pub document_format: Option<DocumentFormat>,
    pub history: DocumentHistory,
    pub document_operation: Option<DocumentOperation>,
    pub document_error: Option<DocumentError>,
    pub message: String,
    pub blocks: Vec<Block>,
    pub stt_requests: u64,
    pub llm_requests: u64,
    pub priority_requests: u64,
    pub audio_ms: u64,
    pub stt_model: String,
    pub stt_configured_model: String,
    pub stt_acknowledged_model: String,
    pub preferences_migrated: bool,
    pub transcript_passage_id: u64,
    pub applied_passage_id: u64,
    pub speech_started_at_epoch_us: Option<u64>,
    pub speech_ended_at_epoch_us: Option<u64>,
    pub applied_speech_started_at_epoch_us: Option<u64>,
    pub applied_speech_ended_at_epoch_us: Option<u64>,
    pub llm_model: String,
    pub last_stt_final_ms: Option<u64>,
    pub last_llm_ms: Option<u64>,
    pub last_priority_ms: Option<u64>,
    pub last_commit_us: Option<u64>,
    pub voice_applied: u64,
    pub voice_clarifications: u64,
    pub voice_rejected: u64,
    pub voice_no_change: u64,
    pub last_voice_result: &'static str,
    pub transcript_enabled: bool,
    pub transcript_provisional: String,
    pub transcript_final: String,
    pub close_pending: bool,
    pub storage_message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Patch {
    pub from: usize,
    pub to: usize,
    pub expected: String,
    pub text: String,
}

#[derive(Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Proposal {
    pub transaction_id: String,
    pub note_id: u64,
    pub manual_period: u64,
    pub base_revision: u64,
    pub patches: Vec<Patch>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ack {
    pub transaction_id: String,
    pub accepted: bool,
    pub message: &'static str,
    pub snapshot: Snapshot,
    pub applied_patches: Vec<Patch>,
}

pub struct Note {
    state: Snapshot,
    confirmed: HashMap<String, Proposal>,
    confirmed_order: std::collections::VecDeque<String>,
    intention: DraftingIntention,
    session: Session,
    destination: Option<std::path::PathBuf>,
    storage: Option<MarkdownFile>,
    storage_conflict: bool,
    pending_dictation_mode: Option<DictationTarget>,
    pending_instruction_segments: u32,
    saved_state: PersistentState,
    opened_context: String,
    history: super::history::History,
    history_transactions: std::collections::VecDeque<(
        String,
        crate::app::document::DocumentVersion,
        DictationTarget,
        bool,
    )>,
}

#[derive(Clone, Default, PartialEq, Eq)]
struct PersistentState {
    source: String,
    intention: DraftingIntention,
}

pub fn blocks(source: &str) -> Vec<Block> {
    let mut result = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    let mut kind = "opaque";
    for (event, range) in Parser::new_ext(source, Options::all()).into_offset_iter() {
        match event {
            Event::Start(ref tag) => {
                if depth == 0 {
                    start = range.start;
                    kind = match tag {
                        Tag::Paragraph => "paragraph",
                        Tag::Heading { .. } => "heading",
                        _ => "opaque",
                    };
                } else if !matches!(tag, Tag::Emphasis | Tag::Strong) {
                    kind = "opaque";
                }
                depth += 1;
            }
            Event::End(_) => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let raw = source[start..range.end].trim_end_matches(['\r', '\n']);
                    result.push(Block {
                        from: start,
                        to: start + raw.len(),
                        raw: raw.into(),
                        kind: kind.into(),
                    });
                }
            }
            Event::Html(_)
            | Event::InlineHtml(_)
            | Event::Rule
            | Event::DisplayMath(_)
            | Event::InlineMath(_)
            | Event::Code(_) => {
                if depth > 0 {
                    kind = "opaque";
                } else {
                    result.push(Block {
                        from: range.start,
                        to: range.end,
                        raw: source[range.clone()].into(),
                        kind: "opaque".into(),
                    });
                }
            }
            _ => {}
        }
    }
    // Whitespace-only and empty notes have a writable insertion point.
    if result.is_empty() && source.trim().is_empty() {
        result.push(Block {
            from: 0,
            to: 0,
            raw: "".into(),
            kind: "paragraph".into(),
        });
    } else if result.is_empty() {
        result.push(Block {
            from: 0,
            to: source.len(),
            raw: source.into(),
            kind: "opaque".into(),
        });
    }
    result
}

fn patched(source: &str, patches: &[Patch]) -> Result<String, &'static str> {
    if patches.is_empty() || patches.len() > 32 {
        return Err("Nombre de modifications invalide.");
    }
    let original_blocks = blocks(source);
    let mut sorted = patches.to_vec();
    sorted.sort_by_key(|p| p.from);
    let mut previous_end = None;
    for p in &sorted {
        if p.from > p.to
            || source.get(p.from..p.to) != Some(p.expected.as_str())
            || previous_end.is_some_and(|end| p.from <= end)
        {
            return Err("Précondition ou frontière UTF-8 invalide.");
        }
        let opaque_overlap = original_blocks.iter().any(|b| {
            b.kind == "opaque"
                && (if p.from == p.to {
                    p.from > b.from && p.from < b.to
                } else {
                    p.from < b.to && p.to > b.from
                })
        });
        // A structural edit may span supported blocks and their whitespace separators,
        // but never raw/unsupported source (including unparsed reference definitions).
        let mut cursor = p.from;
        let mut unsupported = false;
        for b in original_blocks
            .iter()
            .filter(|b| b.kind != "opaque" && b.to > p.from && b.from < p.to)
        {
            let gap_end = b.from.min(p.to).max(cursor);
            if !source[cursor..gap_end].trim().is_empty() {
                unsupported = true;
                break;
            }
            cursor = cursor.max(b.to.min(p.to));
        }
        if !source[cursor..p.to].trim().is_empty() {
            unsupported = true;
        }
        if opaque_overlap || unsupported {
            return Err("Bloc opaque ou portée non prise en charge : modification refusée.");
        }
        previous_end = Some(p.to);
    }
    let mut result = source.to_owned();
    for p in sorted.iter().rev() {
        result.replace_range(p.from..p.to, &p.text);
    }
    if result.len() > MAX_SOURCE {
        return Err("Limite de note atteinte.");
    }
    let rendered = blocks(&result);
    for opaque in original_blocks.iter().filter(|b| b.kind == "opaque") {
        let shift: isize = sorted
            .iter()
            .filter(|p| p.to <= opaque.from)
            .map(|p| p.text.len() as isize - (p.to - p.from) as isize)
            .sum();
        let from = opaque
            .from
            .checked_add_signed(shift)
            .ok_or("Déplacement de bloc invalide.")?;
        if !rendered
            .iter()
            .any(|b| b.kind == "opaque" && b.from == from && b.raw == opaque.raw)
        {
            return Err(
                "La modification altérerait l'interprétation d'un bloc opaque ; aucun effet.",
            );
        }
    }
    Ok(result)
}

fn append_dictation(current: &str, segment: &str, paragraph: bool) -> String {
    let separator =
        if current.is_empty() || current.ends_with(if paragraph { "\n\n" } else { "\n" }) {
            ""
        } else if paragraph && current.ends_with('\n') {
            "\n"
        } else if paragraph {
            "\n\n"
        } else {
            "\n"
        };
    let mut result = String::with_capacity(current.len() + separator.len() + segment.len());
    result.push_str(current);
    result.push_str(separator);
    result.push_str(segment);
    result
}

impl Note {
    pub fn new(source: String) -> Self {
        let dirty = !source.is_empty();
        Self {
            state: Snapshot {
                note_id: 1,
                session_id: 0,
                manual_period: 1,
                revision: 0,
                generation: 1,
                mode: "manual",
                dictation_mode: DictationTarget::Content,
                context: String::new(),
                instructions: String::new(),
                dictation_transition_pending: false,
                input_generation: 1,
                listening_state: "stopped",
                drafting_intention: None,
                change_feedback: None,
                failure_stage: None,
                voice_activity: "idle",
                blocks: blocks(&source),
                source,
                dirty,
                document_name: "Sans titre".into(),
                document_path: None,
                document_format: None,
                history: DocumentHistory::default(),
                document_operation: None,
                document_error: None,
                message: "Manuel — aucune capture. Recettes desktop non exécutées.".into(),
                stt_requests: 0,
                llm_requests: 0,
                priority_requests: 0,
                audio_ms: 0,
                stt_model: String::new(),
                stt_configured_model: String::new(),
                stt_acknowledged_model: String::new(),
                preferences_migrated: false,
                transcript_passage_id: 0,
                applied_passage_id: 0,
                speech_started_at_epoch_us: None,
                speech_ended_at_epoch_us: None,
                applied_speech_started_at_epoch_us: None,
                applied_speech_ended_at_epoch_us: None,
                llm_model: String::new(),
                last_stt_final_ms: None,
                last_llm_ms: None,
                last_priority_ms: None,
                last_commit_us: None,
                voice_applied: 0,
                voice_clarifications: 0,
                voice_rejected: 0,
                voice_no_change: 0,
                last_voice_result: "Aucun résultat vocal traité.",
                transcript_enabled: false,
                transcript_provisional: String::new(),
                transcript_final: String::new(),
                close_pending: false,
                storage_message: String::new(),
            },
            confirmed: HashMap::new(),
            confirmed_order: std::collections::VecDeque::new(),
            intention: DraftingIntention::default(),
            session: Session::default(),
            destination: None,
            storage: None,
            storage_conflict: false,
            pending_dictation_mode: None,
            pending_instruction_segments: 0,
            saved_state: PersistentState::default(),
            opened_context: String::new(),
            history: super::history::History::default(),
            history_transactions: std::collections::VecDeque::new(),
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        let mut snapshot = self.state.clone();
        snapshot.drafting_intention = (!self.intention.is_empty()).then(|| self.intention.clone());
        snapshot.context = self.intention.context().to_owned();
        snapshot.instructions = self.state.instructions.clone();
        snapshot.dictation_transition_pending = self.pending_dictation_mode.is_some();
        snapshot.history = self.history.availability();
        if self.history.capacity_reached() {
            snapshot.message.push_str(" Capacité d'historique atteinte : entrées les plus anciennes retirées, texte conservé.");
        }
        if let Some(path) = &self.destination {
            snapshot.document_name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Sans titre".into());
            snapshot.document_path = Some(path.to_string_lossy().into_owned());
            snapshot.document_format = DocumentFormat::from_path(path);
        }
        snapshot
    }

    fn publish(&mut self, source: String) {
        self.publish_origin(source, super::history::Origin::Manual);
    }

    fn publish_origin(&mut self, source: String, origin: super::history::Origin) {
        self.record_history(
            DictationTarget::Content,
            origin,
            self.state.source.clone(),
            &source,
        );
        self.state.change_feedback = None;
        self.state.source = source;
        self.state.blocks = blocks(&self.state.source);
        self.state.revision += 1;
        self.state.input_generation += 1;
        self.refresh_dirty();
    }

    fn record_history(
        &mut self,
        area: DictationTarget,
        origin: super::history::Origin,
        before: String,
        after: &str,
    ) {
        self.history.record(area, origin, &before, after);
    }

    pub fn history_action(
        &mut self,
        id: String,
        version: crate::app::document::DocumentVersion,
        area: DictationTarget,
        redo: bool,
    ) -> Result<(), crate::app::document::DocumentError> {
        use crate::app::document::{DocumentErrorCode, DocumentVersion};
        if let Some((_, previous, old_area, old_redo)) = self
            .history_transactions
            .iter()
            .find(|(old, ..)| old == &id)
        {
            return if (*previous, *old_area, *old_redo) == (version, area, redo) {
                Ok(())
            } else {
                Err(DocumentErrorCode::StaleRevision.error())
            };
        }
        if DocumentVersion::from(&self.snapshot()) != version {
            return Err(DocumentErrorCode::StaleRevision.error());
        }
        let current = match area {
            DictationTarget::Content => &self.state.source,
            DictationTarget::Context => self.intention.context(),
            DictationTarget::Instructions => &self.state.instructions,
        };
        let text = self
            .history
            .apply(area, redo, current)
            .ok_or_else(|| DocumentErrorCode::HistoryUnavailable.error())?;
        match area {
            DictationTarget::Content => {
                self.state.source = text;
                self.state.blocks = blocks(&self.state.source);
            }
            DictationTarget::Context => {
                self.intention = self
                    .intention
                    .with_context(text)
                    .map_err(|_| DocumentErrorCode::SizeLimit.error())?
            }
            DictationTarget::Instructions => self.state.instructions = text,
        }
        self.state.revision += 1;
        self.state.input_generation += 1;
        self.state.change_feedback = None;
        self.refresh_dirty();
        self.history_transactions
            .push_back((id, version, area, redo));
        if self.history_transactions.len() > 256 {
            self.history_transactions.pop_front();
        }
        Ok(())
    }

    fn refresh_dirty(&mut self) {
        self.state.dirty = self.state.source != self.saved_state.source
            || self.intention != self.saved_state.intention;
    }

    fn remember_saved_state(&mut self) {
        self.saved_state = PersistentState {
            source: self.state.source.clone(),
            intention: self.intention.clone(),
        };
        self.refresh_dirty();
    }

    pub fn set_dictation_mode(&mut self, target: DictationTarget) {
        if self.state.dictation_mode == target {
            self.pending_dictation_mode = None;
            self.state.dictation_transition_pending = false;
            return;
        }
        if self.state.dictation_mode == DictationTarget::Instructions
            && target != DictationTarget::Instructions
            && self.pending_instruction_segments > 0
        {
            self.pending_dictation_mode = Some(target);
            self.state.dictation_transition_pending = true;
            self.state.message =
                "Sortie d'Instructions différée : transcription en attente.".into();
            return;
        }
        self.apply_dictation_mode(target);
    }

    fn apply_dictation_mode(&mut self, target: DictationTarget) {
        if self.state.dictation_mode == DictationTarget::Instructions
            && target != DictationTarget::Instructions
        {
            self.history.clear(Some(DictationTarget::Instructions));
        }
        if self.state.dictation_mode == DictationTarget::Instructions
            && target != DictationTarget::Instructions
            && !self.state.instructions.is_empty()
        {
            self.state.instructions.clear();
            self.state.input_generation += 1;
        }
        self.pending_dictation_mode = None;
        self.state.dictation_transition_pending = false;
        self.state.dictation_mode = target;
        self.state.message = "Mode de dictée modifié.".into();
    }

    pub fn begin_dictation_segment(&mut self) -> Result<DictationTarget, &'static str> {
        let target = self.state.dictation_mode;
        if target == DictationTarget::Instructions {
            self.pending_instruction_segments = self
                .pending_instruction_segments
                .checked_add(1)
                .ok_or("Trop de transcriptions en attente.")?;
        }
        Ok(target)
    }

    pub fn route_dictation_text(
        &mut self,
        target: DictationTarget,
        text: String,
    ) -> Result<(), &'static str> {
        let text = text.trim();
        let result = if text.is_empty() {
            Ok(())
        } else {
            match target {
                DictationTarget::Content => {
                    let source = append_dictation(&self.state.source, text, true);
                    if source.len() > MAX_SOURCE {
                        Err("Limite de note atteinte ; transcription non ajoutée.")
                    } else {
                        self.publish_origin(source, super::history::Origin::Dictation);
                        self.state.message = "Transcription ajoutée au contenu.".into();
                        Ok(())
                    }
                }
                DictationTarget::Context => {
                    let context = append_dictation(self.intention.context(), text, false);
                    self.set_context_origin(context, super::history::Origin::Dictation)
                        .map(|()| {
                            self.state.message = "Transcription ajoutée au contexte.".into();
                        })
                }
                DictationTarget::Instructions => {
                    let instructions = append_dictation(&self.state.instructions, text, false);
                    if instructions.len() > MAX_INSTRUCTIONS_BYTES {
                        Err("Limite d'instructions atteinte ; transcription non ajoutée.")
                    } else {
                        self.record_history(
                            DictationTarget::Instructions,
                            super::history::Origin::Dictation,
                            self.state.instructions.clone(),
                            &instructions,
                        );
                        self.state.instructions = instructions;
                        self.state.input_generation += 1;
                        self.state.message = "Transcription ajoutée aux instructions.".into();
                        Ok(())
                    }
                }
            }
        };
        if target == DictationTarget::Instructions {
            self.pending_instruction_segments = self.pending_instruction_segments.saturating_sub(1);
            if result.is_err() {
                self.pending_dictation_mode = None;
                self.state.dictation_transition_pending = false;
            } else if self.pending_instruction_segments == 0 {
                if let Some(target) = self.pending_dictation_mode {
                    self.apply_dictation_mode(target);
                }
            }
        }
        if result.is_ok() && !text.is_empty() {
            self.state.voice_applied += 1;
            self.state.last_voice_result = "Transcription ajoutée à sa destination.".into();
        }
        result
    }

    pub fn set_context(&mut self, context: String) -> Result<(), &'static str> {
        self.set_context_origin(context, super::history::Origin::Manual)
    }

    fn set_context_origin(
        &mut self,
        context: String,
        origin: super::history::Origin,
    ) -> Result<(), &'static str> {
        let intention = self.intention.with_context(context)?;
        if intention != self.intention {
            self.record_history(
                DictationTarget::Context,
                origin,
                self.intention.context().to_owned(),
                intention.context(),
            );
            self.intention = intention;
            self.state.revision += 1;
            self.state.input_generation += 1;
            self.refresh_dirty();
        }
        Ok(())
    }

    pub fn set_instructions(&mut self, text: String) -> Result<(), &'static str> {
        if self.state.dictation_mode != DictationTarget::Instructions {
            return Err("Les instructions ne sont modifiables qu'en mode Instructions.");
        }
        if text.len() > MAX_INSTRUCTIONS_BYTES {
            return Err("Instructions trop volumineuses ; aucune modification appliquée.");
        }
        if self.state.instructions != text {
            self.record_history(
                DictationTarget::Instructions,
                super::history::Origin::Manual,
                self.state.instructions.clone(),
                &text,
            );
            self.state.instructions = text;
            self.state.input_generation += 1;
        }
        Ok(())
    }

    pub fn reformat_started(&mut self) {
        self.state.llm_requests += 1;
        self.state.message =
            "Reformulation explicitement demandée ; contenu inchangé en attente.".into();
    }

    pub fn apply_reformat(
        &mut self,
        note_id: u64,
        base_revision: u64,
        input_generation: u64,
        input_opaque_blocks: &[String],
        content_markdown: String,
        model: &str,
    ) -> Result<(), &'static str> {
        if self.state.note_id != note_id
            || self.state.revision != base_revision
            || self.state.input_generation != input_generation
        {
            return Err("Résultat de reformulation périmé ; note inchangée.");
        }
        if content_markdown.trim().is_empty() {
            return Err("Réponse LLM vide ; aucun effet.");
        }
        if content_markdown.len() > 65_536 || content_markdown.len() > MAX_SOURCE {
            return Err("Réponse LLM trop volumineuse.");
        }
        let current_opaque: Vec<_> = blocks(&self.state.source)
            .into_iter()
            .filter(|block| block.kind == "opaque")
            .map(|block| block.raw)
            .collect();
        let result_opaque: Vec<_> = blocks(&content_markdown)
            .into_iter()
            .filter(|block| block.kind == "opaque")
            .map(|block| block.raw)
            .collect();
        if &current_opaque != input_opaque_blocks || result_opaque != current_opaque {
            return Err("La réponse altère un bloc Markdown opaque ; aucun effet.");
        }
        if self.state.source != content_markdown {
            self.publish_origin(content_markdown, super::history::Origin::Reformat);
        }
        self.state.llm_model = model.into();
        self.state.message = "Contenu reformulé ; contexte et instructions inchangés.".into();
        Ok(())
    }

    pub fn manual(&mut self, proposal: Proposal) -> Ack {
        self.manual_origin(proposal, super::history::ManualOrigin::Manual)
    }

    pub fn manual_origin(
        &mut self,
        proposal: Proposal,
        origin: super::history::ManualOrigin,
    ) -> Ack {
        let id = proposal.transaction_id.clone();
        let mut accepted = false;
        let mut message = "Proposition refusée : reconstruire depuis le cœur sans rejeu.";
        if !self.state.close_pending
            && proposal.note_id == self.state.note_id
            && proposal.manual_period == self.state.manual_period
        {
            if let Some(previous) = self.confirmed.get(&id) {
                accepted = previous == &proposal;
            } else if !id.is_empty()
                && id.len() <= 128
                && proposal.base_revision == self.state.revision
            {
                match patched(&self.state.source, &proposal.patches) {
                    Ok(source) => {
                        self.publish_origin(source, origin.into());
                        if self.confirmed.len() >= MAX_TRANSACTIONS {
                            if let Some(old) = self.confirmed_order.pop_front() {
                                self.confirmed.remove(&old);
                            }
                        }
                        self.confirmed_order.push_back(id.clone());
                        self.confirmed.insert(id.clone(), proposal);
                        accepted = true;
                    }
                    Err(reason) => message = reason,
                }
            }
        }
        if accepted {
            message = "Modification acceptée par le cœur.";
        }
        self.state.message = message.into();
        Ack {
            transaction_id: id.clone(),
            accepted,
            message,
            snapshot: self.snapshot(),
            applied_patches: if accepted {
                self.confirmed
                    .get(&id)
                    .map(|p| p.patches.clone())
                    .unwrap_or_default()
            } else {
                vec![]
            },
        }
    }

    pub fn ticket(&self) -> Ticket {
        Ticket::current(&self.state)
    }

    pub fn set_transcript_enabled(&mut self, enabled: bool) -> Result<(), &'static str> {
        if enabled && self.state.mode != "voice" {
            return Err("Démarrer le vocal avant d’activer la transcription temporaire.");
        }
        self.clear_transcript();
        self.state.transcript_enabled = enabled;
        Ok(())
    }

    fn clear_transcript(&mut self) {
        self.state.transcript_enabled = false;
        self.state.transcript_provisional.clear();
        self.state.transcript_final.clear();
    }

    pub fn transcript_start_passage(&mut self) {
        self.state.transcript_provisional.clear();
    }

    pub fn transcript_delta(&mut self, text: &str) {
        if self.state.transcript_enabled
            && self.state.mode == "voice"
            && self.state.transcript_provisional.len() + text.len() <= 65_536
        {
            self.state.transcript_provisional.push_str(text);
        }
    }

    pub fn transcript_done(&mut self, text: &str) {
        if self.state.transcript_enabled && self.state.mode == "voice" && text.len() <= 65_536 {
            self.state.transcript_provisional.clear();
            if !text.trim().is_empty() {
                self.state.transcript_final = text.into();
            }
        }
    }

    pub fn start_voice(&mut self, resume: bool) -> Result<(), &'static str> {
        if self.state.close_pending
            || self.state.mode == "voice"
            || (resume && self.state.mode != "paused")
        {
            return Err("État incompatible avec le démarrage explicite.");
        }
        self.clear_transcript();
        self.state.failure_stage = None;
        self.state.stt_model.clear();
        self.state.stt_configured_model.clear();
        self.state.stt_acknowledged_model.clear();
        self.state.transcript_passage_id = 0;
        self.state.applied_passage_id = 0;
        self.state.speech_started_at_epoch_us = None;
        self.state.speech_ended_at_epoch_us = None;
        self.state.applied_speech_started_at_epoch_us = None;
        self.state.applied_speech_ended_at_epoch_us = None;
        if !resume {
            self.state.session_id += 1;
        }
        self.state.generation += 1;
        self.state.manual_period += 1;
        self.confirmed.clear();
        self.confirmed_order.clear();
        self.session = Session::default();
        self.state.mode = "voice";
        self.state.listening_state = "starting";
        self.state.voice_activity = "idle";
        self.state.message = "Écoute — OpenAI STT/LLM ; coût inconnu.".into();
        Ok(())
    }

    pub fn processing(&mut self) {
        if self.state.mode == "voice" {
            self.state.voice_activity = "processing";
        }
    }

    pub fn failure_stage(&mut self, stage: &'static str) {
        self.state.failure_stage = Some(stage);
    }

    pub fn voice_error(&mut self) {
        self.state.voice_activity = "error";
    }

    pub fn listening_ready(&mut self) {
        if self.state.mode == "voice" {
            self.state.listening_state = "listening";
        }
    }

    pub fn pause(&mut self) {
        self.clear_transcript();
        self.state.generation += 1;
        self.state.manual_period += 1;
        self.state.mode = "paused";
        self.state.listening_state = "stopped";
        self.state.voice_activity = "idle";
        self.state.message =
            "Pause — micro arrêté, passages en attente non intégrés ; reprise explicite.".into();
        self.confirmed.clear();
        self.confirmed_order.clear();
        self.session = Session::default();
    }

    pub fn intention(&self) -> &str {
        self.intention.purpose()
    }

    pub fn drafting_intention(&self) -> &DraftingIntention {
        &self.intention
    }

    /// Same IDs as the native resolver; no offsets/envelope fields sent to the LLM.
    pub fn candidates(&self) -> Vec<Candidate> {
        self.state
            .blocks
            .iter()
            .enumerate()
            .filter(|(_, block)| block.kind != "opaque")
            .map(|(index, block)| Candidate {
                block_id: format!("b{index}"),
                source: block.raw.clone(),
            })
            .collect()
    }

    pub fn same_session(&self, ticket: &Ticket) -> bool {
        ticket.authorized(&self.state)
    }

    pub fn voice_ticket(&mut self, sequence: u64) -> Result<Ticket, &'static str> {
        self.voice_ticket_for(sequence, sequence)
    }

    pub fn voice_ticket_for(
        &mut self,
        sequence: u64,
        passage_id: u64,
    ) -> Result<Ticket, &'static str> {
        self.session.issue(&self.state, sequence, passage_id)
    }

    pub fn commit_voice(
        &mut self,
        ticket: &Ticket,
        response: Intervention,
    ) -> Result<(), &'static str> {
        // Count only fresh, authorized results: duplicates and invalidated sessions
        // must not overwrite the diagnostic of the current session.
        if let Err(reason) = self.session.consume(&self.state, ticket) {
            crate::diagnostics::event(reason);
            self.state.message = reason.into();
            return Err(reason);
        }
        let clarification = response.status == Status::Clarify;
        let previous_intention = self.intention.clone();
        let revision = self.state.revision;
        let result = self.intervene(ticket, response);
        self.state.voice_activity = if result.is_err() {
            "error"
        } else if clarification {
            "clarification"
        } else {
            "idle"
        };
        self.state.last_voice_result = match result {
            Ok(()) if clarification => {
                self.state.voice_clarifications += 1;
                "Précision demandée ; aucune action appliquée."
            }
            Ok(()) if self.state.revision != revision => {
                self.state.voice_applied += 1;
                if self.intention != previous_intention {
                    if self
                        .state
                        .change_feedback
                        .as_ref()
                        .is_some_and(|change| change.revision == self.state.revision)
                    {
                        "Note et intention de rédaction mises à jour."
                    } else {
                        "Intention de rédaction mise à jour."
                    }
                } else {
                    "Modification de la note appliquée."
                }
            }
            Ok(()) => {
                self.state.voice_no_change += 1;
                "Passage traité sans modification de la note."
            }
            Err(reason) => {
                self.state.voice_rejected += 1;
                reason
            }
        };
        crate::diagnostics::numbers(
            self.state.last_voice_result,
            &[
                ("applied", self.state.voice_applied),
                ("clarifications", self.state.voice_clarifications),
                ("rejected", self.state.voice_rejected),
                ("no_change", self.state.voice_no_change),
            ],
        );
        if let Err(reason) = result {
            self.state.message = reason.into();
        }
        result
    }

    pub fn message(&mut self, message: &'static str) {
        crate::diagnostics::event(message);
        self.state.message = message.into();
    }

    pub fn measure(&mut self, kind: &str, value: u64, model: &str) {
        match kind {
            "stt" => self.state.stt_requests += value,
            "llm" => self.state.llm_requests += value,
            "priority" => self.state.priority_requests += value,
            "audio" => self.state.audio_ms += value,
            "stt_model" => self.state.stt_model = model.into(),
            "stt_configured_model" => self.state.stt_configured_model = model.into(),
            "stt_acknowledged_model" => self.state.stt_acknowledged_model = model.into(),
            "transcript_passage" => self.state.transcript_passage_id = value,
            "applied_passage" => self.state.applied_passage_id = value,
            "llm_model" => self.state.llm_model = model.into(),
            "stt_final_ms" => self.state.last_stt_final_ms = Some(value),
            "llm_ms" => self.state.last_llm_ms = Some(value),
            "priority_ms" => self.state.last_priority_ms = Some(value),
            "commit_us" => self.state.last_commit_us = Some(value),
            _ => {}
        }
    }

    pub fn speech_passage_started(&mut self, passage_id: u64, captured_at_epoch_us: Option<u64>) {
        self.transcript_start_passage();
        self.state.transcript_passage_id = passage_id;
        self.state.speech_started_at_epoch_us = captured_at_epoch_us;
        self.state.speech_ended_at_epoch_us = None;
    }

    pub fn speech_passage_ended(&mut self, passage_id: u64, captured_at_epoch_us: Option<u64>) {
        if self.state.transcript_passage_id == passage_id {
            self.state.speech_ended_at_epoch_us = captured_at_epoch_us;
        }
    }

    pub fn speech_passage_applied(
        &mut self,
        passage_id: u64,
        started_at_epoch_us: Option<u64>,
        ended_at_epoch_us: Option<u64>,
    ) {
        self.state.applied_passage_id = passage_id;
        self.state.applied_speech_started_at_epoch_us = started_at_epoch_us;
        self.state.applied_speech_ended_at_epoch_us = ended_at_epoch_us;
    }

    pub fn preferences_migrated(&mut self) {
        self.state.preferences_migrated = true;
        self.state.message = "Configuration migrée vers OpenAI ; renseignez la clé OpenAI dans le trousseau si nécessaire.".into();
    }

    pub fn is_current(&self, ticket: &Ticket) -> bool {
        ticket.current_revision(&self.state)
    }

    pub fn stop(&mut self, message: &str) {
        self.pending_instruction_segments = 0;
        self.pending_dictation_mode = None;
        self.state.dictation_transition_pending = false;
        self.clear_transcript();
        self.state.generation += 1;
        self.state.manual_period += 1;
        self.state.mode = "manual";
        self.state.listening_state = "stopped";
        self.state.voice_activity = "idle";
        self.state.message = message.into();
        self.confirmed.clear();
        self.confirmed_order.clear();
        self.session = Session::default();
    }

    pub fn invalidate_for_open(&mut self) {
        self.stop("Changement accepté : traitements précédents invalidés.");
    }

    pub fn invalidate_document_inputs(&mut self) {
        self.state.input_generation += 1;
    }

    pub fn loaded(&mut self, source: String) {
        self.confirmed.clear();
        self.confirmed_order.clear();
        self.history.clear(None);
        self.history_transactions.clear();
        self.intention.clear();
        self.pending_dictation_mode = None;
        self.pending_instruction_segments = 0;
        self.state.dictation_mode = DictationTarget::Content;
        self.state.instructions.clear();
        self.state.dictation_transition_pending = false;
        self.state.input_generation += 1;
        self.state.change_feedback = None;
        self.state.note_id += 1;
        self.state.transcript_passage_id = 0;
        self.state.applied_passage_id = 0;
        self.state.speech_started_at_epoch_us = None;
        self.state.speech_ended_at_epoch_us = None;
        self.state.applied_speech_started_at_epoch_us = None;
        self.state.applied_speech_ended_at_epoch_us = None;
        self.state.source = source;
        self.remember_saved_state();
        self.state.blocks = blocks(&self.state.source);
        self.destination = None;
        self.storage = None;
        self.storage_conflict = false;
        self.state.storage_message.clear();
        self.state.close_pending = false;
        self.opened_context.clear();
    }

    pub fn save_target(
        &self,
    ) -> Option<(
        std::path::PathBuf,
        crate::storage::markdown_file::DiskVersion,
    )> {
        let storage = self.storage.as_ref()?;
        if DocumentFormat::from_path(storage.path()) == Some(DocumentFormat::Markdown)
            && self.intention.context() != self.opened_context
        {
            return None;
        }
        Some((storage.path().to_owned(), storage.version().clone()))
    }

    pub fn document_disk(
        &self,
    ) -> Option<(
        std::path::PathBuf,
        crate::storage::markdown_file::DiskVersion,
    )> {
        self.storage
            .as_ref()
            .map(|storage| (storage.path().to_owned(), storage.version().clone()))
    }

    pub fn adopt_saved(
        &mut self,
        candidate: &crate::storage::markdown_file::WriteCandidate,
        version: crate::storage::markdown_file::DiskVersion,
    ) -> Result<(), crate::app::document::DocumentError> {
        use crate::{app::document::DocumentErrorCode, storage::markdown_file::WriteContent};
        if candidate.note_version().note_id != self.state.note_id {
            return Err(DocumentErrorCode::StaleRevision.error());
        }
        let WriteContent::Document { source, intention } = candidate.content() else {
            return Err(DocumentErrorCode::InvalidDocument.error());
        };
        self.saved_state = PersistentState {
            source: source.clone(),
            intention: intention.clone(),
        };
        self.opened_context = intention.context().to_owned();
        self.destination = Some(candidate.path().to_owned());
        self.storage = Some(MarkdownFile::from_saved(
            candidate.path().to_owned(),
            version,
        ));
        self.storage_conflict = false;
        self.state.storage_message.clear();
        self.refresh_dirty();
        Ok(())
    }

    /// The caller validates and reads the entire candidate before acquiring the core.
    pub fn adopt_opened(
        &mut self,
        storage: MarkdownFile,
        source: String,
        intention: DraftingIntention,
    ) {
        self.invalidate_for_open();
        self.loaded(source);
        self.intention = intention;
        self.opened_context = self.intention.context().to_owned();
        self.destination = Some(storage.path().to_owned());
        self.storage = Some(storage);
        self.remember_saved_state();
    }

    pub fn open(&mut self, path: &Path) -> Result<(), &'static str> {
        if self.state.dirty {
            return Err("Sauvegarder la note modifiée avant de changer de note.");
        }
        self.invalidate_for_open();
        let (storage, source) = MarkdownFile::open(path).map_err(|e| e.message)?;
        let (source, intention) = note_document::decode(&source)?;
        self.loaded(source);
        self.intention = intention;
        self.remember_saved_state();
        self.destination = Some(path.to_owned());
        self.storage = Some(storage);
        self.opened_context = self.intention.context().to_owned();
        self.state.message =
            "Note ouverte ; les modifications seront enregistrées sur demande.".into();
        Ok(())
    }

    pub fn new_note(&mut self) -> Result<(), &'static str> {
        if self.state.dirty {
            return Err("Sauvegarder la note modifiée avant de changer de note.");
        }
        self.invalidate_for_open();
        self.loaded(String::new());
        self.state.message = "Nouvelle note créée — manuel, aucune capture.".into();
        Ok(())
    }

    pub fn save_copy(&mut self, path: &Path) -> Result<(), &'static str> {
        let document = note_document::encode_checked(&self.state.source, &self.intention)?;
        let storage = match MarkdownFile::create_copy(path, &document) {
            Ok(file) => file,
            Err(error) => {
                let message = error.message;
                self.storage_error(error);
                return Err(message);
            }
        };
        self.storage = Some(storage);
        self.storage_conflict = false;
        self.state.storage_message.clear();
        self.remember_saved_state();
        self.destination = Some(path.to_owned());
        self.state.message =
            "Nouvelle copie Markdown enregistrée ; aucun fichier existant remplacé.".into();
        Ok(())
    }

    pub(crate) fn storage_error(&mut self, error: SaveError) {
        crate::diagnostics::event("Sauvegarde : erreur ; note conservée en mémoire.");
        self.state.message = error.message.into();
        for path in error.recovery_paths {
            self.state.message.push_str("\nNote à vérifier : ");
            self.state.message.push_str(&path.to_string_lossy());
        }
        self.state.storage_message = self.state.message.clone();
    }

    pub fn refresh_disk_status(&mut self) {
        if self.storage_conflict {
            return;
        }
        if let Some(storage) = &self.storage {
            match storage.changed_on_disk() {
                Ok(true) => {
                    let warning = "Modification externe détectée ; la note en mémoire est conservée. Tout enregistrement contrôlera les deux versions.";
                    if self.state.storage_message != warning {
                        crate::diagnostics::event("Sauvegarde : modification externe détectée ; note en mémoire conservée.");
                    }
                    self.state.storage_message = warning.into();
                },
                Err(_) => self.state.storage_message = "Fichier disque inaccessible ; note en mémoire conservée, choisir une destination disponible.".into(),
                Ok(false) => {}
            }
        }
    }

    pub fn save_current(&mut self) -> Result<(), &'static str> {
        if self.storage_conflict {
            return Err(
                "Conflit disque : enregistrer une nouvelle copie avant de continuer la sauvegarde.",
            );
        }
        let document = note_document::encode_checked(&self.state.source, &self.intention)?;
        let storage = self
            .storage
            .as_mut()
            .ok_or("Choisir une destination Markdown locale avant la sauvegarde ; aucun effet.")?;
        if !self.state.dirty {
            return Ok(());
        }
        match storage.save(&document) {
            Ok(SaveOutcome::Saved) => {
                self.remember_saved_state();
                self.state.storage_message.clear();
                self.state.message = "Note enregistrée après contrôle de la version disque.".into();
                Ok(())
            }
            Ok(SaveOutcome::Conflict { local, external }) => {
                crate::diagnostics::event(
                    "Sauvegarde : conflit disque ; versions préservées, nouvelle destination requise.",
                );
                self.storage_conflict = true;
                self.state.message = format!(
                    "Conflit disque : note conservée en mémoire. Enregistrer une nouvelle copie.\nVersion locale : {}\nVersion disque préservée : {}",
                    local.display(),
                    external.display()
                );
                self.state.storage_message = self.state.message.clone();
                Err(
                    "Conflit disque : les versions sont préservées ; choisir une nouvelle destination.",
                )
            }
            Ok(SaveOutcome::Copy { path }) => {
                self.state.message = format!(
                    "Remplacement indisponible ; note non enregistrée à la destination demandée. Préparation conservée : {}",
                    path.display()
                );
                self.state.storage_message = self.state.message.clone();
                Err(
                    "Remplacement indisponible ; choisir Enregistrer sous. La préparation n'est pas adoptée.",
                )
            }
            Err(error) => {
                let message = error.message;
                self.storage_error(error);
                Err(message)
            }
        }
    }

    pub fn saved_destination(&self) -> Option<std::path::PathBuf> {
        if self.state.dirty {
            None
        } else {
            self.destination.clone()
        }
    }

    pub fn can_save(&self) -> bool {
        self.destination.is_some()
    }

    pub fn save_selected_copy(&mut self) -> Result<(), &'static str> {
        let destination = self
            .destination
            .clone()
            .ok_or("Choisir une destination Markdown locale avant la sauvegarde ; aucun effet.")?;
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = destination.with_file_name(format!("vocalnotes-copie-{unique}.md"));
        self.save_copy(&path)
    }

    pub fn prepare_close(&mut self) {
        self.state.close_pending = true;
        self.state.message = "Fermeture demandée : micro arrêté. Attendre les acquittements, récupérer la saisie non acceptée si nécessaire, puis Fichier → Fermer après sauvegarde.".into();
    }

    pub fn cancel_close(&mut self) {
        self.state.close_pending = false;
        self.state.message = "Fermeture annulée ; manuel, aucune reprise du microphone.".into();
    }

    pub fn request_close(&mut self) -> bool {
        self.stop("Arrêt et invalidation avant fermeture.");
        if self.state.dirty {
            if self.storage.is_some() {
                if self.save_current().is_ok() {
                    self.clear_transient_instructions();
                    return true;
                }
                // Keep the detailed conflict/recovery paths available in the window.
                self.state
                    .message
                    .insert_str(0, "Fermeture suspendue ; micro arrêté. ");
                return false;
            }
            self.state.message =
                "Fermeture suspendue : enregistrer une nouvelle copie de la note avant de fermer."
                    .into();
            return false;
        }
        self.clear_transient_instructions();
        true
    }

    fn clear_transient_instructions(&mut self) {
        self.history.clear(Some(DictationTarget::Instructions));
        if !self.state.instructions.is_empty() {
            self.state.instructions.clear();
            self.state.input_generation += 1;
        }
        self.state.dictation_mode = DictationTarget::Content;
        self.pending_dictation_mode = None;
        self.pending_instruction_segments = 0;
        self.state.dictation_transition_pending = false;
    }

    pub fn intervene(
        &mut self,
        ticket: &Ticket,
        response: Intervention,
    ) -> Result<(), &'static str> {
        if !self.is_current(ticket) {
            return Err("Résultat périmé ou invalidé.");
        }
        response.validate()?;
        if response.status == Status::Clarify {
            if !response.actions.is_empty() || response.message.trim().is_empty() {
                return Err("Intervention ambiguë incohérente : aucun effet.");
            }
            self.state.message = response.message;
            return Ok(());
        }
        if response.status != Status::Resolved
            || !response.message.is_empty()
            || response.actions.len() > 32
        {
            return Err("Enveloppe d'intentions invalide.");
        }
        // Preflight on a working copy. Nothing is published until ALL actions validate.
        let mut source = self.state.source.clone();
        let mut targets = WorkingTargets::new(&source);
        let mut intention = self.intention.clone();
        let mut intention_reviewed = false;
        let mut change_areas = Vec::new();
        let mut terminal = None;
        for action in response.actions {
            if terminal.is_some() {
                return Err("Action après barrière terminale : aucun effet.");
            }
            match action {
                Action::Edit { operation } => {
                    targets.edit(&mut source, operation, &mut change_areas)?
                }
                Action::DraftingIntent {
                    text,
                    form,
                    tone,
                    context,
                } => {
                    intention_reviewed = true;
                    intention = DraftingIntention::new(text, form, tone, context)?;
                }
                Action::SessionAction {
                    command:
                        command @ (SessionCommand::Pause {}
                        | SessionCommand::Stop {}
                        | SessionCommand::Manual {}),
                } => terminal = Some(command),
                Action::SessionAction {
                    command: SessionCommand::Save {},
                } => {
                    return Err(
                        "Utilisez Enregistrer dans le menu natif ; intervention sans effet.",
                    );
                }
                _ => return Err("Action non disponible dans ce prototype : aucun effet."),
            }
        }
        let intention_changed = self.intention != intention;

        self.record_history(
            DictationTarget::Context,
            super::history::Origin::Dictation,
            self.intention.context().to_owned(),
            intention.context(),
        );
        self.intention = intention;
        if intention_reviewed {
            crate::diagnostics::event(if intention_changed {
                "Intention de rédaction : profil modifié."
            } else {
                "Intention de rédaction : profil inchangé."
            });
        }
        if source != self.state.source {
            self.publish(source);
            self.state.change_feedback = Some(ChangeFeedback {
                revision: self.state.revision,
                areas: change_areas,
            });
        } else if intention_changed {
            self.state.revision += 1;
            self.state.input_generation += 1;
            self.state.change_feedback = None;
            self.refresh_dirty();
        }
        match terminal {
            Some(SessionCommand::Save {}) => self.save_current()?,
            Some(SessionCommand::Pause {}) => self.pause(),
            Some(_) => self.stop("Interruption demandée ; passages en attente non intégrés, aucune reprise automatique."),
            None => {}
        }
        Ok(())
    }
}

/// Candidate identities stay attached to their native ranges while preceding
/// operations shift bytes. A structural replacement that destroys a candidate
/// does not silently retarget its ID to another paragraph.
struct WorkingTargets(Vec<(String, Block)>);

impl WorkingTargets {
    fn new(source: &str) -> Self {
        Self(
            blocks(source)
                .into_iter()
                .enumerate()
                .filter(|(_, block)| block.kind != "opaque")
                .map(|(index, block)| (format!("b{index}"), block))
                .collect(),
        )
    }

    fn block(&self, source: &str, id: &str) -> Result<&Block, &'static str> {
        let block = self
            .0
            .iter()
            .find(|(key, _)| key == id)
            .map(|(_, block)| block)
            .ok_or("Cible absente ou opaque : préciser la demande ; aucun effet.")?;
        if !blocks(source).iter().any(|current| {
            current.kind != "opaque"
                && current.from == block.from
                && current.to == block.to
                && current.raw == block.raw
        }) {
            return Err("Cible modifiée structurellement : préciser la demande ; aucun effet.");
        }
        Ok(block)
    }

    fn target(&self, source: &str, target: &Target) -> Result<(usize, usize), &'static str> {
        let block = self.block(source, &target.block_id)?;
        // Search at UTF-8 boundaries INCLUDING overlapping occurrences.
        let mut occurrences = block
            .raw
            .char_indices()
            .filter(|(index, _)| block.raw[*index..].starts_with(&target.expected));
        let (offset, _) = occurrences
            .next()
            .ok_or("Texte attendu absent : préciser la demande ; aucun effet.")?;
        if occurrences.next().is_some() {
            return Err("Cible multiple : préciser la demande ; aucun effet.");
        }
        Ok((
            block.from + offset,
            block.from + offset + target.expected.len(),
        ))
    }

    fn anchor(&self, source: &str, anchor: &Anchor) -> Result<usize, &'static str> {
        let block = self.block(source, &anchor.block_id)?;
        if block.raw != anchor.expected {
            return Err("Ancre périmée : préciser la demande ; aucun effet.");
        }
        Ok(match anchor.edge {
            Edge::Before => block.from,
            Edge::After => block.to,
        })
    }

    fn edit(
        &mut self,
        source: &mut String,
        operation: EditOperation,
        areas: &mut Vec<ChangeArea>,
    ) -> Result<(), &'static str> {
        let kind = match &operation {
            EditOperation::Insert { .. } => "insert",
            EditOperation::Replace { .. } => "replace",
            EditOperation::Delete { .. } => "delete",
            EditOperation::Move { .. } => "move",
        };
        let patches = match operation {
            EditOperation::Replace { target, text } => {
                let (from, to) = self.target(source, &target)?;
                vec![Patch {
                    from,
                    to,
                    expected: target.expected,
                    text,
                }]
            }
            EditOperation::Delete { target } => {
                let (from, to) = self.target(source, &target)?;
                vec![Patch {
                    from,
                    to,
                    expected: target.expected,
                    text: String::new(),
                }]
            }
            EditOperation::Insert { anchor, text } => {
                let from = self.anchor(source, &anchor)?;
                vec![Patch {
                    from,
                    to: from,
                    expected: String::new(),
                    text,
                }]
            }
            EditOperation::Move { target, anchor } => {
                let (from, to) = self.target(source, &target)?;
                let destination = self.anchor(source, &anchor)?;
                if destination >= from && destination <= to {
                    return Err("Déplacement sur sa propre cible : aucun effet.");
                }
                vec![
                    Patch {
                        from,
                        to,
                        expected: target.expected.clone(),
                        text: String::new(),
                    },
                    Patch {
                        from: destination,
                        to: destination,
                        expected: String::new(),
                        text: target.expected,
                    },
                ]
            }
        };
        let next = patched(source, &patches)?;
        // Apply range bookkeeping in reverse source order, like patched().
        let mut ordered = patches;
        ordered.sort_by_key(|patch| patch.from);
        for patch in ordered.iter().rev() {
            track_patch(areas, patch, kind);
            let shift = patch.text.len() as isize - (patch.to - patch.from) as isize;
            for (_, block) in &mut self.0 {
                if patch.to <= block.from && !(block.from == block.to && patch.from == block.from) {
                    block.from = block
                        .from
                        .checked_add_signed(shift)
                        .ok_or("Cible invalide.")?;
                    block.to = block
                        .to
                        .checked_add_signed(shift)
                        .ok_or("Cible invalide.")?;
                } else if patch.from >= block.from && patch.to <= block.to {
                    block.to = block
                        .to
                        .checked_add_signed(shift)
                        .ok_or("Cible invalide.")?;
                }
            }
        }
        for (_, block) in &mut self.0 {
            block.raw = next.get(block.from..block.to).unwrap_or("").to_owned();
        }
        *source = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaving_instructions_waits_for_the_original_passage_before_clearing_text() {
        let mut note = Note::new("Note".into());
        note.set_dictation_mode(DictationTarget::Instructions);
        note.set_instructions("instruction manuelle".into())
            .unwrap();
        let original_target = note.begin_dictation_segment().unwrap();

        note.set_dictation_mode(DictationTarget::Context);
        let pending = note.snapshot();
        assert_eq!(pending.dictation_mode, DictationTarget::Instructions);
        assert!(pending.dictation_transition_pending);
        assert_eq!(pending.instructions, "instruction manuelle");

        note.route_dictation_text(original_target, "instruction dictée".into())
            .unwrap();
        let completed = note.snapshot();
        assert_eq!(completed.dictation_mode, DictationTarget::Context);
        assert!(!completed.dictation_transition_pending);
        assert!(completed.instructions.is_empty());
        assert_eq!(completed.voice_applied, 1);
    }

    #[test]
    fn manual_content_edits_are_allowed_during_capture_but_stale_edits_are_rejected() {
        let mut note = Note::new("Note".into());
        note.start_voice(false).unwrap();
        let snapshot = note.snapshot();
        let valid = Proposal {
            transaction_id: "manual-during-capture".into(),
            note_id: snapshot.note_id,
            manual_period: snapshot.manual_period,
            base_revision: snapshot.revision,
            patches: vec![Patch {
                from: 0,
                to: 4,
                expected: "Note".into(),
                text: "Note éditée".into(),
            }],
        };

        assert!(note.manual(valid).accepted);
        assert_eq!(note.state.mode, "voice");
        assert_eq!(note.state.source, "Note éditée");

        let stale = Proposal {
            transaction_id: "stale-during-capture".into(),
            note_id: note.state.note_id,
            manual_period: note.state.manual_period,
            base_revision: note.state.revision - 1,
            patches: vec![Patch {
                from: 0,
                to: note.state.source.len(),
                expected: note.state.source.clone(),
                text: "Écrasement périmé".into(),
            }],
        };
        assert!(!note.manual(stale).accepted);
        assert_eq!(note.state.source, "Note éditée");
    }
}

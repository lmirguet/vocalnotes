//! Canonical native note authority, shared by the prototype and foundations.
use super::session::{Session, Ticket};
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
    pub listening_state: &'static str,
    pub drafting_intention: Option<DraftingIntention>,
    pub change_feedback: Option<ChangeFeedback>,
    pub failure_stage: Option<&'static str>,
    pub voice_activity: &'static str,
    pub source: String,
    pub dirty: bool,
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
    intention: DraftingIntention,
    session: Session,
    destination: Option<std::path::PathBuf>,
    storage: Option<MarkdownFile>,
    storage_conflict: bool,
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

impl Note {
    pub fn new(source: String) -> Self {
        Self {
            state: Snapshot {
                note_id: 1,
                session_id: 0,
                manual_period: 1,
                revision: 0,
                generation: 1,
                mode: "manual",
                listening_state: "stopped",
                drafting_intention: None,
                change_feedback: None,
                failure_stage: None,
                voice_activity: "idle",
                blocks: blocks(&source),
                source,
                dirty: false,
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
            intention: DraftingIntention::default(),
            session: Session::default(),
            destination: None,
            storage: None,
            storage_conflict: false,
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        let mut snapshot = self.state.clone();
        snapshot.drafting_intention = (!self.intention.is_empty()).then(|| self.intention.clone());
        snapshot
    }

    fn publish(&mut self, source: String) {
        self.state.change_feedback = None;
        self.state.source = source;
        self.state.blocks = blocks(&self.state.source);
        self.state.revision += 1;
        self.state.dirty = true;
        if self.storage.is_some() {
            let _ = self.save_current();
        }
    }

    pub fn manual(&mut self, proposal: Proposal) -> Ack {
        let id = proposal.transaction_id.clone();
        let mut accepted = false;
        let mut message = "Proposition refusée : reconstruire depuis le cœur sans rejeu.";
        if self.state.mode == "manual"
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
                        if self.confirmed.len() < MAX_TRANSACTIONS {
                            self.publish(source);
                            self.confirmed.insert(id.clone(), proposal);
                            accepted = true;
                        } else {
                            // Never evict deduplication identities inside a manual period.
                            message = "Limite du prototype : rouvrir après sauvegarde pour une nouvelle période.";
                        }
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
        if !resume {
            self.state.session_id += 1;
        }
        self.state.generation += 1;
        self.state.manual_period += 1;
        self.confirmed.clear();
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

    pub fn preferences_migrated(&mut self) {
        self.state.preferences_migrated = true;
        self.state.message = "Configuration migrée vers OpenAI ; renseignez la clé OpenAI dans le trousseau si nécessaire.".into();
    }

    pub fn is_current(&self, ticket: &Ticket) -> bool {
        ticket.current_revision(&self.state)
    }

    pub fn stop(&mut self, message: &str) {
        self.clear_transcript();
        self.state.generation += 1;
        self.state.manual_period += 1;
        self.state.mode = "manual";
        self.state.listening_state = "stopped";
        self.state.voice_activity = "idle";
        self.state.message = message.into();
        self.confirmed.clear();
        self.session = Session::default();
    }

    pub fn invalidate_for_open(&mut self) {
        self.stop("Changement accepté : traitements précédents invalidés.");
    }

    pub fn loaded(&mut self, source: String) {
        self.intention.clear();
        self.state.change_feedback = None;
        self.state.note_id += 1;
        self.state.source = source;
        self.state.revision = 0;
        self.state.dirty = false;
        self.state.blocks = blocks(&self.state.source);
        self.destination = None;
        self.storage = None;
        self.storage_conflict = false;
        self.state.storage_message.clear();
        self.state.close_pending = false;
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
        self.destination = Some(path.to_owned());
        self.storage = Some(storage);
        self.state.message =
            "Note ouverte ; autosauvegarde contrôlée des modifications acceptées.".into();
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
        let storage = match MarkdownFile::create_copy(
            path,
            &note_document::encode(&self.state.source, &self.intention),
        ) {
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
        self.state.dirty = false;
        self.destination = Some(path.to_owned());
        self.state.message =
            "Nouvelle copie Markdown enregistrée ; aucun fichier existant remplacé.".into();
        Ok(())
    }

    fn storage_error(&mut self, error: SaveError) {
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
        let storage = self
            .storage
            .as_mut()
            .ok_or("Choisir une destination Markdown locale avant la sauvegarde ; aucun effet.")?;
        if !self.state.dirty {
            return Ok(());
        }
        match storage.save(&note_document::encode(&self.state.source, &self.intention)) {
            Ok(SaveOutcome::Saved) => {
                self.state.dirty = false;
                self.state.storage_message.clear();
                self.state.message =
                    "Note enregistrée automatiquement après contrôle de la version disque.".into();
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
                // A fallback is an explicit destination change, never a silent overwrite.
                let (storage, _) = MarkdownFile::open(&path).map_err(|e| e.message)?;
                self.destination = Some(path.clone());
                self.storage = Some(storage);
                self.state.dirty = false;
                self.state.message = format!(
                    "Remplacement indisponible ; note sauvegardée dans une nouvelle copie : {}",
                    path.display()
                );
                self.state.storage_message = self.state.message.clone();
                Ok(())
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
        true
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
                    if self.destination.is_none() {
                        return Err(
                            "Destination de sauvegarde absente : intervention entière sans effet, choisir une destination.",
                        );
                    }
                    // A prototype save is last: validate every edit first, then save accepted
                    // source exclusively. No partially interpreted tail is replayed.
                    terminal = Some(SessionCommand::Save {});
                }
                _ => return Err("Action non disponible dans ce prototype : aucun effet."),
            }
        }
        let intention_changed = self.intention != intention;

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
            self.state.dirty = true;
            self.state.revision += 1;
            self.state.change_feedback = None;
            if self.storage.is_some() {
                let _ = self.save_current();
            }
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

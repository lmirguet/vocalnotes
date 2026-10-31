//! Canonical native note authority, shared by the prototype and foundations.
use super::session::{Session, Ticket};
pub use crate::ports::interpreter::{Action, Intervention};
use crate::ports::interpreter::{
    Anchor, Candidate, Edge, EditOperation, SessionCommand, Status, Target,
};
use pulldown_cmark::{Event, Options, Parser, Tag};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Write, path::Path};

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
    pub source: String,
    pub dirty: bool,
    pub message: String,
    pub blocks: Vec<Block>,
    pub stt_requests: u64,
    pub llm_requests: u64,
    pub priority_requests: u64,
    pub audio_ms: u64,
    pub stt_model: String,
    pub llm_model: String,
    pub last_stt_final_ms: Option<u64>,
    pub last_llm_ms: Option<u64>,
    pub last_priority_ms: Option<u64>,
    pub last_commit_us: Option<u64>,
    pub close_pending: bool,
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
    intention: String,
    session: Session,
    destination: Option<std::path::PathBuf>,
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
                blocks: blocks(&source),
                source,
                dirty: false,
                message: "Manuel — aucune capture. Recettes desktop non exécutées.".into(),
                stt_requests: 0,
                llm_requests: 0,
                priority_requests: 0,
                audio_ms: 0,
                stt_model: String::new(),
                llm_model: String::new(),
                last_stt_final_ms: None,
                last_llm_ms: None,
                last_priority_ms: None,
                last_commit_us: None,
                close_pending: false,
            },
            confirmed: HashMap::new(),
            intention: String::new(),
            session: Session::default(),
            destination: None,
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.state.clone()
    }

    fn publish(&mut self, source: String) {
        self.state.source = source;
        self.state.blocks = blocks(&self.state.source);
        self.state.revision += 1;
        self.state.dirty = true;
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

    pub fn start_voice(&mut self, resume: bool) -> Result<(), &'static str> {
        if self.state.close_pending
            || self.state.mode == "voice"
            || (resume && self.state.mode != "paused")
        {
            return Err("État incompatible avec le démarrage explicite.");
        }
        if !resume {
            self.intention.clear();
            self.state.session_id += 1;
        }
        self.state.generation += 1;
        self.state.manual_period += 1;
        self.confirmed.clear();
        self.session = Session::default();
        self.state.mode = "voice";
        self.state.message = "Écoute — Mistral STT/LLM ; coût inconnu.".into();
        Ok(())
    }

    pub fn pause(&mut self) {
        self.state.generation += 1;
        self.state.manual_period += 1;
        self.state.mode = "paused";
        self.state.message =
            "Pause — micro arrêté, passages en attente non intégrés ; reprise explicite.".into();
        self.confirmed.clear();
        self.session = Session::default();
    }

    pub fn intention(&self) -> &str {
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
        let result = self
            .session
            .consume(&self.state, ticket)
            .and_then(|()| self.intervene(ticket, response));
        if let Err(reason) = result {
            self.state.message = reason.into();
        }
        result
    }

    pub fn message(&mut self, message: impl Into<String>) {
        self.state.message = message.into();
    }

    pub fn measure(&mut self, kind: &str, value: u64, model: &str) {
        match kind {
            "stt" => self.state.stt_requests += value,
            "llm" => self.state.llm_requests += value,
            "priority" => self.state.priority_requests += value,
            "audio" => self.state.audio_ms += value,
            "stt_model" => self.state.stt_model = model.into(),
            "llm_model" => self.state.llm_model = model.into(),
            "stt_final_ms" => self.state.last_stt_final_ms = Some(value),
            "llm_ms" => self.state.last_llm_ms = Some(value),
            "priority_ms" => self.state.last_priority_ms = Some(value),
            "commit_us" => self.state.last_commit_us = Some(value),
            _ => {}
        }
    }

    pub fn is_current(&self, ticket: &Ticket) -> bool {
        ticket.current_revision(&self.state)
    }

    pub fn stop(&mut self, message: &str) {
        self.state.generation += 1;
        self.state.manual_period += 1;
        self.state.mode = "manual";
        self.state.message = message.into();
        self.confirmed.clear();
        self.intention.clear();
        self.session = Session::default();
    }

    pub fn invalidate_for_open(&mut self) {
        self.stop("Changement accepté : traitements précédents invalidés.");
    }

    pub fn loaded(&mut self, source: String) {
        self.state.note_id += 1;
        self.state.source = source;
        self.state.revision = 0;
        self.state.dirty = false;
        self.state.blocks = blocks(&self.state.source);
        self.destination = None;
        self.state.close_pending = false;
    }

    pub fn open(&mut self, path: &Path) -> Result<(), &'static str> {
        if self.state.dirty {
            return Err("Sauvegarder la note modifiée avant de changer de note.");
        }
        self.invalidate_for_open();
        validate_path(path)?;
        let metadata = std::fs::symlink_metadata(path).map_err(|_| "Note inaccessible.")?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() > MAX_SOURCE as u64
        {
            return Err("Note ordinaire UTF-8 de moins de 1 Mio requise.");
        }
        let source = std::fs::read_to_string(path).map_err(|_| "Lecture UTF-8 impossible.")?;
        if source.len() > MAX_SOURCE {
            return Err("La note a grandi pendant l'ouverture ; lecture refusée.");
        }
        self.loaded(source);
        self.destination = Some(path.to_owned());
        self.state.message =
            "Note ouverte. Sauvegarde uniquement en nouvelle copie exclusive dans ce prototype."
                .into();
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
        validate_path(path)?;
        // Safe prototype fallback: no replacement of existing files on either OS.
        // No cache, transcript, credentials or context are ever written here.
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(
                |_| "Création exclusive impossible : choisir un nouveau fichier Markdown local.",
            )?;
        file.write_all(self.state.source.as_bytes())
            .and_then(|_| file.sync_all())
            .map_err(|_| "Écriture incomplète : note conservée en mémoire, vérifier la copie.")?;
        self.state.dirty = false;
        self.destination = Some(path.to_owned());
        self.state.message =
            "Nouvelle copie Markdown enregistrée ; aucun fichier existant remplacé.".into();
        Ok(())
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
            // Until conditional atomic replacement is available, a new exclusive sibling
            // is safer than overwriting the opened/saved note. Failure keeps it accessible.
            if let Some(destination) = self.destination.clone() {
                let unique = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos();
                let path = destination.with_file_name(format!("vocalnotes-fermeture-{unique}.md"));
                match self.save_copy(&path) {
                    Ok(()) => return true,
                    Err(_) => {
                        self.state.message = "Fermeture suspendue : sauvegarde exclusive échouée ; note conservée, choisir une autre destination.".into();
                        return false;
                    }
                }
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
        let mut terminal = None;
        for action in response.actions {
            if terminal.is_some() {
                return Err("Action après barrière terminale : aucun effet.");
            }
            match action {
                Action::Edit { operation } => targets.edit(&mut source, operation)?,
                Action::DraftingIntent { text } if text.len() <= 4096 => intention = text,
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
        if source != self.state.source {
            self.publish(source);
        }
        self.intention = intention;
        match terminal {
            Some(SessionCommand::Save {}) => self.save_selected_copy()?,
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

    fn edit(&mut self, source: &mut String, operation: EditOperation) -> Result<(), &'static str> {
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

fn validate_path(path: &Path) -> Result<(), &'static str> {
    if !path.is_absolute() || path.extension().and_then(|s| s.to_str()) != Some("md") {
        return Err("Chemin absolu vers un fichier .md requis.");
    }
    Ok(())
}

//! Strict, bounded ordered interventions; no arbitrary tool or voice undo.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::future::Future;

const MAX_REPLY: usize = 65_536;

pub trait Interpreter {
    fn interpret_final(
        &self,
        passage: &str,
        intention: &str,
        candidates: InterpretationContext,
    ) -> impl Future<Output = Result<(Intervention, String), &'static str>> + Send;
    fn interpret_interruption(
        &self,
        provisional: &str,
    ) -> impl Future<Output = Result<Interruption, &'static str>> + Send;
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Resolved,
    Clarify,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Intervention {
    pub status: Status,
    pub message: String,
    pub actions: Vec<Action>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Edit { operation: EditOperation },
    DraftingIntent { text: String },
    SessionAction { command: SessionCommand },
}

/// IDs belong to the current native candidate set, never to a provider offset.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub block_id: String,
    pub expected: String,
}

#[derive(Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Edge {
    Before,
    After,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Anchor {
    pub block_id: String,
    pub expected: String,
    pub edge: Edge,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EditOperation {
    Insert { anchor: Anchor, text: String },
    Replace { target: Target, text: String },
    Delete { target: Target },
    Move { target: Target, anchor: Anchor },
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum SessionCommand {
    Pause {},
    Stop {},
    Manual {},
    Save {},
    New {},
    Open { path: String },
    SaveAs { path: String },
    Preferences {},
    Close {},
}

#[derive(Serialize)]
pub struct Candidate {
    pub block_id: String,
    pub source: String,
}

#[derive(Serialize)]
pub struct InterpretationContext {
    pub blocks: Vec<Candidate>,
    pub save_available: bool,
}

impl Target {
    fn validate(&self) -> bool {
        !self.block_id.is_empty()
            && self.block_id.len() <= 128
            && !self.expected.is_empty()
            && self.expected.len() <= MAX_REPLY
    }
}

impl Anchor {
    fn validate(&self) -> bool {
        !self.block_id.is_empty() && self.block_id.len() <= 128 && self.expected.len() <= MAX_REPLY
    }
}

impl EditOperation {
    fn validate(&self) -> bool {
        match self {
            Self::Insert { anchor, text } => anchor.validate() && text.len() <= MAX_REPLY,
            Self::Replace { target, text } => target.validate() && text.len() <= MAX_REPLY,
            Self::Delete { target } => target.validate(),
            Self::Move { target, anchor } => target.validate() && anchor.validate(),
        }
    }
}

impl Intervention {
    /// Validate the WHOLE envelope, including terminal order, before any native effect.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.actions.len() > 32
            || self.message.len() > 4096
            || (self.status == Status::Clarify
                && (!self.actions.is_empty() || self.message.trim().is_empty()))
            || (self.status == Status::Resolved && !self.message.is_empty())
        {
            crate::diagnostics::numbers(
                "LLM : enveloppe refusée (métadonnées uniquement)",
                &[
                    ("actions", self.actions.len() as u64),
                    ("message_bytes", self.message.len() as u64),
                    ("status_clarify", u64::from(self.status == Status::Clarify)),
                    ("message_blank", u64::from(self.message.trim().is_empty())),
                ],
            );
            return Err("Réponse du service incohérente avec le contrat : aucun effet.");
        }
        for (index, action) in self.actions.iter().enumerate() {
            let invalid = match action {
                Action::SessionAction { command } => {
                    index + 1 != self.actions.len()
                        || match command {
                            SessionCommand::Open { path } | SessionCommand::SaveAs { path } => {
                                path.len() > 4096
                                    || path.chars().any(char::is_control)
                                    || !absolute_markdown_path(path)
                            }
                            _ => false,
                        }
                }
                Action::DraftingIntent { text } => text.len() > 4096,
                Action::Edit { operation } => !operation.validate(),
            };
            if invalid {
                return Err("Intervention incohérente ou ordre terminal ambigu : aucun effet.");
            }
        }
        Ok(())
    }
}

/// Validate the syntax of either target OS, without touching its filesystem.
/// Native path/identity/permission checks remain mandatory at the execution port.
fn absolute_markdown_path(path: &str) -> bool {
    let windows_drive = path.as_bytes().get(1..3) == Some(b":\\")
        && path.as_bytes().first().is_some_and(u8::is_ascii_alphabetic);
    (path.starts_with('/') || windows_drive)
        && path.ends_with(".md")
        && !path
            .split(['/', '\\'])
            .any(|part| part == ".." || part == ".")
        && !path.starts_with("//")
}

pub(crate) fn object(properties: Value, required: Value) -> Value {
    json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
}

pub fn response_format() -> Value {
    let string = |max| json!({"type":"string","maxLength":max});
    let target = object(
        json!({
            "block_id":{"type":"string","minLength":1,"maxLength":128},
            "expected":{"type":"string","minLength":1,"maxLength":MAX_REPLY}
        }),
        json!(["block_id", "expected"]),
    );
    let anchor = object(
        json!({
            "block_id":{"type":"string","minLength":1,"maxLength":128},
            "expected":string(MAX_REPLY),
            "edge":{"type":"string","enum":["before","after"]}
        }),
        json!(["block_id", "expected", "edge"]),
    );
    let insert = object(
        json!({"kind":{"type":"string","enum":["insert"]},"anchor":anchor,"text":string(MAX_REPLY)}),
        json!(["kind", "anchor", "text"]),
    );
    let replace = object(
        json!({"kind":{"type":"string","enum":["replace"]},"target":target,"text":string(MAX_REPLY)}),
        json!(["kind", "target", "text"]),
    );
    let delete = object(
        json!({"kind":{"type":"string","enum":["delete"]},"target":target}),
        json!(["kind", "target"]),
    );
    let movement = object(
        json!({"kind":{"type":"string","enum":["move"]},"target":target,"anchor":anchor}),
        json!(["kind", "target", "anchor"]),
    );
    let edit = object(
        json!({"type":{"type":"string","enum":["edit"]},
            "operation":{"anyOf":[insert,replace,delete,movement]}}),
        json!(["type", "operation"]),
    );
    let intent = object(
        json!({"type":{"type":"string","enum":["drafting_intent"]},"text":string(4096)}),
        json!(["type", "text"]),
    );
    let simple_command = object(
        json!({"action":{"type":"string","enum":["pause","stop","manual","save","new","preferences","close"]}}),
        json!(["action"]),
    );
    let path_command = object(
        json!({"action":{"type":"string","enum":["open","save_as"]},"path":string(4096)}),
        json!(["action", "path"]),
    );
    let session = object(
        json!({"type":{"type":"string","enum":["session_action"]},
            "command":{"anyOf":[simple_command,path_command]}}),
        json!(["type", "command"]),
    );
    let mut format = json!({"type":"json_schema","json_schema":{"name":"intervention","strict":true,"schema":
        object(json!({
            "status":{"type":"string","enum":["resolved","clarify"]},
            "message":string(4096),
            "actions":{"type":"array","maxItems":32,"items":{"anyOf":[edit,intent,session]}}
        }), json!(["status","message","actions"]))
    }});
    constrain_envelope(&mut format);
    format
}

/// Match the provider-visible envelope to the native status/message/actions guard.
/// Keep root object properties for providers expecting an object at the root.
fn constrain_envelope(format: &mut Value) {
    let schema = &mut format["json_schema"]["schema"];
    let mut base = schema.clone();
    base.as_object_mut().unwrap().remove("anyOf");
    let mut resolved = base.clone();
    resolved["properties"]["status"]["enum"] = json!(["resolved"]);
    resolved["properties"]["message"]["enum"] = json!([""]);
    let mut clarify = base;
    clarify["properties"]["status"]["enum"] = json!(["clarify"]);
    clarify["properties"]["message"]["minLength"] = json!(1);
    clarify["properties"]["message"]["pattern"] = json!(r"\S");
    clarify["properties"]["actions"]["maxItems"] = json!(0);
    schema["anyOf"] = json!([resolved, clarify]);
}

pub fn response_format_for_context(context: &InterpretationContext) -> Value {
    let mut format = response_format();
    // An empty native anchor permits insertion, not targeting text from the dictation
    // as if it were already in the note. Keep all native validation at commit.
    if context.blocks.len() == 1 && context.blocks[0].source.is_empty() {
        let operation = &mut format["json_schema"]["schema"]["properties"]["actions"]["items"]["anyOf"]
            [0]["properties"]["operation"];
        let mut insert = operation["anyOf"][0].clone();
        let anchor = &mut insert["properties"]["anchor"]["properties"];
        anchor["block_id"]["enum"] = json!([context.blocks[0].block_id]);
        anchor["expected"]["enum"] = json!([""]);
        operation["anyOf"] = json!([insert]);
        // Rebuild alternatives after specializing the native empty-note anchor.
        constrain_envelope(&mut format);
    }
    format
}

pub fn decode_intervention(text: &str) -> Result<Intervention, &'static str> {
    if text.len() > MAX_REPLY {
        return Err("Réponse LLM trop volumineuse.");
    }
    let result: Intervention = serde_json::from_str(text).map_err(|error| {
        // Never print serde's error text: it can quote provider/user content.
        crate::diagnostics::numbers(
            "LLM : réponse hors contrat (métadonnées uniquement)",
            &[
                ("response_bytes", text.len() as u64),
                ("json_syntax", u64::from(error.is_syntax())),
                ("json_data", u64::from(error.is_data())),
                ("json_eof", u64::from(error.is_eof())),
                ("line", error.line() as u64),
                ("column", error.column() as u64),
            ],
        );
        "Réponse LLM hors contrat strict."
    })?;
    result.validate()?;
    Ok(result)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Interrupt {
    interrupt: Interruption,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Interruption {
    None,
    Pause,
    Stop,
    Manual,
}

pub fn decode_interrupt(text: &str) -> Result<Interruption, &'static str> {
    if text.len() > MAX_REPLY {
        return Err("Réponse prioritaire trop volumineuse.");
    }
    let value: Interrupt = serde_json::from_str(text).map_err(|_| "Interruption hors contrat.")?;
    Ok(value.interrupt)
}

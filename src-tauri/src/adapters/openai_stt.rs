use crate::{
    config::preferences::ServiceSelection,
    platform::{secrets, stt_transport},
    ports::stt::{LiveStt, ModelProvenance, SttEvent, SttInput, SttOutput},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{future::Future, time::Duration};
use tokio::{
    net::TcpStream,
    time::{Instant, timeout, timeout_at},
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};

const TIMEOUT: Duration = Duration::from_secs(45);
const MAX_REPLY: usize = 65_536;
const MAX_AUDIO: usize = 24_000 * 2 * 60;

pub trait JsonSender: Send {
    fn send_json(&mut self, value: Value) -> impl Future<Output = Result<(), &'static str>> + Send;
}
pub trait JsonReceiver: Send {
    fn receive_text(&mut self) -> impl Future<Output = Result<String, &'static str>> + Send;
}
pub struct NativeSocket(WebSocketStream<MaybeTlsStream<TcpStream>>);
impl JsonSender for NativeSocket {
    async fn send_json(&mut self, value: Value) -> Result<(), &'static str> {
        timeout(
            TIMEOUT,
            self.0.send(Message::Text(value.to_string().into())),
        )
        .await
        .map_err(|_| "STT [timeout] : envoi expiré.")?
        .map_err(|e| stt_transport::classify(&e).message())
    }
}
impl JsonReceiver for NativeSocket {
    async fn receive_text(&mut self) -> Result<String, &'static str> {
        loop {
            match self.0.next().await {
                Some(Ok(Message::Text(t))) => return Ok(t.to_string()),
                Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {}
                Some(Err(e)) => return Err(stt_transport::classify(&e).message()),
                _ => return Err("STT [closed] : connexion interrompue."),
            }
        }
    }
}
async fn read<T: JsonReceiver>(socket: &mut T) -> Result<Value, &'static str> {
    let text = socket.receive_text().await?;
    if text.len() > MAX_REPLY {
        return Err("STT [capacity] : réponse trop volumineuse.");
    }
    serde_json::from_str(&text).map_err(|_| "STT [protocol] : JSON invalide.")
}
fn item(value: &Value) -> Result<String, &'static str> {
    let id = value["item_id"]
        .as_str()
        .ok_or("STT [protocol] : identité absente.")?;
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
    {
        return Err("STT [protocol] : identité invalide.");
    }
    Ok(id.into())
}
fn identity(value: &Value) -> Result<(String, u32), &'static str> {
    let index = value["content_index"]
        .as_u64()
        .and_then(|i| u32::try_from(i).ok())
        .ok_or("STT [protocol] : index invalide.")?;
    Ok((item(value)?, index))
}
fn configuration(model: &str) -> Value {
    json!({"type":"transcription","audio":{"input":{
        "format":{"type":"audio/pcm","rate":24000},
        "transcription":{"model":model,"delay":"minimal"},
        "turn_detection":null
    }}})
}

pub struct SttSession<T> {
    socket: T,
    model: String,
    active: bool,
    failed: bool,
    sent: usize,
    text_bytes: usize,
    current: Option<(String, u32)>,
    committed: Option<String>,
    last_final: Option<(String, u32)>,
    deadline: Option<Instant>,
    duplicates: u64,
}
pub type OpenAiStt = SttSession<NativeSocket>;

impl<T: JsonSender + JsonReceiver> SttSession<T> {
    pub async fn open(mut socket: T, model: &str) -> Result<Self, &'static str> {
        ServiceSelection {
            provider: "openai".into(),
            model: model.into(),
        }
        .validate()?;
        timeout(TIMEOUT, async {
            if read(&mut socket).await?["type"] != "session.created" {
                return Err("STT [configuration] : session non créée.");
            }
            socket
                .send_json(json!({"type":"session.update","session":configuration(model)}))
                .await?;
            loop {
                let ack = read(&mut socket).await?;
                match ack["type"].as_str() {
                    Some("error") => return Err(remote_error(&ack)),
                    Some(
                        "rate_limits.updated"
                        | "conversation.item.created"
                        | "conversation.item.added"
                        | "conversation.item.done",
                    ) => continue,
                    Some("session.updated") => Self::validate_configuration_ack(&ack, model)?,
                    _ => {
                        return Err(
                            "STT [protocol] : événement inattendu avant l’acquittement de configuration.",
                        );
                    }
                }
                break;
            }
            Ok(())
        })
        .await
        .map_err(|_| "STT [timeout] : configuration expirée.")??;
        Ok(Self {
            socket,
            model: model.into(),
            active: false,
            failed: false,
            sent: 0,
            text_bytes: 0,
            current: None,
            committed: None,
            last_final: None,
            deadline: None,
            duplicates: 0,
        })
    }

    fn validate_configuration_ack(ack: &Value, model: &str) -> Result<(), &'static str> {
        let session = &ack["session"];
        if session["type"] != "transcription" {
            return Err("STT [configuration-session] : type de session incompatible.");
        }

        let input = &session["audio"]["input"];
        if input["format"]["type"] != "audio/pcm" || input["format"]["rate"] != 24000 {
            return Err("STT [configuration-format] : format audio incompatible.");
        }
        let transcription = &input["transcription"];
        let model_match = transcription["model"] == model;
        let delay = transcription.get("delay");
        let delay_present = delay.is_some();
        let delay_match = delay.and_then(Value::as_str) == Some("minimal");
        let delay_supported = match delay {
            None => true,
            Some(value) => matches!(
                value.as_str(),
                Some("minimal" | "low" | "medium" | "high" | "xhigh")
            ),
        };
        if !model_match || !delay_supported {
            crate::diagnostics::numbers(
                "STT configuration acknowledgement rejected",
                &[
                    ("model_match", u64::from(model_match)),
                    ("delay_match", u64::from(delay_match)),
                ],
            );
            if !model_match {
                return Err("STT [configuration-model] : modèle de transcription incompatible.");
            }
            return Err("STT [configuration-delay] : délai de transcription non pris en charge.");
        }
        if !delay_present || !delay_match {
            crate::diagnostics::numbers(
                "STT transcription delay not confirmed minimal",
                &[
                    ("delay_present", u64::from(delay_present)),
                    ("delay_match", u64::from(delay_match)),
                ],
            );
        }
        if input.get("turn_detection") != Some(&Value::Null)
            || transcription.get("language").is_some_and(|v| !v.is_null())
            || transcription.get("languages").is_some_and(|v| !v.is_null())
            || transcription.get("prompt").is_some_and(|v| !v.is_null())
        {
            return Err("STT [configuration-options] : options de transcription incompatibles.");
        }
        Ok(())
    }

    pub fn begin_passage(&mut self) -> Result<(), &'static str> {
        if self.failed || self.active {
            return Err("STT [protocol] : passage déjà actif ou interrompu.");
        }
        self.active = true;
        self.sent = 0;
        self.text_bytes = 0;
        self.current = None;
        self.committed = None;
        self.deadline = None;
        Ok(())
    }
    pub fn duplicate_finals(&self) -> u64 {
        self.duplicates
    }
    pub async fn append_pcm(&mut self, pcm: &[u8]) -> Result<(), &'static str> {
        if self.failed
            || !self.active
            || self.deadline.is_some()
            || pcm.is_empty()
            || pcm.len() % 2 != 0
            || pcm.len() > 32_000
            || self.sent + pcm.len() > MAX_AUDIO
        {
            self.failed = true;
            return Err("STT [capacity] : format ou limite audio invalide.");
        }
        // Poison before awaiting: cancelling an in-flight write cannot permit replay.
        self.failed = true;
        self.socket
            .send_json(json!({"type":"input_audio_buffer.append","audio":STANDARD.encode(pcm)}))
            .await?;
        self.sent += pcm.len();
        self.failed = false;
        Ok(())
    }
    pub async fn end_passage(&mut self) -> Result<(), &'static str> {
        if self.failed || !self.active || self.sent == 0 || self.deadline.is_some() {
            self.failed = true;
            return Err("STT [protocol] : commit vide, répété ou interrompu.");
        }
        self.failed = true;
        self.deadline = Some(Instant::now() + TIMEOUT);
        self.socket
            .send_json(json!({"type":"input_audio_buffer.commit"}))
            .await?;
        self.failed = false;
        Ok(())
    }
    pub async fn next_event(&mut self) -> Result<SttEvent, &'static str> {
        if self.failed {
            return Err("STT [closed] : session interrompue.");
        }
        let result = match self.deadline {
            Some(deadline) => timeout_at(deadline, read(&mut self.socket))
                .await
                .map_err(|_| "STT [timeout] : finalisation expirée.")
                .and_then(|r| r),
            None => read(&mut self.socket).await,
        }
        .and_then(|v| self.accept(v));
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn correlate(&mut self, value: &Value) -> Result<(), &'static str> {
        let key = identity(value)?;
        if !self.active
            || self.current.as_ref().is_some_and(|id| id != &key)
            || self.committed.as_ref().is_some_and(|id| id != &key.0)
        {
            return Err("STT [protocol] : identité contradictoire.");
        }
        self.current = Some(key);
        Ok(())
    }
    fn accept(&mut self, value: Value) -> Result<SttEvent, &'static str> {
        match value["type"].as_str() {
            Some("conversation.item.input_audio_transcription.delta") => {
                self.correlate(&value)?;
                let delta = value["delta"]
                    .as_str()
                    .ok_or("STT [protocol] : delta invalide.")?;
                self.text_bytes += delta.len();
                if self.text_bytes > MAX_REPLY {
                    return Err("STT [capacity] : transcription trop volumineuse.");
                }
                Ok(SttEvent::Provisional(delta.into()))
            }
            Some("input_audio_buffer.committed") => {
                let id = item(&value)?;
                if !self.active
                    || self.deadline.is_none()
                    || self.committed.is_some()
                    || self.current.as_ref().is_some_and(|key| key.0 != id)
                {
                    return Err("STT [protocol] : commit inattendu.");
                }
                if let Some(previous) = value.get("previous_item_id").filter(|p| !p.is_null()) {
                    if previous.as_str() != self.last_final.as_ref().map(|k| k.0.as_str()) {
                        return Err("STT [protocol] : prédécesseur contradictoire.");
                    }
                }
                self.committed = Some(id);
                Ok(SttEvent::Other)
            }
            Some("conversation.item.input_audio_transcription.completed") => {
                let key = identity(&value)?;
                if self.last_final.as_ref() == Some(&key) {
                    self.duplicates = self.duplicates.saturating_add(1);
                    return Ok(SttEvent::Other);
                }
                self.correlate(&value)?;
                if self.deadline.is_none() || self.committed.as_deref() != Some(key.0.as_str()) {
                    return Err("STT [protocol] : final sans commit acquitté.");
                }
                let text = value["transcript"]
                    .as_str()
                    .ok_or("STT [protocol] : final invalide.")?;
                if text.len() > MAX_REPLY {
                    return Err("STT [capacity] : final trop volumineux.");
                }
                self.last_final = Some(key);
                self.active = false;
                self.deadline = None;
                Ok(SttEvent::Final {
                    text: text.into(),
                    model: ModelProvenance {
                        configured: Some(self.model.clone()),
                        acknowledged: Some(self.model.clone()),
                        served: None,
                    },
                })
            }
            Some("conversation.item.input_audio_transcription.failed") => {
                Err("STT [recognition] : transcription refusée.")
            }
            Some("error") => Err(remote_error(&value)),
            Some(
                "rate_limits.updated"
                | "conversation.item.created"
                | "conversation.item.added"
                | "conversation.item.done",
            ) => Ok(SttEvent::Other),
            _ => {
                let (code, fingerprint) = rejected_event_diagnostic(&value);
                match fingerprint {
                    Some(fingerprint) => crate::diagnostics::numbers(
                        "STT streaming event rejected",
                        &[
                            ("event_type_code", code),
                            ("event_type_fingerprint", fingerprint),
                        ],
                    ),
                    None => crate::diagnostics::numbers(
                        "STT streaming event rejected",
                        &[("event_type_code", code)],
                    ),
                }
                Err("STT [protocol] : événement inattendu.")
            }
        }
    }
}
fn rejected_event_diagnostic(value: &Value) -> (u64, Option<u64>) {
    // Closed diagnostic codes: never copy an arbitrary provider string into logs.
    let kind = match value.get("type") {
        None => return (1, None),
        Some(Value::String(kind)) => kind,
        Some(_) => return (2, None),
    };
    if kind.is_empty()
        || kind.len() > 128
        || !kind
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
    {
        return (3, None);
    }
    let code = match kind.as_str() {
        "session.created" => 10,
        "session.updated" => 11,
        "input_audio_buffer.cleared" => 20,
        "input_audio_buffer.speech_started" => 21,
        "input_audio_buffer.speech_stopped" => 22,
        "conversation.item.input_audio_transcription.segment" => 30,
        "conversation.item.done" => 31,
        "conversation.item.deleted" => 32,
        "conversation.item.retrieved" => 33,
        "conversation.item.added" => 34,
        _ => 0,
    };
    if code == 0 {
        let hash = Sha256::digest(kind.as_bytes());
        let mut bytes = [0; 8];
        bytes.copy_from_slice(&hash[..8]);
        (code, Some(u64::from_be_bytes(bytes)))
    } else {
        (code, None)
    }
}
fn remote_error(value: &Value) -> &'static str {
    match value["error"]["code"].as_str() {
        Some("invalid_api_key") => "STT [authentication] : authentification OpenAI refusée.",
        Some("model_not_found" | "permission_denied") => {
            "STT [access] : accès ou modèle OpenAI refusé."
        }
        Some("rate_limit_exceeded" | "insufficient_quota") => {
            "STT [quota] : quota ou limite OpenAI atteinte."
        }
        Some("server_error") => "STT [service] : service OpenAI indisponible.",
        _ => "STT [remote-error] : erreur OpenAI non catégorisée.",
    }
}
impl SttSession<NativeSocket> {
    pub async fn connect_selected(selection: &ServiceSelection) -> Result<Self, &'static str> {
        selection.validate()?;
        let secret = tokio::task::spawn_blocking(secrets::load)
            .await
            .map_err(|_| "Trousseau indisponible.")??;
        let request = stt_transport::request_for_model(&selection.model, Some(secret.as_str()))
            .map_err(|e| e.message())?;
        let (socket, _) = timeout(
            TIMEOUT,
            tokio_tungstenite::connect_async_with_config(
                request,
                Some(
                    tokio_tungstenite::tungstenite::protocol::WebSocketConfig::default()
                        .max_message_size(Some(MAX_REPLY))
                        .max_frame_size(Some(MAX_REPLY)),
                ),
                false,
            ),
        )
        .await
        .map_err(|_| "STT [timeout] : connexion expirée.")?
        .map_err(|e| stt_transport::classify(&e).message())?;
        Self::open(NativeSocket(socket), &selection.model).await
    }
}
impl<T: JsonSender + JsonReceiver> SttInput for SttSession<T> {
    async fn append_pcm(&mut self, pcm: &[u8]) -> Result<(), &'static str> {
        Self::append_pcm(self, pcm).await
    }
    async fn end_passage(&mut self) -> Result<(), &'static str> {
        Self::end_passage(self).await
    }
}
impl<T: JsonSender + JsonReceiver> SttOutput for SttSession<T> {
    async fn next_event(&mut self) -> Result<SttEvent, &'static str> {
        Self::next_event(self).await
    }
}
impl<T: JsonSender + JsonReceiver> LiveStt for SttSession<T> {
    fn begin_passage(&mut self) -> Result<(), &'static str> {
        Self::begin_passage(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejected_event_codes_are_closed_and_distinguish_missing_invalid_and_unknown() {
        for (event, code) in [
            (json!({"type":"synthetic-private-data"}), 0),
            (json!({"other":"synthetic-private-data"}), 1),
            (json!({"type":null}), 2),
            (json!({"type":42}), 2),
            (json!({"type":{"private":"synthetic-private-data"}}), 2),
            (json!({"type":"session.created"}), 10),
            (json!({"type":"session.updated"}), 11),
            (json!({"type":"input_audio_buffer.cleared"}), 20),
            (json!({"type":"input_audio_buffer.speech_started"}), 21),
            (json!({"type":"input_audio_buffer.speech_stopped"}), 22),
            (
                json!({"type":"conversation.item.input_audio_transcription.segment"}),
                30,
            ),
            (json!({"type":"conversation.item.done"}), 31),
            (json!({"type":"conversation.item.deleted"}), 32),
            (json!({"type":"conversation.item.retrieved"}), 33),
            (json!({"type":"conversation.item.added"}), 34),
        ] {
            let (actual, fingerprint) = rejected_event_diagnostic(&event);
            assert_eq!(actual, code);
            assert_eq!(fingerprint.is_some(), code == 0);
        }
    }

    #[test]
    fn unknown_event_fingerprint_is_stable_and_only_hashes_the_type() {
        let event = json!({"type":"abc","text":"synthetic-private-data"});
        let expected = (0, Some(0xba7816bf8f01cfea));
        assert_eq!(rejected_event_diagnostic(&event), expected);
        assert_eq!(
            rejected_event_diagnostic(&json!({"type":"abc","unrelated":42})),
            expected
        );
    }

    #[test]
    fn event_fingerprints_enforce_token_alphabet_and_length_bounds() {
        for kind in [
            "a".to_string(),
            "a".repeat(128),
            "Token_1.test-event".into(),
        ] {
            let (code, fingerprint) = rejected_event_diagnostic(&json!({"type":kind}));
            assert_eq!(code, 0);
            assert!(fingerprint.is_some());
        }
        for kind in ["", "a b", "a\nb", "a/b", "a:b", "é", "\0"] {
            assert_eq!(rejected_event_diagnostic(&json!({"type":kind})), (3, None));
        }
        assert_eq!(
            rejected_event_diagnostic(&json!({"type":"a".repeat(129)})),
            (3, None)
        );
    }

    struct Waiting {
        incoming: tokio::sync::mpsc::Receiver<Value>,
    }
    impl JsonSender for Waiting {
        async fn send_json(&mut self, _: Value) -> Result<(), &'static str> {
            Ok(())
        }
    }
    impl JsonReceiver for Waiting {
        async fn receive_text(&mut self) -> Result<String, &'static str> {
            self.incoming
                .recv()
                .await
                .map(|v| v.to_string())
                .ok_or("closed")
        }
    }

    #[tokio::test]
    async fn quiet_listening_is_cancellable_but_commit_has_an_absolute_deadline() {
        let (sender, receiver) = tokio::sync::mpsc::channel(4);
        sender
            .send(json!({"type":"session.created"}))
            .await
            .unwrap();
        sender
            .send(json!({"type":"session.updated","session":configuration("gpt-realtime-whisper")}))
            .await
            .unwrap();
        let mut session = SttSession::open(Waiting { incoming: receiver }, "gpt-realtime-whisper")
            .await
            .unwrap();
        assert!(
            timeout(Duration::from_millis(5), session.next_event())
                .await
                .is_err()
        );
        assert!(!session.failed);
        session.begin_passage().unwrap();
        session.append_pcm(&[0, 0]).await.unwrap();
        session.end_passage().await.unwrap();
        session.deadline = Some(Instant::now() - Duration::from_millis(1));
        assert!(
            session
                .next_event()
                .await
                .err()
                .unwrap()
                .contains("[timeout]")
        );
        assert!(session.failed);
    }
}

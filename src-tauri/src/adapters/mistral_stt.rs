//! Shared protocol for native WSS and deterministic in-memory transport tests.
use crate::{
    platform::{secrets, stt_transport},
    ports::stt::{Finality, SttEvent},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    time::Duration,
};
use tokio::{net::TcpStream, time::timeout};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};
const TIMEOUT: Duration = Duration::from_secs(45);
const MAX_REPLY: usize = 65_536;
const MAX_AUDIO: usize = 16_000 * 2 * 60;

pub trait JsonSender: Send {
    fn send_json(&mut self, value: Value) -> impl Future<Output = Result<(), &'static str>> + Send;
}
pub trait JsonReceiver: Send {
    fn receive_text(&mut self) -> impl Future<Output = Result<String, &'static str>> + Send;
}
type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub struct NativeSocket(Socket);
pub struct NativeSender(futures_util::stream::SplitSink<Socket, Message>);
pub struct NativeReceiver(futures_util::stream::SplitStream<Socket>);

async fn send(
    sink: &mut (impl futures_util::Sink<Message> + Unpin),
    value: Value,
) -> Result<(), &'static str> {
    timeout(TIMEOUT, sink.send(Message::Text(value.to_string().into())))
        .await
        .map_err(|_| "Envoi STT expiré.")?
        .map_err(|_| "Envoi STT impossible.")
}
async fn receive(
    stream: &mut (
             impl futures_util::Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>
             + Unpin
         ),
) -> Result<String, &'static str> {
    // The deadline covers ping/other frames too, not a fresh timer per frame.
    timeout(TIMEOUT, async {
        loop {
            match stream.next().await {
                Some(Ok(Message::Text(text))) => return Ok(text.to_string()),
                Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {}
                _ => return Err("Connexion STT interrompue ; passage non intégré."),
            }
        }
    })
    .await
    .map_err(|_| "Réponse STT expirée.")?
}
impl JsonSender for NativeSocket {
    async fn send_json(&mut self, value: Value) -> Result<(), &'static str> {
        send(&mut self.0, value).await
    }
}
impl JsonSender for NativeSender {
    async fn send_json(&mut self, value: Value) -> Result<(), &'static str> {
        send(&mut self.0, value).await
    }
}
impl JsonReceiver for NativeSocket {
    async fn receive_text(&mut self) -> Result<String, &'static str> {
        receive(&mut self.0).await
    }
}
impl JsonReceiver for NativeReceiver {
    async fn receive_text(&mut self) -> Result<String, &'static str> {
        receive(&mut self.0).await
    }
}
async fn read_json<T: JsonReceiver>(socket: &mut T) -> Result<Value, &'static str> {
    let text = socket
        .receive_text()
        .await
        .map_err(|_| "Réception STT impossible ; passage non intégré.")?;
    if text.len() > MAX_REPLY {
        return Err("Réponse STT trop volumineuse.");
    }
    serde_json::from_str(&text).map_err(|_| "Événement STT illisible.")
}

// 0 ready, 1 ending, 2 ended, 3 failed, 4 append in flight.
struct InputState {
    sent: usize,
    phase: Arc<AtomicU8>,
}
impl Default for InputState {
    fn default() -> Self {
        Self {
            sent: 0,
            phase: Arc::new(AtomicU8::new(0)),
        }
    }
}
struct SendAttempt {
    phase: Arc<AtomicU8>,
    expected: u8,
    complete: bool,
}
impl SendAttempt {
    fn begin(phase: &Arc<AtomicU8>, expected: u8) -> Result<Self, &'static str> {
        phase
            .compare_exchange(0, expected, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| "Passage déjà terminé ou interrompu.")?;
        Ok(Self {
            phase: phase.clone(),
            expected,
            complete: false,
        })
    }
    fn finish(mut self, next: u8) -> Result<(), &'static str> {
        self.phase
            .compare_exchange(self.expected, next, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| "Passage interrompu pendant l'envoi.")?;
        self.complete = true;
        Ok(())
    }
}
impl Drop for SendAttempt {
    fn drop(&mut self) {
        if !self.complete {
            self.phase.store(3, Ordering::Release);
        }
    }
}
impl InputState {
    async fn append<T: JsonSender>(
        &mut self,
        socket: &mut T,
        pcm: &[u8],
    ) -> Result<(), &'static str> {
        if pcm.is_empty()
            || pcm.len() % 2 != 0
            || pcm.len() > 32_000
            || self.sent + pcm.len() > MAX_AUDIO
        {
            self.phase.store(3, Ordering::Release);
            return Err("Limite ou format du passage audio : passage non intégré.");
        }
        let attempt = SendAttempt::begin(&self.phase, 4)?;
        self.sent += pcm.len();
        socket
            .send_json(json!({"type":"input_audio.append","audio":STANDARD.encode(pcm)}))
            .await
            .map_err(|_| "Envoi STT impossible ; passage abandonné sans réémission.")?;
        attempt.finish(0)
    }
    async fn end<T: JsonSender>(&mut self, socket: &mut T) -> Result<(), &'static str> {
        let attempt = SendAttempt::begin(&self.phase, 1)?;
        // The documented realtime finish message drains the remaining transcript.
        // An extra undocumented flush is not needed before input_audio.end.
        socket
            .send_json(json!({"type":"input_audio.end"}))
            .await
            .map_err(|_| "Fin STT interrompue ; aucun nouvel envoi.")?;
        attempt.finish(2)
    }
}
async fn event<T: JsonReceiver>(
    socket: &mut T,
    finality: &mut Finality,
    phase: &AtomicU8,
) -> Result<SttEvent, &'static str> {
    if finality.is_finished() {
        return Err("Passage déjà finalisé ; événement rejeté.");
    }
    if phase.load(Ordering::Acquire) == 3 {
        return Err("Passage STT interrompu ; aucun nouvel événement accepté.");
    }
    let value = match read_json(socket).await {
        Ok(value) => value,
        Err(error) => {
            phase.store(3, Ordering::Release);
            return Err(error);
        }
    };
    if value["type"] == "transcription.done" && phase.load(Ordering::Acquire) != 2 {
        phase.store(3, Ordering::Release);
        return Err("Final STT prématuré ; passage non intégré.");
    }
    let result = finality.accept(&value);
    if result.is_err() {
        phase.store(3, Ordering::Release);
    }
    result
}

pub struct SttSession<T> {
    socket: T,
    input: InputState,
    finality: Finality,
}
pub type MistralStt = SttSession<NativeSocket>;
impl<T: JsonSender + JsonReceiver> SttSession<T> {
    pub async fn open(mut socket: T) -> Result<Self, &'static str> {
        if read_json(&mut socket).await?["type"] != "session.created" {
            return Err("Session STT non créée.");
        }
        socket.send_json(json!({"type":"session.update","session":{"audio_format":{"encoding":"pcm_s16le","sample_rate":16000},"language":"fr","target_streaming_delay_ms":480}})).await.map_err(|_|"Configuration STT impossible.")?;
        if read_json(&mut socket).await?["type"] != "session.updated" {
            return Err("Format STT non acquitté.");
        }
        Ok(Self {
            socket,
            input: InputState::default(),
            finality: Finality::default(),
        })
    }
    pub async fn append_pcm(&mut self, pcm: &[u8]) -> Result<(), &'static str> {
        self.input.append(&mut self.socket, pcm).await
    }
    pub async fn end_passage(&mut self) -> Result<(), &'static str> {
        self.input.end(&mut self.socket).await
    }
    pub async fn next_event(&mut self) -> Result<SttEvent, &'static str> {
        event(&mut self.socket, &mut self.finality, &self.input.phase).await
    }
}
impl SttSession<NativeSocket> {
    pub async fn connect_explicit() -> Result<Self, &'static str> {
        Self::connect_selected(&crate::config::preferences::Preferences::default().stt).await
    }
    pub async fn connect_selected(
        selection: &crate::config::preferences::ServiceSelection,
    ) -> Result<Self, &'static str> {
        selection.validate()?;
        let secret = tokio::task::spawn_blocking(secrets::load)
            .await
            .map_err(|_| "Trousseau indisponible.")??;
        let request = stt_transport::request_for_model(&selection.model, Some(secret.as_str()))
            .map_err(stt_transport::Failure::message)?;
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
        .map_err(|_| stt_transport::Failure::Timeout.message())?
        .map_err(|e| stt_transport::classify(&e).message())?;
        crate::diagnostics::event("STT : connexion WebSocket établie");
        let stream = Self::open(NativeSocket(socket)).await?;
        crate::diagnostics::event("STT : format audio acquitté");
        Ok(stream)
    }
    pub fn into_parts(self) -> (SttWriter, SttReader) {
        let (writer, reader) = self.socket.0.split();
        let phase = self.input.phase.clone();
        (
            SttWriter {
                socket: NativeSender(writer),
                input: self.input,
            },
            SttReader {
                socket: NativeReceiver(reader),
                finality: self.finality,
                phase,
            },
        )
    }
    pub async fn close(mut self) {
        let _ = timeout(Duration::from_secs(2), self.socket.0.close(None)).await;
    }
}
pub struct SttWriter<T = NativeSender> {
    socket: T,
    input: InputState,
}
pub struct SttReader<T = NativeReceiver> {
    socket: T,
    finality: Finality,
    phase: Arc<AtomicU8>,
}
impl<T: JsonSender> SttWriter<T> {
    pub async fn append_pcm(&mut self, pcm: &[u8]) -> Result<(), &'static str> {
        self.input.append(&mut self.socket, pcm).await
    }
    pub async fn end_passage(&mut self) -> Result<(), &'static str> {
        self.input.end(&mut self.socket).await
    }
}
impl<T: JsonReceiver> SttReader<T> {
    pub async fn next_event(&mut self) -> Result<SttEvent, &'static str> {
        event(&mut self.socket, &mut self.finality, &self.phase).await
    }
}
impl<T: JsonSender> crate::ports::stt::SttInput for SttWriter<T> {
    async fn append_pcm(&mut self, pcm: &[u8]) -> Result<(), &'static str> {
        SttWriter::append_pcm(self, pcm).await
    }
    async fn end_passage(&mut self) -> Result<(), &'static str> {
        SttWriter::end_passage(self).await
    }
}
impl<T: JsonReceiver> crate::ports::stt::SttOutput for SttReader<T> {
    async fn next_event(&mut self) -> Result<SttEvent, &'static str> {
        SttReader::next_event(self).await
    }
}

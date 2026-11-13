use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use vocalnotes_lib::{
    adapters::{
        mistral_interpreter::{ChatTransport, build_request},
        mistral_stt::{JsonReceiver, JsonSender, SttSession},
    },
    ports::stt::SttEvent,
};

struct FakeSocket {
    incoming: VecDeque<Result<String, &'static str>>,
    sent: Arc<Mutex<Vec<Value>>>,
    fail: Arc<AtomicBool>,
}
impl JsonSender for FakeSocket {
    async fn send_json(&mut self, value: Value) -> Result<(), &'static str> {
        self.sent.lock().unwrap().push(value);
        if self.fail.load(Ordering::SeqCst) {
            Err("synthetic-private-error")
        } else {
            Ok(())
        }
    }
}
impl JsonReceiver for FakeSocket {
    async fn receive_text(&mut self) -> Result<String, &'static str> {
        self.incoming
            .pop_front()
            .unwrap_or(Err("synthetic-private-error"))
    }
}
fn socket(events: Vec<Value>) -> (FakeSocket, Arc<Mutex<Vec<Value>>>) {
    let sent = Arc::new(Mutex::new(vec![]));
    (
        FakeSocket {
            incoming: events.into_iter().map(|v| Ok(v.to_string())).collect(),
            sent: sent.clone(),
            fail: Arc::new(AtomicBool::new(false)),
        },
        sent,
    )
}
fn events(extra: Vec<Value>) -> Vec<Value> {
    let mut e = vec![
        json!({"type":"session.created"}),
        json!({"type":"session.updated"}),
    ];
    e.extend(extra);
    e
}

#[test]
fn legacy_native_transports_cannot_send_openai_credentials_to_mistral() {
    use vocalnotes_lib::adapters::mistral_interpreter::NativeChatTransport;
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        assert!(
            NativeChatTransport
                .execute(json!({"synthetic":"request"}))
                .await
                .is_err()
        );
        assert!(
            vocalnotes_lib::adapters::mistral_stt::MistralStt::connect_explicit()
                .await
                .is_err()
        );
    });
}

#[tokio::test]
async fn stt_requires_ack_and_only_done_is_final_including_corrections() {
    let (transport, sent) = socket(events(vec![
        json!({"type":"transcription.text.delta","text":"dix"}),
        json!({"type":"transcription.segment","text":"dix","start":0,"end":1}),
        json!({"type":"transcription.done","text":"douze","model":"served-stt"}),
    ]));
    let mut stream = SttSession::open(transport).await.unwrap();
    assert_eq!(
        sent.lock().unwrap()[0]["session"]["audio_format"]["sample_rate"],
        16000
    );
    assert_eq!(sent.lock().unwrap()[0]["session"]["language"], "fr");
    assert_eq!(
        sent.lock().unwrap()[0]["session"]["target_streaming_delay_ms"],
        480
    );
    stream.append_pcm(&[0, 1]).await.unwrap();
    stream.end_passage().await.unwrap();
    assert!(matches!(
        stream.next_event().await.unwrap(),
        SttEvent::Provisional(_)
    ));
    assert!(matches!(
        stream.next_event().await.unwrap(),
        SttEvent::Other
    ));
    assert!(
        matches!(stream.next_event().await.unwrap(),SttEvent::Final{text,model} if text=="douze" && model.served.as_deref()==Some("served-stt"))
    );
    assert!(stream.next_event().await.is_err());
    assert!(stream.end_passage().await.is_err());
    let sent = sent.lock().unwrap();
    assert!(!sent.iter().any(|v| v["type"] == "input_audio.flush"));
    assert_eq!(
        sent.iter()
            .filter(|v| v["type"] == "input_audio.end")
            .count(),
        1
    );
    assert_eq!(
        sent.iter()
            .filter(|v| v["type"] == "input_audio.append")
            .count(),
        1
    );
}

#[tokio::test]
async fn stt_bad_ack_early_done_invalid_json_and_network_errors_fail_closed() {
    let (transport, sent) = socket(vec![
        json!({"type":"session.created"}),
        json!({"type":"error","detail":"synthetic-secret"}),
    ]);
    assert!(SttSession::open(transport).await.is_err());
    assert_eq!(sent.lock().unwrap().len(), 1);
    let (transport, _) = socket(events(vec![
        json!({"type":"transcription.done","text":"early","model":"served"}),
    ]));
    let mut stream = SttSession::open(transport).await.unwrap();
    assert!(stream.next_event().await.is_err());
    let (mut transport, _) = socket(events(vec![]));
    transport
        .incoming
        .push_back(Ok("invalid JSON synthetic-secret".into()));
    let mut stream = SttSession::open(transport).await.unwrap();
    let error = stream.next_event().await.err().unwrap();
    assert!(!error.contains("synthetic-secret"));
    let (transport, _) = socket(events(vec![]));
    let mut stream = SttSession::open(transport).await.unwrap();
    assert!(
        !stream
            .next_event()
            .await
            .err()
            .unwrap()
            .contains("synthetic-private-error")
    );
}

#[tokio::test]
async fn failed_audio_send_is_terminal_and_is_never_replayed() {
    let (transport, sent) = socket(events(vec![]));
    let fail = transport.fail.clone();
    let mut stream = SttSession::open(transport).await.unwrap();
    fail.store(true, Ordering::SeqCst);
    assert!(stream.append_pcm(&[0, 1]).await.is_err());
    assert!(stream.append_pcm(&[0, 1]).await.is_err());
    assert!(stream.end_passage().await.is_err());
    assert_eq!(
        sent.lock()
            .unwrap()
            .iter()
            .filter(|v| v["type"] == "input_audio.append")
            .count(),
        1
    );
}

#[test]
fn auth_is_sensitive_and_cannot_enter_url_debug_or_json_payload() {
    let body = json!({"model":"mistral-large-latest"});
    let request = build_request(&reqwest::Client::new(), "synthetic-secret-auth", &body).unwrap();
    assert!(request.headers()["Authorization"].is_sensitive());
    assert!(!request.url().as_str().contains("synthetic-secret-auth"));
    assert!(!format!("{request:?}").contains("synthetic-secret-auth"));
}

struct BlockingSocket {
    incoming: VecDeque<String>,
    sent: Arc<Mutex<Vec<Value>>>,
    blocked: &'static str,
}
impl JsonSender for BlockingSocket {
    async fn send_json(&mut self, value: Value) -> Result<(), &'static str> {
        let blocked = value["type"] == self.blocked;
        self.sent.lock().unwrap().push(value);
        if blocked {
            std::future::pending().await
        } else {
            Ok(())
        }
    }
}
impl JsonReceiver for BlockingSocket {
    async fn receive_text(&mut self) -> Result<String, &'static str> {
        self.incoming.pop_front().ok_or("closed")
    }
}
#[tokio::test]
async fn cancelling_audio_send_poisoned_the_passage_before_any_possible_replay() {
    let sent = Arc::new(Mutex::new(vec![]));
    let socket = BlockingSocket {
        incoming: events(vec![]).into_iter().map(|v| v.to_string()).collect(),
        sent: sent.clone(),
        blocked: "input_audio.append",
    };
    let mut stream = SttSession::open(socket).await.unwrap();
    assert!(
        tokio::time::timeout(Duration::from_millis(5), stream.append_pcm(&[0, 1]))
            .await
            .is_err()
    );
    let retry = tokio::time::timeout(Duration::from_millis(5), stream.append_pcm(&[0, 1])).await;
    assert!(matches!(retry, Ok(Err(_))));
    assert_eq!(
        sent.lock()
            .unwrap()
            .iter()
            .filter(|v| v["type"] == "input_audio.append")
            .count(),
        1
    );
    assert!(stream.end_passage().await.is_err());
}

#[tokio::test]
async fn stt_capacity_is_an_error_without_a_second_passage_or_truncation() {
    let (socket, sent) = socket(events(vec![]));
    let mut stream = SttSession::open(socket).await.unwrap();
    let pcm = vec![0u8; 32_000];
    for _ in 0..60 {
        stream.append_pcm(&pcm).await.unwrap();
    }
    assert!(stream.append_pcm(&[0, 0]).await.is_err());
    assert_eq!(
        sent.lock()
            .unwrap()
            .iter()
            .filter(|v| v["type"] == "input_audio.append")
            .count(),
        60
    );
}

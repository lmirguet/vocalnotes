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
        mistral_interpreter::{ChatReply, ChatTransport, MistralInterpreter, build_request},
        mistral_stt::{JsonReceiver, JsonSender, SttSession},
    },
    config::preferences::Preferences,
    ports::{
        interpreter::{InterpretationContext, Interruption},
        stt::SttEvent,
    },
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
        matches!(stream.next_event().await.unwrap(),SttEvent::Final{text,model} if text=="douze" && model=="served-stt")
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

struct FakeChat {
    replies: Mutex<VecDeque<Result<ChatReply, &'static str>>>,
    requests: Mutex<Vec<Value>>,
}
impl ChatTransport for FakeChat {
    async fn execute(&self, request: Value) -> Result<ChatReply, &'static str> {
        self.requests.lock().unwrap().push(request);
        self.replies
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or(Err("synthetic-secret-error"))
    }
}
fn chat(content: &str, reason: &str) -> ChatReply {
    ChatReply{status:200,body:json!({"model":"served-llm","choices":[{"finish_reason":reason,"message":{"content":content}}]}).to_string().into_bytes()}
}
fn context() -> InterpretationContext {
    InterpretationContext {
        blocks: vec![],
        save_available: false,
    }
}
fn adapter(replies: Vec<Result<ChatReply, &'static str>>) -> MistralInterpreter<FakeChat> {
    MistralInterpreter::with_transport(
        &Preferences::default().llm,
        FakeChat {
            replies: Mutex::new(replies.into()),
            requests: Mutex::new(vec![]),
        },
    )
    .unwrap()
}

#[test]
fn auth_is_sensitive_and_cannot_enter_url_debug_or_json_payload() {
    let body = json!({"model":"mistral-large-latest"});
    let request = build_request(&reqwest::Client::new(), "synthetic-secret-auth", &body).unwrap();
    assert!(request.headers()["Authorization"].is_sensitive());
    assert!(!request.url().as_str().contains("synthetic-secret-auth"));
    assert!(!format!("{request:?}").contains("synthetic-secret-auth"));
}

#[tokio::test]
async fn strict_ordinary_and_priority_requests_are_separate_and_countable() {
    let llm = adapter(vec![
        Ok(chat(
            r#"{"status":"resolved","message":"","actions":[]}"#,
            "stop",
        )),
        Ok(chat(r#"{"interrupt":"pause"}"#, "stop")),
    ]);
    let (_, model) = llm
        .interpret_final(
            "un passage",
            &vocalnotes_lib::domain::intention::DraftingIntention::new(
                "intention".into(),
                "".into(),
                "".into(),
                "".into(),
            )
            .unwrap(),
            context(),
        )
        .await
        .unwrap();
    assert_eq!(model, "served-llm");
    assert!(matches!(
        llm.interpret_interruption("mets en pause").await.unwrap(),
        Interruption::Pause
    ));
    let transport = llm.transport_for_test();
    let requests = transport.requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    let system = requests[0]["messages"][0]["content"].as_str().unwrap();
    assert!(system.contains("exige une action drafting_intent"));
    assert!(system.contains("synthèse concise"));
    assert!(system.contains(r#""tone":"neutre""#));
    let user: Value =
        serde_json::from_str(requests[0]["messages"][1]["content"].as_str().unwrap()).unwrap();
    assert_eq!(user["intention"]["purpose"], "intention");
    assert_eq!(user["intention"]["form"], "");
    let variants = requests[0]["response_format"]["json_schema"]["schema"]["properties"]["actions"]
        ["items"]["anyOf"]
        .as_array()
        .unwrap();
    let intent = variants
        .iter()
        .find(|v| v["properties"]["type"]["enum"][0] == "drafting_intent")
        .unwrap();
    assert_eq!(
        intent["required"],
        json!(["type", "text", "form", "tone", "context"])
    );
    for r in requests.iter() {
        assert_eq!(r["model"], "mistral-large-latest");
        assert_eq!(r["response_format"]["json_schema"]["strict"], true);
        assert!(r.get("tools").is_none());
        assert_eq!(r["messages"].as_array().unwrap().len(), 2);
    }
    assert!(
        !requests[1]["messages"][1]["content"]
            .as_str()
            .unwrap()
            .contains("intention")
    );
}

#[tokio::test]
async fn llm_errors_truncation_and_invalid_json_have_no_retry_or_fallback() {
    for reply in [
        Err("synthetic-secret-error"),
        Ok(ChatReply {
            status: 401,
            body: b"synthetic-secret-body".to_vec(),
        }),
        Ok(chat("invalid JSON synthetic-secret", "stop")),
        Ok(chat(
            r#"{"status":"resolved","message":"","actions":[]}"#,
            "length",
        )),
        Ok(chat(r#"{"interrupt":"replace","actions":[]}"#, "stop")),
    ] {
        let llm = adapter(vec![reply]);
        let error = llm
            .interpret_final("texte", &Default::default(), context())
            .await
            .err()
            .unwrap();
        assert!(!error.contains("synthetic-secret"));
        assert_eq!(llm.transport_for_test().requests.lock().unwrap().len(), 1);
    }
}

struct PendingChat {
    started: Arc<AtomicBool>,
    dropped: Arc<AtomicBool>,
}
struct Dropped(Arc<AtomicBool>);
impl Drop for Dropped {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}
impl ChatTransport for PendingChat {
    async fn execute(&self, _: Value) -> Result<ChatReply, &'static str> {
        self.started.store(true, Ordering::SeqCst);
        let _guard = Dropped(self.dropped.clone());
        std::future::pending().await
    }
}
#[tokio::test]
async fn cancelling_pending_llm_drops_request_without_retry() {
    let started = Arc::new(AtomicBool::new(false));
    let dropped = Arc::new(AtomicBool::new(false));
    let llm = MistralInterpreter::with_transport(
        &Preferences::default().llm,
        PendingChat {
            started: started.clone(),
            dropped: dropped.clone(),
        },
    )
    .unwrap();
    assert!(
        tokio::time::timeout(
            Duration::from_millis(5),
            llm.interpret_final("texte", &Default::default(), context())
        )
        .await
        .is_err()
    );
    assert!(started.load(Ordering::SeqCst));
    assert!(dropped.load(Ordering::SeqCst));
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

#[tokio::test]
async fn llm_oversize_tools_and_priority_edit_attempts_are_refused() {
    let mut tools = chat(r#"{"status":"resolved","message":"","actions":[]}"#, "stop");
    let mut body: Value = serde_json::from_slice(&tools.body).unwrap();
    body["choices"][0]["message"]["tool_calls"] = json!([{"name":"shell"}]);
    tools.body = body.to_string().into_bytes();
    for reply in [
        tools,
        ChatReply {
            status: 200,
            body: vec![b' '; 65_537],
        },
        chat("{}", "tool_calls"),
    ] {
        let llm = adapter(vec![Ok(reply)]);
        assert!(
            llm.interpret_final("texte", &Default::default(), context())
                .await
                .is_err()
        );
        assert_eq!(llm.transport_for_test().requests.lock().unwrap().len(), 1);
    }
    let llm = adapter(vec![Ok(chat(
        r#"{"interrupt":"pause","actions":[{"type":"edit"}]}"#,
        "stop",
    ))]);
    assert!(llm.interpret_interruption("pause").await.is_err());
}

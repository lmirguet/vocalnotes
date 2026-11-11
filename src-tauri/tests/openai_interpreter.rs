use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use serde_json::{Value, json};
use vocalnotes_lib::{
    adapters::openai_interpreter::{
        OpenAIInterpreter, ResponsesReply, ResponsesTransport, build_request,
    },
    config::preferences::{Preferences, ServiceSelection},
    domain::intention::DraftingIntention,
    ports::interpreter::{Candidate, InterpretationContext, Status},
};

#[derive(Clone)]
struct FakeTransport {
    replies: Arc<Mutex<VecDeque<ResponsesReply>>>,
    requests: Arc<Mutex<Vec<Value>>>,
}

impl FakeTransport {
    fn new(replies: impl IntoIterator<Item = ResponsesReply>) -> Self {
        Self {
            replies: Arc::new(Mutex::new(replies.into_iter().collect())),
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn requests(&self) -> Vec<Value> {
        self.requests.lock().unwrap().clone()
    }
}

impl ResponsesTransport for FakeTransport {
    async fn execute(&self, request: reqwest::Request) -> Result<ResponsesReply, &'static str> {
        let body = request
            .body()
            .and_then(reqwest::Body::as_bytes)
            .expect("request JSON body");
        self.requests
            .lock()
            .unwrap()
            .push(serde_json::from_slice(body).unwrap());
        self.replies
            .lock()
            .unwrap()
            .pop_front()
            .ok_or("Unexpected fake request")
    }
}

fn selection() -> ServiceSelection {
    Preferences::default().llm
}

fn success(content: &str) -> ResponsesReply {
    ResponsesReply {
        status: 200,
        body: json!({
            "id": "resp_synthetic",
            "model": "gpt-6-luna",
            "status": "completed",
            "output": [
                {"type": "reasoning", "id": "rs_synthetic", "summary": []},
                {"type": "message", "role": "assistant", "content": [
                    {"type": "output_text", "text": content}
                ]}
            ]
        })
        .to_string()
        .into_bytes(),
    }
}

fn context() -> InterpretationContext {
    InterpretationContext {
        blocks: vec![Candidate {
            block_id: "b0".into(),
            source: "Acheter des cahiers.".into(),
        }],
        save_available: false,
    }
}

#[tokio::test]
async fn sends_non_streaming_private_responses_request_with_complete_root_schema() {
    let fake = FakeTransport::new([success(
        r#"{"status":"resolved","message":"","actions":[]}"#,
    )]);
    let interpreter =
        OpenAIInterpreter::with_transport(&selection(), "synthetic-test-key", fake.clone())
            .unwrap();

    let (intervention, model) = interpreter
        .interpret_final(
            "Prévoir aussi des stylos.",
            &DraftingIntention::default(),
            context(),
        )
        .await
        .unwrap();

    assert!(matches!(intervention.status, Status::Resolved));
    assert_eq!(model, "gpt-6-luna");
    let requests = fake.requests();
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert_eq!(request["model"], "gpt-6-luna");
    assert_eq!(request["store"], false);
    assert_eq!(request["stream"], false);
    assert_eq!(request["max_output_tokens"], 4096);
    assert!(request.get("temperature").is_none());
    assert!(request.get("max_tokens").is_none());
    assert!(request.get("reasoning").is_none());

    let format = &request["text"]["format"];
    assert_eq!(format["type"], "json_schema");
    assert_eq!(format["name"], "intervention");
    assert_eq!(format["strict"], true);
    let schema = &format["schema"];
    assert_eq!(schema["type"], "object");
    assert!(schema.get("anyOf").is_none(), "root must remain an object");
    assert_eq!(schema["required"], json!(["status", "message", "actions"]));
    assert_eq!(
        schema["properties"]["status"]["enum"],
        json!(["resolved", "clarify"])
    );
    assert_schema_objects_are_closed_and_complete(schema);
}

fn assert_schema_objects_are_closed_and_complete(value: &Value) {
    match value {
        Value::Object(object) => {
            if object.get("type") == Some(&json!("object")) {
                assert_eq!(object["additionalProperties"], false);
                let properties = object["properties"].as_object().unwrap();
                let required = object["required"].as_array().unwrap();
                assert_eq!(required.len(), properties.len());
                for key in properties.keys() {
                    assert!(required.iter().any(|entry| entry == key));
                }
            }
            for child in object.values() {
                assert_schema_objects_are_closed_and_complete(child);
            }
        }
        Value::Array(values) => {
            for child in values {
                assert_schema_objects_are_closed_and_complete(child);
            }
        }
        _ => {}
    }
}

#[test]
fn request_builder_uses_fixed_https_origin_and_only_header_authentication() {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let request = build_request(
        &client,
        "synthetic-test-key",
        &json!({"model":"gpt-6-luna","store":false}),
    )
    .unwrap();

    assert_eq!(request.url().scheme(), "https");
    assert_eq!(request.url().host_str(), Some("api.openai.com"));
    assert_eq!(request.url().path(), "/v1/responses");
    assert!(request.url().query().is_none());
    assert_eq!(
        request.headers()[reqwest::header::AUTHORIZATION],
        "Bearer synthetic-test-key"
    );
    assert!(request.headers()[reqwest::header::AUTHORIZATION].is_sensitive());
    let body = request.body().and_then(reqwest::Body::as_bytes).unwrap();
    assert!(!String::from_utf8_lossy(body).contains("synthetic-test-key"));
}

#[tokio::test]
async fn extracts_only_one_assistant_output_text_and_ignores_reasoning() {
    let fake = FakeTransport::new([success(
        r#"{"status":"clarify","message":"Quelle cible ?","actions":[]}"#,
    )]);
    let interpreter =
        OpenAIInterpreter::with_transport(&selection(), "synthetic-test-key", fake).unwrap();
    let (intervention, served) = interpreter
        .interpret_final("Replace this.", &DraftingIntention::default(), context())
        .await
        .unwrap();
    assert!(matches!(intervention.status, Status::Clarify));
    assert_eq!(served, "gpt-6-luna");
}

#[tokio::test]
async fn rejects_non_completed_refusal_tools_ambiguous_or_malformed_output_and_missing_model() {
    let mut replies = Vec::new();
    for (status, model, output, incomplete) in [
        (
            "incomplete",
            "gpt-6-luna",
            json!([]),
            json!({"reason":"max_output_tokens"}),
        ),
        (
            "completed",
            "gpt-6-luna",
            json!([{"type":"message","role":"assistant","content":[{"type":"refusal","refusal":"No"}]}]),
            Value::Null,
        ),
        (
            "completed",
            "gpt-6-luna",
            json!([{"type":"function_call","name":"tool","arguments":"{}"}]),
            Value::Null,
        ),
        (
            "completed",
            "gpt-6-luna",
            json!([
                {"type":"message","role":"assistant","content":[{"type":"output_text","text":"{}"}]},
                {"type":"message","role":"assistant","content":[{"type":"output_text","text":"{}"}]}
            ]),
            Value::Null,
        ),
        (
            "completed",
            "gpt-6-luna",
            json!([{"type":"message","role":"assistant","content":[{"type":"output_text","text":"{"}]}]),
            Value::Null,
        ),
        (
            "completed",
            "",
            json!([{"type":"message","role":"assistant","content":[{"type":"output_text","text":"{}"}]}]),
            Value::Null,
        ),
    ] {
        replies.push(ResponsesReply {
            status: 200,
            body: json!({
                "model":model,
                "status":status,
                "output":output,
                "incomplete_details":incomplete
            })
            .to_string()
            .into_bytes(),
        });
    }
    replies.push(ResponsesReply {
        status: 200,
        body: br#"{"model":"gpt-6-luna","status":"completed","output_text":"{}"}"#.to_vec(),
    });
    replies.push(ResponsesReply {
        status: 200,
        body: br#"{"status":"completed","output":[]}"#.to_vec(),
    });
    replies.push(ResponsesReply {
        status: 200,
        body: b"not-json".to_vec(),
    });
    for (field, invalid) in [
        ("model", Value::Null),
        ("model", json!("model with whitespace")),
        ("error", json!({"message":"private provider failure"})),
        ("incomplete_details", json!({"reason":"max_output_tokens"})),
        ("output", json!([{"type":"unexpected"}])),
        (
            "output",
            json!([{
                "type":"message","role":"assistant","content":[
                    {"type":"output_text","text":r#"{"status":"resolved","message":"","actions":[]}"#},
                    {"type":"output_text","text":r#"{"status":"resolved","message":"","actions":[]}"#}
                ]
            }]),
        ),
    ] {
        let valid = success(r#"{"status":"resolved","message":"","actions":[]}"#);
        let mut envelope: Value = serde_json::from_slice(&valid.body).unwrap();
        envelope[field] = invalid;
        replies.push(ResponsesReply {
            status: 200,
            body: envelope.to_string().into_bytes(),
        });
    }

    for reply in replies {
        let fake = FakeTransport::new([reply]);
        let interpreter =
            OpenAIInterpreter::with_transport(&selection(), "synthetic-test-key", fake).unwrap();
        assert!(
            interpreter
                .interpret_final(
                    "Donne une réponse.",
                    &DraftingIntention::default(),
                    context(),
                )
                .await
                .is_err()
        );
    }
}

#[tokio::test]
async fn bounds_context_and_response_without_sending_or_truncating() {
    let fake = FakeTransport::new([]);
    let interpreter =
        OpenAIInterpreter::with_transport(&selection(), "synthetic-test-key", fake.clone())
            .unwrap();
    let too_large_context = InterpretationContext {
        blocks: vec![Candidate {
            block_id: "b0".into(),
            source: "x".repeat(16_384),
        }],
        save_available: false,
    };
    assert!(
        interpreter
            .interpret_final("passage", &DraftingIntention::default(), too_large_context,)
            .await
            .is_err()
    );
    assert!(fake.requests().is_empty());

    let fake = FakeTransport::new([ResponsesReply {
        status: 200,
        body: vec![b'x'; 65_537],
    }]);
    let interpreter =
        OpenAIInterpreter::with_transport(&selection(), "synthetic-test-key", fake).unwrap();
    let error = interpreter
        .interpret_final("passage", &DraftingIntention::default(), context())
        .await
        .err()
        .unwrap();
    assert!(error.contains("volumineuse"));
    assert!(!error.contains(&"x".repeat(32)));
}

#[tokio::test]
async fn maps_http_and_transport_failures_to_closed_categories_without_body_text() {
    for (status, expected) in [
        (401, "Authentification LLM refusée (401)."),
        (403, "Accès au modèle LLM refusé (403/404)."),
        (404, "Accès au modèle LLM refusé (403/404)."),
        (429, "Limitation ou quota LLM (429)."),
        (500, "Service LLM indisponible (5xx)."),
        (503, "Service LLM indisponible (5xx)."),
        (302, "Réponse HTTP LLM refusée ; aucun effet."),
    ] {
        let fake = FakeTransport::new([ResponsesReply {
            status,
            body: b"synthetic private provider message; secret; header".to_vec(),
        }]);
        let interpreter =
            OpenAIInterpreter::with_transport(&selection(), "synthetic-test-key", fake.clone())
                .unwrap();
        let error = interpreter
            .interpret_final("passage", &DraftingIntention::default(), context())
            .await
            .err()
            .unwrap();
        assert_eq!(error, expected);
        assert_eq!(fake.requests().len(), 1, "no retry or fallback");
    }
}

struct FailureTransport(&'static str, Arc<AtomicUsize>);

impl ResponsesTransport for FailureTransport {
    async fn execute(&self, _: reqwest::Request) -> Result<ResponsesReply, &'static str> {
        self.1.fetch_add(1, Ordering::SeqCst);
        Err(self.0)
    }
}

#[tokio::test]
async fn transport_categories_are_allowlisted_and_unknown_errors_are_redacted() {
    for (supplied, expected) in [
        (
            "Connexion LLM impossible ; aucun effet.",
            "Connexion LLM impossible ; aucun effet.",
        ),
        (
            "Certificat TLS LLM invalide ; aucun effet.",
            "Certificat TLS LLM invalide ; aucun effet.",
        ),
        (
            "Délai LLM dépassé ; aucun effet.",
            "Délai LLM dépassé ; aucun effet.",
        ),
        (
            "private body Authorization: synthetic-test-key",
            "Transport LLM en échec ; aucun effet.",
        ),
    ] {
        let calls = Arc::new(AtomicUsize::new(0));
        let adapter = OpenAIInterpreter::with_transport(
            &selection(),
            "synthetic-test-key",
            FailureTransport(supplied, calls.clone()),
        )
        .unwrap();
        let error = adapter
            .interpret_final("passage", &DraftingIntention::default(), context())
            .await
            .err()
            .unwrap();
        assert_eq!(error, expected);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}

#[derive(Clone, Default)]
struct PendingTransport {
    started: Arc<AtomicBool>,
    dropped: Arc<AtomicBool>,
    progress: Arc<AtomicUsize>,
}

struct DropSignal(Arc<AtomicBool>);

impl Drop for DropSignal {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

impl ResponsesTransport for PendingTransport {
    async fn execute(&self, _: reqwest::Request) -> Result<ResponsesReply, &'static str> {
        let _drop_signal = DropSignal(self.dropped.clone());
        self.started.store(true, Ordering::SeqCst);
        loop {
            tokio::time::sleep(Duration::from_millis(10)).await;
            self.progress.fetch_add(1, Ordering::SeqCst);
        }
    }
}

#[tokio::test]
async fn absolute_45_second_deadline_drops_inflight_work_despite_continuous_progress() {
    let transport = PendingTransport::default();
    let adapter =
        OpenAIInterpreter::with_transport(&selection(), "synthetic-test-key", transport.clone())
            .unwrap();
    let start = Instant::now();
    let error = tokio::time::timeout(
        Duration::from_secs(60),
        adapter.interpret_final("passage", &DraftingIntention::default(), context()),
    )
    .await
    .expect("absolute deadline must finish even while the transport makes progress")
    .err()
    .unwrap();
    assert_eq!(error, "Délai LLM dépassé ; aucun effet.");
    assert!(start.elapsed() >= Duration::from_secs(45));
    assert!(transport.progress.load(Ordering::SeqCst) > 10);
    assert!(transport.dropped.load(Ordering::SeqCst));
    let progress = transport.progress.load(Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(30)).await;
    assert_eq!(transport.progress.load(Ordering::SeqCst), progress);
}

#[tokio::test]
async fn caller_cancellation_drops_the_transport_without_background_work() {
    let transport = PendingTransport::default();
    let adapter =
        OpenAIInterpreter::with_transport(&selection(), "synthetic-test-key", transport.clone())
            .unwrap();
    assert!(
        tokio::time::timeout(
            Duration::from_millis(50),
            adapter.interpret_final("passage", &DraftingIntention::default(), context()),
        )
        .await
        .is_err()
    );
    assert!(transport.started.load(Ordering::SeqCst));
    assert!(transport.dropped.load(Ordering::SeqCst));
    let progress = transport.progress.load(Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(30)).await;
    assert_eq!(transport.progress.load(Ordering::SeqCst), progress);
}

#[tokio::test]
async fn ordinary_context_accepts_exactly_16384_serialized_bytes_and_rejects_one_more() {
    let intention = DraftingIntention::default();
    let overhead = json!({
        "passage":"",
        "intention":intention,
        "sections_candidates":context()
    })
    .to_string()
    .len();
    let fake = FakeTransport::new([success(
        r#"{"status":"resolved","message":"","actions":[]}"#,
    )]);
    let adapter =
        OpenAIInterpreter::with_transport(&selection(), "synthetic-test-key", fake.clone())
            .unwrap();
    assert!(
        adapter
            .interpret_final(&"x".repeat(16_384 - overhead), &intention, context())
            .await
            .is_ok()
    );
    assert_eq!(fake.requests()[0]["input"].as_str().unwrap().len(), 16_384);
    assert!(
        adapter
            .interpret_final(&"x".repeat(16_385 - overhead), &intention, context())
            .await
            .is_err()
    );
    assert_eq!(fake.requests().len(), 1);
}

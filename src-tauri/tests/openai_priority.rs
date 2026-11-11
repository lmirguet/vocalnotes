use std::sync::{Arc, Mutex};

use serde_json::{Value, json};
use vocalnotes_lib::{
    adapters::openai_interpreter::{OpenAIInterpreter, ResponsesReply, ResponsesTransport},
    config::preferences::Preferences,
    ports::interpreter::Interruption,
};

#[derive(Clone)]
struct RecordingTransport {
    requests: Arc<Mutex<Vec<Value>>>,
    output: String,
}

impl Default for RecordingTransport {
    fn default() -> Self {
        Self {
            requests: Arc::new(Mutex::new(Vec::new())),
            output: r#"{"interrupt":"pause"}"#.into(),
        }
    }
}

impl ResponsesTransport for RecordingTransport {
    async fn execute(&self, request: reqwest::Request) -> Result<ResponsesReply, &'static str> {
        let body = request
            .body()
            .and_then(reqwest::Body::as_bytes)
            .ok_or("Missing synthetic request body")?;
        self.requests
            .lock()
            .map_err(|_| "Recording transport unavailable")?
            .push(serde_json::from_slice(body).map_err(|_| "Invalid synthetic request")?);
        Ok(ResponsesReply {
            status: 200,
            body: json!({
                "model": "gpt-6-luna",
                "status": "completed",
                "output": [{
                    "type": "message",
                    "role": "assistant",
                    "content": [{
                        "type": "output_text",
                        "text": self.output
                    }]
                }]
            })
            .to_string()
            .into_bytes(),
        })
    }
}

#[tokio::test]
async fn priority_schema_is_restricted_and_prompt_covers_bilingual_citations() {
    let fake = RecordingTransport::default();
    let interpreter = OpenAIInterpreter::with_transport(
        &Preferences::default().llm,
        "synthetic-test-key",
        fake.clone(),
    )
    .unwrap();

    let result = interpreter
        .interpret_interruption("« Ne fais pas pause », dit-elle ; stoppez le partage.")
        .await
        .unwrap();
    assert_eq!(result, Interruption::Pause);

    let requests = fake.requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert_eq!(request["model"], "gpt-6-luna");
    assert_eq!(request["store"], false);
    assert_eq!(request["stream"], false);
    assert_eq!(
        request["text"]["format"]["schema"]["properties"]["interrupt"]["enum"],
        json!(["none", "pause", "stop", "manual"])
    );
    assert_eq!(
        request["text"]["format"]["schema"]["required"],
        json!(["interrupt"])
    );
    assert_eq!(
        request["text"]["format"]["schema"]["additionalProperties"],
        false
    );
    let instructions = request["instructions"].as_str().unwrap();
    for phrase in ["quoted", "citation", "pause", "pause", "stop", "manual"] {
        assert!(
            instructions.to_lowercase().contains(phrase),
            "missing prompt term: {phrase}"
        );
    }
    drop(requests);

    for (response, expected) in [
        (r#"{"interrupt":"none"}"#, Interruption::None),
        (r#"{"interrupt":"pause"}"#, Interruption::Pause),
        (r#"{"interrupt":"stop"}"#, Interruption::Stop),
        (r#"{"interrupt":"manual"}"#, Interruption::Manual),
    ] {
        let transport = RecordingTransport {
            requests: Arc::new(Mutex::new(Vec::new())),
            output: response.into(),
        };
        let adapter = OpenAIInterpreter::with_transport(
            &Preferences::default().llm,
            "synthetic-test-key",
            transport,
        )
        .unwrap();
        assert_eq!(
            adapter
                .interpret_interruption("Résumé de la phrase")
                .await
                .unwrap(),
            expected
        );
    }
}

#[tokio::test]
async fn priority_rejects_oversized_provisional_context_without_calling_transport() {
    let fake = RecordingTransport::default();
    let interpreter = OpenAIInterpreter::with_transport(
        &Preferences::default().llm,
        "synthetic-test-key",
        fake.clone(),
    )
    .unwrap();
    assert!(
        interpreter
            .interpret_interruption(&"x".repeat(4097))
            .await
            .is_err()
    );
    assert!(fake.requests.lock().unwrap().is_empty());
}

#[tokio::test]
async fn priority_context_accepts_exactly_4096_serialized_bytes_and_rejects_one_more() {
    let overhead = json!({"passage_provisoire":""}).to_string().len();
    let fake = RecordingTransport::default();
    let adapter = OpenAIInterpreter::with_transport(
        &Preferences::default().llm,
        "synthetic-test-key",
        fake.clone(),
    )
    .unwrap();
    assert!(
        adapter
            .interpret_interruption(&"x".repeat(4096 - overhead))
            .await
            .is_ok()
    );
    assert_eq!(
        fake.requests.lock().unwrap()[0]["input"]
            .as_str()
            .unwrap()
            .len(),
        4096
    );
    assert!(
        adapter
            .interpret_interruption(&"x".repeat(4097 - overhead))
            .await
            .is_err()
    );
    assert_eq!(fake.requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn priority_reply_cannot_express_an_edit_or_unknown_command() {
    for content in [
        r#"{"interrupt":"edit"}"#,
        r#"{"interrupt":"none","actions":[{"type":"edit"}]}"#,
        r#"{"interrupt":"pause","unexpected":true}"#,
    ] {
        let fake = FixedReply {
            body: content.into(),
        };
        let interpreter = OpenAIInterpreter::with_transport(
            &Preferences::default().llm,
            "synthetic-test-key",
            fake,
        )
        .unwrap();
        assert!(interpreter.interpret_interruption("pause").await.is_err());
    }
}

#[derive(Clone)]
struct FixedReply {
    body: String,
}

impl ResponsesTransport for FixedReply {
    async fn execute(&self, _request: reqwest::Request) -> Result<ResponsesReply, &'static str> {
        Ok(ResponsesReply {
            status: 200,
            body: json!({
                "model": "gpt-6-luna",
                "status": "completed",
                "output": [{
                    "type": "message",
                    "role": "assistant",
                    "content": [{"type":"output_text","text":self.body}]
                }]
            })
            .to_string()
            .into_bytes(),
        })
    }
}

use serde_json::{Value, json};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use vocalnotes_lib::adapters::openai_stt::{JsonReceiver, JsonSender, SttSession};
use vocalnotes_lib::ports::stt::SttEvent;

struct Socket {
    incoming: VecDeque<Value>,
    sent: Arc<Mutex<Vec<Value>>>,
}
impl JsonSender for Socket {
    async fn send_json(&mut self, value: Value) -> Result<(), &'static str> {
        self.sent.lock().unwrap().push(value);
        Ok(())
    }
}
impl JsonReceiver for Socket {
    async fn receive_text(&mut self) -> Result<String, &'static str> {
        self.incoming
            .pop_front()
            .map(|v| v.to_string())
            .ok_or("closed")
    }
}
fn updated() -> Value {
    json!({"type":"session.updated","session":{"type":"transcription","audio":{"input":{
        "format":{"type":"audio/pcm","rate":24000},
        "transcription":{"model":"gpt-realtime-whisper","delay":"minimal"},
        "turn_detection":null
    }}}})
}
async fn session(events: Vec<Value>) -> (SttSession<Socket>, Arc<Mutex<Vec<Value>>>) {
    let sent = Arc::new(Mutex::new(vec![]));
    let mut incoming = VecDeque::from([json!({"type":"session.created"}), updated()]);
    incoming.extend(events);
    let stream = SttSession::open(
        Socket {
            incoming,
            sent: sent.clone(),
        },
        "gpt-realtime-whisper",
    )
    .await
    .unwrap();
    (stream, sent)
}

#[tokio::test]
async fn partial_before_commit_final_once_and_next_passage_share_one_session() {
    let (mut stream, sent) = session(vec![
        json!({"type":"conversation.item.input_audio_transcription.delta","item_id":"item_1","content_index":0,"delta":"Hello"}),
        json!({"type":"input_audio_buffer.committed","item_id":"item_1","previous_item_id":null}),
        json!({"type":"conversation.item.input_audio_transcription.completed","item_id":"item_1","content_index":0,"transcript":"Hello."}),
        json!({"type":"conversation.item.input_audio_transcription.completed","item_id":"item_1","content_index":0,"transcript":"Hello."}),
        json!({"type":"conversation.item.input_audio_transcription.delta","item_id":"item_2","content_index":0,"delta":"Bonjour"}),
        json!({"type":"input_audio_buffer.committed","item_id":"item_2","previous_item_id":"item_1"}),
        json!({"type":"conversation.item.input_audio_transcription.completed","item_id":"item_2","content_index":0,"transcript":"Bonjour."}),
    ]).await;
    stream.begin_passage().unwrap();
    stream.append_pcm(&[0, 0]).await.unwrap();
    assert!(matches!(stream.next_event().await.unwrap(), SttEvent::Provisional(t) if t=="Hello"));
    stream.end_passage().await.unwrap();
    assert!(matches!(
        stream.next_event().await.unwrap(),
        SttEvent::Other
    ));
    assert!(
        matches!(stream.next_event().await.unwrap(), SttEvent::Final{text,model} if text=="Hello." && model.served.is_none() && model.acknowledged.as_deref()==Some("gpt-realtime-whisper"))
    );
    assert!(matches!(
        stream.next_event().await.unwrap(),
        SttEvent::Other
    ));
    assert_eq!(stream.duplicate_finals(), 1);
    stream.begin_passage().unwrap();
    stream.append_pcm(&[0, 0]).await.unwrap();
    assert!(matches!(stream.next_event().await.unwrap(), SttEvent::Provisional(t) if t=="Bonjour"));
    stream.end_passage().await.unwrap();
    stream.next_event().await.unwrap();
    assert!(
        matches!(stream.next_event().await.unwrap(), SttEvent::Final{text,..} if text=="Bonjour.")
    );
    let sent = sent.lock().unwrap();
    assert_eq!(
        sent.iter()
            .filter(|v| v["type"] == "session.update")
            .count(),
        1
    );
    assert_eq!(
        sent.iter()
            .filter(|v| v["type"] == "input_audio_buffer.commit")
            .count(),
        2
    );
    let input = &sent[0]["session"]["audio"]["input"];
    assert_eq!(input["format"]["rate"], 24000);
    assert_eq!(input["transcription"]["delay"], "minimal");
    assert!(input["turn_detection"].is_null());
    for key in ["language", "languages", "prompt"] {
        assert!(input["transcription"].get(key).is_none());
    }
}

#[tokio::test]
async fn contradictory_ack_and_early_final_fail_closed() {
    for events in [
        vec![
            json!({"type":"conversation.item.input_audio_transcription.completed","item_id":"x","content_index":0,"transcript":"early"}),
        ],
        vec![
            json!({"type":"conversation.item.input_audio_transcription.delta","item_id":"x","content_index":0,"delta":"a"}),
            json!({"type":"input_audio_buffer.committed","item_id":"wrong"}),
        ],
    ] {
        let (mut stream, _) = session(events).await;
        stream.begin_passage().unwrap();
        stream.append_pcm(&[0, 0]).await.unwrap();
        if matches!(stream.next_event().await, Ok(SttEvent::Provisional(_))) {
            stream.end_passage().await.unwrap();
            assert!(stream.next_event().await.is_err());
        } else {
            assert!(stream.append_pcm(&[0, 0]).await.is_err());
        }
    }
    let sent = Arc::new(Mutex::new(vec![]));
    let mut bad = updated();
    bad["session"]["audio"]["input"]["format"]["rate"] = json!(16000);
    assert!(
        SttSession::open(
            Socket {
                incoming: VecDeque::from([json!({"type":"session.created"}), bad]),
                sent
            },
            "gpt-realtime-whisper"
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn audio_and_text_bounds_reject_without_replay() {
    let (mut stream, _) = session(vec![]).await;
    stream.begin_passage().unwrap();
    assert!(stream.end_passage().await.is_err());
    let (mut stream, _) = session(vec![]).await;
    stream.begin_passage().unwrap();
    assert!(stream.append_pcm(&vec![0; 32002]).await.is_err());
    assert!(stream.append_pcm(&[0, 0]).await.is_err());
    let (mut stream, _) = session(vec![json!({"type":"conversation.item.input_audio_transcription.delta","item_id":"x","content_index":0,"delta":"x".repeat(65537)})]).await;
    stream.begin_passage().unwrap();
    assert!(stream.next_event().await.is_err());
}

#[tokio::test]
async fn exact_audio_capacity_and_cumulative_transcript_are_enforced() {
    let (mut stream, sent) = session(vec![]).await;
    stream.begin_passage().unwrap();
    for _ in 0..90 {
        stream.append_pcm(&vec![0; 32000]).await.unwrap();
    }
    assert!(stream.append_pcm(&[0, 0]).await.is_err());
    assert_eq!(
        sent.lock()
            .unwrap()
            .iter()
            .filter(|v| v["type"] == "input_audio_buffer.append")
            .count(),
        90
    );
    let delta = |text: String| json!({"type":"conversation.item.input_audio_transcription.delta","item_id":"x","content_index":0,"delta":text});
    let (mut stream, _) = session(vec![
        delta("a".repeat(32000)),
        delta("b".repeat(32000)),
        delta("c".repeat(2000)),
    ])
    .await;
    stream.begin_passage().unwrap();
    assert!(stream.next_event().await.is_ok());
    assert!(stream.next_event().await.is_ok());
    assert!(stream.next_event().await.is_err());
}

#[tokio::test]
async fn foreign_identity_and_remote_errors_never_expose_payloads() {
    for event in [
        json!({"type":"conversation.item.input_audio_transcription.delta","item_id":"x".repeat(129),"content_index":0,"delta":"private"}),
        json!({"type":"conversation.item.input_audio_transcription.delta","item_id":"x","content_index":-1,"delta":"private"}),
        json!({"type":"error","error":{"code":"invalid_api_key","message":"synthetic-private-key"}}),
        json!({"type":"conversation.item.input_audio_transcription.failed","error":{"message":"synthetic-private-key"}}),
        json!({"unexpected":"synthetic-private-key"}),
    ] {
        let (mut stream, _) = session(vec![event]).await;
        stream.begin_passage().unwrap();
        let error = stream.next_event().await.err().unwrap();
        assert!(!error.contains("private"));
        assert!(stream.append_pcm(&[0, 0]).await.is_err());
    }
}

#[test]
fn transport_host_and_authentication_are_fixed_and_sensitive() {
    use vocalnotes_lib::platform::stt_transport::request_for_model;
    let request = request_for_model("gpt-realtime-whisper", Some("synthetic-secret")).unwrap();
    assert_eq!(request.uri().host(), Some("api.openai.com"));
    assert_eq!(request.uri().query(), Some("intent=transcription"));
    assert!(request.headers()["Authorization"].is_sensitive());
    assert!(!request.uri().to_string().contains("synthetic-secret"));
    assert!(request_for_model("x&key=y", None).is_err());
}

#[test]
fn bilingual_acceptance_corpus_has_fixed_counts_and_critical_oracles() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../tests/fixtures/openai-fidelity.json")).unwrap();
    let passages = corpus["passages"].as_array().unwrap();
    assert_eq!(passages.len(), 20);
    for (n, p) in passages.iter().enumerate() {
        assert_eq!(p["language"], if n % 2 == 0 { "fr" } else { "en" });
        let text = p["text"].as_str().unwrap();
        assert!(text.split_whitespace().count() >= 15);
        for critical in p["critical"].as_array().unwrap() {
            assert!(text.contains(critical.as_str().unwrap()));
        }
    }
    let drafting = corpus["drafting"].as_array().unwrap();
    assert_eq!(drafting.len(), 12);
    for language in ["fr", "en"] {
        let cases: Vec<_> = drafting
            .iter()
            .filter(|v| v["language"] == language)
            .collect();
        assert_eq!(cases.len(), 6);
        for kind in ["draft", "correction", "intention", "ambiguity"] {
            assert!(cases.iter().any(|v| v["kind"] == kind));
        }
    }
    let interruptions = corpus["interruptions"].as_array().unwrap();
    assert_eq!(interruptions.len(), 10);
    for event in ["pause", "stop", "lock", "sleep", "close"] {
        assert_eq!(
            interruptions.iter().filter(|v| v["event"] == event).count(),
            2
        );
    }
    assert_eq!(corpus["thresholds"]["maxWerPerLanguage"], 0.1);
    assert_eq!(corpus["thresholds"]["firstRenderedPartialMs"], 2000);
    assert_eq!(corpus["thresholds"]["renderedNoteAfterSpeechEndMs"], 5000);
    assert_eq!(corpus["thresholds"]["minimumLatencySuccesses"], 18);
}

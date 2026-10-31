use serde_json::json;
use vocalnotes_lib::platform::mistral::*;
use vocalnotes_lib::ports::interpreter::Interruption;

#[test]
fn done_is_the_only_final_text_and_is_consumed_once() {
    let mut stream = Finality::default();
    assert!(matches!(
        stream
            .accept(&json!({"type":"transcription.text.delta","text":"dix"}))
            .unwrap(),
        SttEvent::Provisional(_)
    ));
    assert!(matches!(
        stream
            .accept(&json!({"type":"transcription.segment","text":"dix","start":0,"end":1}))
            .unwrap(),
        SttEvent::Other
    ));
    match stream
        .accept(&json!({"type":"transcription.done","text":"douze","model":"test-model"}))
        .unwrap()
    {
        SttEvent::Final { text, model } => {
            assert_eq!(text, "douze");
            assert_eq!(model, "test-model");
        }
        _ => panic!("missing done"),
    }
    assert!(
        stream
            .accept(&json!({"type":"transcription.done","text":"douze","model":"test-model"}))
            .is_err()
    );
}

#[test]
fn strict_schema_has_wire_schema_and_no_arbitrary_tools() {
    let format = response_format();
    assert_eq!(format["type"], "json_schema");
    assert_eq!(format["json_schema"]["strict"], true);
    assert_eq!(
        format["json_schema"]["schema"]["additionalProperties"],
        false
    );
    assert!(format["json_schema"].get("schema_definition").is_none());
    assert!(decode_intervention(r#"{"status":"resolved","message":"","actions":[{"type":"session_action","command":{"action":"undo"}}]}"#).is_err());
    assert!(decode_intervention(r#"{"status":"clarify","message":"préciser","actions":[{"type":"drafting_intent","text":"mail"}]}"#).is_err());
}

#[test]
fn provider_errors_never_return_payloads() {
    let mut stream = Finality::default();
    let message = stream
        .accept(&json!({"type":"error","message":"secret synthetic content"}))
        .err()
        .unwrap();
    assert!(!message.contains("secret"));
    assert!(decode_intervention("not JSON with private content").is_err());
}

#[test]
fn interruption_channel_accepts_only_interruptions_not_editing() {
    assert_eq!(
        decode_interrupt(r#"{"interrupt":"pause"}"#).unwrap(),
        Interruption::Pause
    );
    assert!(decode_interrupt(r#"{"interrupt":"pause","edit":"x"}"#).is_err());
    assert!(decode_interrupt(r#"{"interrupt":"replace"}"#).is_err());
}

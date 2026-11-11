use serde_json::{Value, json};
use vocalnotes_lib::adapters::mistral_stt::Finality;
use vocalnotes_lib::ports::{
    interpreter::{
        Interpreter, Interruption, decode_interrupt, decode_intervention, response_format,
    },
    stt::{SttEvent, SttInput, SttOutput},
};

fn strict_objects(value: &Value) {
    if value["type"] == "object" {
        assert_eq!(value["additionalProperties"], false);
        let properties = value["properties"].as_object().unwrap();
        let required = value["required"].as_array().unwrap();
        assert_eq!(properties.len(), required.len());
        for key in properties.keys() {
            assert!(required.contains(&json!(key)));
        }
    }
    match value {
        Value::Object(map) => map.values().for_each(strict_objects),
        Value::Array(list) => list.iter().for_each(strict_objects),
        _ => {}
    }
}

#[test]
fn every_wire_object_is_strict_and_has_required_fields() {
    let format = response_format();
    assert_eq!(format["json_schema"]["strict"], true);
    strict_objects(&format["json_schema"]["schema"]);
}

#[test]
fn malformed_or_ambiguous_envelopes_and_terminal_order_never_become_actions() {
    for text in [
        r#"{"status":"clarify","message":"?","actions":[{"type":"drafting_intent","text":"mail"}]}"#,
        r#"{"status":"resolved","message":"?","actions":[]}"#,
        r#"{"status":"resolved","message":"","actions":[{"type":"session_action","command":{"action":"undo"}}]}"#,
        r#"{"status":"resolved","message":"","actions":[{"type":"edit","patch":{"from":2,"to":1,"expected":"","text":""}}]}"#,
        r#"{"status":"resolved","message":"","actions":[{"type":"session_action","command":{"action":"pause"}},{"type":"drafting_intent","text":"mail"}]}"#,
        r#"{"status":"resolved","message":"","actions":[],"shell":"rm"}"#,
    ] {
        assert!(decode_intervention(text).is_err(), "{text}");
    }
    assert!(
        decode_intervention(r#"{"status":"clarify","message":"Préciser","actions":[]}"#).is_ok()
    );
    assert!(decode_intervention(r#"{"status":"resolved","message":"","actions":[]}"#).is_ok());
}

#[test]
fn priority_can_only_interrupt_and_has_bounded_expurgated_errors() {
    assert_eq!(
        decode_interrupt(r#"{"interrupt":"manual"}"#).unwrap(),
        Interruption::Manual
    );
    assert!(decode_interrupt(r#"{"interrupt":"stop","actions":[]}"#).is_err());
    let oversized = "synthetic-private".repeat(5000);
    let error = decode_interrupt(&oversized).unwrap_err();
    assert!(!error.contains("synthetic"));
}

#[test]
fn final_stt_requires_nonempty_served_model_and_never_promotes_segments() {
    let mut finality = Finality::default();
    assert!(
        finality
            .accept(&json!({"type":"transcription.done","text":"", "model":""}))
            .is_err()
    );
    assert!(matches!(
        finality
            .accept(&json!({"type":"transcription.segment","text":"x"}))
            .unwrap(),
        SttEvent::Other
    ));
    assert!(matches!(
        finality
            .accept(&json!({"type":"transcription.done","text":"corrigé","model":"test"}))
            .unwrap(),
        SttEvent::Final { .. }
    ));
    assert!(
        finality
            .accept(&json!({"type":"transcription.text.delta","text":"tardif"}))
            .is_err()
    );
}

#[test]
fn native_adapters_implement_separate_provider_independent_ports() {
    fn input<T: SttInput>() {}
    fn output<T: SttOutput>() {}
    fn interpreter<T: Interpreter>() {}
    input::<vocalnotes_lib::platform::mistral::SttWriter>();
    output::<vocalnotes_lib::platform::mistral::SttReader>();
    interpreter::<vocalnotes_lib::platform::mistral::MistralInterpreter>();
    input::<vocalnotes_lib::adapters::openai_stt::OpenAiStt>();
    output::<vocalnotes_lib::adapters::openai_stt::OpenAiStt>();
    interpreter::<vocalnotes_lib::adapters::openai_interpreter::OpenAiInterpreter>();
    // Compile-time contract only: no constructor that connects, no provider calls.
}

#[test]
fn stt_exact_payload_and_model_boundaries_are_enforced_before_finalization() {
    for (size, valid) in [(65_536, true), (65_537, false)] {
        let mut finality = Finality::default();
        assert_eq!(
            finality
                .accept(&json!({
                    "type":"transcription.text.delta","text":"x".repeat(size)
                }))
                .is_ok(),
            valid
        );
    }
    for (size, valid) in [(256, true), (257, false)] {
        let mut finality = Finality::default();
        assert_eq!(
            finality
                .accept(&json!({
                    "type":"transcription.done","text":"été","model":"m".repeat(size)
                }))
                .is_ok(),
            valid
        );
        if !valid {
            assert!(
                finality
                    .accept(&json!({
                        "type":"transcription.done","text":"été","model":"m"
                    }))
                    .is_ok()
            );
        }
    }
    let mut finality = Finality::default();
    assert!(
        finality
            .accept(&json!({
                "type":"transcription.done","text":false,"model":"m"
            }))
            .is_err()
    );
    assert!(
        finality
            .accept(&json!({
                "type":"transcription.done","text":"été","model":"m"
            }))
            .is_ok()
    );
}

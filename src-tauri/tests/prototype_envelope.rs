use vocalnotes_lib::platform::probe::{Intervention, Probe};
use vocalnotes_lib::ports::interpreter::Status;

#[test]
fn document_codec_keeps_exact_source_and_checks_the_serialized_size() {
    use vocalnotes_lib::{
        domain::intention::DraftingIntention,
        storage::note_document::{MAX_DOCUMENT_BYTES, decode, encode_checked},
    };
    let intention = DraftingIntention::new(
        "but".into(),
        "forme".into(),
        "ton".into(),
        "--> & <\r\nété".into(),
    )
    .unwrap();
    for source in [
        "",
        "# Été\r\n\r\ntexte  \n",
        "e\u{301} 😀\r\n",
        "<!-- vocalnotes:intention:v2\nbody",
    ] {
        let document = encode_checked(source, &intention).unwrap();
        let (body, decoded) = decode(&document).unwrap();
        assert_eq!(body.as_bytes(), source.as_bytes());
        assert!(decoded == intention);
        assert_eq!(
            document.matches("-->").count(),
            if source.contains("-->") { 2 } else { 1 }
        );
    }
    let body = "x".repeat(MAX_DOCUMENT_BYTES);
    assert!(encode_checked(&body, &DraftingIntention::default()).is_ok());
    assert!(decode(&body).is_ok());
    assert!(decode(&format!("{body}x")).is_err());
    let overhead = encode_checked("", &intention).unwrap().len();
    assert!(encode_checked(&"x".repeat(MAX_DOCUMENT_BYTES - overhead), &intention).is_ok());
    assert!(encode_checked(&"x".repeat(MAX_DOCUMENT_BYTES - overhead + 1), &intention).is_err());
    let expanded = DraftingIntention::new(
        String::new(),
        String::new(),
        String::new(),
        ">".repeat(4096),
    )
    .unwrap();
    assert!(encode_checked(&"x".repeat(MAX_DOCUMENT_BYTES - 4096), &expanded).is_err());
}

#[test]
fn document_codec_enforces_metadata_limits_and_rejects_unknown_fields() {
    use vocalnotes_lib::domain::intention::DraftingIntention;
    use vocalnotes_lib::storage::note_document::{decode, encode_checked};
    let json = r#"{"purpose":"","form":"","tone":"","context":""}"#;
    let document = |length| {
        format!(
            "<!-- vocalnotes:intention:v1\n{}{json}\n-->\n\nbody",
            " ".repeat(length - json.len())
        )
    };
    assert!(decode(&document(32_768)).is_ok());
    assert!(decode(&document(32_769)).is_err());
    let invalid =
        document(json.len()).replace("\"context\":\"\"", "\"context\":\"\",\"extra\":\"ignored\"");
    assert!(decode(&invalid).is_err());
    let oversized: DraftingIntention = serde_json::from_value(serde_json::json!({
        "purpose":"", "form":"", "tone":"", "context":"x".repeat(4097)
    }))
    .unwrap();
    assert!(encode_checked("body", &oversized).is_err());
}

fn no_action() -> Intervention {
    Intervention {
        status: Status::Resolved,
        message: "".into(),
        actions: vec![],
    }
}

#[test]
fn native_request_operation_sequence_and_passage_are_consumed_once() {
    let mut core = Probe::new("synthétique".into());
    core.start_voice(false).unwrap();
    // The preceding empty STT final may be discarded without reusing its passage identity.
    let first = core.voice_ticket_for(1, 2).unwrap();
    core.commit_voice(&first, no_action()).unwrap();
    assert!(core.commit_voice(&first, no_action()).is_err());
    assert!(core.voice_ticket_for(2, 2).is_err());
    assert!(core.voice_ticket_for(3, 3).is_err());
    let second = core.voice_ticket_for(2, 3).unwrap();
    core.commit_voice(&second, no_action()).unwrap();
    let session_id = core.snapshot().session_id;
    core.pause();
    core.start_voice(true).unwrap();
    assert_eq!(core.snapshot().session_id, session_id);
    assert!(core.commit_voice(&second, no_action()).is_err());
    core.stop("manuel");
    core.start_voice(false).unwrap();
    assert!(core.snapshot().session_id > session_id);
}

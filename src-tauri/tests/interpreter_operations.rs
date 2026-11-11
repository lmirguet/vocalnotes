use serde_json::{Value, json};
use vocalnotes_lib::{
    domain::note::Note,
    ports::interpreter::{
        InterpretationContext, decode_intervention, response_format, response_format_for_context,
    },
};

fn replace(id: &str, expected: &str, text: &str) -> Value {
    json!({"type":"edit","operation":{"kind":"replace",
        "target":{"block_id":id,"expected":expected},"text":text}})
}

fn anchor(id: &str, expected: &str, edge: &str) -> Value {
    json!({"block_id":id,"expected":expected,"edge":edge})
}

fn session(action: &str) -> Value {
    json!({"type":"session_action","command":{"action":action}})
}

fn reply(actions: Vec<Value>) -> String {
    json!({"status":"resolved","message":"","actions":actions}).to_string()
}

fn apply(note: &mut Note, sequence: u64, actions: Vec<Value>) -> Result<(), &'static str> {
    let ticket = note.voice_ticket(sequence)?;
    note.commit_voice(&ticket, decode_intervention(&reply(actions))?)
}

#[test]
fn empty_context_only_offers_insertion_on_the_exact_native_anchor() {
    let mut note = Note::new(String::new());
    let context = InterpretationContext {
        blocks: note.candidates(),
        save_available: false,
    };
    let format = response_format_for_context(&context);
    let operations = format["json_schema"]["schema"]["properties"]["actions"]["items"]["anyOf"][0]
        ["properties"]["operation"]["anyOf"]
        .as_array()
        .unwrap();
    assert_eq!(
        operations.len(),
        1,
        "No replace/delete/move target exists in an empty note"
    );
    let insert = &operations[0];
    assert_eq!(insert["properties"]["kind"]["enum"], json!(["insert"]));
    let anchor_properties = &insert["properties"]["anchor"]["properties"];
    let id = anchor_properties["block_id"]["enum"][0].as_str().unwrap();
    let expected = anchor_properties["expected"]["enum"][0].as_str().unwrap();
    assert_eq!(id, context.blocks[0].block_id);
    assert_eq!(expected, "");
    // The provider-visible native anchor must actually be executable in the core.
    note.start_voice(false).unwrap();
    apply(
        &mut note,
        1,
        vec![json!({"type":"edit", "operation": {
            "kind":"insert", "anchor":anchor(id, expected, "before"), "text":"Trois cahiers."
        }})],
    )
    .unwrap();
    assert_eq!(note.snapshot().source, "Trois cahiers.");
    assert_eq!(note.snapshot().voice_applied, 1);
}

#[test]
fn nonempty_context_keeps_all_operations_and_empty_opaque_only_context_is_not_an_empty_note() {
    for source in ["bonjour", "```rs\nopaque\n```"] {
        let note = Note::new(source.into());
        let context = InterpretationContext {
            blocks: note.candidates(),
            save_available: false,
        };
        assert_eq!(response_format_for_context(&context), response_format());
    }
}

#[test]
fn voice_diagnostics_distinguish_outcomes_and_survive_stop_without_content() {
    let mut note = Note::new("bonjour".into());
    note.start_voice(false).unwrap();
    let ticket = note.voice_ticket(1).unwrap();
    note.commit_voice(
        &ticket,
        decode_intervention(
            r#"{"status":"clarify","message":"Précision synthétique privée","actions":[]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    apply(&mut note, 2, vec![]).unwrap();
    assert!(apply(&mut note, 3, vec![replace("b404", "bonjour", "soir")]).is_err());
    apply(&mut note, 4, vec![replace("b0", "bonjour", "bonsoir")]).unwrap();
    note.stop("Arrêt explicite");
    let snapshot = serde_json::to_value(note.snapshot()).unwrap();
    assert_eq!(snapshot["voiceApplied"], 1);
    assert_eq!(snapshot["voiceClarifications"], 1);
    assert_eq!(snapshot["voiceRejected"], 1);
    assert_eq!(snapshot["voiceNoChange"], 1);
    assert_eq!(
        snapshot["lastVoiceResult"],
        "Modification de la note appliquée."
    );
    assert_eq!(snapshot["source"], "bonsoir");
    assert!(
        !snapshot
            .to_string()
            .contains("Précision synthétique privée")
    );
    // Neither a stale response nor a duplicate is counted as new work.
    assert!(
        note.commit_voice(&ticket, decode_intervention(&reply(vec![])).unwrap())
            .is_err()
    );
    assert_eq!(
        serde_json::to_value(note.snapshot()).unwrap()["voiceNoChange"],
        1
    );
    note.start_voice(false).unwrap();
    let current = note.voice_ticket(1).unwrap();
    note.commit_voice(&current, decode_intervention(&reply(vec![])).unwrap())
        .unwrap();
    assert!(
        note.commit_voice(&current, decode_intervention(&reply(vec![])).unwrap())
            .is_err()
    );
    assert_eq!(
        serde_json::to_value(note.snapshot()).unwrap()["voiceNoChange"],
        2
    );
    assert_eq!(
        serde_json::to_value(note.snapshot()).unwrap()["voiceRejected"],
        1
    );
}

#[test]
fn voice_diagnostics_keep_clarification_or_native_refusal_after_explicit_stop() {
    let mut note = Note::new("bonjour".into());
    note.start_voice(false).unwrap();
    let ticket = note.voice_ticket(1).unwrap();
    note.commit_voice(
        &ticket,
        decode_intervention(r#"{"status":"clarify","message":"Quelle précision ?","actions":[]}"#)
            .unwrap(),
    )
    .unwrap();
    note.stop("Arrêt explicite");
    assert_eq!(
        serde_json::to_value(note.snapshot()).unwrap()["lastVoiceResult"],
        "Précision demandée ; aucune action appliquée."
    );
    note.start_voice(false).unwrap();
    assert!(apply(&mut note, 1, vec![session("new")]).is_err());
    let refusal = note.snapshot().message;
    note.stop("Arrêt explicite");
    assert_eq!(
        serde_json::to_value(note.snapshot()).unwrap()["lastVoiceResult"],
        refusal
    );
    assert_eq!(note.snapshot().source, "bonjour");
}

#[test]
fn provider_offsets_and_envelope_identities_are_never_trusted() {
    for action in [
        json!({"type":"edit","patch":{"from":0,"to":1,"expected":"a","text":"b"}}),
        json!({"type":"edit","operation":{"kind":"replace",
            "target":{"block_id":"b0","expected":"a","from":0},"text":"b"}}),
        json!({"type":"edit","operation":{"kind":"replace",
            "target":{"block_id":"b0","expected":"a"},"text":"b","revision":0}}),
        session("undo"),
        session("redo"),
        session("shell"),
    ] {
        assert!(decode_intervention(&reply(vec![action])).is_err());
    }
    let mut value: Value = serde_json::from_str(&reply(vec![])).unwrap();
    value["generation"] = json!(1);
    assert!(decode_intervention(&value.to_string()).is_err());
}

#[test]
fn all_four_operations_decode_and_only_required_variant_fields_are_allowed() {
    for action in [
        replace("b0", "été", "hiver"),
        json!({"type":"edit","operation":{"kind":"delete",
            "target":{"block_id":"b0","expected":"été"}}}),
        json!({"type":"edit","operation":{"kind":"insert",
            "anchor":anchor("b0","été","after"),"text":"\r\n\r\nsuite"}}),
        json!({"type":"edit","operation":{"kind":"move",
            "target":{"block_id":"b0","expected":"été"},
            "anchor":anchor("b1","fin","after")}}),
    ] {
        assert!(decode_intervention(&reply(vec![action])).is_ok());
    }
    for operation in [
        json!({"kind":"delete","target":{"block_id":"b0","expected":"a"},"text":"b"}),
        json!({"kind":"insert","anchor":anchor("b0","a","inside"),"text":"b"}),
        json!({"kind":"replace","target":{"block_id":"","expected":"a"},"text":"b"}),
        json!({"kind":"replace","target":{"block_id":"b0","expected":""},"text":"b"}),
        json!({"kind":"replace","target":{"block_id":"b0","expected":"a"}}),
    ] {
        assert!(
            decode_intervention(&reply(vec![json!({"type":"edit","operation":operation})]))
                .is_err()
        );
    }
}

#[test]
fn whole_preflight_rejects_missing_multiple_or_opaque_targets_without_effect() {
    for (id, expected) in [("b404", "été"), ("b0", "été"), ("b1", "opaque")] {
        let mut note = Note::new("été été\r\n\r\n```rs\r\nopaque\r\n```\r\n".into());
        note.start_voice(false).unwrap();
        let before = note.snapshot();
        assert!(
            apply(
                &mut note,
                1,
                vec![
                    json!({"type":"drafting_intent","text":"courriel"}),
                    replace(id, expected, "hiver"),
                    session("pause"),
                ]
            )
            .is_err()
        );
        assert_eq!(note.snapshot().source, before.source);
        assert_eq!(note.snapshot().revision, before.revision);
        assert_eq!(note.snapshot().mode, "voice");
        assert_eq!(note.intention(), "");
        apply(&mut note, 2, vec![]).unwrap();
        assert_eq!(note.snapshot().source, before.source); // No deferred replay.
    }
}

#[test]
fn ordered_edits_resolve_on_working_source_not_provider_offsets() {
    let mut note = Note::new("été\r\n\r\nfin\r\n\r\n```rs\r\nopaque\r\n```\r\n".into());
    note.start_voice(false).unwrap();
    apply(
        &mut note,
        1,
        vec![
            json!({"type":"drafting_intent","text":"sobre"}),
            replace("b0", "été", "hiver"),
            replace("b0", "hiver", "printemps"),
            replace("b1", "fin", "conclusion"),
        ],
    )
    .unwrap();
    assert_eq!(
        note.snapshot().source,
        "printemps\r\n\r\nconclusion\r\n\r\n```rs\r\nopaque\r\n```\r\n"
    );
    assert_eq!(note.snapshot().revision, 1);
    assert_eq!(note.intention(), "sobre");
}

#[test]
fn later_failed_precondition_cancels_earlier_edits_and_intention() {
    let mut note = Note::new("été\r\n\r\nfin".into());
    note.start_voice(false).unwrap();
    assert!(
        apply(
            &mut note,
            1,
            vec![replace("b0", "été", "hiver"), replace("b0", "été", "AUTRE"),]
        )
        .is_err()
    );
    assert_eq!(note.snapshot().source, "été\r\n\r\nfin");
    assert_eq!(note.snapshot().revision, 0);
}

#[test]
fn insertion_delete_and_move_preserve_untargeted_utf8_crlf_and_opaque_bytes() {
    let mut note = Note::new("été\r\n\r\nfin\r\n\r\n```rs\r\nopaque\r\n```\r\n".into());
    note.start_voice(false).unwrap();
    apply(
        &mut note,
        1,
        vec![json!({"type":"edit","operation":{"kind":"insert",
        "anchor":anchor("b0","été","after"),"text":" : résumé"}})],
    )
    .unwrap();
    apply(
        &mut note,
        2,
        vec![json!({"type":"edit","operation":{"kind":"delete",
        "target":{"block_id":"b0","expected":" : résumé"}}})],
    )
    .unwrap();
    apply(
        &mut note,
        3,
        vec![json!({"type":"edit","operation":{"kind":"move",
        "target":{"block_id":"b0","expected":"été"},
        "anchor":anchor("b1","fin","after")}})],
    )
    .unwrap();
    assert_eq!(
        note.snapshot().source,
        "\r\n\r\nfinété\r\n\r\n```rs\r\nopaque\r\n```\r\n"
    );
}

#[test]
fn overlapping_or_self_move_and_structurally_invalid_tail_are_zero_effect() {
    for actions in [
        vec![json!({"type":"edit","operation":{"kind":"move",
            "target":{"block_id":"b0","expected":"été"},
            "anchor":anchor("b0","été","after")}})],
        vec![
            replace("b0", "été", "un\n\ndeux"),
            replace("b0", "un", "trois"),
        ],
    ] {
        let mut note = Note::new("été\r\n\r\nfin".into());
        note.start_voice(false).unwrap();
        assert!(apply(&mut note, 1, actions).is_err());
        assert_eq!(note.snapshot().source, "été\r\n\r\nfin");
        assert_eq!(note.snapshot().revision, 0);
    }
}

#[test]
fn empty_note_has_native_anchor_but_no_empty_replacement_target() {
    let mut note = Note::new(String::new());
    note.start_voice(false).unwrap();
    apply(
        &mut note,
        1,
        vec![json!({"type":"edit","operation":{"kind":"insert",
        "anchor":anchor("b0","","before"),"text":"été"}})],
    )
    .unwrap();
    assert_eq!(note.snapshot().source, "été");
}

#[test]
fn terminal_barriers_are_typed_and_reject_every_tail_before_any_effect() {
    for command in [
        "pause",
        "stop",
        "manual",
        "save",
        "new",
        "preferences",
        "close",
    ] {
        assert!(
            decode_intervention(&reply(vec![session(command), replace("b0", "a", "b")])).is_err()
        );
    }
    for command in ["new", "preferences", "close"] {
        let mut note = Note::new("a".into());
        note.start_voice(false).unwrap();
        assert!(
            apply(
                &mut note,
                1,
                vec![replace("b0", "a", "b"), session(command)]
            )
            .is_err()
        );
        assert_eq!(note.snapshot().source, "a");
        assert_eq!(note.snapshot().mode, "voice");
    }
    for command in ["open", "save_as"] {
        assert!(
            decode_intervention(&reply(vec![json!({"type":"session_action",
            "command":{"action":command,"path":"relative.md"}})]))
            .is_err()
        );
        assert!(
            decode_intervention(&reply(vec![json!({"type":"session_action",
            "command":{"action":command,"path":"/synthetic/note.md"}})]))
            .is_ok()
        );
    }
}

#[test]
fn valid_manual_interrupt_invalidates_and_never_restores_voice_history() {
    let mut note = Note::new("a".into());
    note.start_voice(false).unwrap();
    let old = note.voice_ticket(1).unwrap();
    note.commit_voice(
        &old,
        decode_intervention(&reply(vec![replace("b0", "a", "b"), session("manual")])).unwrap(),
    )
    .unwrap();
    assert_eq!(note.snapshot().source, "b");
    assert_eq!(note.snapshot().mode, "manual");
    note.start_voice(false).unwrap();
    assert!(
        note.commit_voice(&old, decode_intervention(&reply(vec![])).unwrap())
            .is_err()
    );
}

#[test]
fn local_limits_match_schema_and_are_byte_bounded_before_effects() {
    let format = response_format();
    assert_eq!(
        format["json_schema"]["schema"]["properties"]["message"]["maxLength"],
        4096
    );
    for actions in [
        vec![replace(&"b".repeat(129), "a", "b")],
        vec![json!({"type":"drafting_intent","text":"é".repeat(2049)})],
        (0..33).map(|_| replace("b0", "a", "b")).collect(),
    ] {
        assert!(decode_intervention(&reply(actions)).is_err());
    }
}

#[test]
fn exact_envelope_thresholds_and_nested_unknown_fields_are_checked() {
    for (size, valid) in [(4096, true), (4097, false)] {
        let clarification = json!({"status":"clarify","message":"x".repeat(size),"actions":[]});
        assert_eq!(
            decode_intervention(&clarification.to_string()).is_ok(),
            valid
        );
        assert_eq!(
            decode_intervention(&reply(vec![
                json!({"type":"drafting_intent","text":"x".repeat(size)})
            ]))
            .is_ok(),
            valid
        );
    }
    for (size, valid) in [(32, true), (33, false)] {
        assert_eq!(
            decode_intervention(&reply((0..size).map(|_| replace("b0", "a", "b")).collect()))
                .is_ok(),
            valid
        );
    }
    for (size, valid) in [(128, true), (129, false)] {
        assert_eq!(
            decode_intervention(&reply(vec![replace(&"b".repeat(size), "a", "b")])).is_ok(),
            valid
        );
    }
    for action in [
        json!({"type":"session_action","command":{"action":"save","path":"/synthetic/note.md"}}),
        json!({"type":"session_action","command":{"action":"open","path":"/synthetic/note.md","shell":"x"}}),
        json!({"type":"session_action","command":{"action":"open"}}),
    ] {
        assert!(decode_intervention(&reply(vec![action])).is_err());
    }
    for command in [
        "pause",
        "stop",
        "manual",
        "save",
        "new",
        "preferences",
        "close",
    ] {
        assert!(
            decode_intervention(&reply(vec![json!({
                "type":"session_action","command":{"action":command,"extra":"forbidden"}
            })]))
            .is_err()
        );
    }
    let private = format!("synthetic-private{}", "x".repeat(65_536));
    assert_eq!(
        decode_intervention(&private).err().unwrap(),
        "Réponse LLM trop volumineuse."
    );
}

#[test]
fn candidate_context_is_native_and_cannot_offer_opaque_blocks_or_offsets() {
    use vocalnotes_lib::ports::interpreter::InterpretationContext;
    let note = Note::new("été\r\n\r\n```rs\r\nopaque\r\n```\r\n\r\nfin".into());
    let context = serde_json::to_value(InterpretationContext {
        blocks: note.candidates(),
        save_available: note.can_save(),
    })
    .unwrap();
    assert_eq!(
        context,
        json!({"save_available":false,"blocks":[
            {"block_id":"b0","source":"été"},
            {"block_id":"b2","source":"fin"}
        ]})
    );
    // b1 cannot silently resolve to b2 merely because opaque candidates are filtered.
    let mut note = note;
    note.start_voice(false).unwrap();
    assert!(apply(&mut note, 1, vec![replace("b1", "fin", "perte")]).is_err());
    assert!(apply(&mut note, 2, vec![replace("b2", "fin", "suite")]).is_ok());
}

#[test]
fn overlapping_text_matches_and_stale_anchors_are_ambiguous_without_replay() {
    let mut note = Note::new("aaa\r\n\r\nfin".into());
    note.start_voice(false).unwrap();
    assert!(apply(&mut note, 1, vec![replace("b0", "aa", "b")]).is_err());
    assert!(
        apply(
            &mut note,
            2,
            vec![
                replace("b0", "aaa", "été"),
                json!({"type":"edit","operation":{"kind":"insert",
            "anchor":anchor("b0","aaa","after"),"text":"b"}})
            ]
        )
        .is_err()
    );
    assert_eq!(note.snapshot().source, "aaa\r\n\r\nfin");
    apply(&mut note, 3, vec![]).unwrap();
    assert_eq!(note.snapshot().revision, 0);
}

#[test]
fn typed_file_commands_not_yet_integrated_never_do_io_or_change_the_note() {
    for command in ["open", "save_as"] {
        let mut note = Note::new("a".into());
        note.start_voice(false).unwrap();
        let before = note.snapshot();
        assert!(apply(&mut note, 1, vec![
            replace("b0","a","b"),
            json!({"type":"session_action","command":{"action":command,"path":"/synthetic/note.md"}})
        ]).is_err());
        assert_eq!(note.snapshot().source, before.source);
        assert_eq!(note.snapshot().generation, before.generation);
        assert_eq!(note.snapshot().note_id, before.note_id);
        assert_eq!(note.snapshot().mode, "voice");
    }
}

#[test]
fn inconsistent_provider_status_is_rejected_with_service_error() {
    for value in [
        json!({"status":"resolved","message":"Résumé fourni par le service","actions":[]}),
        json!({"status":"clarify","message":"","actions":[]}),
        json!({"status":"clarify","message":"Quelle cible ?","actions":[replace("b0","a","b")]}),
    ] {
        assert_eq!(
            decode_intervention(&value.to_string()).err(),
            Some("Réponse du service incohérente avec le contrat : aucun effet.")
        );
    }
    assert!(decode_intervention(&reply(vec![])).is_ok());
    assert!(
        decode_intervention(r#"{"status":"clarify","message":"Quelle cible ?","actions":[]}"#)
            .is_ok()
    );
}

#[test]
fn wire_schema_is_a_closed_root_object_and_local_guards_separate_outcomes() {
    for source in ["", "Première idée."] {
        let note = Note::new(source.into());
        let format = response_format_for_context(&InterpretationContext {
            blocks: note.candidates(),
            save_available: false,
        });
        let schema = &format["json_schema"]["schema"];
        assert_eq!(schema["type"], "object");
        assert!(schema.get("anyOf").is_none());
        assert_eq!(schema["required"], json!(["status", "message", "actions"]));
        assert_eq!(
            schema["properties"]["status"]["enum"],
            json!(["resolved", "clarify"])
        );
        assert_eq!(schema["properties"]["actions"]["maxItems"], 32);
        assert_eq!(schema["additionalProperties"], false);
        // Cross-field consistency remains enforced by decode_intervention locally.
        println!("ENVELOPE_SCHEMA {}", schema);
    }
}

#[test]
fn delete_then_replace_missing_target_keeps_cahiers_and_allows_later_insert() {
    let original = "Acheter des cahiers.";
    let mut note = Note::new(original.into());
    note.start_voice(false).unwrap();
    assert!(
        apply(
            &mut note,
            1,
            vec![
                json!({"type":"edit","operation":{"kind":"delete",
            "target":{"block_id":"b0","expected":"cahiers"}}}),
                replace("b0", "cahiers", "cahiers et des stylos"),
            ]
        )
        .is_err()
    );
    assert_eq!(note.snapshot().source, original);
    assert_eq!(note.snapshot().revision, 0);
    apply(
        &mut note,
        2,
        vec![json!({"type":"edit","operation":{
        "kind":"insert","anchor":anchor("b0", original, "after"),
        "text":"\n\nPrévoir aussi des stylos."}})],
    )
    .unwrap();
    assert_eq!(
        note.snapshot().source,
        "Acheter des cahiers.\n\nPrévoir aussi des stylos."
    );
    assert_eq!(note.snapshot().voice_rejected, 1);
    assert_eq!(note.snapshot().voice_applied, 1);
}

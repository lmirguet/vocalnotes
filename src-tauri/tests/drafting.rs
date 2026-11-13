//! Synthetic reference scenarios, not a measurement of provider intelligence.
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use vocalnotes_lib::{domain::note::Note, ports::interpreter::decode_intervention};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Corpus {
    version: u32,
    synthetic: bool,
    scope: String,
    cases: Vec<Case>,
    recognition_pairs: Vec<RecognitionPair>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    use_case: String,
    tags: Vec<String>,
    initial_note: String,
    steps: Vec<Step>,
    required: Vec<String>,
    forbidden: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    spoken: String,
    transcript: String,
    response: Value,
    expected_note: String,
    expected_intention: String,
    decision: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecognitionPair {
    id: String,
    spoken: String,
    transcript: String,
    expected_note: String,
    expected_error: String,
}
fn corpus() -> Corpus {
    serde_json::from_str(include_str!("../../tests/fixtures/fidelity.json")).unwrap()
}
// Exact reference plus explicit semantic review markers. Alternative valid wording
// requires human review, not automatic acceptance by substring matching.
fn conforms(case: &Case, actual: &str) -> bool {
    actual == case.steps.last().unwrap().expected_note
        && case.required.iter().all(|s| actual.contains(s))
        && case.forbidden.iter().all(|s| !actual.contains(s))
}
#[test]
fn reference_decisions_preserve_facts_nuances_and_existing_content() {
    for case in corpus().cases {
        let mut note = Note::new(case.initial_note.clone());
        note.start_voice(false).unwrap();
        for (i, step) in case.steps.iter().enumerate() {
            assert_eq!(
                step.spoken, step.transcript,
                "{}: clean STT fixture",
                case.id
            );
            let before = note.snapshot().source;
            let ticket = note.voice_ticket(i as u64 + 1).unwrap();
            note.commit_voice(
                &ticket,
                decode_intervention(&step.response.to_string()).unwrap(),
            )
            .unwrap_or_else(|e| panic!("{} step {i}: {e}", case.id));
            assert_eq!(
                note.snapshot().source,
                step.expected_note,
                "{} step {i}",
                case.id
            );
            assert_eq!(note.intention(), step.expected_intention, "{}", case.id);
            if step.decision == "clarify" {
                assert_eq!(
                    note.snapshot().source,
                    before,
                    "{}: no partial edit",
                    case.id
                );
                assert_eq!(note.snapshot().voice_clarifications, 1);
            }
        }
        assert!(conforms(&case, &note.snapshot().source), "{}", case.id);
    }
}
#[test]
fn reference_oracle_rejects_added_facts_omissions_and_lost_nuance() {
    for case in corpus().cases {
        let expected = &case.steps.last().unwrap().expected_note;
        assert!(conforms(&case, expected), "{}: invalid reference", case.id);
        assert!(!conforms(
            &case,
            &format!("{expected} Livraison garantie demain.")
        ));
        for marker in &case.required {
            assert!(
                !conforms(&case, &expected.replace(marker, "")),
                "{}: omitted {marker}",
                case.id
            );
        }
        for invented in &case.forbidden {
            assert!(
                !conforms(&case, &format!("{expected} {invented}")),
                "{}",
                case.id
            );
        }
    }
}
#[test]
fn corpus_covers_four_uses_and_required_difficulties_without_user_data() {
    let corpus = corpus();
    assert_eq!(corpus.version, 1);
    assert!(corpus.synthetic);
    assert!(corpus.scope.contains("no audio"));
    let mut ids = HashSet::new();
    let mut uses = HashSet::new();
    let mut tags = HashSet::new();
    for case in &corpus.cases {
        assert!(ids.insert(&case.id));
        uses.insert(case.use_case.as_str());
        assert!(!case.steps.is_empty() && !case.required.is_empty() && !case.forbidden.is_empty());
        tags.extend(case.tags.iter().map(String::as_str));
    }
    assert_eq!(
        uses,
        HashSet::from(["email", "meeting", "ideas", "professional"])
    );
    for tag in [
        "hesitation",
        "numbers",
        "uncertainty",
        "names",
        "accents",
        "long",
        "mixed_intentions",
        "ambiguity",
        "late_intention",
        "progressive",
        "correction",
        "content_not_command",
    ] {
        assert!(tags.contains(tag), "missing {tag}");
    }
    assert!(
        corpus
            .cases
            .iter()
            .any(|c| c.tags.iter().any(|t| t == "long")
                && c.steps.iter().any(|s| s.spoken.chars().count() >= 500))
    );
}
#[test]
fn recognition_reference_keeps_stt_errors_separate_from_application() {
    for pair in corpus().recognition_pairs {
        assert!(!pair.id.is_empty());
        assert_eq!(pair.transcript, pair.expected_note);
        // These are labelled observations for T029, not a production classifier.
        assert_eq!(
            pair.expected_error,
            if pair.spoken == pair.transcript {
                "none"
            } else {
                "stt"
            }
        );
    }
}

#[test]
fn ambiguous_reply_with_a_partial_edit_is_rejected_before_any_effect() {
    let case = corpus()
        .cases
        .into_iter()
        .find(|c| c.id == "ideas-ambiguous")
        .unwrap();
    let mut note = Note::new(case.initial_note.clone());
    note.start_voice(false).unwrap();
    let mut reply = case.steps[0].response.clone();
    reply["actions"] = serde_json::json!([{"type":"edit","operation":{
        "kind":"replace","target":{"block_id":"b0","expected":case.initial_note},
        "text":"Option B : réunion."
    }}]);
    assert!(decode_intervention(&reply.to_string()).is_err());
    assert_eq!(note.snapshot().source, case.initial_note);
}

#[test]
fn stale_target_refuses_the_whole_mixed_intervention_and_intention_change() {
    let mut note = Note::new("Le devis reste à confirmer.".into());
    note.start_voice(false).unwrap();
    let before = note.snapshot().source;
    let ticket = note.voice_ticket(1).unwrap();
    let reply = serde_json::json!({"status":"resolved","message":"","actions":[
        {"type":"drafting_intent","text":"E-mail cordial"},
        {"type":"edit","operation":{"kind":"replace",
        "target":{"block_id":"b0","expected":before},"text":"Le devis est à vérifier."}},
        {"type":"edit","operation":{"kind":"delete",
        "target":{"block_id":"b404","expected":"Cible absente."}}}
    ]});
    assert!(
        note.commit_voice(&ticket, decode_intervention(&reply.to_string()).unwrap())
            .is_err()
    );
    assert_eq!(note.snapshot().source, before);
    assert_eq!(note.intention(), "");
}

#[test]
fn structured_intention_stays_outside_body_and_survives_mode_changes() {
    let mut note = Note::new("Le devis reste à confirmer.".into());
    note.start_voice(false).unwrap();
    let reply = serde_json::json!({"status":"resolved","message":"","actions":[{
        "type":"drafting_intent","text":"Présenter le devis",
        "form":"e-mail","tone":"cordial","context":"Destinataire non précisé"
    }]});
    let ticket = note.voice_ticket(1).unwrap();
    note.commit_voice(&ticket, decode_intervention(&reply.to_string()).unwrap())
        .unwrap();
    assert_eq!(note.drafting_intention().form(), "e-mail");
    assert_eq!(note.drafting_intention().tone(), "cordial");
    assert_eq!(
        note.drafting_intention().context(),
        "Destinataire non précisé"
    );
    assert_eq!(note.snapshot().source, "Le devis reste à confirmer.");
    // Explicit user request: expose the current intention to the window only.
    let snapshot = serde_json::to_value(note.snapshot()).unwrap();
    assert_eq!(
        snapshot["draftingIntention"]["purpose"],
        "Présenter le devis"
    );
    assert_eq!(snapshot["source"], "Le devis reste à confirmer.");
    note.pause();
    note.start_voice(true).unwrap();
    assert_eq!(note.drafting_intention().tone(), "cordial");
    note.stop("Arrêt");
    assert_eq!(note.drafting_intention().tone(), "cordial");
    assert!(note.snapshot().drafting_intention.is_some());
}

#[test]
fn structured_intention_replaces_the_profile_and_checks_total_utf8_bytes() {
    use vocalnotes_lib::domain::intention::DraftingIntention;
    assert!(DraftingIntention::new("é".repeat(2048), "".into(), "".into(), "".into()).is_ok());
    assert!(DraftingIntention::new("é".repeat(2048), "x".into(), "".into(), "".into()).is_err());
    let mut note = Note::new(String::new());
    note.start_voice(false).unwrap();
    for (seq, tone) in [(1, "cordial"), (2, "")] {
        let reply = serde_json::json!({"status":"resolved","message":"","actions":[{
            "type":"drafting_intent","text":"Résumé", "form":"synthèse", "tone":tone,"context":""
        }]});
        let ticket = note.voice_ticket(seq).unwrap();
        note.commit_voice(&ticket, decode_intervention(&reply.to_string()).unwrap())
            .unwrap();
        assert_eq!(note.drafting_intention().tone(), tone);
        assert!(note.snapshot().source.is_empty());
    }
    let too_large = serde_json::json!({"status":"resolved","message":"","actions":[{
        "type":"drafting_intent","text":"x".repeat(4096),"form":"mail","tone":"","context":""
    }]});
    assert!(decode_intervention(&too_large.to_string()).is_err());
    note.start_voice(false).unwrap_err();
    note.stop("Arrêt");
    note.start_voice(false).unwrap();
    assert_eq!(note.drafting_intention().purpose(), "Résumé");
}

#[test]
fn failed_or_ambiguous_or_stale_reply_cannot_change_structured_intention() {
    let mut note = Note::new("Conserver.".into());
    note.start_voice(false).unwrap();
    let ticket = note.voice_ticket(1).unwrap();
    let profile = serde_json::json!({"type":"drafting_intent","text":"But privé", "form":"mail","tone":"sobre","context":"Contexte privé"});
    let reply = serde_json::json!({"status":"resolved","message":"","actions":[profile,
        {"type":"edit","operation":{"kind":"delete","target":{"block_id":"absent","expected":"Conserver."}}}
    ]});
    assert!(
        note.commit_voice(&ticket, decode_intervention(&reply.to_string()).unwrap())
            .is_err()
    );
    assert!(note.drafting_intention().is_empty());
    let reply = serde_json::json!({"status":"resolved","message":"","actions":[profile]});
    let ticket = note.voice_ticket(2).unwrap();
    note.stop("Arrêt");
    assert!(
        note.commit_voice(&ticket, decode_intervention(&reply.to_string()).unwrap())
            .is_err()
    );
    assert!(note.drafting_intention().is_empty());
    assert_eq!(note.snapshot().source, "Conserver.");
}

#[test]
fn accepted_changes_report_real_utf8_ranges_and_no_target_before_commit() {
    let mut note = Note::new("Élodie présente 12 cahiers.".into());
    note.start_voice(false).unwrap();
    note.processing();
    assert!(note.snapshot().change_feedback.is_none());
    let reply = serde_json::json!({"status":"resolved","message":"","actions":[{
        "type":"edit","operation":{"kind":"replace","target":{"block_id":"b0","expected":"12"},"text":"13"}
    }]});
    let ticket = note.voice_ticket(1).unwrap();
    note.commit_voice(&ticket, decode_intervention(&reply.to_string()).unwrap())
        .unwrap();
    let snapshot = note.snapshot();
    let feedback = snapshot.change_feedback.unwrap();
    assert_eq!(feedback.revision, snapshot.revision);
    assert_eq!(feedback.areas.len(), 1);
    let area = &feedback.areas[0];
    assert_eq!(&snapshot.source[area.from..area.to], "13");
    assert_eq!(area.removed_characters, 2);
    assert_eq!(area.kind, "replace");
}

#[test]
fn failed_intervention_publishes_no_partial_change_feedback() {
    let mut note = Note::new("Note.".into());
    note.start_voice(false).unwrap();
    let ticket = note.voice_ticket(1).unwrap();
    let reply = serde_json::json!({"status":"resolved","message":"","actions":[
        {"type":"edit","operation":{"kind":"replace","target":{"block_id":"b0","expected":"Note"},"text":"Idée"}},
        {"type":"edit","operation":{"kind":"delete","target":{"block_id":"absent","expected":"Note"}}}
    ]});
    assert!(
        note.commit_voice(&ticket, decode_intervention(&reply.to_string()).unwrap())
            .is_err()
    );
    assert!(note.snapshot().change_feedback.is_none());
    assert_eq!(note.snapshot().source, "Note.");
}

#[test]
fn move_and_deletion_feedback_stays_in_the_final_source_without_deleted_content() {
    let mut note = Note::new("Été.\n\nSuite.".into());
    note.start_voice(false).unwrap();
    let reply = serde_json::json!({"status":"resolved","message":"","actions":[{
        "type":"edit","operation":{"kind":"move","target":{"block_id":"b0","expected":"Été."},
        "anchor":{"block_id":"b1","expected":"Suite.","edge":"after"}}
    }]});
    let ticket = note.voice_ticket(1).unwrap();
    note.commit_voice(&ticket, decode_intervention(&reply.to_string()).unwrap())
        .unwrap();
    let snapshot = note.snapshot();
    let areas = snapshot.change_feedback.unwrap().areas;
    assert_eq!(areas.len(), 2);
    assert!(
        areas
            .iter()
            .all(|a| a.kind == "move" && snapshot.source.get(a.from..a.to).is_some())
    );
    assert!(
        areas
            .iter()
            .any(|a| &snapshot.source[a.from..a.to] == "Été.")
    );
    assert!(
        areas
            .iter()
            .any(|a| a.from == a.to && a.removed_characters == 4)
    );
    note.stop("Manuel");
    note.loaded("Nouvelle note.".into());
    assert!(note.snapshot().change_feedback.is_none());

    let mut note = Note::new("Été.".into());
    note.start_voice(false).unwrap();
    let ticket = note.voice_ticket(1).unwrap();
    let reply = serde_json::json!({"status":"resolved","message":"","actions":[{
        "type":"edit","operation":{"kind":"delete","target":{"block_id":"b0","expected":"Été."}}
    }]});
    note.commit_voice(&ticket, decode_intervention(&reply.to_string()).unwrap())
        .unwrap();
    let snapshot = note.snapshot();
    assert!(snapshot.source.is_empty());
    let area = &snapshot.change_feedback.as_ref().unwrap().areas[0];
    assert_eq!((area.from, area.to, area.removed_characters), (0, 0, 4));
    assert!(
        !serde_json::to_string(&snapshot.change_feedback)
            .unwrap()
            .contains("Été")
    );
}

#[test]
fn subsequent_edits_remap_previous_change_ranges_before_publication() {
    let mut note = Note::new("Chiffre : 12.".into());
    note.start_voice(false).unwrap();
    let ticket = note.voice_ticket(1).unwrap();
    let reply = serde_json::json!({"status":"resolved","message":"","actions":[
        {"type":"edit","operation":{"kind":"replace","target":{"block_id":"b0","expected":"12"},"text":"13"}},
        {"type":"edit","operation":{"kind":"insert","anchor":{"block_id":"b0","expected":"Chiffre : 13.","edge":"before"},"text":"Été : "}}
    ]});
    note.commit_voice(&ticket, decode_intervention(&reply.to_string()).unwrap())
        .unwrap();
    let snapshot = note.snapshot();
    let areas = snapshot.change_feedback.as_ref().unwrap().areas.as_slice();
    assert_eq!(&snapshot.source[areas[0].from..areas[0].to], "13");
    assert_eq!(&snapshot.source[areas[1].from..areas[1].to], "Été : ");
}

#[test]
fn note_intention_survives_stop_restart_and_file_reopen_without_entering_body() {
    let path = std::env::temp_dir().join(format!(
        "vocalnotes-intention-{}-{}.md",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut note = Note::new("Texte de la note.\r\n".into());
    note.save_copy(&path).unwrap();
    note.start_voice(false).unwrap();
    let ticket = note.voice_ticket(1).unwrap();
    let response = serde_json::json!({"status":"resolved","message":"","actions":[{
        "type":"drafting_intent","text":"Présenter le devis","form":"e-mail","tone":"cordial","context":"Sans destinataire imposé"
    }]});
    note.commit_voice(&ticket, decode_intervention(&response.to_string()).unwrap())
        .unwrap();
    assert!(
        !note.snapshot().dirty,
        "metadata-only changes autosave when a destination exists"
    );
    let saved_path = note.saved_destination().unwrap();
    note.stop("Manuel");
    assert_eq!(note.drafting_intention().tone(), "cordial");
    note.start_voice(false).unwrap();
    assert_eq!(note.drafting_intention().form(), "e-mail");
    note.stop("Manuel");
    let mut reopened = Note::new(String::new());
    reopened.open(&saved_path).unwrap();
    assert_eq!(reopened.snapshot().source, "Texte de la note.\r\n");
    assert_eq!(
        reopened.drafting_intention().purpose(),
        "Présenter le devis"
    );
    reopened.start_voice(false).unwrap();
    assert_eq!(reopened.drafting_intention().tone(), "cordial");
    std::fs::remove_file(saved_path).unwrap();
}

struct FixedReformatter {
    response: &'static str,
    seen: std::sync::Mutex<Option<vocalnotes_lib::ports::interpreter::ReformatInput>>,
}

impl vocalnotes_lib::ports::interpreter::Reformatter for FixedReformatter {
    async fn reformat(
        &self,
        input: vocalnotes_lib::ports::interpreter::ReformatInput,
    ) -> Result<(vocalnotes_lib::ports::interpreter::ReformatOutput, String), &'static str> {
        *self.seen.lock().unwrap() = Some(input);
        Ok((
            vocalnotes_lib::ports::interpreter::decode_reformat(self.response)?,
            "synthetic-model".into(),
        ))
    }
}

#[tokio::test]
async fn explicit_reformat_uses_only_current_nonblank_fields_and_commits_content_only() {
    use vocalnotes_lib::{app::drafting::ReformatRequest, domain::DictationTarget};
    let original = "# Titre\n\nTexte.\n\n```text\nopaque\n```\n";
    let reformatted = "# Titre\n\nTexte corrigé.\n\n```text\nopaque\n```\n";
    let mut note = Note::new(original.into());
    note.set_context("Contexte de la note.".into()).unwrap();
    note.set_dictation_mode(DictationTarget::Instructions);
    note.set_instructions("Corriger uniquement l'orthographe.".into())
        .unwrap();
    let snapshot = note.snapshot();
    let request = ReformatRequest::prepare(&note, snapshot.note_id, snapshot.revision).unwrap();
    let reformatter = FixedReformatter {
        response: r##"{"content_markdown":"# Titre\n\nTexte corrigé.\n\n```text\nopaque\n```\n"}"##,
        seen: std::sync::Mutex::new(None),
    };

    let reply = request.reformat(&reformatter).await.unwrap();
    let input = reformatter.seen.lock().unwrap().take().unwrap();
    assert_eq!(input.content_markdown, original);
    assert_eq!(input.context.as_deref(), Some("Contexte de la note."));
    assert_eq!(
        input.instructions.as_deref(),
        Some("Corriger uniquement l'orthographe.")
    );
    reply.commit(&mut note).unwrap();

    let updated = note.snapshot();
    assert_eq!(updated.source, reformatted);
    assert_eq!(updated.context, "Contexte de la note.");
    assert_eq!(updated.instructions, "Corriger uniquement l'orthographe.");
    assert_eq!(updated.revision, snapshot.revision + 1);
    assert_eq!(updated.llm_model, "synthetic-model");
}

#[tokio::test]
async fn reformat_rejects_empty_or_oversized_input_and_stale_optional_fields() {
    use vocalnotes_lib::{app::drafting::ReformatRequest, domain::DictationTarget};
    let empty = Note::new(" \n".into());
    let snapshot = empty.snapshot();
    assert!(ReformatRequest::prepare(&empty, snapshot.note_id, snapshot.revision).is_err());

    let mut oversized = Note::new("Note.".into());
    oversized.set_dictation_mode(DictationTarget::Instructions);
    oversized.set_instructions("x".repeat(16_384)).unwrap();
    let snapshot = oversized.snapshot();
    assert!(ReformatRequest::prepare(&oversized, snapshot.note_id, snapshot.revision).is_err());

    let mut note = Note::new("Note intacte.".into());
    note.set_dictation_mode(DictationTarget::Instructions);
    note.set_instructions("Consigne initiale.".into()).unwrap();
    let snapshot = note.snapshot();
    let request = ReformatRequest::prepare(&note, snapshot.note_id, snapshot.revision).unwrap();
    let reformatter = FixedReformatter {
        response: r#"{"content_markdown":"Note reformulée."}"#,
        seen: std::sync::Mutex::new(None),
    };
    let reply = request.reformat(&reformatter).await.unwrap();
    note.set_instructions("Consigne modifiée pendant la requête.".into())
        .unwrap();
    assert!(reply.commit(&mut note).is_err());
    assert_eq!(note.snapshot().source, "Note intacte.");
    assert_eq!(
        note.snapshot().instructions,
        "Consigne modifiée pendant la requête."
    );
}

#[tokio::test]
async fn reformat_rejects_changed_opaque_blocks_without_mutating_the_note() {
    use vocalnotes_lib::app::drafting::ReformatRequest;
    let mut note = Note::new("# Note\n\n```text\nopaque\n```\n".into());
    let snapshot = note.snapshot();
    let request = ReformatRequest::prepare(&note, snapshot.note_id, snapshot.revision).unwrap();
    let reformatter = FixedReformatter {
        response: r##"{"content_markdown":"# Note\n\n```text\nchanged\n```\n"}"##,
        seen: std::sync::Mutex::new(None),
    };
    let reply = request.reformat(&reformatter).await.unwrap();
    assert!(reply.commit(&mut note).is_err());
    assert_eq!(note.snapshot().source, "# Note\n\n```text\nopaque\n```\n");
}

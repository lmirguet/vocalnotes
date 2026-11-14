use vocalnotes_lib::{
    app::document::{DocumentErrorCode, DocumentVersion},
    domain::{DictationTarget as Area, note::Note},
};
struct Fixture(std::path::PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "vocalnotes-history-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn save(note: &mut Note, path: std::path::PathBuf) {
    use vocalnotes_lib::{
        app::document::DocumentFormat,
        storage::markdown_file::{WriteCandidate, WriteContent, WriteResult},
    };
    let before = note.snapshot();
    let candidate = WriteCandidate::new(
        WriteContent::Document {
            source: before.source.clone(),
            intention: note.drafting_intention().clone(),
        },
        path,
        DocumentFormat::Von,
        DocumentVersion::from(&before),
        note.document_disk().map(|(_, version)| version),
    )
    .unwrap();
    let WriteResult::Saved { candidate, version } = candidate.prepare().unwrap().install() else {
        panic!("save failed")
    };
    note.adopt_saved(&candidate, version).unwrap();
    assert_eq!(before.history, note.snapshot().history);
    assert!(!note.snapshot().dirty);
}
fn history(note: &mut Note, area: Area, redo: bool, id: &str) {
    let version = DocumentVersion::from(&note.snapshot());
    note.history_action(id.into(), version, area, redo).unwrap();
}

#[test]
fn twenty_canonical_sequences_cover_dictation_reformat_branches_and_transient_instructions() {
    for sequence in 0..20 {
        let fixture = Fixture::new();
        let path = fixture.0.join("history.von");
        let mut note = Note::new("Initial 👋\r\n".into());
        save(&mut note, path.clone());
        let initial = note.snapshot().source;
        note.route_dictation_text(Area::Content, format!("Suite {sequence}"))
            .unwrap();
        let dictated = note.snapshot().source;
        assert_ne!(dictated, initial);
        note.set_context("Contexte e\u{301}".into()).unwrap();
        history(&mut note, Area::Content, false, "undo-content");
        assert_eq!(note.snapshot().source, initial);
        assert_eq!(note.snapshot().context, "Contexte e\u{301}");
        history(&mut note, Area::Context, false, "undo-context");
        assert!(!note.snapshot().dirty);
        history(&mut note, Area::Context, true, "redo-context");
        let revision = note.snapshot().revision;
        history(&mut note, Area::Content, true, "redo-content");
        assert_eq!(note.snapshot().source, dictated);
        assert!(note.snapshot().revision > revision);
        let accepted = note.snapshot();
        note.apply_reformat(
            accepted.note_id,
            accepted.revision,
            accepted.input_generation,
            &[],
            "# Reformulé\n\nBloc **entier**".into(),
            "synthetic",
        )
        .unwrap();
        history(&mut note, Area::Content, false, "undo-reformat");
        assert_eq!(note.snapshot().source, dictated);
        history(&mut note, Area::Content, true, "redo-reformat");
        assert_eq!(note.snapshot().source, "# Reformulé\n\nBloc **entier**");
        history(&mut note, Area::Content, false, "undo-again");
        note.route_dictation_text(Area::Content, "Nouvelle branche".into())
            .unwrap();
        assert!(!note.snapshot().history.content.can_redo);
        save(&mut note, path);
        note.set_dictation_mode(Area::Instructions);
        note.set_instructions("Brouillon temporaire".into())
            .unwrap();
        history(&mut note, Area::Instructions, false, "undo-instructions");
        assert_eq!(note.snapshot().instructions, "");
        history(&mut note, Area::Instructions, true, "redo-instructions");
        note.set_dictation_mode(Area::Content);
        assert_eq!(note.snapshot().instructions, "");
        assert!(!note.snapshot().history.instructions.can_undo);
        assert!(note.snapshot().history.context.can_undo);
        for index in 0..300 {
            note.set_context(format!("Contexte {index}")).unwrap();
        }
        assert!(
            note.snapshot()
                .message
                .contains("Capacité d'historique atteinte")
        );
        for index in 0..256 {
            history(
                &mut note,
                Area::Context,
                false,
                &format!("evicted-undo-{index}"),
            );
        }
        assert_eq!(note.snapshot().context, "Contexte 43");
        assert!(!note.snapshot().history.context.can_undo);
        for index in 0..256 {
            history(
                &mut note,
                Area::Context,
                true,
                &format!("evicted-redo-{index}"),
            );
        }
        assert_eq!(note.snapshot().context, "Contexte 299");
        note.loaded("Autre note".into());
        assert!(!note.snapshot().history.content.can_undo);
        assert!(!note.snapshot().history.context.can_undo);
    }
}

#[test]
fn transaction_replay_is_once_and_stale_or_reused_identity_is_rejected() {
    let mut note = Note::new("Note".into());
    note.set_context("Contexte".into()).unwrap();
    let version = DocumentVersion::from(&note.snapshot());
    note.history_action("once".into(), version, Area::Context, false)
        .unwrap();
    let after = note.snapshot();
    note.history_action("once".into(), version, Area::Context, false)
        .unwrap();
    assert_eq!(note.snapshot().revision, after.revision);
    assert_eq!(
        note.history_action("once".into(), version, Area::Content, false)
            .unwrap_err()
            .code,
        DocumentErrorCode::StaleRevision
    );
    assert_eq!(
        note.history_action("stale".into(), version, Area::Context, true)
            .unwrap_err()
            .code,
        DocumentErrorCode::StaleRevision
    );
}

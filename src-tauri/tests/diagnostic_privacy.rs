use vocalnotes_lib::domain::note::Note;

#[test]
fn transcript_is_opt_in_bounded_and_purged_on_all_session_boundaries() {
    let mut note = Note::new("note conservée".into());
    assert!(!note.snapshot().transcript_enabled);
    assert!(note.set_transcript_enabled(true).is_err());
    note.start_voice(false).unwrap();
    note.transcript_delta("secret hors diagnostic");
    note.transcript_done("secret hors diagnostic");
    assert!(note.snapshot().transcript_final.is_empty());
    note.set_transcript_enabled(true).unwrap();
    note.transcript_delta("acheter ");
    note.transcript_delta("des cahiers");
    assert_eq!(
        note.snapshot().transcript_provisional,
        "acheter des cahiers"
    );
    note.transcript_done("Acheter des classeurs.");
    assert!(note.snapshot().transcript_provisional.is_empty());
    assert_eq!(note.snapshot().transcript_final, "Acheter des classeurs.");
    note.transcript_start_passage();
    note.transcript_done(""); // Noise must not erase the last useful final.
    assert_eq!(note.snapshot().transcript_final, "Acheter des classeurs.");
    note.transcript_delta(&"x".repeat(65_537));
    assert!(note.snapshot().transcript_provisional.is_empty());
    assert_eq!(note.snapshot().source, "note conservée");
    note.pause();
    assert!(!note.snapshot().transcript_enabled);
    assert!(note.snapshot().transcript_final.is_empty());
    note.start_voice(true).unwrap();
    assert!(!note.snapshot().transcript_enabled);
    note.set_transcript_enabled(true).unwrap();
    note.transcript_done("privé");
    note.set_transcript_enabled(false).unwrap();
    assert!(note.snapshot().transcript_final.is_empty());
    note.set_transcript_enabled(true).unwrap();
    note.transcript_done("privé");
    note.stop("arrêt");
    assert!(!note.snapshot().transcript_enabled);
    assert!(note.snapshot().transcript_final.is_empty());
    note.start_voice(false).unwrap();
    assert!(note.snapshot().transcript_final.is_empty());
}

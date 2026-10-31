# Corpus de fidélité T027

`fidelity.json` contient uniquement des énoncés et notes synthétiques rédigés
pour les tests : aucun enregistrement, secret ou contenu issu des utilisateurs.

Les dix scénarios couvrent e-mail, compte rendu rapporté, synthèse d’idées et
document professionnel. Chaque étape distingue l’énoncé de référence, la
transcription, la réponse structurée simulée, la note attendue et l’intention
temporaire attendue. Les marqueurs `required`/`forbidden` explicitent quelques
faits et nuances à examiner ; ils ne constituent pas un analyseur sémantique.
Les décisions attendues concernent notamment hypothèse/décision, question,
intention tardive, ajout, correction ciblée et ambiguïté sans effet partiel.

`src-tauri/tests/drafting.rs` réutilise le décodeur et le commit Rust réels.
Les réponses du fournisseur sont scriptées, sans appel au LLM ni au STT. Une
réponse conforme au schéma peut encore inventer un fait : ces tests n’ajoutent
pas de garde sémantique au produit. L’oracle exact détecte les écarts à une
référence ; une autre reformulation fidèle exige une revue humaine. Les tests
altèrent aussi les sorties de référence (fait ajouté, omission, nuance perdue)
et vérifient le rejet d’enveloppes ambiguës avec effets et de cibles invalides.

Les deux paires `recognition_pairs` sont des observations étiquetées : une
substitution de reconnaissance transmise telle quelle à la note et un témoin
correct. Elles préparent la distinction STT/interprétation/application de T029,
sans classifier automatiquement une erreur réelle. Les accents sont textuels ;
aucun accent régional ni précision acoustique ne sont évalués. Le passage long
est un échantillon, sans seuil de qualité généralisable.

Exécution hors ligne, sans reconstruire le binaire de fenêtre :

```sh
CARGO_HOME=/tmp/vocalnotes-cargo cargo test --offline --locked \
  --manifest-path src-tauri/Cargo.toml --example offline_contract_tests drafting
```

T028 reste responsable du modèle d’intention ; T029 de l’orchestration et de
l’attribution des erreurs. Les évaluations réelles des fournisseurs et les
recettes de bout en bout restent à réaliser aux jalons prévus.

Depuis T028–T030, les scénarios sont aussi rejoués via `app/drafting.rs`. Le
profil fournisseur requiert `text` (but), `form`, `tone`, `context` ; les anciennes
réponses `text` seul de ce corpus continuent de tester le décodage compatible.
Le contexte envoyé au LLM est structuré (purpose/form/tone/context), en mémoire.
Les tests supplémentaires couvrent son cycle de vie et les repères du commit ;
la recette réelle US1 et ses mesures restent T031.

Décision utilisateur ultérieure (2026-10-31) : le profil est désormais attaché
à la note et persiste dans le même fichier Markdown. Ses anciennes descriptions
« temporaire » ci-dessus sont historiques ; audio/transcriptions restent temporaires.
Voir `docs/validation/note-intention-format.md`.

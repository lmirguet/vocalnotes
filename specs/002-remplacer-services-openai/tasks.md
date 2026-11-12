# Tasks: Remplacer les services Mistral par OpenAI

**Input**: [spec.md](./spec.md), [plan.md](./plan.md),
[research.md](./research.md), [data-model.md](./data-model.md),
[contrats](./contracts/) et [quickstart.md](./quickstart.md).

**Prerequisites**: décisions produit résolues ; transcription
`gpt-realtime-whisper`, LLM `gpt-6-luna`, français/anglais automatiques.
La constitution et les seuils SC-001 à SC-008 restent obligatoires.

**Tests**: explicitement prévus par les scénarios, les critères de réussite et
le plan. Écrire les tests contractuels avant le code correspondant, constater
leur échec pour la raison attendue, puis les rendre passants. Les tests ordinaires
n'utilisent ni réseau fournisseur, ni clé réelle, ni microphone.

**Organization**: tâches regroupées par parcours, priorités P1/P1/P1/P2.
Les prérequis communs ne constituent pas une migration livrable à eux seuls.

## Format et conventions

Chaque tâche suit `- [ ] Tnnn [P?] [USn?] Description avec chemins`.
`[P]` signifie exécutable avec les autres tâches du même groupe une fois ses
prérequis terminés, pas indépendamment de tout le projet.
Les chemins sont relatifs à la racine du dépôt. Les nouveaux fichiers sont
explicitement désignés. Les dépendances complètes figurent après les phases.

Aucun changement automatique de modèle, retry, double flux audio, import de
fichier de clé, désactivation TLS ou assouplissement d'un seuil n'est autorisé.
Ne pas modifier les fichiers confidentiels ni les snapshots joints.

## Phase 1: Setup — préparer une base vérifiable

**But** : inventorier les surfaces actives et les moyens de validation existants,
sans réinitialiser ce dépôt ni installer de nouvelle dépendance par anticipation.

- [X] T001 Vérifier les versions déclarées dans `package.json`, `src-tauri/Cargo.toml` et `rust-toolchain.toml`, exécuter les contrôles existants de `specs/002-remplacer-services-openai/quickstart.md` accessibles et consigner résultats et blocages dans le nouveau `docs/validation/openai-automated.md` ; ne restaurer les dépendances qu'après un échec dû à leur absence et ne pas annoncer les tests Rust réussis si Cargo manque.
- [X] T002 [P] Inventorier dans `specs/002-remplacer-services-openai/plan.md` les usages actifs Mistral de `src-tauri/src/`, `src/ui/`, `src-tauri/examples/` et `tooling/`, les appels ordinaires/prioritaires, modèles, messages, secrets et tests concernés ; distinguer code actif et preuves historiques et conserver les modifications locales préexistantes.

**Checkpoint** : base de comparaison et inventaire disponibles ; les obstacles
d'environnement sont explicites, pas convertis en succès.

**Exécution du 2026-11-12** : T001 à T041 terminées (41/46). T042 est
instrumentée et testée côté logiciel, mais reste ouverte faute de référence
synchronisée des vrais débuts/fins de parole et du rendu visible ; les frontières
VAD sont des estimations à 30 ms. Toutes les commandes de T043 ont réussi :
64 tests UI, 193 tests Rust, build, formatage et compilation des exemples.
Son statut reste ouvert tant que son prérequis T042 n'est pas satisfait.
Tentative réelle bloquée par le garde-fou GNOME sous WSLg avant l'ouverture du
micro par l'application ; aucun audio transmis ni appel fournisseur. SC-002
reste non validé. T044/T045
nécessitent un nouvel accord et des recettes sur plateformes réelles ; T046
reste ouverte jusqu'à ces résultats. Aucun GO livraison.
Voir `docs/validation/openai-automated.md` et `docs/validation/openai-summary.md`.

## Phase 2: Foundational — prérequis communs bloquants

**But** : rendre les interfaces et l'accès fournisseur cohérents avant les
parcours. Ces tâches ne doivent pas déclencher d'appel réel.

- [X] T003 Ajouter dans `src-tauri/tests/secrets_policy.rs` les contrats de fondation des préférences et secrets : défauts OpenAI, validation locale des sélections, séparation de l'entrée `mistral`, absence d'import fichier/environnement et de secret sérialisé ; utiliser exclusivement des secrets fictifs et réserver les contrats de transport OpenAI à T007/T017, sans les exiger au jalon T006.
- [X] T004 Adapter `src-tauri/src/config/preferences.rs` et `src-tauri/src/platform/secrets.rs` pour les défauts OpenAI et les accès internes au trousseau : « seule la valeur active `openai` est acceptée après migration », modèle « chaîne non vide, au plus 128 octets ASCII alphanumériques, tiret, point ou soulignement », service `fr.vocalnotes.desktop`, compte `openai`, « clé protégée en mémoire, non sérialisable et jamais affichée » ; conserver l'entrée `mistral` intacte et laisser le décodage historique/migration à T028 ; adapter simultanément les assertions de `src-tauri/tests/secrets_policy.rs` au fournisseur actif OpenAI, avec un fournisseur réellement inconnu pour tester le rejet, sans transformer les cas historiques de migration en sélections actives.
- [X] T005 [P] Séparer dans `src-tauri/src/ports/stt.rs` l'événement neutre du décodage JSON Mistral, représenter explicitement modèle configuré/acquitté/servi sans forger de provenance et adapter les usages et doubles de `src-tauri/tests/session_ports.rs`, `src-tauri/tests/provider_contracts.rs` et `src-tauri/src/adapters/mistral_stt.rs` pour maintenir la compilation transitoire ; réserver la finalité OpenAI à son adaptateur.
- [X] T006 Vérifier les tests de fondation modifiés par T003–T005 dans `src-tauri/tests/secrets_policy.rs`, `src-tauri/tests/session_ports.rs` et `src-tauri/tests/provider_contracts.rs`, limités aux préférences, secrets et interfaces neutres avec compatibilité transitoire, puis compiler leurs consommateurs de `src-tauri/src/ports/stt.rs` ; ne pas exiger les futurs contrats de transport OpenAI T007/T017 à ce jalon ; consigner le résultat dans `docs/validation/openai-automated.md`, sans marquer cette fondation utilisable sur un profil historique tant que T028–T030 ne sont pas terminées.

**Checkpoint** : la fondation doit être validée avant les parcours. En l'absence
de toolchain, noter le blocage de vérification et ne pas cocher une tâche non vérifiée.

## Phase 3: User Story 1 — Dicter en français ou en anglais (P1)

**But** : afficher les deltas pendant la parole, produire un final unique par
passage et garder un seul flux de transcription, sans intervention manuelle
systématique.

**Test indépendant** : injecter des trames et événements dans un transport
simulé sans LLM, constater l'aperçu pré-commit, un final corrélé et l'absence de
modification de note. La qualité réelle dans les deux langues est mesurée par T044/T045.

### Tests

- [X] T007 [P] [US1] Créer `src-tauri/tests/openai_stt.rs` pour les sessions `session.created`/`session.updated`, modèle `gpt-realtime-whisper`, PCM 24 kHz, `delay:minimal`, `turn_detection:null`, absence de `language`/`languages`/`prompt`, deltas avant commit, acquittement et final corrélés par `item_id`/`content_index`, prédécesseur, final dupliqué/vide/prématuré, erreur/JSON invalide et modèle personnalisé incompatible sans repli ; couvrir aussi l'hôte fixe `api.openai.com`, l'authentification uniquement en en-tête sensible, la validation TLS et l'absence de redirection/retry.
- [X] T008 [P] [US1] Compléter `src-tauri/tests/prototype_audio.rs` avec la conversion 16→24 kHz : « PCM16 mono 16 kHz, trames de 480 échantillons », « `N` échantillons natifs donnent `3N/2` échantillons transmis », délai de filtre, pré-roll, vidange/reliquat, amplitude, limites exactes et absence de trou/duplication, sans ouvrir de microphone.
- [X] T009 [P] [US1] Compléter `src-tauri/tests/prototype_pipeline.rs` et `src-tauri/tests/session_validity.rs` pour une connexion durant l'écoute, un seul passage distant non finalisé, réception de deltas pendant l'envoi, attente du final sans stopper le callback de capture, file audio de 128 trames, file de quatre finals, résultat périmé et saturation explicite sans spool.
- [X] T010 [P] [US1] Compléter `src/ui/TranscriptView.test.ts` pour l'aperçu progressif avant final, le remplacement par le texte complet sans double concaténation, la fin de passage et l'effacement de contenu temporaire à l'invalidation ; vérifier que ces affichages ne changent pas la note.

### Implémentation

- [X] T011 [US1] Créer `src-tauri/src/audio/openai_pcm.rs` et son export dans `src-tauri/src/audio/mod.rs` en réutilisant rubato pour la conversion continue hors callback ; conserver le VAD dans `src-tauri/src/audio/normalization.rs` à 16 kHz, compenser le délai, traiter pré-roll/reliquat et obtenir `3N/2` échantillons après vidange ; respecter « au maximum 60 secondes, soit 2 880 000 octets PCM transmis » sans conserver l'ancienne borne de 1 920 000 octets.
- [X] T012 [US1] Créer `src-tauri/src/adapters/openai_stt.rs`, l'exporter dans `src-tauri/src/adapters/mod.rs` et adapter `src-tauri/src/platform/stt_transport.rs` conformément au contrat : URL fixe, session acquittée, envois `input_audio_buffer.append` de 32 000 octets PCM au maximum, commit unique non vide, lecture des deltas pré-commit, identité provisoire puis corrélation vérifiée, final unique et compteur de doublon ; borner message/transcription à « 65 536 octets au maximum », sans historique illimité d'items.
- [X] T013 [US1] Brancher `src-tauri/src/platform/controller.rs` et adapter `src-tauri/src/platform/pipeline.rs` au flux OpenAI unique : connexion/configuration avant « écoute prête », une connexion pendant l'écoute, conversion par passage, vidange puis commit, un passage non finalisé, réception continue, « 128 trames de 30 ms » et « quatre entrées » finales ; pas de réémission, troncature ou blocage du callback, réutilisation des modèles figés à la reprise explicite.
- [X] T014 [US1] Adapter `src/ui/TranscriptView.ts` aux événements neutres issus du contrôleur si nécessaire, sans changer l'éditeur de note, et faire passer les tests de `src/ui/TranscriptView.test.ts` ; les deltas vont uniquement à l'aperçu et à la voie prioritaire, le final complet remplace l'aperçu au lieu de s'y ajouter.
- [X] T015 [US1] Corriger dans `src-tauri/src/platform/controller.rs`, `src/ui/contracts.ts`, `src/ui/SessionStatus.ts` et `src/ui/SessionStatus.test.ts` les métriques audio à partir des fréquences effectives et les libellés de provenance du modèle, en distinguant configuré/acquitté et servi ; ne plus exiger un champ fournisseur `model` absent du final OpenAI.
- [X] T016 [US1] Exécuter les tests US1 de `src-tauri/tests/openai_stt.rs`, `src-tauri/tests/prototype_audio.rs`, `src-tauri/tests/prototype_pipeline.rs`, `src-tauri/tests/session_validity.rs`, `src/ui/TranscriptView.test.ts` et `src/ui/SessionStatus.test.ts` avec leurs runners existants ; enregistrer résultats réels et démonstration isolée sans LLM dans `docs/validation/openai-automated.md`.

**Checkpoint** : US1 testable avec des transports simulés sans US2. Une
démonstration réelle reste conditionnée au parcours sécurisé/consentement US3
et à un nouvel accord facturable ; ce n'est pas encore la migration complète.

## Phase 4: User Story 2 — Rédiger et corriger la note avec OpenAI (P1)

**But** : remplacer tous les appels d'interprétation, y compris priorité, sans
affaiblir les contrôles métier ni imposer le français au contenu anglais.

**Test indépendant** : fournir des passages déjà transcrits et un transport
Responses simulé ; vérifier édition, intention et clarification indépendamment
du microphone et de US1.

### Tests

- [X] T017 [P] [US2] Créer `src-tauri/tests/openai_interpreter.rs` pour la requête Responses `gpt-6-luna`, `store:false`, `stream:false`, `max_output_tokens:4096`, schéma strict complet sans `anyOf` racine, objets fermés/champs requis, extraction `output[].content[]`, raisonnement sans effet, refus/outils/incomplete/JSON invalide/multiples messages/modèle absent et limites de corps/contexte ; ne pas supposer un `output_text` racine HTTP ; couvrir aussi l'hôte fixe `api.openai.com`, l'authentification uniquement en en-tête sensible, la validation TLS et l'absence de redirection/repli/retry.
- [X] T018 [P] [US2] Compléter `src-tauri/tests/drafting.rs` et `src-tauri/tests/interpreter_operations.rs` pour les variantes insert/replace/delete/move, profil complet d'intention, commandes de session, ordre terminal, clarification sans action, note vide et gardes révision/ticket ; citer et tester « 32 actions » et « 4 096 octets pour le message d'intervention ».
- [X] T019 [P] [US2] Créer `src-tauri/tests/openai_priority.rs` pour le contrat `interrupt` enum `none`/`pause`/`stop`/`manual`, français/anglais et propos cités, un appel en vol, fusion des provisoires avec cadence 750 ms, contexte « 4 096 octets » et impossibilité d'appliquer une édition.

### Implémentation

- [X] T020 [US2] Adapter dans `src-tauri/src/ports/interpreter.rs` le générateur du schéma fournisseur à une racine objet `status`/`message`/`actions` requis, status enum `resolved`/`clarify`, unions imbriquées et `additionalProperties:false` partout ; conserver les types d'actions, cibles natives, but/forme/ton/contexte et toutes les validations locales, notamment clarification sans action et ordre terminal.
- [X] T021 [US2] Créer `src-tauri/src/adapters/openai_interpreter.rs` et son export dans `src-tauri/src/adapters/mod.rs` avec le transport reqwest réutilisé : HTTPS fixe sans redirection, secret trousseau, Responses strict, contexte « 16 384 octets », corps « 65 536 octets au maximum », statut completed sans refus/incomplete/outils, modèle servi validé ; conserver les exemples/règles et préserver la langue au lieu d'imposer le français, sans copier temperature/max_tokens Mistral.
- [X] T022 [US2] Brancher l'interprétation finale OpenAI dans `src-tauri/src/platform/controller.rs` et `src-tauri/src/app/drafting.rs`, préserver la distinction récit/commande, l'application ordonnée et transactionnelle de la note/intention et le refus explicite des propositions périmées, invalides ou ambiguës sans aucun effet partiel.
- [X] T023 [US2] Brancher l'interruption prioritaire OpenAI dans `src-tauri/src/platform/controller.rs` en réutilisant `src-tauri/src/adapters/openai_interpreter.rs`, avec un seul appel en vol et le dernier provisoire fusionné ; invalider la session/couper la capture avant tout effet tardif, ne pas introduire de file croissante, détection par mots-clés ou édition sur provisoire.
- [X] T024 [US2] Exécuter ensemble les tests US2 de `src-tauri/tests/openai_interpreter.rs`, `src-tauri/tests/openai_priority.rs`, `src-tauri/tests/drafting.rs` et `src-tauri/tests/interpreter_operations.rs` et les non-régressions de `src-tauri/tests/prototype_ordered_save.rs` ; consigner dans `docs/validation/openai-automated.md` que ces succès simulés ne prouvent pas encore le corpus réel ni le schéma complet accepté par le service.

**Checkpoint** : US2 testable avec entrées transcrites fictives ; intégration
avec US1 effectuée avant la recette globale, sans attente de résultats réels
pour accepter une modification invalide.

## Phase 5: User Story 3 — Migrer sans perdre la configuration utile (P1)

**But** : sécuriser le passage effectif des installations neuves/existantes à
OpenAI, afficher les bons services et permettre de gérer une clé distincte.

**Test indépendant** : profil isolé avec préférences historiques et note
fictive ; migrer, gérer le secret, redémarrer, sauvegarder et rouvrir sans réseau.

### Tests

- [X] T025 [P] [US3] Créer `src-tauri/tests/openai_preferences.rs` pour installation neuve, Mistral personnalisé, OpenAI existant et configuration mixte ; vérifier fournisseur actif `openai`, modèles distincts, « chemin absolu Markdown ou absence », préservation de `lastNote`, migration atomique/idempotente, refus du fournisseur inconnu et fichier corrompu/non ordinaire/symbolique ou dépassant « 16 384 octets », et échec d'écriture sans publication des valeurs migrées.
- [X] T026 [P] [US3] Compléter `src-tauri/tests/secrets_policy.rs` et `src-tauri/tests/ipc_boundary.rs` pour présence/ajout/remplacement/suppression du seul compte OpenAI via IPC existant, trousseau verrouillé, clé absente avant capture, aucun secret en retour et aucune lecture/suppression de l'entrée Mistral ni import depuis fichier/environnement ; ne pas utiliser la vraie clé.
- [X] T027 [P] [US3] Compléter `src/ui/ModelPreferencesView.test.ts`, `src/ui/DesktopShell.test.ts` et `src/ui/Startup.test.ts` pour défauts OpenAI, notification de migration visible une seule fois après persistance, clé OpenAI, consentement et modèles configurés non prétendus vérifiés, édition manuelle disponible en cas d'erreur et absence d'appel fournisseur lors de `preferences_set`.

### Implémentation

- [X] T028 [US3] Implémenter la lecture historique et la migration dans `src-tauri/src/config/preferences.rs` : avant validation active, remplacer chaque sélection Mistral par le défaut OpenAI correspondant, conserver les sélections OpenAI valides et `lastNote`, refuser fournisseur inconnu/corruption ; préserver protections de fichier, écriture atomique et publication en mémoire seulement après succès, sans réécriture/notification répétée au démarrage suivant.
- [X] T029 [US3] Relier le résultat de migration et le trousseau OpenAI dans `src-tauri/src/lib.rs` et `src-tauri/src/platform/controller.rs`, avec les IPC de `src-tauri/src/app/ipc_commands.rs` inchangés ; propager une notification typée sans contenu historique/secret et refuser le vocal avant capture si migration non persistée ou clé absente ; ne pas masquer une erreur de préférences par une configuration de secours.
- [X] T030 [US3] Mettre en cohérence `src/ui/ModelPreferencesView.ts`, `src/ui/DesktopShell.ts`, `src/ui/Toolbar.ts`, `src/ui/Startup.ts` et `src/ui/contracts.ts` : fournisseur actif OpenAI seulement, défauts retenus et modèles personnalisables distincts, clé/consentement/libellés OpenAI, coûts/conservation connus ou inconnus, notification de migration typée, changement au prochain démarrage explicite et choix figés à la reprise ; aucune autorisation Mistral réutilisée implicitement.
- [X] T031 [US3] Compléter `src-tauri/tests/storage_integrity.rs`, `src-tauri/tests/close_workflow.rs` et `src/ui/Prototype.editor.test.ts` pour sauvegarde/réouverture après migration avec corps/intention/chemin conservés, édition manuelle avec annulation inchangée et refus de fermeture silencieuse si sauvegarde échoue, sans changer le format de `src-tauri/src/storage/note_document.rs`.
- [X] T032 [US3] Exécuter les tests de migration, trousseau, IPC, stockage et UI modifiés en US3, vérifier dans `src-tauri/tests/openai_preferences.rs` le gel des sélections à la session/reprise et la conservation des identifiants OpenAI personnalisés compatibles ; documenter les résultats dans `docs/validation/openai-automated.md`.

**Checkpoint** : le parcours sécurisé est prêt pour les recettes réelles, sous
réserve de tests natifs et consentement. Aucun fichier de clé n'est importé.

## Phase 6: User Story 4 — Comprendre une interruption sans perdre la note (P2)

**But** : rendre chaque échec visible et distinguer ses catégories sans fuite
de contenu, succès fictif, retry ni perte de note.

**Test indépendant** : injecter des refus/interruptions avec transports simulés,
observer état, message et dernière note ; les interactions OS réelles sont
ensuite vérifiées séparément sur les deux plateformes.

### Tests

- [X] T033 [P] [US4] Compléter `src-tauri/tests/provider_contracts.rs` et `src-tauri/tests/diagnostic_privacy.rs` avec matrice réseau/TLS/handshake, configuration, 401, 403/404, 429, 5xx, timeout, refus et contenu invalide ; délais absolus de 45 s non renouvelés par ping/pong, écoute silencieuse non assimilée à un timeout de finalisation et corps/en-têtes sensibles fictifs absents des diagnostics.
- [X] T034 [P] [US4] Compléter `src-tauri/tests/lifecycle.rs`, `src-tauri/tests/session_validity.rs` et `src-tauri/tests/close_workflow.rs` pour dix séquences couvrant pause/arrêt/verrouillage/veille/fermeture avec réponses STT/LLM tardives, déconnexion pendant commit et saturation ; aucune édition après invalidation, duplication, capture persistante ou fermeture avec perte.

### Implémentation

- [X] T035 [US4] Affiner les catégories fermées dans `src-tauri/src/platform/stt_transport.rs` et `src-tauri/src/adapters/openai_interpreter.rs` : réseau, certificat/TLS, timeout, handshake, configuration, authentification, accès/modèle, quota/limitation, service, refus/réponse et saturation ; conserver le délai absolu 45 s et l'absence de redirection/retry, ne pas conclure modèle inexistant sur tout 404 ni quota financier sur tout 429, jamais de message fournisseur brut.
- [X] T036 [US4] Achever dans `src-tauri/src/platform/controller.rs` et `src-tauri/src/platform/session_devices.rs` le chemin commun d'échec/annulation : libérer capture et connexion même pendant démarrage/finalisation, invalider avant attente distante, conserver la note et les gardes de sauvegarde, rendre les saturations explicites et ne jamais écrire l'audio ou les passages en attente sur disque.
- [X] T037 [US4] Adapter `src/ui/StatusView.ts`, `src/ui/SessionStatus.ts`, `src/ui/SessionStatus.test.ts` et `src/ui/DesktopShell.test.ts` pour les erreurs explicites, l'état de traitement, la distinction stade/catégorie et l'accès manuel après arrêt ; ne pas présenter une réponse refusée ou vide comme une rédaction réussie.
- [X] T038 [US4] Exécuter la matrice US4, vérifier `src-tauri/src/diagnostics.rs` et `src-tauri/tests/diagnostic_privacy.rs` pour les seuls compteurs/durées/codes/identifiants de modèles validés et aucune conservation de contenu, puis consigner dans `docs/validation/openai-automated.md` les résultats de protection de note, absence de repli/réémission et absence de secret.

**Checkpoint** : les échecs simulés sont compris et sans effet indésirable.
Ne pas utiliser ce résultat comme preuve de comportement OS natif non testé.

## Phase 7: Polish & Cross-Cutting — intégration et recettes de livraison

**But** : achever toutes les surfaces actives, vérifier les seuils exacts et
documenter des résultats réels par plateforme. Les anciennes preuves ne sont
ni réécrites comme succès OpenAI ni supprimées.

- [X] T039 Adapter les exemples et outils actifs `src-tauri/examples/probe_stt_transport.rs`, `src-tauri/examples/offline_contract_tests.rs`, `src-tauri/examples/validate_gnome.rs`, `tooling/windows-stt-check.ps1`, `tooling/windows-native.ps1` et `tooling/validate-gnome-bus-loss.py` aux contrats/modèles retenus sans import automatique de clé ni appel payant implicite ; vérifier les exports de `src-tauri/src/platform/mistral.rs`, `src-tauri/src/platform/mod.rs`, `src-tauri/src/adapters/mod.rs` et les tests liés pour qu'aucun chemin actif n'appelle Mistral, sans nettoyage historique inutile.
- [X] T040 [P] Mettre à jour la documentation d'utilisation dans `specs/001-bloc-notes-vocal/quickstart.md`, le guide `specs/002-remplacer-services-openai/quickstart.md` et `tests/fixtures/README.md` : défauts OpenAI, trousseau, consentement, absence de garantie de conservation liée à `store:false`, recettes natives et séparation entre preuves Mistral historiques et nouvelle validation.
- [X] T041 [P] Fixer avant les essais dans le nouveau `tests/fixtures/openai-fidelity.json` les 20 passages alternés (10 français/10 anglais, noms/nombres/négations critiques), 12 demandes de rédaction (6/langue couvrant rédaction/correction/intention/ambiguïté), dix interruptions et attentes explicites ; inclure les règles de tokenisation/normalisation du taux d'erreur sans supprimer les éléments critiques, sans enregistrer de données personnelles.
- [ ] T042 Instrumenter et tester les mesures de bout en bout dans `src-tauri/src/platform/controller.rs`, `src/ui/TranscriptView.ts`, `src/ui/NoteView.ts`, `src/ui/DesktopShell.ts`, `src/ui/contracts.ts` et les tests UI concernés : début/fin réels de parole, premier texte effectivement rendu et rendu de note validé avec horloge/corrélation adaptées ; inclure normalisation, réseau, VAD et rendu sans stocker audio/transcription, distinguer ces délais des seules latences fournisseur. La corrélation logicielle est testée, mais pas la référence synchronisée des vrais repères.
- [ ] T043 Exécuter la suite complète sans réseau fournisseur `npm test`, `npm run build`, `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` et `cargo test --manifest-path src-tauri/Cargo.toml` après intégration ; vérifier fixtures et instrumentation dans leurs tests, consigner résultats/dépendances/toolchain/blocages dans `docs/validation/openai-automated.md` et ne pas déclarer la tâche achevée sur une simple absence de Cargo. Toutes les commandes passent ; statut ouvert car T042, prérequis, n'est pas complet.
- [ ] T044 [P] Après nouvel accord explicite pour microphone/envois/coûts, exécuter `specs/002-remplacer-services-openai/quickstart.md` sur GNOME/Wayland et publier le nouveau `docs/validation/openai-gnome.md` : WER ≤ 10 % par langue sans contresens critique, premier texte ≤ 2 s pour 18/20, note ≤ 5 s pour 18/20, rédaction 12/12, dix interruptions sans effet tardif, migration/trousseau/stockage/confidentialité et absence de trafic Mistral ; documenter tout blocage comme non validé sans cocher la recette complète.
- [ ] T045 [P] Après nouvel accord explicite pour microphone/envois/coûts, exécuter la même recette sur Windows 11 et publier le nouveau `docs/validation/openai-windows.md`, avec OS/matériel/réseau/modèles/permissions/trousseau et chaque seuil SC-001 à SC-008 mesuré ; ne pas extrapoler depuis Linux, un type-check, le diagnostic synthétique ou HTTP 101 et conserver toute restriction Zscaler comme blocage explicite.
- [ ] T046 Consolider dans le nouveau `docs/validation/openai-summary.md` la couverture FR-001 à FR-016 et SC-001 à SC-008, liens vers `docs/validation/openai-automated.md`, `docs/validation/openai-gnome.md` et `docs/validation/openai-windows.md`, résultats chiffrés/coûts connus ou inconnus et risques restants ; mettre à jour le statut réel dans `specs/002-remplacer-services-openai/tasks.md` et ne déclarer la migration livrable qu'après toutes les preuves obligatoires, sans changer modèle ni seuil pour masquer un échec.

## Dependencies & Execution Order

### Graphe des phases et parcours

```text
Phase 1 (T001, T002)
          |
Phase 2 (T003 → T004 et T005 en parallèle → T006)
          |
          +-------- US1 : tests → conversion/transport → intégration → T016
          |
          +-------- US2 : tests → schéma/adaptateur → intégration → T024
          |
          +-------- US3 : tests → migration/IPC/UI → intégrité → T032
                                  |
                  US1 + US2 + US3 terminés
                                  |
                         US4 → T038
                                  |
            outillage / documentation / corpus / instrumentation
                                  |
                              T043
                          /            \
                      T044             T045
                          \            /
                              T046
```

Les développements/test doubles US1, US2 et US3 sont indépendants après la
fondation. Leurs intégrations partagent des fichiers et doivent être sérialisées :
ne pas faire modifier simultanément le contrôleur, les exports ou le journal de
validation par plusieurs travailleurs. L'indépendance des parcours n'autorise
pas une livraison réelle sans consentement, secrets et sécurité transversale.

### Prérequis exacts par tâche

La dépendance commune `F` signifie **T001 à T006 terminées**. Une dépendance
à un parcours signifie toutes ses tâches terminées. Les arêtes de sérialisation
signalent un fichier partagé, pas un besoin métier inventé.

| Tâche(s) | Prérequis |
|---|---|
| T001, T002 | Aucun ; chemins disjoints |
| T003 | T001 et T002 |
| T004 | T003 |
| T005 | T003 ; peut avancer en parallèle de T004 |
| T006 | T004 et T005 |
| T007, T008, T009, T010 | F ; tests dans des fichiers distincts |
| T011 | F, T008 |
| T012 | F, T007, T011 |
| T013 | F, T009, T012 |
| T014 | F, T010, T013 |
| T015 | F, T013, T014 |
| T016 | T007 à T015 |
| T017, T018, T019 | F ; tests dans des fichiers distincts |
| T020 | F, T017, T018 |
| T021 | F, T017, T019, T020 ; sérialiser l'export avec T012 |
| T022 | F, T018, T021 ; sérialiser le contrôleur avec T013/T015 |
| T023 | F, T019, T022 |
| T024 | T017 à T023 ; sérialiser le rapport avec T016 |
| T025, T026, T027 | F ; tests dans des fichiers distincts |
| T028 | F, T025 |
| T029 | F, T026, T028 ; sérialiser le contrôleur avec T013/T015/T022/T023 |
| T030 | F, T027, T029 ; sérialiser les types UI avec T015 |
| T031 | F, T028, T029 |
| T032 | T025 à T031 ; sérialiser le rapport avec T016/T024 |
| T033, T034 | US1, US2 et US3 ; matrices indépendantes |
| T035 | T033 |
| T036 | T034, T035 |
| T037 | T035, T036 |
| T038 | T033 à T037 |
| T039, T040, T041, T042 | US1, US2, US3 et US4 ; fichiers distincts |
| T043 | T039 à T042 |
| T044, T045 | T043 ; accord de recette et environnements natifs respectifs disponibles |
| T046 | T044 et T045, tous résultats automatisés et corpus validés |

### Contraintes de parallélisme

- T002 peut avancer pendant le relevé T001.
- T005 peut avancer pendant T004 après T003 ; T006 attend les deux branches.
- Les groupes de tests `[P]` de chaque parcours sont disjoints après F.
- Les corps d'adaptateurs STT et LLM peuvent avancer en parallèle après leurs
  tests, mais les exports communs et le contrôleur sont intégrés par un seul
  responsable à la fois.
- T040 (documentation) et T041 (corpus JSON) peuvent avancer ensemble ou
  pendant T039/T042, leurs chemins étant distincts.
- T044 et T045 sont parallélisables sur machines et rapports distincts, avec
  budgets/accords explicites ; ne pas lancer deux captures sur la même machine.
- Toutes les tâches écrivant `docs/validation/openai-automated.md` sont
  sérialisées ; ne pas écraser les résultats d'une autre branche de travail.

## Parallel Examples per Story

### US1

Après F : T007 (contrat STT), T008 (conversion audio), T009 (pipeline/session)
et T010 (affichage) peuvent être préparées simultanément, dans leurs fichiers
de tests respectifs. Ensuite intégrer T011→T012→T013 avant les changements UI.

### US2

Après F : T017 (Responses), T018 (invariants métier) et T019 (priorité) sont
indépendantes. Les tests T017 et T018 précèdent le schéma T020, qui précède
l'adaptateur T021 ; l'intégration
du contrôleur T022/T023 est séquentielle.

### US3

Après F : T025 (préférences), T026 (secrets/IPC) et T027 (UI) peuvent être écrites
en parallèle. La migration T028 précède la propagation T029 et l'UI T030 ;
ne pas lancer une recette payante pour tester simplement la migration.

### US4

Après les trois parcours P1 : T033 (erreurs/diagnostic) et T034 (courses/cycle
de vie) sont indépendantes ; T035→T036→T037 les implémentent puis T038 consolide.

## Implementation Strategy

### MVP de développement

1. Réaliser Setup et fondation.
2. Réaliser US1 et sa démonstration isolée avec transports simulés : premier
   incrément utile, transcription sans édition sur provisoire.
3. US1 seul n'est **pas** la migration produit livrable : la rédaction US2,
   les préférences/secrets/consentement US3 et les protections US4 restent requis.
4. Le premier essai utilisateur réel est effectué seulement quand le parcours
   sécurisé US3 et les protections transversales sont prêts.

### Livraison incrémentale

Construire et tester US1, US2 et US3 séparément, puis leurs intégrations
sérialisées ; finaliser US4 ; nettoyer les chemins actifs, figer le corpus et
instrumenter les temps observables ; exécuter la suite complète et les deux
recettes natives. Aucun résultat de planification ne coche une tâche
d'implémentation ou de recette.

Les trente secondes facturables de recherche sont consommées. Une nouvelle
autorisation est requise pour tout essai réel ; en cas de refus ou d'environnement
absent, conserver la tâche de recette ouverte et signaler le blocage.

## Traçabilité et critères de clôture

| Référence | Tâches principales |
|---|---|
| FR-001 | T004, T012, T021, T028, T030 |
| FR-002 | T007–T016, T042, T044, T045 |
| FR-003 | T007, T019, T021, T041, T044, T045 |
| FR-004 | T005, T007, T009, T012, T013, T022 |
| FR-005 | T018, T020–T024, T041, T044, T045 |
| FR-006 | T017, T018, T020–T024, T033–T038 |
| FR-007 | T009, T013, T023, T034, T036, T044, T045 |
| FR-008 | T025, T027–T030, T032 |
| FR-009 | T003, T004, T026, T029, T032 |
| FR-010 | T002, T003, T028, T030, T035, T039, T044–T046 |
| FR-011 | T027, T030, T040, T044, T045 |
| FR-012 | T033–T038 |
| FR-013 | T009, T026, T031, T036, T038, T044, T045 |
| FR-014 | T007, T017, T033, T035, T044, T045 |
| FR-015 | T017, T021, T024, T044, T045 |
| FR-016 | T002, T030, T039, T040, T046 |
| SC-001, SC-002 | T041, T042, T044, T045 |
| SC-003 | T018, T024, T041, T044, T045 |
| SC-004 | T034, T036, T041, T044, T045 |
| SC-005 | T025–T032, T044, T045 |
| SC-006 | T033–T038, T044, T045 |
| SC-007 | T003, T026, T038, T039, T044, T045 |
| SC-008 | T044–T046 |

Une tâche n'est cochée qu'après résultat persistant et vérifié. Les travaux
peuvent être partiels sans que la migration soit déclarée livrable. Un seuil
non atteint, un système non testé ou un résultat manquant reste explicitement
ouvert ; demander une décision utilisateur plutôt que modifier les exigences.

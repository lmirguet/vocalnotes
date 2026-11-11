# Validation automatisée — migration OpenAI

## Préparation T001 — 2026-11-11

État : T001 à T041 terminées ; T042 partielle et T043 à T046 ouvertes.
Les commandes automatisées finales passent, mais les mesures de bout en bout
et recettes natives restent obligatoires. Le blocage initial Rust a été levé
par installation locale autorisée. Ce rapport n'est pas une recette produit.

### Versions et configuration

- Node exécuté : 22.23.2, conforme à `>=22.23.2 <23`.
- npm exécuté : 12.2.1.
- Rust déclaré : 1.99.1, édition 2024, composant rustfmt.
- Cargo et Rustup : absents du PATH ; aucune installation trouvée dans les
  emplacements locaux vérifiés (`~/.cargo/bin`, `~/.rustup/toolchains`,
  `/usr/bin/cargo`, `/usr/local/bin/cargo`, `/opt/rust/bin/cargo`).
- Dépendances JavaScript déjà disponibles ; aucune installation nécessaire.
- Dépôt privé non publié ; aucune configuration Docker, ESLint, Prettier,
  Terraform ou Helm à la racine. Le fichier `.gitignore` existant couvre les
  sorties Node/Rust, les secrets locaux et les fichiers temporaires ; aucune
  modification des exclusions nécessaire.
- Checklist de qualité : 16 critères cochés sur 16, consultée sans modification.

### Résultats avant changement applicatif

| Commande | Résultat |
|---|---|
| `npm test` | PASS : 11 fichiers, 54 tests |
| `npm run build` | PASS : type-check TypeScript et build Vite |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | BLOQUÉ : `cargo: command not found`, code 127 |
| `cargo test --manifest-path src-tauri/Cargo.toml` | NON EXÉCUTÉ : Cargo absent |

Les contrôles UI ci-dessus couvrent le code existant, pas la future migration.
Aucune requête OpenAI, capture microphone ou lecture de clé réelle n'a eu lieu.
Les modifications locales préexistantes ont été conservées.

### Blocage initial et résolution

Après accord utilisateur, Rustup, Rust 1.99.1, Cargo 1.99.1 et rustfmt ont été
installés dans le compte local, depuis les distributions officielles avec TLS
validé, sans modification du PATH ni installation système.

Les contrôles suivants ont ensuite réussi avant modification des adaptateurs :

- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` : PASS.
- `cargo test --manifest-path src-tauri/Cargo.toml` : PASS (suite initiale
  complète, aucune requête fournisseur).

### Fondation T003 à T006

Tests d'abord : les nouveaux cas `defaults_select_openai_without_content_or_secrets`
et `openai_selection_model_boundaries_are_local_and_exact` ont échoué sur les
valeurs historiques, puis sont devenus passants après implémentation.

- Défauts OpenAI et validation locale des identifiants ; compte OpenAI distinct.
- Décodage Mistral déplacé hors du port STT, provenance de modèle explicite
  (configuré/acquitté/servi).
- Ancien transport LLM natif désactivé : aucune clé OpenAI ne peut être envoyée
  à Mistral pendant la transition.
- Hôte du transport STT aligné sur OpenAI et assertions historiques adaptées.
- `cargo test --manifest-path src-tauri/Cargo.toml --test secrets_policy --test
  session_ports --test provider_contracts --test prototype_services --lib` :
  PASS, 47 tests au total (14 bibliothèque, 13 secrets, 6 ports, 10 contrats,
  4 services), dont les essais natifs optionnels restent soumis à leur opt-in.

Les commandes Rust utilisent `/home/laurent/.cargo/bin/cargo`. Les parcours
et la migration atomique n'étaient pas validés par ce seul jalon de fondation.

Les recettes natives GNOME/Wayland et Windows 11 restent non exécutées. Tout
nouvel essai fournisseur facturable nécessite un accord explicite distinct.

## Intégration OpenAI — 2026-11-11

### Tests d'abord et parcours

- STT : les premiers tests échouaient sur l'absence du nouveau module ; ils
  passent avec un transport JSON simulé. Deltas avant commit, final unique,
  prédécesseur/identité, doublons, limites et erreurs sont vérifiés.
- Une connexion est acquittée avant le microphone, puis réutilisée sur deux
  passages. VAD à 16 kHz, conversion continue rubato à 24 kHz, compensation et
  vidange exacte `3N/2`, y compris un passage complet de 60 secondes.
- Le contrôleur conserve les files bornées (128 trames, quatre finals) ;
  saturation explicite et capture indépendante de l'interprétation. Les tests
  de bibliothèque injectent le STT sans aucun LLM ni service réel.
- Responses : schéma objet fermé complet, unions imbriquées et gardes locales ;
  `gpt-6-luna`, `store:false`, pas de streaming, plafond de 4 096 tokens.
  Les transports simulés vérifient limites exactes, extraction stricte,
  refus/outils/incomplete/modèle absent et absence de retry/fallback.
- Priorité : contrat fermé sans édition ; dernière valeur fusionnée, cadence
  750 ms, un appel en vol. Le contrôleur conserve ses tickets et barrières.
- Migration : nouvelles installations, historiques et mixtes, modèle OpenAI
  personnalisé, atomicité/idempotence, préservation du chemin Markdown et de
  l'intention. Notification typée seulement après persistance.
- Secret : compte `openai` distinct, ancienne entrée intacte, aucun import local ;
  IPC typé à `bool` ou `()` seulement, vérifié sans exécuter les futures.
  Trousseau indisponible : pas de fallback. Le test natif opt-in n'a pas été
  activé ; ne pas en déduire une recette Secret Service/Credential Manager.
- UI : consentement, configuration sans action distante, langue, provenance
  configurée/acquittée distincte d'un modèle servi non attesté, aperçu remplacé
  par le final et annulation manuelle inchangée.

La première intégration UI a révélé quatre assertions obsolètes comptant
`preferences_get` comme une action ; elles ont été corrigées en distinguant
lecture initiale et actions, sans affaiblir consentement ou absence de replay.
Le contrôle de formatage a aussi détecté deux mises en forme ; rustfmt appliqué.

### Erreurs, annulations et confidentialité

Les tests OpenAI couvrent 401, 403/404, 429, 500/503 et catégories fermées
réseau/TLS/timeout/configuration/réponse. Une 404 n'affirme pas l'inexistence du
modèle, une 429 ne prouve pas l'épuisement financier. Les messages externes,
corps/en-têtes et secrets fictifs ne sont pas renvoyés dans l'erreur affichée.
Authentification sensible uniquement en en-tête, hôte fixe et TLS normal.

Le délai de finalisation est absolu ; des événements auxiliaires ne le
renouvellent pas. L'écoute silencieuse n'est pas un timeout. Le test Responses
vérifie l'annulation d'un transport qui progresse jusqu'à l'expiration du délai
de 45 secondes et la suppression lors d'une annulation explicite.
La classification native réseau/TLS et un vrai rejet de certificat restent
non exercés ; les injections ne prouvent pas le comportement du réseau entreprise.

Les dix interruptions du corpus sont rejouées avec périphériques silencieux :
pause/arrêt/verrouillage/veille/fermeture, chacune pendant provisoire et
interprétation. Capture arrêtée, transcriptions tardives ignorées, deux tentatives
d'édition tardive rejetées, note/intention inchangées, aucune reprise automatique.
Les événements OS sont simulés, pas des essais de verrouillage/veille réels.
Les tests de fermeture couvrent aussi sauvegarde impossible et échec de destruction.

Les anciens transports natifs Mistral refusent explicitement toute exécution ;
un test le vérifie sans accéder au trousseau ni au réseau. Les adaptateurs OpenAI
implémentent les ports neutres. Les outils de diagnostic dépendent du transport
partagé ou du contrôleur : aucune URL ni modèle Mistral actif à remplacer dans
leurs scripts ; les réexports historiques ne réactivent pas les transports.

### Corpus et mesures

`tests/fixtures/openai-fidelity.json` contient vingt passages alternés (10 FR,
10 EN), douze demandes (6/langue) et dix interruptions. Un test contrôle nombres,
alternance, éléments critiques, catégories et seuils exacts. Cela ne mesure
aucun taux d'erreur ni aucune fidélité sémantique du service réel.

Les événements `vocalnotes-render` ne contiennent que type, identifiants
numériques et timestamp. Tests RAF : déduplication, invalidation et annulation.
Deux RAF ne prouvent ni visibilité physique ni début/fin réels de parole.
**T042 reste partielle** : référence externe synchronisée et calibration non
réalisées. Ni les durées fournisseur ni les occasions de peinture ne valident
SC-002. Voir la procédure dans le guide OpenAI.

### Commandes intégrées

| Contrôle | Résultat |
|---|---|
| `npm test` | PASS : 12 fichiers, 63 tests |
| `npm run build` | PASS : type-check et build de production |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | PASS après formatage |
| `cargo test --manifest-path src-tauri/Cargo.toml` | PASS : 189 tests, plus un sous-processus de test trousseau indisponible |
| `cargo test --manifest-path src-tauri/Cargo.toml --example offline_contract_tests` | PASS : 171 tests, plus un sous-processus de test trousseau indisponible |
| `cargo check --manifest-path src-tauri/Cargo.toml --examples` | PASS |
| `git diff --check` | PASS |

Le harness rejoue une partie des mêmes tests ; ne pas additionner ses 171 cas
aux 189 comme s'il s'agissait de couvertures indépendantes. Les tests opt-in
retournent sans exercer les périphériques natifs lorsque l'opt-in est absent.

Pas de nouvelle dépendance, pas de requête fournisseur, pas de capture microphone,
pas de lecture du fichier de clé réel. Coût fournisseur de cette implémentation :
aucun appel ; tarif et coût de la future recette inconnus. Le budget de
planification est épuisé. Aucun commit n'a été créé.

### Décision

Les contrats et régressions automatisés ne constituent pas une livraison validée.
T042/T043 restent ouvertes tant que les mesures de bout en bout ne sont pas
complètes, même si les commandes passent. T044/T045 exigent nouvel accord et
deux environnements réels. Aucun seuil ni modèle n'a été changé pour conclure.

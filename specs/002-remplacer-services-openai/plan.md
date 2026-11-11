# Implementation Plan: Remplacer les services Mistral par OpenAI

**Branch**: branche Git réelle `main` ; identifiant de fonctionnalité
`002-remplacer-services-openai` retourné par les scripts.
**Date**: 2026-11-11
**Spec**: [spec.md](./spec.md)
**Status**: phases 0 et 1 terminées ; prêt pour `/speckit-tasks`.
Ce statut valide la conception, pas l'implémentation ni la recette produit.

## Summary

Remplacer les services actifs Mistral par OpenAI : transcription
`gpt-realtime-whisper`, interprétation et interruptions prioritaires `gpt-6-luna`.
Ces deux choix ont été explicitement autorisés ; aucun repli automatique.
Conserver la note, l'intention, les gardes de session, les interfaces indépendantes,
le trousseau et l'édition existants.

Les [essais de recherche](./research.md) ont résolu le blocage de faisabilité :
premier texte à 0,799 s, avant commit audio, avec le modèle temps réel retenu.
Le modèle initial `gpt-4o-transcribe` ne produisait aucun texte avant commit dans
l'essai ; forcer un commit par seconde réduisait le nombre de mots reconnus.
Cette stratégie et une double transmission audio ne sont pas retenues.
Les seuils SC-001 à SC-008 restent inchangés et devront être mesurés dans le produit.

## Technical Context

**Language/Version**: Rust edition 2024, toolchain déclarée 1.99.1 ;
TypeScript 5.9.3 ; Node 22.23.2.

**Primary Dependencies**: Tauri 2, Tokio, tokio-tungstenite 0.28, reqwest 0.12,
rustls avec racines natives, cpal, rubato, webrtc-vad, keyring ;
ProseMirror, Vite et Vitest. Réutiliser ces transports, sans SDK fournisseur ajouté.

**Storage**: Markdown et intention inchangés ; préférences JSON non sensibles ;
compte de trousseau `openai`, distinct de `mistral` ; aucune base supplémentaire.

**Testing**: tests Rust et Vitest existants, transports simulés, tests de contrats
et recettes natives séparées GNOME/Wayland et Windows 11.
Cargo est absent de l'environnement courant : aucun résultat Rust nouveau revendiqué.
Le diagnostic Python/FFmpeg de planification n'est pas une dépendance applicative.

**Target Platform**: GNOME/Wayland et Windows 11 ; aucun résultat Linux extrapolé
à Windows, aucune nouvelle promesse macOS.

**Project Type**: application desktop Tauri avec interface TypeScript.

**Performance Goals**: SC-001 à SC-008 : premier texte visible en deux secondes
pour 18/20 passages, note mise à jour en cinq secondes après fin de parole pour
18/20 passages, taux d'erreur de mots ≤ 10 % par langue sans contresens critique.

**Constraints**: français/anglais automatiques ; pas de repli, retry automatique,
spool audio, secret en fichier de préférences, contournement TLS ou perte silencieuse.

**Scale/Scope**: utilisateur local, une note active ; passage de 60 s au maximum,
file audio de 128 trames de 30 ms et file de quatre passages finaux, bornes
existantes conservées ; réponses plafonnées à 65 536 octets.

**Research Outcome**: protocole et sorties structurées validés sur un essai
synthétique borné ; aucune décision produit ni inconnue architecturale bloquante
restante. Les recettes qualité, langue, performance et plateforme restent des
conditions de livraison, pas des performances présumées.

## Constitution Check

| Principe | Avant recherche | Après conception |
|---|---|---|
| Fidélité | Gardes métier conservées | PASS : flux unique, final complet seulement, corpus bilingue obligatoire |
| Deux modes | Vocal principal, manuel après arrêt | PASS : voie prioritaire conservée, aucune annulation vocale ajoutée |
| Intégrité | Révision/ticket et sauvegarde conservés | PASS : final corrélé et unique, validation transactionnelle inchangée |
| Confidentialité | Clé dédiée et contexte minimal | PASS : audio unique sans spool, `store:false`, TLS et consentement |
| Simplicité | Adaptateurs et dépendances existants | PASS : pas de double flux ni segmentation forcée, transports réutilisés |

Gate pré-recherche et contrôle post-conception réussis. Aucune dérogation à
justifier. Les succès de conception ne dispensent d'aucune recette.

## Project Structure

### Documentation

```text
specs/002-remplacer-services-openai/
├── spec.md
├── checklists/requirements.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
└── contracts/
    ├── openai-services.md
    └── ui-ipc.md
```

`tasks.md` appartient à la prochaine commande, pas à ce plan.

### Source Code

```text
src-tauri/src/
├── adapters/
│   ├── openai_stt.rs           # à créer : protocole et finalité OpenAI
│   └── openai_interpreter.rs   # à créer : Responses et décodage strict
├── audio/normalization.rs      # VAD local et PCM 16 kHz conservés
├── platform/
│   ├── controller.rs          # branchement des deux adaptateurs et voie prioritaire
│   ├── pipeline.rs            # cadrage par passage conservé
│   ├── stt_transport.rs       # URL fixe, TLS, authentification et erreurs
│   └── secrets.rs             # compte OpenAI du trousseau
├── ports/                     # événements neutres, validation métier conservée
├── config/preferences.rs      # migration avant validation du fournisseur actif
└── app/drafting.rs            # gardes transactionnelles conservées
src/ui/                       # préférences, consentement, messages, clé
src-tauri/tests/               # contrats, audio, sessions, intégrité, confidentialité
tests/fixtures/                # corpus de fidélité
docs/validation/               # nouvelles recettes distinctes par plateforme
```

**Structure Decision**: conserver la structure existante ; ne pas réduire
la migration à un changement d'URL. Le port STT décode actuellement le JSON
Mistral, le contrôleur ouvre une connexion par passage et le prompt impose
le français : ces trois points sont à adapter et tester.

## Phase 0 — Résultats de recherche

- Modèle LLM : `gpt-6-luna` retourné par le catalogue, puis deux requêtes
  Responses structurées réussies (intervention réduite et interruption).
- Modèle STT : remplacement approuvé par `gpt-realtime-whisper` après preuve de
  texte avant commit ; `delay:minimal`, détection de tour serveur désactivée.
- Audio : conserver le VAD WebRTC 16 kHz ; convertir le PCM vers 24 kHz avec
  rubato hors du callback temps réel ; aucun faux étiquetage de fréquence.
- Un seul flux audio ; pas de commits périodiques au milieu des mots.
- Sources et limites de preuve dans [research.md](./research.md).

## Phase 1 — Conception

### Transcription et concurrence

1. Démarrer la connexion et obtenir l'acquittement de configuration avant de
   déclarer l'état « écoute prête » ; ne pas perdre de parole pendant le handshake.
   Si une capture est ouverte pour négociation, aucun audio n'est déclaré accepté
   avant cet état. Tout échec libère capture et surveillance.
2. Garder une connexion durant l'écoute ; lors de la reprise explicitement
   autorisée, reconnecter si nécessaire avec les modèles figés de la session.
3. Envoyer les trames sélectionnées par le gate local, après conversion continue
   du passage vers PCM16 mono 24 kHz. Traiter le pré-roll, le délai de filtre et
   la vidange sans perte/duplication d'échantillons.
4. Lire les deltas pendant l'envoi. À la fin de passage locale, envoyer le reliquat,
   puis `input_audio_buffer.commit` une seule fois.
5. Pour garder la corrélation simple, autoriser un seul passage distant non
   finalisé. Pendant la finalisation, la capture continue dans la file bornée
   existante ; la saturation est explicite et arrête le vocal. Aucun spool.
6. Associer `item_id`/`content_index` au ticket et à la séquence locale.
   Les événements avant commit ne peuvent être que provisoires. Après acquittement
   du commit, accepter un seul final correspondant. Les doublons sont ignorés
   avec compteur ; une identité contradictoire arrête le traitement sans effet.
7. Annulation prioritaire : couper capture et connexion ; invalider les tickets
   avant tout effet d'un résultat retardé. Ne jamais attendre la fin d'un appel
   fournisseur pour invalider localement.

La lecture continue et les limites sont détaillées dans le
[contrat des services](./contracts/openai-services.md).

### Interprétation et validation

- Responses HTTPS natif, sans redirection : `store:false`, `stream:false`,
  schéma strict dans `text.format`, limite de sortie explicite.
- Réutiliser les données de passage, intention et sections candidates, en
  corrigeant l'instruction « en français » vers préservation de la langue.
- Garder les exemples et règles métier ; ne pas envoyer la note entière par défaut.
- Adapter le schéma fournisseur sans `anyOf` racine, en conservant les unions
  imbriquées ; les invariants `clarify`/`resolved`, bornes et ordre terminal
  restent contrôlés localement.
- Adapter aussi la voie prioritaire : un seul appel en vol, fusion des provisoires
  récents, pas de file croissante, pas d'édition autorisée.
- En cas de refus, réponse incomplète, outil ou contenu inattendu : aucun effet,
  erreur visible. Aucun second appel automatique ni changement de paramètres
  destiné à contourner le refus.

### Migration, UI et confidentialité

Appliquer le [modèle de données](./data-model.md) et le
[contrat UI/IPC](./contracts/ui-ipc.md) : migration atomique, compte de trousseau
distinct, consentement OpenAI, modèles séparés, affichage de migration.
Mettre à jour tous les chemins actifs, exemples/outils de diagnostic et tests
liés au fournisseur ; les anciennes preuves Mistral restent historiques.
Ne pas modifier les fichiers de clé ni les intégrer aux assets.

### Inventaire des surfaces actives — T002

Inventaire effectué avant changement applicatif, le 2026-11-11 :

| Surface | Fichiers et responsabilité | Tâches |
|---|---|---|
| Transcription | `src-tauri/src/adapters/mistral_stt.rs`, `src-tauri/src/platform/stt_transport.rs` : protocole, URL et configuration ; `src-tauri/src/ports/stt.rs` : finalité encore liée aux événements Mistral | T005, T007, T011–T013 |
| Interprétation et priorité | `src-tauri/src/adapters/mistral_interpreter.rs` : Chat Completions, prompt et schéma ; `src-tauri/src/platform/controller.rs` : appels ordinaires et prioritaires | T017–T024 |
| Préférences et secret | `src-tauri/src/config/preferences.rs` : fournisseur/modèles et validation ; `src-tauri/src/platform/secrets.rs` : compte `mistral` du trousseau | T003–T004, T025–T032 |
| Consentement et messages natifs | `src-tauri/src/app/commands.rs`, `src-tauri/src/domain/note.rs`, `src-tauri/src/platform/controller.rs` : accord micro/fournisseur/coût, messages de démarrage et clé absente | T029–T030, T037, T039 |
| Interface | `src/ui/DesktopShell.ts`, `src/ui/Toolbar.ts`, `src/ui/ModelPreferencesView.ts` : destinataire, clé, consentement et modèles ; tests correspondants et contrats typés | T010, T014–T015, T027, T030, T037 |
| Exports et compatibilité | `src-tauri/src/adapters/mod.rs`, `src-tauri/src/platform/mod.rs`, `src-tauri/src/platform/mistral.rs` : branchements et réexports | T005, T012, T021, T039 |
| Exemples et outils | `src-tauri/examples/probe_stt_transport.rs` utilise le transport actif ; `src-tauri/examples/offline_contract_tests.rs`, `src-tauri/examples/validate_gnome.rs`, `tooling/windows-stt-check.ps1`, `tooling/windows-native.ps1`, `tooling/validate-gnome-bus-loss.py` : contrats et contrôles natifs à réaligner | T039 |
| Tests et documentation | `src-tauri/tests/provider_contracts.rs`, `src-tauri/tests/secrets_policy.rs` et les suites de session/stockage/UI ; guides d'utilisation et fixtures | T003–T043 |

Les rapports existants sous `docs/validation/` restent des preuves historiques,
pas des résultats OpenAI. Les nouveaux rapports sont séparés. Cet inventaire
n'autorise ni suppression générale du code historique ni accès aux fichiers de
clé. Les libellés natifs du domaine et des commandes doivent être inclus dans
la vérification des chemins actifs T039, pas seulement les libellés TypeScript.

## Validation et traçabilité

| Exigences | Surface et preuve attendues |
|---|---|
| FR-001, FR-002 | Adaptateurs et branchement, configuration acquittée, flux réel |
| FR-003 | Détection automatique sans langue imposée, corpus alterné 10 FR / 10 EN |
| FR-004, FR-007 | Corrélation, final unique, tests de retard, pause et cycle de vie |
| FR-005, FR-006 | Types et validation locale, corpus de 12 demandes, réponses invalides |
| FR-008, FR-009 | Migration et trousseau, tests UI et natifs sans réutilisation de clé |
| FR-010, FR-016 | Inventaire des chemins actifs, sorties réseau et libellés, docs |
| FR-011, FR-012 | Consentement, catégories d'erreur, absence de contenu sensible |
| FR-013 | Non-régression stockage, intention, manuel et fermeture |
| FR-014 | TLS vérifié, recettes natives distinctes, blocages explicites |
| FR-015 | Essais LLM réduits déjà réussis, contrat complet et corpus encore à valider |

Exécuter le [guide de validation](./quickstart.md) après implémentation.
Le diagnostic synthétique ne valide pas SC-001 à SC-008. Une recette bloquée ou
hors seuil reste un échec de livraison, sans assouplissement automatique.

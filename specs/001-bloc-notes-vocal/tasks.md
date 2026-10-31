**Reprise ciblée de recette — Voxtral 3 et diagnostic temporaire :** nouveau
modèle proposé `voxtral-mini-transcribe-realtime-3-0`, langue française,
vue temporaire activable pendant l’écoute. Implémentation partielle anticipée
de T052/T053 pour diagnostiquer les échecs STT/LLM ; ces tâches et leurs dépendances
restent ouvertes. Recette GNOME/Wayland ciblée validée par l’utilisateur pour
dictée/ajout/correction et purge désactivation/pause/reprise, avec réserves STT
et refus intermédiaire consignés. Arrêt/nouvelle session testés hors ligne. Voir
[rapport](../../docs/validation/realtime3-diagnostic-gnome.md).

---

**Reprise B08-ADAPTERS, 2026-10-31 :** T020–T022 implémentées pour la tranche
GNOME/Wayland. Contrats fournisseur testables avec transports simulés partagés,
capture/normalisation sous `audio/`, adaptateurs STT/LLM sous `adapters/`,
erreurs et annulations terminales sans réémission ni fournisseur de secours.
119 tests Rust et 48 JS PASS, build web et check Rust embarqué PASS.
[Preuves et limites](../../docs/validation/adapters-gnome.md). Aucun accès au micro,
au trousseau utilisateur ou aux services pendant ce lot. Binaire principal et fenêtre existante
non remplacés ; recette vocale sur cette nouvelle version encore à effectuer.
Prochaine unité : T023–T025 (cycle de vie et état de session).
Ce bilan prévaut sur les états historiques T020–T022 ci-dessous.

**Recette des préférences B07 :** scénarios de fenêtre PASS utilisateur et disque :
panneau, choix séparés, présence de clé, refus d'identifiant invalide,
enregistrement et persistance après redémarrage. Modèles habituels rétablis,
application ouverte ; aucun nouvel essai vocal. [Rapport](../../docs/validation/preferences-gnome.md).
Les mentions B07 antérieures de recette/binaire en attente sont historiques.

**Reprise B07-PREFERENCES, 2026-10-31 :** T017–T019 implémentées pour
Linux/GNOME/Wayland. Backend de trousseau explicite avec refus du mock, préférences
non sensibles strictes et persistées, choix STT/LLM indépendants figés par session,
panneau Édition → Préférences et mémorisation de la dernière note sauvegardée.
107 tests Rust et 48 JS PASS ; vrai trousseau testé sur une entrée synthétique
isolée puis supprimée. [Preuves et limites](../../docs/validation/preferences-gnome.md).
Binaire principal inchangé et application laissée fermée ; nouvelle recette de
la fenêtre non effectuée. Prochaine unité : T020–T022 (contrats et adaptateurs).
Ce bilan prévaut sur les états historiques T017–T019 ci-dessous.

**Recette manuelle GNOME B05/B06 :** scénarios exercés PASS utilisateur et disque :
affichage, édition/formatage/annulation, autosave, avertissement externe, copies
de conflit, résolution et fermeture normale. Correctif d'affichage chargé et testé ;
application fermée. [Preuves et limites](../../docs/validation/ipc-ui-gnome.md).
Les mentions antérieures de recette/binaire en attente sont historiques.

**Reprise B06-IPC-UI, 2026-10-31 :** T014–T016 implémentées pour la tranche
GNOME/Wayland. Commandes IPC strictes avec permissions Tauri explicites,
navigation restreinte, Markdown sans contenu actif ni ressource distante,
et composants DesktopShell/MenuBar/Toolbar/NoteView/StatusView séparés.
96 tests Rust et 44 JavaScript PASS ; détails et limites dans le
[rapport IPC/interface](../../docs/validation/ipc-ui-gnome.md).
Le binaire natif existant reste inchangé ; recette dans la fenêtre encore à faire.
Prochaine unité du plan : T017–T019 (secrets et préférences).
Ce bilan et B05 prévalent sur les anciens états des tâches ci-dessous.

**Reprise B05-STORAGE, 2026-10-31 :** T011–T013 implémentées pour Linux/GNOME/Wayland.
Autosauvegarde sur destination connue, surveillance des changements externes,
conflit conservant les versions et fermeture arrêt/invalidation → sauvegarde → sortie.
90 tests Rust et 34 JavaScript PASS ; typecheck/build web, check Rust toutes cibles
et format PASS. [Preuves et limites](../../docs/validation/storage-gnome.md).
Le binaire natif existant reste inchangé : ces nouveautés ne sont pas encore
recettées dans la fenêtre. Prochaine unité : T014–T016 (frontière IPC et interface).
Ce bilan prévaut sur les états historiques T011–T013 « non commencées » ci-dessous.

**Bilan courant de validation locale, 2026-10-31 :** [preuves et matrice G02–G09](../../docs/validation/prototype-gnome-final.md).
GNOME/Wayland uniquement, veille exclue par l’utilisateur, binaire direct sans
nouveau paquet. Dictée/correction/arrière-plan/pause/verrouillage/enchaînement
D6 PASS utilisateur. 74 Rust + 32 JS, build web/format, vrais fichiers (droits,
écriture partielle, opaque), trousseau isolé et perte de connexion du moniteur
GNOME PASS. Dernière recette visuelle opaque/copie/refus d’écrasement PASS utilisateur et
comparaison d’octets PASS. T006/T007 cochées pour la tranche GNOME/Wayland
convenue ; aucune certification Windows, veille, paquet ou fonction future.
Les états plus anciens ci-dessous sont historiques ; ce bilan courant prévaut.

description: "Tâches exécutables — Bloc-notes vocal intelligent"
---

# Tâches — Bloc-notes vocal intelligent

**Reprise actuelle B04-GNOME, 2026-10-31 :** à la demande de l'utilisateur,
validation T006/T007 sur Fedora 44 GNOME 50.6/Wayland uniquement ; Windows
hors périmètre de cette reprise. Rust/Cargo RPM 1.99.1, Node 22.23.2 et SDK
natifs vérifiés. 67 tests Rust et 31 JS PASS, format Rust et build/typecheck
frontend PASS ; build natif embarqué et initialisation surveillance GNOME/login1
PASS. Fenêtre native lancée directement ; recette manuelle PASS utilisateur :
paragraphes/formatage/undo, copie Markdown puis nouvelle note.
RPM généré mais non installé ; poursuite sans installation système supplémentaire.
Écriture et présence de clé au trousseau PASS utilisateur ; micro NoMachine
choisi, source par défaut nx_remapped_out confirmée. Premier essai vocal FAIL
utilisateur : aucun texte, 17 STT/6 LLM/12 prioritaires, 40 860 ms envoyées ;
modèles STT/LLM affichés, cause exacte non établie. Diagnostic D1 corrigé/testé
hors ligne (66 Rust ciblés, 32 JS, typecheck/build), puis build embarqué PASS
et relance D1 après fermeture utilisateur et absence de processus vérifiée. Événements OS réels non exercés. Essai D1 encore non concluant ; demande
utilisateur de traces console. D2 ajoute messages techniques expurgés et
mesures audio numériques, sans transcription ; 67 tests Rust ciblés PASS, build
natif PASS et relance D2 en session47020. D1 fermé par SIGTERM à la demande
utilisateur (fermeture normale difficile, à diagnostiquer), aucun SIGKILL.
Essai D2 console : audio non silencieux reçu, final STT non vide et HTTP200 LLM,
mais refus natif Texte attendu absent. D3 restreint les éditions sur note vide
à insert avec ancre native exacte ; 69 Rust ciblés PASS, build natif et relance
D3 PASS après fermeture D2 code0. Première dictée D3 PASS utilisateur et console
(insert, applied1/rejected0), arrêt et fermeture micro confirmés. Correction et pause PASS utilisateur ; console replace, applied2/rejected0,
puis commande pause et fermeture micro. Prochaine recette : arrière-plan,
session45903 ; aucun GO global.
Essai arrière-plan D3 FAIL : replace au lieu d’ajout confirmé utilisateur/console.
D4 renforce la conservation et l’ajout par défaut dans le prompt ; 69 Rust PASS,
build PASS, chargé en session5444 après fermeture D3 code0. Recette réelle
D4 : correction PASS utilisateur, deux idées fusionnées et conservées selon
son retour ; fidélité de nuance et changement de fenêtre non certifiés.
Critère de deux phrases distinctes trop restrictif pour S1.1/S1.3.
Recette suivante D4 fidélité/réserve en arrière-plan PASS utilisateur ; console
commit sans refus et fermeture micro, sortie code0. Relance même binaire
session4710. Verrouillage/déverrouillage PASS utilisateur : retour manuel,
interruption OS et fermeture micro confirmées, aucune auto-reprise observée.
Veille exclue de la recette locale à la demande utilisateur (non testable sur
cet ordinateur), non validée et sans blocage local. Prochaine recette :
enchaînement de trois passages distincts dans une même session vocale.
Cette recette D4 FAIL : réponses du service hors contrat/invariants refusées.
D5 corrige le message trompeur, ajoute diagnostics numériques sans contenu
et renforce le prompt ; 70 Rust PASS, build natif PASS, chargé session56340
après fermeture utilisateur code0. Cause précise non établie ; nouvel essai
enchaînement D5 FAIL à la deuxième idée : clarify avec deux actions et message
vide (diagnostic établi). D6 contraint ces combinaisons dans le schéma JSON,
validation native conservée ; 71 Rust et 18 cas JSON Schema PASS, build
natif PASS ; chargé session26421 après fermeture code0. Recette réelle
enchaînement D6 PASS utilisateur/console : schéma accepté, applied5, aucun
refus/clarification/erreur de contrat, pause et fermeture micro confirmées.
Application ouverte en pause session26421. Restent les autres gates trousseau/
fichiers/mesures ; veille exclue du périmètre local.
T006 partielle/T007 ouverte ; aucune tâche supplémentaire cochée, T011–T013
non commencées. [Preuves GNOME](../../docs/validation/prototype-gnome.md).
Les mentions GNOME différée des lots B03/B02 ci-dessous sont historiques.

**État autoritaire B03-OFFLINE, 2026-10-31 :** lot borné **T010 terminée**,
sans API/micro/recette réseau. Ports raccordés au contrôleur existant :
statut/interruption/commandes typés, `edit.operation` insert/replace/delete/move,
cibles `block_id` attribuées par le cœur, préconditions UTF-8 et résolution
sur copie de travail avant publication. Plus aucun offset fourni par le modèle.
Ambiguïté, cible multiple/opaque/absente, ancre périmée ou ordre terminal
invalide : zéro effet, aucun rejeu. U1/U2/U3 conservés.

- [X] Tests avant code : 11 tests initiaux, 9 rouges ; complétés à 15 tests
  opérations et 6 tests ports. Test de champs superflus sur commande sans
  paramètre rouge puis corrigé (variantes Serde structurées vides).
- [X] Validation finale source : 67 Rust (`--lib --tests`), 31 JS,
  typecheck, fmt/check Rust PASS. Windows : 9 Rust lib + 55 tests partagés
  dans un harnais example = **64 ciblés**, 31 JS, fmt et check lib MSVC
  custom-protocol PASS ; pas de total Windows de 67 revendiqué.
- [X] Synchronisation allowlist de 14 fichiers avec contrôle des différences,
  hashes anti-conflit et parité finale. Garde interdisant écriture/suppression
  de l'exe pendant les tests ; aucun arrêt/remplacement/lancement.

**Point d'arrêt explicite du lot :** T011–T013 NON COMMENCÉES, pas bloquées
par T010 ni par la seule dérogation T007. T011 reste le prochain lot TDD,
T012 attend T009+T011, T013 attend T012. Ne pas amorcer une implémentation
partielle de remplacement disque sans tests réels des courses/permissions
et de la branche Windows. Copies exclusives du prototype inchangées :
ni autosave contrôlé, ni watch/conflit/remplacement atomique annoncés.
`new/open/save_as/preferences/close` sont formalisées au port mais refusées
sans effet dans le parcours vocal jusqu'à leurs tâches d'intégration ; les
commandes manuelles existantes et l'historique acquitté restent inchangés.
T006 partielle, T007 ouverte, GNOME différée, aucun GO. La fenêtre existante
reste sur son ancien binaire ; ces modifications ne sont pas chargées.
Les états B02 suivants sont historiques. [Preuves actuelles](batch-report.yaml).

**État autoritaire B02-W05, 2026-10-31 :** accord ≤30s/fictif/coût possible
et nouvelle clé Credential Manager confirmés par l'utilisateur. Premier
passage vocal **FAIL**, aucun texte intégré ; ancien message de connexion
générique. Probe Windows WSS sans clé : `WrongHttpVersion`, réponse HTTP/1.0
incompatible ; catégorie du premier appel authentifié inconnue.
Diagnostic typé/expurgé corrigé, pas de TLS/proxy/fournisseur alternatif.
31 JS/51 Rust Linux PASS ; 31 JS/9 Rust lib Windows PASS, typecheck/build
frontend et build MSVC lib custom-protocol PASS. Synchronisation ciblée
sept fichiers PASS. Exe PID16144 conservé, **correctif non chargé** :
nouveau link attend sauvegarde/fermeture confirmée ; aucune capture/appel
authentifié supplémentaire. T006/T007 ouvertes, GNOME différée.

- [X] Enregistrer le premier échec STT utilisateur et distinguer les preuves
  de transport sans clé des hypothèses sur le compte.
- [X] Corriger les erreurs de handshake typées avec tests synthétiques de
  headers sensibles, redaction, HTTP1.0 et erreurs TLS encapsulées en IO.
- [X] Valider source et copie ciblée Windows sans toucher l'exe actif.
- [ ] Après confirmation de sauvegarde/fermeture, linker le diagnostic natif ;
  rétablir un handshake conforme sans clé avant nouvelle recette consentie.

Ces sous-travaux restent dans T006/T007. [Preuves et étape sûre](../../docs/validation/prototype-windows.md).

**État autoritaire B02-W04, 2026-10-31 à 02:23.** Les attentes W03/W02
ci-dessous sont historiques : build/link MSVC custom-protocol W03 PASS ;
recette manuelle Entrée/deux paragraphes/copie/nouvelle note **PASS utilisateur
à 02:20**, ancienne instance PID19624 terminée code0. Barre PASS à 01:57.
Préflight Windows `probe_lifecycle` PASS code0 sans micro/clé/API ;
permissions microphone lues Allow, pas preuve WASAPI. Fenêtre W03 corrigée
relancée sans doublon, PID16144/session détachée `windows-voice-recipe`,
titre Bloc-notes vocal, répondante et conservée, sans Vite.

- [X] Préparer la recette vocale graduelle et vérifier les prérequis Windows
  sans capture : WTS natif, permissions lecture seule, parité source/copie,
  messages exacts du contrôleur/trousseau/UI et lancement répondant.
- [X] Obtenir le consentement explicite micro ET appels Mistral/coût possible,
  puis retour utilisateur sur nouvelle clé masquée/Credential Manager.
- [ ] Exécuter par l'utilisateur le premier passage synthétique court,
  puis modification simple et arrêt/pause ; premier passage tenté FAIL W05,
  suite non exécutée, arrêt/Manuel à confirmer.

Ces sous-travaux appartiennent à T006/T007, sans nouvelle tâche de fondations.
T006/T007 restent décochées ; T008/T009 cochées, T010 partielle, GNOME
différée, aucun GO. Constitution inchangée. Voir le
[guide Windows actuel](../../docs/validation/prototype-windows.md).

**Fonctionnalité :** `001-bloc-notes-vocal` — **Date :** 2026-10-30.
**État au 2026-10-31 :** reprise B01-R3 : T001–T005 réalisées ;
T005 compilée/testée nativement après installation utilisateur des SDK.
T006 assemblée/compilée avec CPAL, mais partielle faute de vérification
réelle micro/services/trousseau/cycle OS ; aucun blocage SDK ALSA restant.
T007 NON EXÉCUTÉE. Lot B02 : T008–T009 réalisées ; T010 partielle
(ports raccordés, contrat complet encore à formaliser) ; T011–T060 non exécutées.
Preuves : [batch-report.yaml](batch-report.yaml) et
[checkpoint partiel](checkpoints/tasks-to-impl.yaml). Aucun GO desktop.
**B02-W01 :** copie Windows native préparée, 44 tests Rust/22 JS,
fmt/check/build MSVC et frontend PASS. Fenêtre native lancée
PID 25360, assets embarqués ; WTS initialisé sans capture.
T006/T007 restent décochées : recettes interactives micro/services/trousseau
et lock/suspend NON EXÉCUTÉES. GNOME différée, autres dépendances inchangées.
Node Windows 25.8.2 hors plage 22 signalé ; aucune installation Node/SDK.
Ancienne UI WSLg fermée, pas relancée. Guide :
[prototype-windows.md](../../docs/validation/prototype-windows.md).
**Racine :** `/home/laurent/projects/vocalnotes`, mode sans Git via [feature.json](../../.specify/feature.json).

**B02-W03 — bugfix concret T006/S3/S4.** Recette Windows FAIL :
Entrée/paragraphes/focus et commandes Fichier ; barre visuelle PASS à 01:57
conservée. Correctifs bornés du pont PM (insertions vides, sélection et
historique acquitté, file rapide) et diagnostics Fichier persistants/expurgés,
succès Nouvelle note explicite. Aucun autre lot commencé ni constitution modifiée.

- [X] Diagnostic racine et tests rouges : Entrée réelle PM, ack et clics Fichier.
- [X] Correctifs source et tests : 31 JS Linux/Windows, 46 Rust Linux/Windows,
  build/typecheck frontend, format et checks natifs PASS. Rust Windows :
  neuf artefacts de tests distincts, pas succès de `cargo test --tests` global.
- [X] Synchronisation allowlist de cinq fichiers ; garde Validate/Build contre
  exe actif vérifiée. Première validation Cargo Windows FAIL 101 sur tentative
  de retrait de l'exe verrouillé ; aucun arrêt, exe B02-W02 inchangé.
- [X] Build/link final application et relancement W03 après accord :
  MSVC custom-protocol PASS, instance PID19624 puis fermeture normale code0.
- [X] Recette interactive manuelle W03 deux paragraphes/save copie/new :
  utilisateur à 02:20 « OK, les tests sont concluants ». Pas recette vocale.

T006/T007 restent décochées, T008/T009 cochées, T010 partielle ; GNOME différée.
Preuves, limites et procédure sans micro : [recette Windows](../../docs/validation/prototype-windows.md).
Les mentions W02 de PID 25360/link en attente ci-dessous sont historiques :
le parent a depuis build/linké et lancé PID 19316 ; ne pas réutiliser
l'ancienne autorisation de fermeture pour cette nouvelle fenêtre.

**B02-W02 — correctif UI borné.** Barre DOM horizontale Fichier/Édition/Aide,
vrais boutons disclosure et panneaux superposés masqués par défaut ; un seul
ouvert, clic extérieur/action/Escape, focus et clavier natifs. Actions existantes,
champ chemin, trousseau et protections vocales conservés ; aucun fichier Rust modifié.
Source et copie Windows : parité SHA-256 des trois fichiers UI ; 25 tests JS,
typecheck/build frontend PASS sur les deux hôtes. Cargo check MSVC avec
assets embarqués PASS. **Link/build du nouvel exe et relancement NON EXÉCUTÉS** :
PID 25360 reste sur B02-W01, en attente de sauvegarde/fermeture propre confirmée.
Pas d'arrêt de fenêtre ni de helper. T006 partielle/T007 ouverte,
T008/T009 cochées et T010 partielle inchangées ; aucun lot fondations développé.

**Sources :** [plan final](plan.md), [spécification](spec.md), [recherche résolue](research.md),
[guide de validation](quickstart.md), [constitution actuelle 3.0.0](../../.specify/memory/constitution.md),
[traçabilité exigences → plan](checkpoints/spec-to-plan.yaml) et
[traçabilité plan → tâches](checkpoints/plan-to-tasks.yaml).
Le modèle local est [tasks-template.md](../../.specify/templates/tasks-template.md).
Les 60 tâches inline du plan sont conservées, sans renumérotation ni plan parallèle.
Le plan et le checkpoint exigences → plan décrivent leur étape antérieure ; les mentions de tâches
« inline seulement » y restent historiques, pas une interdiction du présent livrable.
Les interfaces et le modèle nécessaires sont dans le plan ; aucun `data-model.md`, `contracts/`,
knowledge graph ou artefact de migration n'est attendu pour commencer.

## Conventions et conditions d'exécution

### Décision utilisateur du 2026-10-31 : Windows en premier

L'utilisateur demande de tester d'abord sur son poste Windows 11, de différer
les essais GNOME/Wayland et de poursuivre les développements entre-temps.
Cette décision prévaut sur les interdictions de poursuivre lorsque seule
la recette GNOME manque. T007 reste ouverte : ses dépendances peuvent être
franchies pour le développement et les tests automatisés, sans la cocher ni
annoncer un GO multiplateforme. Les autres dépendances restent obligatoires.
Un défaut observé doit être corrigé ou explicitement circonscrit avant de
développer une fonctionnalité qui en dépend. Chaque recette distingue Windows
réussi/échoué/non exécuté et GNOME/Wayland non exécuté jusqu'aux essais réels.
Le report ne retire aucune exigence Linux et ne permet pas de certifier une
distribution ou un jalon multiplateforme avant la validation des deux cibles.

- Format : `- [ ] Tnnn [P?] [USn?] [Plan:X.Y] Description avec chemin exact`.
  `[USn]` désigne le parcours de la spécification, pas le numéro de phase ; aucun US5.
  `[P]` autorise seulement les groupes explicités plus bas, après leurs prérequis.
- Tous les chemins des tâches sont relatifs à la racine du projet. Les fichiers de code,
  fixtures et rapports sont **à créer**, sauf les sources documentaires existantes.
  Les dépendances inline sont les prédécesseurs directs, également inventoriés plus bas.
  L'ordre des IDs n'ajoute pas de dépendance implicite.
- Les tests déterministes Rust/Vitest et transports simulés restent exécutables hors ligne,
  sans compte, sans clé et sans facturation. Écrire les tests correspondants avant leur
  implémentation ; les tests ne certifient pas les résultats sémantiques d'un modèle.
- L'accès aux deux desktops interactifs, au micro et aux trousseaux réels est à établir.
  Tout contact API réel attend l'information sur les données envoyées, l'autorisation
  explicite de coût et les contrôles de compte/conditions. Une clé disponible ne vaut
  aucune autorisation. Sans tarif vérifié, afficher « inconnu », pas zéro ; pas de
  données sensibles sans conditions externes vérifiées.
- Aucun fichier de clé en clair ni pièce jointe de clé n'est une entrée des tâches.
  La seule configuration produit prévue est une future saisie masquée dans l'application,
  remise au trousseau natif ; aucun import permanent de fichier, variable d'environnement
  ou stockage de substitution en clair.
- Un essai desktop/API absent reste **NON EXÉCUTÉ**. Cela n'empêche pas de créer le socle,
  les tests ou le code nécessaires au prototype. En revanche T007, T026 et chaque
  jalon fonctionnel demandent un GO réel sur GNOME/Wayland **et** Windows 11 :
  ni mocks, ni navigateur, ni résultat d'un seul OS ne remplacent cette recette.
  Après un NO-GO, corriger ou réviser techniquement avant de franchir le jalon.

## Règles d'implémentation communes

Ces règles complètent les descriptions des tâches, sans ajouter de tâches ni changer leur périmètre.

**Pile retenue.** Tauri 2, Rust édition 2024, TypeScript 5/DOM sans framework UI supplémentaire,
Node 22 LTS/Vite, ProseMirror 1 ; versions de patch et lockfiles figés en T005.
Capture CPAL native, downmix/rubato hors callback vers PCM mono s16le 16 kHz, anneau mémoire
borné et VAD WebRTC local. Passages délimités naturellement, rotation `flush/end`, traitement
ordinaire uniquement sur `transcription.done` ; ni delta ni segment horodaté ne prouve la
finalité. Capture suivante continue et bornée pendant finalisation ; saturation explicite,
pas de spool. Le silence ne termine pas la session. Une voie prioritaire interprète les
deltas pour interrompre seulement, sans édition provisoire ni mot-clé rigide.

**Services et opérations.** Deux ports indépendants, transports natifs avec TLS vérifié.
Voxtral `voxtral-mini-transcribe-realtime-2602` et Mistral `mistral-large-latest`,
alias mutable dont le modèle servi reste à relever au premier essai autorisé.
Sortie `json_schema`, champ filaire `schema`, `strict=true`, propriétés obligatoires,
`additionalProperties=false` sur chaque objet. Intervention résolue : liste ordonnée d'actions
`edit`, `session_action`, `drafting_intent` (vide pour `no_action`) ; intervention ambiguë :
`clarify` avec message et sans action exécutable. Aucune variante undo/redo. Éditions `insert/replace/delete/move`,
cibles locales et préconditions vérifiées ; aucun outil arbitraire. Le contenu de note
est une donnée, jamais une instruction de confiance. Contexte envoyé limité au passage,
à l'intention/lexique utiles et aux sections candidates ; élargissement explicite au besoin.

Valider l'intervention entière avant tout effet. Une ambiguïté n'exécute aucune de ses
actions, même si certaines sont claires ; la signaler dans l'application. La précision
vocale est une nouvelle demande, sans reprise automatique d'actions suspendues.
Sinon respecter l'ordre, notamment ton → édition → sauvegarde. Un changement de note
est une barrière terminale ; une portée incertaine après cette barrière est ambiguë.
Les interruptions prioritaires restent indépendantes et peuvent arrêter/invalider,
sans appliquer de modification de contenu.

**Sessions.** Autorité native unique et contrôle atomique au commit :
identité de note, session, génération, opération, séquence, révision, portée et préconditions UTF-8.
Résultat d'une génération invalidée rejeté définitivement, même après reprise.
Résultat actif mais périmé : recalcul unique sur le courant ou conflit visible.
Pause : micro arrêté, génération invalidée, intention/lexique suspendus ; reprise locale
explicite sans écoute cachée de réveil. Arrêt/manuel/verrouillage/veille : fin et purge,
aucune auto-reprise. Nouvelle session autorisée à modifier intentionnellement le texte manuel.
Undo en mémoire uniquement durant la période manuelle, aucune pile vocale.
Une création, ouverture ou réouverture acceptée invalide tous les traitements précédents
avant le changement de note : génération incrémentée, files/passages précédents abandonnés,
résultats tardifs rejetés même si note ou révision identiques. Une ouverture en échec
laisse la note accessible sans réactiver les anciens traitements. Le mode vocal peut
rester actif pour les nouveaux passages.

**UI et intégration.** Une fenêtre inspirée de WordPad, menus DOM accessibles
Fichier/Édition/Aide, bandeau Markdown et démarrage/arrêt vocal, note visuelle centrale.
Un seul registre d'actions typées, porté par `src-tauri/src/app/commands.rs`, puis alimenté
par `voice_actions.rs` et `note_actions.rs` dans ce même répertoire : menus, bandeau et
intentions vocales appellent les mêmes opérations. ProseMirror en lecture seule et sans
history en vocal ; sélection/copie possibles, aucune saisie concurrente.
États et précisions **dans la fenêtre uniquement**, même masquée, consultables au retour ;
pas de tray, indicateur hors fenêtre, notification bureau, extension GNOME, TTS ou focus forcé.
L'écoute arrière-plan et la pause à voix restent actives. Les indicateurs propres à l'OS
ne sont ni ajoutés ni masqués par l'application.

**Intégrité.** Markdown canonique dans Rust, offsets `pulldown-cmark`, ProseMirror en projection.
Le cœur a priorité sur l'affichage (U2). Une proposition manuelle identifiée par transaction,
note, période manuelle et révision reçoit un acquittement `accepted`/`rejected` du cœur.
Une seule proposition est en vol ; les suivantes restent ordonnées. La projection locale
en attente n'est jamais présentée comme confirmée ou sauvegardée. Dédupliquer les
acquittements et ignorer les événements anciens ; un décalage impose un instantané du cœur.
Après refus ou divergence : signaler, abandonner la file non acceptée, reconstruire depuis
le cœur sans rejeu automatique et réinitialiser l'historique avec avertissement. La saisie
non acceptée reste récupérable séparément en mémoire pendant la période manuelle ;
signaler sa suppression si non récupérée à la fin de cette période. Undo/redo manuel
soumis aux mêmes contrôles, historique confirmé uniquement pour transactions acceptées,
sans double entrée lors d'un acquittement ou d'une projection. Changement de note ou
de mode : aucune proposition de l'ancienne période ne peut être reportée.
T039 relie transactions et plages source : patches ciblés, plus petite unité prise en charge
affectée, octets hors cible inchangés, CRLF/Unicode/espaces conservés. Tableaux/code/HTML et
autres constructions non représentables : îlots opaques, refus expliqué avant altération,
pas de sérialisation globale. Rendu sans HTML actif, protocole dangereux ni ressource distante.
Copie/export depuis la source brute. Une note unique, dirty protégé avant changement,
autosauvegarde après commit sérialisée avec manuel et fermeture.
Identité et empreinte disque revérifiées ; watcher indicatif seulement. Écriture voisine exclusive
de note, flush/sync, échange contrôlé Linux ou `ReplaceFileW` Windows avec version déplacée
conservée jusqu'au contrôle. En conflit, préserver les deux versions sans rollback destructeur ;
primitive indisponible = nouvelle copie exclusive et avertissement. Liens symboliques,
fichiers non ordinaires ou volumes non locaux : proposer une destination locale ordinaire.
Fermeture : arrêt/invalidation → sauvegarde → fin/sortie ; erreur ou destination absente
laisse la note accessible sans capture et permet une autre sauvegarde.

**Confidentialité et OS.** Audio/transcription/conversation/lexique et undo manuel temporaires ;
seule la note est du contenu durable, préférences non sensibles séparées.
Keyring 3.6.3 : `sync-secret-service`+`crypto-rust` GNOME, `windows-native` Windows,
`apple-native` pour l'objectif macOS ; backend réel, jamais mock en production.
Champ secret bref, commande IPC dédiée, champ vidé après envoi, aucun secret dans le store UI,
réponse native limitée à présence/succès/erreur expurgée. CSP, ressources embarquées, pas de
réseau webview ni API générique filesystem/shell. Limites swap/crash/fournisseur exposées,
sans promesse d'effacement physique. GNOME : ScreenSaver et login1 ; Windows : WTS et
WM_POWERBROADCAST. Perte de surveillance = arrêt sûr/erreur, manuel utilisable.

## Phase 1 — Setup et premier prototype (7 tâches)

**Clôture courante :** T006/T007 terminées pour GNOME/Wayland dans le périmètre
utilisateur documenté en tête et dans le bilan final. Les paragraphes B01-R3
qui suivent décrivent l’état historique ; ils ne rouvrent pas ces tâches.
Les gates deux OS restent nécessaires pour une future annonce multiplateforme.


**Reprise B01-R3 (2026-10-31).** ALSA **1.2.11** détecté après installation
utilisateur ; CPAL compile sans changement système par l'agent. Contrôleur
continu capture→VAD→WSS/rotation→LLM→commit raccordé, tampon suivant borné,
priorité sémantique coalescée, annulation et enveloppe native
note/session/génération/requête/opération/passage/séquence/révision.
GNOME ScreenSaver/login1 et récepteur caché Windows WTS/alimentation ajoutés ;
Windows vérifié seulement par compilation isolée des types API **sur Linux**.
Accord micro/Mistral contextualisé, pause/reprise explicite, structure simple,
undo, sauvegarde ton→édition→copie exclusive et fermeture coordonnée raccordés.
Le premier X arrête/invalide et suspend la fermeture ; Fichier → Fermer après
sauvegarde attend les acquittements UI, avertit sur la saisie non acceptée,
sauvegarde l'accepté puis détruit la fenêtre.

**Final : 22 tests JS, 32 tests Rust, typecheck/build web,
cargo check --tests/build/fmt PASS.** Contrôle de types Windows = 0 test runtime.
Sondage OS réel sans micro/clé : `cargo run --example probe_lifecycle`
**code 2**, surveillance indisponible WSL2. T006 reste décochée :
qualité/finalité/rotation/pause rapide, vrais micros/trousseaux et événements
lock/suspend non vérifiés sur les cibles. Copie exclusive seulement ; pont
complet/remplacement conditionnel et création/ouverture à voix non annoncés.
T007 NON EXÉCUTÉE sur les deux OS ; ses rapports consignent cette absence.
Aucune T008+ engagée. **0 appel Mistral, 0 micro démarré, clé racine non lue**.

**But :** reprendre les décisions arrêtées, créer un socle compilable puis une tranche
réelle réutilisable, pas seulement des mocks.
**Vérification indépendante :** T007 installe et exerce cette tranche sur les deux desktops,
avec comptes rendus séparés et mesures. Aucun essai n'est déjà réussi.

- [X] T001 [P] [Plan:1.1] Relire les décisions de ce plan et consigner leur prise en compte dans `specs/001-bloc-notes-vocal/design-review.md`, dont Q01 résolue, UI WordPad et Tauri 2 retenu, sans redemander leur validation ; dépend de : aucune.
- [X] T002 [P] [Plan:1.2] Préparer `docs/validation/provider-check.md` à partir de R02/R03/R09 : modèles retenus, PCM mono 16 kHz, WSS, json_schema strict et test synthétique explicitement informé ; au premier essai autorisé relever catalogue du compte, modèle réellement servi, tarif officiel daté si accessible et conditions applicables, sinon afficher inconnues et exclure données sensibles ; dépend de : aucune.
- [X] T003 [P] [Plan:1.3] Inventorier dans `docs/validation/environments.md` l'accès à GNOME/Wayland Fedora et Windows 11, matériel, micro, trousseau, versions et formats de distribution, puis planifier macOS ; dépend de : aucune ; aucun desktop complet présumé.
- [X] T004 [Plan:1.4] Revoir dans `specs/001-bloc-notes-vocal/design-review.md` les types et interfaces inline avant leur traduction en code : barrière de commit, schéma strict, pont Markdown et conflits, secrets et frontières de session ; définir les assertions de prototype, sans attendre ni produire d'artefact de design autonome ; dépend de : T001, T002, T003.
- [X] T005 [Plan:1.5] Créer `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/lib.rs`, `package.json`, `rust-toolchain.toml` et verrouillages pour la pile retenue, scripts `tauri`/`test:unit` avec Vitest, cibles RPM/NSIS et capacités minimales ; figer les patches réellement résolus, sans initialiser Git ; dépend de : T004.
- [X] T006 [Plan:1.6] **Clôture locale GNOME/Wayland** — Réaliser une tranche verticale réutilisable dans `src-tauri/src/platform/probe.rs` et `src/ui/Prototype.ts` : capture CPAL, normalisation/VAD/rotation WSS, LLM strict, génération et interruption prioritaire, ProseMirror visuel en manuel/read-only vocal, préservation d'un bloc opaque, trousseau, cycle OS et sauvegarde ; menus et boutons WordPad minimaux, aucune notification ni cache de contenu ; dépend de : T005 ; ne pas annoncer compatible avant T007.
- [X] T007 [Plan:1.6] **Clôture locale GNOME/Wayland, périmètre réduit par l’utilisateur** — Recetter la tranche prototype dans `docs/validation/prototype-gnome-final.md` et `docs/validation/prototype-metrics.md` (Windows et veille exclus ; aucun paquet certifié) : G02–G09, passage finalisé sans perte à la rotation, pause naturelle rapide, édition visuelle/protection opaque et invariants transversaux ; dépend de : T006 ; NO-GO exige correction ou révision technique avant stories, un essai absent reste NON EXÉCUTÉ.

**Point d'arrêt B01.** T005 : manifestes, entrées Tauri/DOM, capacités minimales,
RPM/NSIS configurés, versions exactes, deux lockfiles et 12 tests frontend réels
créés. `npm test`, `test:unit`, `typecheck`, `build`, `cargo fmt --check` et
métadonnées Cargo passent. `cargo check --locked` échoue dans libdbus-sys ;
`cargo test --locked --lib` dans GLib/GObject (glib-sys initialement,
gobject-sys au dernier essai) : exécutable pkg-config absent,
dépendances natives non disponibles. Le diagnostic Tauri indique aussi WebKitGTK
4.1/rsvg2 manquants. Aucun test Rust exécuté, aucun binaire natif validé.
T005 reste décochée, donc aucun début de T006, ni contournement par mocks,
feature sans desktop ou shim. Aucun paquet système installé sans autorisation.
T007 reste NON EXÉCUTÉ : aucun prototype assemblé, aucun des deux desktops
recetté. L'autorisation Mistral synthétique/coût reçue pendant le lot n'a pas
été utilisée (0 appel, clé non lue) ; tarifs/modèles servis restent inconnus.

**Reprise B01-R1 (2026-10-30).** Validations rejouées : 12/12 tests JS,
typecheck/build frontend, format Rust et metadata verrouillée OK.
Check Rust échoue encore dans libdbus-sys ; tests bloqués dans GDK/GIO/GLib
avant tout test. Diagnostic complémentaire : bibliothèques runtime présentes
mais en-têtes et fichiers `.pc` absents, aucun SDK local établi.
Simulation APT T005 : 96 nouveaux paquets et 12 mises à jour (GLib/Mesa),
0 suppression ; installation non exécutée, autorisation explicite attendue.
Commande bornée et preuves dans `docs/validation/environments.md#reprise-t005--b01-r1-2026-10-30`.
Aucune case supplémentaire cochée, T006/T007 et T008+ inchangées.

**Reprise B01-R2 (2026-10-30).** Les SDK installés par l'utilisateur sont
détectés ; T005 passe check/test/build natifs, format, 12 tests JS et
typecheck/build frontend. **T005 cochée** sur ces preuves, pas sur l'existence
du scaffold. T006 commencée avec tests avant code : canonique/patches/ack,
barrières, ambiguïté sans effet, éditeur/blocs opaques/récupération mémoire,
keyring réel, transports Mistral natifs et downmix/rubato/VAD.
Avant CPAL : **17 tests Rust, check et build natifs PASS**.
Frontend final : **19 tests, typecheck et build PASS**.
Au raccordement réel CPAL : `cargo check` échoue dans `alsa-sys 0.3.1`,
`alsa.pc` absent ; capture nouvelle **non compilée**. Format et metadata
verrouillées finaux PASS. Aucune installation système par l'agent.
ALSA dev : simulation seulement, 1 nouveau paquet, 0 mise à jour/suppression ;
autorisation ou installation utilisateur attendue.

T006 reste décochée : assemblage continu/rotation/priorité et cycle OS
manquants ; édition limitée aux blocs existants, sauvegarde exclusive en
nouvelle copie seulement. Le bouton vocal est explicitement indisponible,
aucun succès simulé. Les 17 tests pré-CPAL ne certifient pas l'arbre final.
T007 et T008+ restent non exécutées. Micro/trousseau/API non exercés,
clé locale jamais lue, constitution identique (SHA-256 dans rapport).

**Précisions d'exécution T005–T007.** T005 comprend le scaffold complet :
`src-tauri/build.rs`, `src-tauri/src/main.rs`, `src/main.ts`, `index.html`,
`tsconfig.json`, `vite.config.ts`, `src-tauri/capabilities/main.json`,
`Cargo.lock` dans `src-tauri/` et `package-lock.json` à la racine.
Prévoir les commandes de compilation initiale et de test dans `package.json`,
vérifier le build initial avant d'enchaîner sur les tests et le prototype.
Ne pas tenter les commandes de quickstart avant existence de leurs manifestes/tests.
T006 écrit les assertions de prototype avant les composants correspondants dans
`src-tauri/tests/prototype_contracts.rs` et `src/ui/Prototype.test.ts` (données synthétiques,
sans compte) et assemble réellement la capture, les services, l'éditeur et les fichiers.
Ces précisions appartiennent aux mêmes IDs ; elles ne créent pas un lot de remplacement.
T007 réalise les essais desktop/API autorisés, pas T002 ni les tests déterministes.

## Phase 2 — Fondations transversales (19 tâches)

**But :** consolider les composants réutilisables du prototype et les invariants partagés.
**Entrée initiale :** prototype GO T007 ; dérogation utilisateur du 2026-10-31
appliquée au développement/tests B02 seulement. **Vérification indépendante :**
T026, matrice des deux OS, toujours obligatoire.
Les tests de domaines distincts peuvent être écrits en parallèle, pas les stories avant T026.

**Lot B02 — 2026-10-31.** Interface native Linux lancée sous WSLg, sans
inspection visuelle distante ni recette GNOME/Windows ; session détachée
`wslg-ui`, processus application `126037`, Vite `126008`, HTTP 200.
T008 : 7 tests écrits d'abord (échec attendu avant domaine), puis verts.
T009 : `Note` canonique extraite depuis le prototype, `Session`/`Ticket`
privés pour les identités, barrière native unique et consommation des conflits
de révision sans rejeu ; erreurs visibles. Façade `platform/probe.rs` sans
seconde implémentation, contrôleur/UI existants conservés.
T010 partielle : ports STT/LLM indépendants utilisés par le contrôleur,
finalité et schéma strict/validation complète réutilisés, 5 tests nouveaux.
Reste à formaliser le contrat complet d'édition/cibles locales
`insert/replace/delete/move` et actions note/session : le contrat filaire
reste celui du prototype (`edit.patch`, pause/stop/save terminal).
Ne pas considérer cette allowlist bornée comme toutes les actions du plan,
ni franchir les dépendances T010 comme si elle était terminée.
T011–T013 non commencées : priorité T008–T010 respectée.
Final : 44 tests Rust et 22 JS, cargo fmt/check/build et frontend
typecheck/build PASS. Pas de mesure de couverture chiffrée ni recette réelle
micro/services/keyring. Vocal toujours refusé faute surveillance OS.
Application lancée avant ce lot avec `--no-watch` : processus B01-R3 conservé
pour ne pas interrompre l'utilisateur ; code B02 compilé/testé, pas relancé.

- [X] T008 [P] [Plan:2.1] Écrire d'abord de petits tests dans `src-tauri/tests/session_validity.rs` : anciennes sessions, reprise, révision périmée, doublons, ordre, pause, résultat retardé après création/ouverture/réouverture de note avec contenus et révisions identiques, et aucune pile undo vocale ; dépend de : T007.
- [X] T009 [Plan:2.1] Implémenter les entités Note/Session/Traitement et la barrière de commit dans `src-tauri/src/domain/session.rs` et `src-tauri/src/domain/note.rs`, avec noteId vérifié, génération monotone, invalidation de tous les traitements précédents au changement de note et nettoyage aux frontières ; dépend de : T008.
- [X] T010 [Plan:2.1] Formaliser les ports STT/LLM et le schéma strict inline en types validés dans `src-tauri/src/ports/stt.rs` et `src-tauri/src/ports/interpreter.rs`, puis réutiliser les composants de prototype sans duplication ; dépend de : T009 ; terminée B03-OFFLINE : opérations/cibles/commandes typées, résolution native sur copie de travail, garde entière et tests Linux/MSVC ; intégration des actions fichiers complète reste T024, pas annoncée par le contrat seul.
- [X] T011 [P] [Plan:2.2] Écrire `src-tauri/tests/storage_integrity.rs` : octets hors cible, version disque, disque plein, droits, interruption et course externe avant remplacement ; dépend de : T007.
- [X] T012 [Plan:2.2] Implémenter la note unique, ouverture/destination, sauvegarde automatique contrôlée et conflit non destructif dans `src-tauri/src/storage/markdown_file.rs` ; dépend de : T009, T011.
- [X] T013 [Plan:2.2] Implémenter l'ordre arrêt/invalidation → sauvegarde → fin/sortie et l'échec sans fermeture silencieuse dans `src-tauri/src/app/close.rs` ; dépend de : T012.
- [X] T014 [P] [Plan:2.3] Écrire `src-tauri/tests/ipc_boundary.rs`, `src/ui/markdown.test.ts` et `src/ui/DesktopShell.test.ts` pour contenu hostile, chemins, HTML/protocoles, images distantes, absence de cible fictive et présence des menus Fichier/Édition/Aide et du bandeau de mise en forme et de démarrage/arrêt vocal (S4.5) ; dépend de : T007.
- [X] T015 [Plan:2.3] Implémenter commandes IPC allowlistées et événements d'état français sans données secrètes dans `src-tauri/src/app/commands.rs` et `src-tauri/capabilities/main.json` ; dépend de : T009, T012, T014.
- [X] T016 [Plan:2.3] Créer la fenêtre simple inspirée de WordPad dans `src/ui/DesktopShell.ts`, les menus Fichier/Édition/Aide dans `src/ui/MenuBar.ts`, le bandeau de mise en forme et démarrage/arrêt vocal dans `src/ui/Toolbar.ts`, la note mise en forme et les états lisibles sans vol de focus dans `src/ui/NoteView.ts` et `src/ui/StatusView.ts` ; relier les commandes aux actions partagées à mesure de leur implémentation, sans dupliquer leur logique ni autoriser l'édition manuelle pendant le vocal ; dépend de : T015.
- [X] T017 [P] [Plan:2.4] Écrire `src-tauri/tests/secrets_policy.rs` pour création/remplacement/suppression, commande de saisie dédiée, absence de clé dans état durable UI/config/logs et trousseau indisponible sans fallback ; dépend de : T007.
- [X] T018 [Plan:2.4] Intégrer keyring 3.6.3 avec backends Secret Service/Credential Manager explicites et commande IPC de secret masqué à durée de vie courte dans `src-tauri/src/platform/secrets.rs`, rejeter le backend mock en production ; dépend de : T017.
- [X] T019 [Plan:2.4] Implémenter préférences non sensibles, présence des clés, choix STT/LLM séparés et prise d'effet à la session suivante dans `src-tauri/src/config/preferences.rs` ; dépend de : T010, T018.
- [X] T020 [Plan:2.5] Écrire `src-tauri/tests/provider_contracts.rs` avec transport simulé : auth expurgée, segments/corrections, JSON invalide, erreur réseau, annulation et absence de fournisseur de secours ; dépend de : T010.
- [X] T021 [P] [Plan:2.5] Intégrer capture CPAL, downmix/rubato 16 kHz, VAD et passages WSS dans `src-tauri/src/audio/capture.rs` et `src-tauri/src/adapters/mistral_stt.rs` : modèle retenu, bornes mémoire et erreur de saturation, réconciliation done sans édition ordinaire sur delta ni réémission indue ; dépend de : T019, T020.
- [X] T022 [P] [Plan:2.5] Intégrer `mistral-large-latest`, schéma strict validé localement, voie prioritaire limitée à l'interruption et compteurs de requêtes dans `src-tauri/src/adapters/mistral_interpreter.rs`, sans outils arbitraires ni conversation persistante ; dépend de : T019, T020.
- [ ] T023 [Plan:2.6] Écrire `src-tauri/tests/lifecycle.rs` avec capture simulée : lancement explicite, silence sans arrêt, pause, lock/sleep, fermeture, pertes micro/service et événements OS indisponibles ; dépend de : T009.
- [ ] T024 [Plan:2.6] Implémenter abonnement OS vérifié et invalidation immédiate dans `src-tauri/src/platform/lifecycle.rs`, sans reprise automatique au retour ; dépend de : T013, T021, T023.
- [ ] T025 [Plan:2.6] Afficher l'état courant d'écoute, les états français et l'avertissement tiers dans `src/ui/SessionStatus.ts` uniquement, sans indicateur hors fenêtre ni notification du bureau ; relier la pause vocale rapide et garder l'état consultable au retour dans l'application ; dépend de : T016, T022, T024.
- [ ] T026 [Plan:2.7] Rejouer la matrice desktop et mesures dans `docs/validation/foundations-gnome.md`, `docs/validation/foundations-windows.md` et `docs/validation/foundations-metrics.md` ; dépend de : T016, T019, T025 ; un échec bloque les stories.

## Phase 3 — US1 : construire une note fidèle (P1, 5 tâches)

**But :** rédaction structurée progressive, intention libre, fidélité et perception des changements.
**Test indépendant :** note vide, récit français avec hésitations/chiffres/réserves, forme puis
intention modifiée ; relire et rouvrir le fichier. S1.1–S1.4, quatre usages synthétiques,
aucun fait ajouté accepté, traitement distinct du commit, pas de saut de navigation.

- [ ] T027 [US1] [Plan:3.1] Créer corpus synthétique et tests de décisions/nuances dans `tests/fixtures/fidelity.json` et `src-tauri/tests/drafting.rs`, sur les quatre usages sans prétendre garantir tout énoncé ; dépend de : T026.
- [ ] T028 [US1] [Plan:3.1] Modéliser intention, forme, ton et contexte minimal temporaire dans `src-tauri/src/domain/intention.rs` sans les insérer automatiquement dans la note ; dépend de : T027.
- [ ] T029 [US1] [Plan:3.1] Implémenter orchestration de rédaction progressive sur informations fournies dans `src-tauri/src/app/drafting.rs` et classifier erreurs STT/LLM/application ; dépend de : T028.
- [ ] T030 [US1] [Plan:3.2] Afficher traitement distinct du commit, zones réelles, déplacements/suppressions perceptibles et navigation préservée dans `src/ui/ChangeView.ts` ; dépend de : T029.
- [ ] T031 [US1] [Plan:3.2] Valider S1 et mesurer sur deux desktops dans `docs/validation/us1-gnome.md`, `docs/validation/us1-windows.md`, `docs/validation/us1-metrics.md` ; dépend de : T030.

## Phase 4 — US2 : piloter et corriger naturellement (P1, 6 tâches)

**But :** opérations naturelles ciblées, clarification, épellation et actions de note/session.
**Test indépendant :** note préparée, modifications à la voix dont une ambiguë et une épelée ;
S2.1–S2.5, intentions ordonnées, récit impératif non exécuté, clarification interne consultable
après masquage, correction ciblée et refus expliqué de l'undo vocal sans interdire la correction.

- [ ] T032 [US2] [Plan:4.1] Écrire `src-tauri/tests/voice_operations.rs` : ton puis édition puis sauvegarde, ambiguïté au début ou à la fin sans aucun effet de l'intervention, absence de reprise automatique, interruption prioritaire, récit impératif, contenu hostile, portée, correction contextuelle et refus de l'undo vocal ; dépend de : T031.
- [ ] T033 [US2] [Plan:4.1] Implémenter dans `src-tauri/src/app/operations.rs` l'enveloppe ordonnée d'intentions et sa validation complète avant effet ; en cas d'ambiguïté ne rien exécuter et signaler, sinon respecter l'ordre et les barrières de changement de note, sans confirmation ordinaire ni historique vocal ; dépend de : T032.
- [ ] T034 [US2] [Plan:4.2] Implémenter lexique éphémère, épellation casse/accents/tiret/apostrophe et tests ciblé/global dans `src-tauri/src/domain/lexicon.rs` et `src-tauri/tests/spelling.rs` ; dépend de : T033.
- [ ] T035 [US2] [Plan:4.2] Implémenter signalement d'ambiguïté sans action de l'intervention et précision vocale comme nouvelle demande dans `src-tauri/src/app/clarification.rs`, affichée dans `src/ui/ClarificationView.ts` uniquement, sans reprise automatique, notification du bureau, lecture vocale, focus imposé, cible aléatoire ni épellation insérée par défaut ; dépend de : T034.
- [ ] T036 [US2] [Plan:4.3] Relier intentions naturelles aux actions note/session/préférences dans `src-tauri/src/app/voice_actions.rs`, y compris destination/nom, retrouver une note, pause/arrêt/manuel/fermeture, hors configuration de secrets et autorisations OS ; dépend de : T035.
- [ ] T037 [US2] [Plan:4.3] Recetter S2, actions vocales et interruptions transversales dans `docs/validation/us2-gnome.md`, `docs/validation/us2-windows.md`, `docs/validation/us2-metrics.md`, notamment ordre ton/édition/sauvegarde et zéro effet d'une intervention ambiguë ; dépend de : T036.

## Phase 5 — US3 : manuel puis reprise vocale (P1, 4 tâches)

**But :** éditeur visuel manuel exclusif avec annulation mémoire, puis reprise explicite.
**Test indépendant :** retarder une réponse, passer en manuel, saisir/annuler, reprendre et
reformuler intentionnellement le texte manuel. S3.1–S3.4 : ancien résultat sans effet avant
et après reprise ; résultat actif périmé réévalué ou signalé ; historique non restauré.

- [ ] T038 [US3] [Plan:5.1] Écrire `src-tauri/tests/mode_handoff.rs` et `src/ui/ManualEditor.test.ts` : résultat tardif avant/après reprise, undo manuel éphémère, saisies rapprochées, acceptation/refus natif, acquittement dupliqué, événement retardé, resynchronisation sur le cœur, récupération temporaire de saisie refusée, formatage puis undo et undo après refus sans réintroduire la transaction rejetée ; dépend de : T037.
- [ ] T039 [US3] [Plan:5.1] Intégrer ProseMirror visuel, bandeau Markdown et undo mémoire uniquement en manuel dans `src/ui/ManualEditor.ts`, et pont de patches/source dans `src/ui/MarkdownBridge.ts` avec contrôle natif dans `src-tauri/src/domain/note.rs` ; implémenter transactionId/noteId/période/révision, acquittement accepted/rejected, envoi ordonné, priorité du cœur et récupération canonique sans rejeu automatique, historique confirmé et reset signalé après refus ; tester dans `src/ui/MarkdownBridge.test.ts` l'égalité UI/source après stabilisation, octets hors cible inchangés, opaque non éditable et consultation read-only sans historique vocal ; dépend de : T038.
- [ ] T040 [US3] [Plan:5.1] Intégrer transfert immédiat, signalement du non-intégré, reprise explicite et modification intentionnelle du contenu manuel dans `src-tauri/src/app/mode_handoff.rs` ; dépend de : T039.
- [ ] T041 [US3] [Plan:5.1] Recetter S3 et SC-004 dans `docs/validation/us3-gnome.md`, `docs/validation/us3-windows.md`, `docs/validation/us3-metrics.md` ; dépend de : T040.

## Phase 6 — US4 : fichiers locaux intègres (P1, 3 tâches)

**But :** parcours complet d'une seule note Markdown, source préservée et conflits non destructifs.
**Test indépendant :** fichiers synthétiques incluant tableaux/code/Unicode/CRLF, actions de
note en vocal et manuel, modification externe puis échec d'écriture. S4.1–S4.5 : aucun
écrasement silencieux, dernière sauvegarde réellement retrouvable, menus et bandeau présents.

- [ ] T042 [US4] [Plan:6.1] Intégrer créer/ouvrir/retrouver/enregistrer/enregistrer sous/copier/exporter en vocal et manuel, avec protection de la note sale et invalidation de tous les traitements précédents avant changement de note, dans `src/ui/NoteActions.ts` et `src-tauri/src/app/note_actions.rs` ; dépend de : T041.
- [ ] T043 [US4] [Plan:6.2] Ajouter fixtures Markdown unsupported/Unicode/CRLF et essais d'échec/conflit externe dans `tests/fixtures/preservation.md` et `src-tauri/tests/file_workflows.rs` ; vérifier aussi les résultats retardés après création/ouverture/réouverture avec mêmes contenus et révisions, et après échec d'ouverture, sans modification tardive d'aucune note ; dépend de : T042.
- [ ] T044 [US4] [Plan:6.2] Exécuter S4.1–S4.5 et les recettes disque plein, fermeture sans destination, conflit pendant sauvegarde et interruption dans `docs/validation/us4-gnome.md`, `docs/validation/us4-windows.md`, `docs/validation/us4-metrics.md` ; dépend de : T043.

## Phase 7 — US6 : services, secrets et coûts (P1, 5 tâches)

**But :** préférences transparentes, clés au trousseau et estimation qualifiée.
**Test indépendant :** ouvrir les préférences, saisir/remplacer/supprimer un secret d'essai,
tester explicitement une configuration valide/invalide après information et autorisation,
puis observer session et panne. S6.1–S6.5 : choix séparés, pas de secret révélé ni de fallback,
réglages à la prochaine session et coûts mesurés/estimés/inconnus distincts.

- [ ] T045 [US6] [Plan:7.1] Intégrer panneau FR, modèles intégrés et accès vérifié distincts, champ secret masqué vidé après IPC, présence/suppression des clés et test explicite informé dans `src/ui/PreferencesView.ts` ; dépend de : T044 ; pas d'OpenAI intégré affiché.
- [ ] T046 [US6] [Plan:7.2] Écrire `src-tauri/tests/usage_estimates.rs` pour composantes STT/LLM, tarifs absents/périmés, métriques manquantes et différence estimation/facture ; dépend de : T045.
- [ ] T047 [US6] [Plan:7.2] Implémenter compteurs mémoire, source/date de tarif et inconnues explicites dans `src-tauri/src/domain/usage.rs` et `src/ui/UsageView.ts` ; dépend de : T046.
- [ ] T048 [US6] [Plan:7.2] Documenter dans `docs/privacy.md` et l'écran `src/ui/PrivacyView.ts` les envois minimaux, conditions vérifiées, limites externes et absence de bascule cachée ; dépend de : T047.
- [ ] T049 [US6] [Plan:7.2] Exécuter S6 et recherche de traces avec secrets synthétiques dans `docs/validation/us6-gnome.md`, `docs/validation/us6-windows.md`, `docs/validation/us6-metrics.md` ; dépend de : T048.

## Phase 8 — US8 : portabilité et mesures (P1, 2 tâches)

**But :** consolider la portabilité déjà vérifiée à chaque jalon, sans la différer ici.
**Test indépendant :** même recette sur GNOME/Wayland et Windows 11 au prototype et aux
jalons ; S8.1–S8.3, rapports datés, conditions/mesures et limites, macOS objectif séparé
non confirmé tant qu'il n'a pas ses propres résultats.

- [ ] T050 [US8] [Plan:8.1] Préparer `docs/validation/macos-status.md` : moyens macOS, Keychain, capture et même recette ; laisser objectif non confirmé si non exécuté, ne pas annoncer un support ; dépend de : T049.
- [ ] T051 [US8] [Plan:8.1] Consolider sans remplacer les recettes par jalon dans `docs/validation/p1-platforms.md` les deux desktops, quatre usages, mesures et limites, avec statut séparé macOS ; dépend de : T050.

## Phase 9 — US7 : diagnostic temporaire (P2, 3 tâches)

**But :** inspection opt-in de reconnaissance, sans nouvel historique.
**Test indépendant :** session sans diagnostic puis session avec diagnostic et correction,
fin puis nouvelle session. S7.1–S7.3 : défaut désactivé, provisoire/final/corrections
qualifiés selon le service, vue/contexte/lexique vidés, seules corrections de note conservées.

- [ ] T052 [US7] [Plan:9.1] Écrire `src-tauri/tests/diagnostic_privacy.rs` : défaut désactivé, aucune console persistante, corrections/provisoire/final si exposés, purge et nouvelle session vide ; dépend de : T051.
- [ ] T053 [US7] [Plan:9.1] Implémenter opt-in par session, avertissement, segments qualifiés et vidage dans `src/ui/TranscriptView.ts` et `src-tauri/src/app/diagnostics.rs`, sans enregistrement audio ; dépend de : T052.
- [ ] T054 [US7] [Plan:9.1] Exécuter S7 puis matrice transversale dans `docs/validation/us7-gnome.md`, `docs/validation/us7-windows.md`, `docs/validation/us7-metrics.md` ; dépend de : T053.

## Phase 10 — Validation finale et sobriété (6 tâches)

**But :** stress, confidentialité, fidélité, recette complète et décision de livraison.
**Vérification indépendante :** toutes les preuves des jalons, 29 scénarios et SC-001 à
SC-007, mêmes fonctions sur les deux OS ; aucune annonce macOS sans vérification.

- [ ] T055 [Plan:10.1] Ajouter stress déterministe de courses, transport retardé, doublons et fautes d'écriture dans `src-tauri/tests/fault_matrix.rs` sans élargir le produit ; dépend de : T054.
- [ ] T056 [Plan:10.1] Vérifier fichiers/cache/temp/logs/crash et limites OS/fournisseur dans `docs/validation/privacy-audit.md`, sans conserver de vrai secret ou contenu personnel dans la preuve ; dépend de : T055.
- [ ] T057 [Plan:10.2] Évaluer quatre usages, hésitations, chiffres, accents, noms, longues interventions, ambiguïtés et épellation dans `docs/validation/fidelity.md`, en séparant STT/LLM/application et sans fait ajouté accepté ; dépend de : T056.
- [ ] T058 [Plan:10.2] Exécuter toute la matrice SC-001 à SC-007 sur les deux desktops dans `docs/validation/final-gnome.md`, `docs/validation/final-windows.md`, `docs/validation/final-metrics.md` ; dépend de : T057.
- [ ] T059 [Plan:10.3] Proposer après mesures seulement des objectifs chiffrés à décision utilisateur et mettre à jour le guide `specs/001-bloc-notes-vocal/quickstart.md` avec les commandes réellement validées ; dépend de : T058.
- [ ] T060 [Plan:10.3] Vérifier couverture, contrats, preuves par jalon et constitution dans `docs/validation/release-gates.md` ; marquer ERROR si une gate reste ouverte, macOS non vérifié restant objectif sans annonce ; dépend de : T059.

## Dépendances et ordre d'exécution

### Inventaire des prédécesseurs directs

Cet inventaire reproduit exactement les dépendances inline et le checkpoint ; aucun
numéro seul, ordre d'affichage, accès compte supposé ou document de design absent n'ajoute
une arête. Les conditions de passage GO/NO-GO décrites plus haut restent applicables.

| Tâche | Prédécesseurs directs |
|---|---|
| T001 | aucun |
| T002 | aucun |
| T003 | aucun |
| T004 | T001, T002, T003 |
| T005 | T004 |
| T006 | T005 |
| T007 | T006 |
| T008 | T007 |
| T009 | T008 |
| T010 | T009 |
| T011 | T007 |
| T012 | T009, T011 |
| T013 | T012 |
| T014 | T007 |
| T015 | T009, T012, T014 |
| T016 | T015 |
| T017 | T007 |
| T018 | T017 |
| T019 | T010, T018 |
| T020 | T010 |
| T021 | T019, T020 |
| T022 | T019, T020 |
| T023 | T009 |
| T024 | T013, T021, T023 |
| T025 | T016, T022, T024 |
| T026 | T016, T019, T025 |
| T027 | T026 |
| T028 | T027 |
| T029 | T028 |
| T030 | T029 |
| T031 | T030 |
| T032 | T031 |
| T033 | T032 |
| T034 | T033 |
| T035 | T034 |
| T036 | T035 |
| T037 | T036 |
| T038 | T037 |
| T039 | T038 |
| T040 | T039 |
| T041 | T040 |
| T042 | T041 |
| T043 | T042 |
| T044 | T043 |
| T045 | T044 |
| T046 | T045 |
| T047 | T046 |
| T048 | T047 |
| T049 | T048 |
| T050 | T049 |
| T051 | T050 |
| T052 | T051 |
| T053 | T052 |
| T054 | T053 |
| T055 | T054 |
| T056 | T055 |
| T057 | T056 |
| T058 | T057 |
| T059 | T058 |
| T060 | T059 |

### Graphe des phases et parcours

```text
{T001,T002,T003} → T004 → T005 → T006 → T007 (GO deux OS)
  → fondations T008–T025, branches selon inventaire → T026 (GO deux OS)
  → US1 T027–T031 → US2 T032–T037 → US3 T038–T041
  → US4 T042–T044 → US6 T045–T049 → US8 T050–T051
  → US7 T052–T054 → transversal T055–T060
```

Les parcours sont volontairement séquentiels dans le plan final. « Test indépendant »
signifie une recette ciblée à partir d'une note/session préparée, pas une autorisation
de supprimer leurs prérequis. US8 P1 précède US7 P2. À chaque jalon, rejouer les invariants
arrière-plan/pause, états internes, sessions, lock/sleep, trousseau, fichiers/fermeture et
mesures sur les deux OS, selon [quickstart](quickstart.md#3-recette-obligatoire-à-chaque-jalon-dès-le-premier-prototype).

**Parcours critique structurel :** sans durées mesurées, il n'existe pas encore de chemin
critique temporel. En comptant une unité par tâche, une chaîne la plus longue est :
T001 → T004 → T005 → T006 → T007 → T008 → T009 → T010 → T019 → T021 →
T024 → T025 → T026 → chaque tâche de T027 à T060 dans l'ordre (47 tâches).
T002 ou T003 peut remplacer T001 ; T020 peut remplacer T019 sur cette chaîne.
Les autres branches, notamment stockage, UI et secrets, rejoignent les barrières de
l'inventaire et ne peuvent pas être omises. Les disponibilités des deux OS et les
autorisations d'essai peuvent devenir le facteur limitant réel.

## Parallélisme réel

**9 tâches marquées `[P]`, trois groupes sûrs :**

| Précondition satisfaite | Tâches concurrentes | Isolation |
|---|---|---|
| Aucune | T001, T002, T003 | Revue, contrôle fournisseur et inventaire OS dans trois fichiers différents |
| T007 GO | T008, T011, T014, T017 | Tests session, fichiers, IPC/UI et secrets dans des fichiers distincts |
| T019 et T020 terminées | T021, T022 | Adaptateurs STT/audio et LLM distincts ; ports déjà stabilisés |

Dans ces groupes, chaque tâche attend bien ses prédécesseurs avant lancement.
Les intégrations communes, manifestes, exports de modules et lockfiles restent à un
responsable unique ; les tâches concurrentes ne les modifient pas simultanément.
Aucun `[P]` n'est ajouté aux stories : leur graphe est une chaîne.

### Exemples par parcours

- **US1 :** T027 → T028 → T029 → T030 → T031 ; pas de parallèle entre tâches.
  Au sein de T031, après le même build, essais GNOME et Windows possibles simultanément
  sur deux hôtes, chacun dans son rapport ; mesures consolidées ensuite par un seul rédacteur.
- **US2 :** T032 → T033 → T034 → T035 → T036 → T037 ; même séparation des deux
  hôtes/rapports pour T037, pas d'écriture concurrente du rapport de mesures.
- **US3 :** T038 → T039 → T040 → T041 ; essais deux OS de T041 parallèles seulement
  après intégration du pont et du transfert. T039 modifie `note.rs` : pas de travail
  concurrent sur cette autorité de mutation.
- **US4 :** T042 → T043 → T044 ; essais deux OS possibles en parallèle dans T044,
  fichiers synthétiques et répertoires d'essai distincts, consolidation ultérieure.
- **US6 :** T045 → T046 → T047 → T048 → T049 ; essais deux OS de T049 parallèles
  après accord d'essai, trousseaux distincts et budget total explicité, jamais deux
  tests payants implicites sous prétexte de parallélisme.
- **US8 :** T050 → T051 ; pas de parallèle interne. T051 rassemble les résultats
  GNOME/Windows déjà produits ; il ne remplace ni ne répète arbitrairement leurs essais.
- **US7 :** T052 → T053 → T054 ; essais deux OS de T054 parallèles avec sessions
  synthétiques séparées ; compte rendu de mesures consolidé après.

## Traçabilité tâches → items → exigences

La table ci-dessous reprend les 28 items et leurs exigences du plan et des checkpoints.
Chaque tâche porte un item existant ; l'union couvre exactement REQ-001 à REQ-053.
Une association est une obligation d'implémentation/recette, pas une preuve déjà obtenue.
Les preuves attendues par exigence restent détaillées dans la table finale du plan.

| Item | Tâches | Exigences |
|---|---|---|
| 1.1 | T001 | REQ-029, REQ-050 |
| 1.2 | T002 | REQ-001, REQ-039, REQ-042, REQ-045 |
| 1.3 | T003 | REQ-050, REQ-051, REQ-052 |
| 1.4 | T004 | REQ-017, REQ-022, REQ-024, REQ-039, REQ-041 |
| 1.5 | T005 | REQ-039, REQ-050 |
| 1.6 | T006, T007 | REQ-028, REQ-029, REQ-032, REQ-033, REQ-035, REQ-036, REQ-037, REQ-041, REQ-050, REQ-052 |
| 2.1 | T008, T009, T010 | REQ-005, REQ-015, REQ-016, REQ-017, REQ-018, REQ-019, REQ-032, REQ-039, REQ-046 |
| 2.2 | T011, T012, T013 | REQ-020, REQ-022, REQ-023, REQ-024, REQ-036, REQ-037 |
| 2.3 | T014, T015, T016 | REQ-004, REQ-021, REQ-022, REQ-026, REQ-027, REQ-030 |
| 2.4 | T017, T018, T019 | REQ-038, REQ-039, REQ-040, REQ-041, REQ-042, REQ-043, REQ-045 |
| 2.5 | T020, T021, T022 | REQ-001, REQ-033, REQ-039, REQ-043, REQ-045, REQ-048 |
| 2.6 | T023, T024, T025 | REQ-013, REQ-028, REQ-029, REQ-030, REQ-031, REQ-032, REQ-033, REQ-034, REQ-035, REQ-036 |
| 2.7 | T026 | REQ-050, REQ-052 |
| 3.1 | T027, T028, T029 | REQ-001, REQ-002, REQ-003, REQ-010, REQ-053 |
| 3.2 | T030, T031 | REQ-025, REQ-026, REQ-027, REQ-050, REQ-052 |
| 4.1 | T032, T033 | REQ-004, REQ-005, REQ-006, REQ-007, REQ-008, REQ-009, REQ-015 |
| 4.2 | T034, T035 | REQ-007, REQ-011, REQ-012, REQ-029, REQ-046 |
| 4.3 | T036, T037 | REQ-013, REQ-020, REQ-029, REQ-031, REQ-032, REQ-038, REQ-050, REQ-052 |
| 5.1 | T038, T039, T040, T041 | REQ-014, REQ-015, REQ-016, REQ-017, REQ-018, REQ-019, REQ-021, REQ-022, REQ-050, REQ-052 |
| 6.1 | T042 | REQ-017, REQ-020, REQ-021, REQ-023, REQ-024 |
| 6.2 | T043, T044 | REQ-017, REQ-022, REQ-023, REQ-024, REQ-036, REQ-037, REQ-050, REQ-052 |
| 7.1 | T045 | REQ-038, REQ-039, REQ-040, REQ-041, REQ-042, REQ-043 |
| 7.2 | T046, T047, T048, T049 | REQ-044, REQ-045, REQ-046, REQ-050, REQ-052 |
| 8.1 | T050, T051 | REQ-050, REQ-051, REQ-052, REQ-053 |
| 9.1 | T052, T053, T054 | REQ-046, REQ-047, REQ-048, REQ-049, REQ-050, REQ-052 |
| 10.1 | T055, T056 | REQ-017, REQ-024, REQ-035, REQ-036, REQ-037, REQ-041, REQ-046, REQ-049 |
| 10.2 | T057, T058 | REQ-002, REQ-007, REQ-050, REQ-051, REQ-052, REQ-053 |
| 10.3 | T059, T060 | REQ-039, REQ-045, REQ-050, REQ-051, REQ-052 |

## Couverture des 29 scénarios

Les IDs désignent les scénarios exacts de [spec.md](spec.md), non de nouvelles exigences.
Chaque ligne associe préparation/implémentation et jalon de recette sur les deux OS.
T058 rejoue l'ensemble en fin de parcours ; toutes les recettes restent non exécutées.

| Scénario | Critère observable condensé | Tâches de réalisation | Tâche de recette |
|---|---|---|---|
| S1.1 | Français continu, structure progressive, chiffres/nuances fidèles | T021, T027, T028, T029 | T031 |
| S1.2 | Quatre formes/tons sans intention insérée ni faits inventés | T027, T028, T029 | T031 |
| S1.3 | Intention tardive, ajout/développement sans perte ni invention | T027, T028, T029 | T031 |
| S1.4 | Attente/commit distincts, vraie zone, navigation et lisibilité | T014, T016, T030 | T031 |
| S2.1 | Intentions ordonnées, portée ciblée, opérations naturelles perceptibles | T008, T009, T030, T032, T033 | T037 |
| S2.2 | Impératifs rapportés et contenu hostile restent des données | T014, T032, T033 | T037 |
| S2.3 | Ambiguïté précisée à voix, affichée seulement dans l'application | T033, T035 | T037 |
| S2.4 | Épellation précise, lexique temporaire, portée non abusive | T034, T035 | T037 |
| S2.5 | Refus undo/redo vocal, correction de contenu toujours possible | T008, T032, T033 | T037 |
| S3.1 | Arrêt/invalidation immédiats, manuel disponible, non-intégré signalé | T038, T039, T040 | T041 |
| S3.2 | Saisie et undo manuel exclusifs, historique éphémère | T038, T039 | T041 |
| S3.3 | Reprise explicite sur texte manuel, ancien résultat sans effet | T008, T009, T038, T040 | T041 |
| S3.4 | Résultat actif périmé réévalué/signalé, sans doublon | T008, T009, T038, T040 | T041 |
| S4.1 | Actions de note unique et Markdown autonome en vocal/manuel | T012, T036, T042 | T044 |
| S4.2 | Rendu lisible, unsupported protégé avant altération | T011, T014, T039, T043 | T044 |
| S4.3 | Conflits et dirty protégés sans écrasement silencieux | T011, T012, T042, T043 | T044 |
| S4.4 | Dernière sauvegarde réelle retrouvable après interruption | T012, T042, T043 | T044 |
| S4.5 | Menus Fichier/Édition/Aide et bandeau, modes respectés | T014, T016, T039, T042 | T044 |
| S6.1 | Préférences à voix, services/données transparents, choix séparés | T019, T036, T045, T048 | T049 |
| S6.2 | Saisie/remplacement/suppression masqués, trousseau ou erreur | T017, T018, T045 | T049 |
| S6.3 | Test explicite informé, résultat expurgé | T002, T019, T045 | T049 |
| S6.4 | Aucun fournisseur caché, réglages à session suivante | T019, T020, T021, T022, T045 | T049 |
| S6.5 | Coût STT/LLM estimé, mesures/inconnues distinctes | T046, T047 | T049 |
| S7.1 | Diagnostic inactif par défaut, pas de transcription persistante | T052, T053 | T054 |
| S7.2 | Opt-in sensible, provisoire/final/corrections qualifiés | T020, T021, T052, T053 | T054 |
| S7.3 | Fin/reprise sans audio/contexte/lexique/undo antérieurs | T009, T034, T039, T052, T053 | T054 |
| S8.1 | Deux desktops pour chaque jalon et invariants transversaux | T003, T005, T006, T024, T051 | T007, T026, T031, T037, T041, T044, T049, T051, T054, T058 |
| S8.2 | macOS objectif sans support annoncé avant recette | T003, T050 | T051, T060 |
| S8.3 | Conditions et mesures démarrage/ressources/délais/coût | T002, T003, T051, T059 | T007, T026, T031, T037, T041, T044, T049, T051, T054, T058 |

**Critères de réussite de la spécification :** SC-001 : T027–T037 et T057 ;
SC-002 : T032–T041 ; SC-003 : T011–T013, T039 et T042–T044 ;
SC-004 : T007, T023–T026, T038–T041 et T055 ;
SC-005 : T017–T022, T045–T049 et T052–T056 ;
SC-006 : tous les jalons deux OS et T050–T051 ;
SC-007 : mesures à tous les jalons, T030–T031, T046–T047 et T059.
T058 recouvre SC-001 à SC-007 ; T060 décide de la livraison d'après les preuves.

## Stratégie d'implémentation — première tranche et progression

1. **Socle puis prototype utilisable T001–T007.** Préparer documentation et environnements
   sans appel payant, créer/build le socle T005, écrire les assertions puis assembler T006.
   Une note réelle peut être rédigée, affichée, modifiée manuellement après arrêt et
   sauvegardée avec les protections minimales ; capture/services/trousseau/cycle OS réels
   sont vérifiés en T007 après autorisation. Ce n'est pas une livraison uniquement mockée.
2. **Premier incrément fonctionnel US1 : T001–T031**, fondations et GO des deux OS compris.
   La tranche permet de raconter, structurer, consulter et retrouver une note fidèle.
   Elle n'est ni toute la V1 ni une dispense de l'exclusivité des modes ou de l'intégrité.
   Le prototype apporte le minimum bout en bout ; les tâches US3/US4 étendent et éprouvent
   ensuite l'éditeur complet et les parcours de fichiers, sans différer leurs invariants.
3. **Incréments suivants** : US2 → US3 → US4 → US6 → US8 (P1), puis US7 (P2),
   chacun recetté sur les deux OS avant le suivant. Aucune redéfinition des exigences
   sous le nom de MVP ; toutes restent nécessaires pour la version complète.
4. **Livraison** après T055–T060 et preuves finales. Les objectifs chiffrés attendent les
   mesures et une décision ; macOS reste objectif non confirmé sans recette propre.

## Bilan documentaire et passage à l'étape suivante

**60 tâches**, toutes décochées : setup 7, fondations 19, US1 5, US2 6, US3 4,
US4 3, US6 5, US8 2, US7 3, transversal 6.
**28 items, 53 exigences, 29 scénarios, 7 critères SC**, aucun US5/S5.
Les 9 marqueurs de parallélisme n'altèrent aucune dépendance.
Les descriptions inline et IDs du plan sont repris ; les précisions de scaffold/tests
du prototype explicitent T005/T006, sans ajouter de tâche ni déplacer un GO/NO-GO.
G10 reste la revue interne T004, pas une attente d'artefacts externes.

Le checkpoint plan → tâches est actualisé pour viser ce fichier, sans prétendre qu'un
test produit ou un essai API/desktop a été exécuté. Les contrôles formels portent sur
le format, la concordance inline/checkpoint, les références et le DAG, pas sur le produit.
La préservation des sources est vérifiée par empreintes des seuls documents nommés,
jamais de fichier de secret. Les chemins applicatifs et rapports restent attendus,
non des liens vers des preuves présentes. Aucun marqueur de cadrage non résolu n'est
introduit ; accès deux OS, compte, tarifs et conditions restent des prérequis d'essai
explicites, pas des succès supposés.

**Prochaine commande : `/speckit.analyze`**, avant `/speckit.implement`.

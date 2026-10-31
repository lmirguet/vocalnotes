# Plan d'implémentation — Bloc-notes vocal intelligent

**État autoritaire B03-OFFLINE, 2026-10-31 :** T010 terminée ; 67 tests Rust
Linux et 64 Rust Windows ciblés (9 lib + 55 tests partagés), 31 JS sur chaque
hôte, typecheck Linux, fmt/check Linux et check lib MSVC embarqué PASS.
Contrat strict désormais sans offsets LLM : `edit.operation` typée et
`block_id` native, préconditions résolues sur copie de travail. Même autorité
Note/Ticket, mêmes actions manuelles/undo ; aucun changement constitutionnel.
T011–T013 non commencées, prochain lot stockage TDD avec leurs dépendances
inchangées ; arrêt à une unité complète, pas blocage artificiel T010/T007.
Les commandes fichiers vocales complètes attendent leur intégration T024 :
contrat formalisé ne vaut pas effet implémenté. Copie exclusive actuelle
conservée, aucune garantie nouvelle d'atomicité/watch/conflicts.
Exécutable Windows inchangé et non relinké, aucune API/micro/recette réseau.
T006 partielle, T007 ouverte, GNOME différée ; aucun GO desktop. Les états
B02 ci-dessous sont historiques. [Rapport actuel](batch-report.yaml).

**État autoritaire B02-W05 :** premier passage vocal Windows utilisateur FAIL
après accord ≤30s/coût possible et nouvelle clé confirmée. WSS Windows sans
clé : réponse HTTP/1.0 (`WrongHttpVersion`), compte/statut/authenticated
root cause inconnus. Correctif diagnostic typé uniquement, sans changement
de protocole/TLS/fournisseur. 31 JS/51 Rust Linux, 31 JS/9 Rust lib Windows,
typecheck/build frontend et build MSVC lib custom-protocol PASS ; nouveau
link exe non exécuté, PID16144 conservé jusqu'à fermeture/sauvegarde
confirmée. T006/T007 ouvertes, GNOME différée ; aucune replanification.
Les états W04 ci-dessous sont historiques :
[preuves et prochaine étape sûre](../../docs/validation/prototype-windows.md).

**État actuel B02-W04, 2026-10-31 à 02:23 — aucune replanification.**
Build/link MSVC custom-protocol W03 PASS ; manuel Windows Entrée/copie/nouvelle
note PASS utilisateur à 02:20, barre PASS à 01:57. Ancienne instance terminée
code0. Préflight Windows actuel WTS PASS code0, permissions micro lues Allow,
sans capture/trousseau/API. Fenêtre corrigée PID16144 répondante, conservée
en session détachée `windows-voice-recipe`, pas de Vite.
Recette vocale graduelle préparée dans
[prototype-windows.md](../../docs/validation/prototype-windows.md) :
consentement micro ET API encore absent, nouvelle clé masquée puis trousseau
par l'utilisateur, essai synthétique court seulement après accord.
Aucun défaut préflight imposant correctif, aucun autre lot commencé.
T006/T007 ouvertes, T010 partielle, GNOME différée, aucun GO ; constitution
inchangée. Les mentions « link/recette W03 en attente » ci-dessous sont historiques.

**Date :** 2026-10-30  
**Fonctionnalité :** `001-bloc-notes-vocal`  
**Branche Git :** aucune. La valeur `BRANCH=001-bloc-notes-vocal` renvoyée par setup est l'identifiant issu du répertoire, pas une branche créée.  
**Entrées :** [spécification actuelle](spec.md), [checklist](checklists/requirements.md), [constitution 3.0.0](../../.specify/memory/constitution.md), [recherche](research.md).  
**Statut au 2026-10-31 : reprise B01-R3 ; T001–T005 réalisées, scaffold T005
compilé/testé nativement et frontend vérifié. T006 assemblée/compilée avec CPAL,
22 tests JS/32 tests Rust PASS mais runtime cible non vérifié ; T007 NON EXÉCUTÉE ;
aucun GO desktop. Lot B02 : T008–T009 réalisées, T010 partielle,
T011+ non exécutées.**
Voir [rapport d'exécution](batch-report.yaml) et [tasks.md](tasks.md).

**B02-W03 — correction T006, pas nouvelle planification.**
Après retour Windows FAIL Entrée/focus/actions Fichier, conservation de l'état
PM sur ack source/révision identiques, insertions vides explicites et patches
UTF-8 du seul segment modifié ; historique acquitté et resynchronisation
sur refus U2 conservés, barrières U1/U3 inchangées. Diagnostics natifs Fichier
statiques allowlistés et persistants, succès Nouvelle note explicite.
31 tests JS et build frontend sur Linux/Windows, 46 tests Rust sur les deux
hôtes PASS ; check MSVC lib avec assets embarqués PASS. Validation Cargo globale
Windows FAIL sur exe actif verrouillé ; garde ajoutée au script.
Instance B02-W02 PID 19316 conservée : build/link application et recette W03
attendent récupération du texte/accord de fermeture, aucun GO desktop.
La barre visuelle W02 confirmée à 01:57 reste PASS ; les états antérieurs
ci-dessous sont historiques. T006/T007/T010 et constitution inchangées.

**B02-W02 — correctif de REQ-021/S4.5, dans T006 uniquement.**
Barre DOM horizontale à boutons disclosure, panneaux superposés exclusifs ;
Tab/Enter/Espace natifs, flèches/Home/End sur les intitulés, Escape avec retour
du focus et clic extérieur sans vol de focus. Pas de rôles menu/menuitem
inadaptés au champ chemin et au texte d'aide ; aucun menu OS/framework ajouté.
Actions et Rust inchangés. 25 tests JS et build frontend Linux/Windows PASS ;
check MSVC embarqué PASS, **nouveau link/build et rendu WebView2 en attente**.
L'ancienne fenêtre B02-W01 reste active jusqu'à fermeture propre confirmée.
T006/T007/T010 restent partielles/ouvertes, constitution inchangée.

**B02-W01 — Windows natif, 2026-10-31.** Copie exclusive utilisateur
`C:\Users\lmirguet\Development\vocalnotes-native-B02-W01`, sans secrets/données
ou sorties Linux. Rust 1.99.1 MSVC/VS Build Tools 18.11.3/SDK 10.0.26100.0
réels : 44 tests Rust/22 JS et fmt/check/build PASS ; mêmes validations
source Linux PASS. Ressources embarquées, fenêtre Windows PID 25360
avec WebView2, WTS initialisé ; gestes/micro/keyring/services et événements
OS non recettés. T006/T007 ouvertes, GNOME différée.
Node Windows 25.8.2 hors plage Node 22 : écart documenté, pile cible inchangée.
Aucun nouveau lot fondations commencé, T010 toujours partielle.
Ancienne instance WSLg désormais fermée ; description active B02 ci-dessous
historique. [Commandes et recette](../../docs/validation/prototype-windows.md).

**B02 — 2026-10-31.** Dérogation de développement T007 appliquée, pas
de changement de dépendances. `domain/note.rs` réutilise l'autorité canonique
du prototype ; `domain/session.rs` porte Session/Ticket et consommation
atomique au commit sous mutex. Conflit périmé visible, pas de rejeu.
Ports `ports/stt.rs`/`ports/interpreter.rs` extraits et raccordés : T010
reste partielle car le schéma conserve `edit.patch` et pause/stop/save
terminal du prototype ; contrat complet cibles/opérations/actions encore
à formaliser avant clôture T010. T011–T013 non engagées.
44 tests Rust, 22 JS et compilations PASS. Application native Linux WSLg
active, pas recette Windows ou GNOME ; protections vocales inchangées.

**Décision utilisateur du 2026-10-31 — recette échelonnée.** Tester Windows 11
en premier ; GNOME/Wayland sera testé ultérieurement. La poursuite du
développement et des tests automatisés est autorisée malgré cette recette
différée, y compris les tâches dépendant du gate T007, sans déclarer T007
terminée. Cette dérogation prévaut sur les blocages de développement dus à la
seule absence de recette GNOME ; les autres dépendances et les corrections
des défauts observés restent obligatoires. Les exigences des deux OS et le GO
multiplateforme restent inchangés. Consigner séparément les résultats Windows
et les essais GNOME non exécutés ; aucune certification des deux cibles avant
leur validation réelle.

## Résumé

Application desktop francophone : rédaction et modifications à la voix naturelle, sans syntaxe rigide ni confirmations ordinaires ; une note Markdown locale, deux modes exclusifs et des services externes explicitement choisis. Le manuel est un éditeur classique accessible après arrêt vocal, avec undo éphémère. Aucun historique, pile ou commande undo/redo vocale. La reprise vocale peut modifier intentionnellement le contenu manuel, jamais par un ancien résultat invalidé.

Choix d'ingénierie retenu : **Tauri 2, Rust, TypeScript avec DOM natif, ProseMirror 1 pour l'édition visuelle**, capture native CPAL et transports STT/LLM natifs. Mistral : `voxtral-mini-transcribe-realtime-2602` et `mistral-large-latest` avec `json_schema` strict. Ce dernier est un **identifiant API documenté mais un alias mutable**, pas une preuve de « Large 4 » ni une version figée. Le modèle effectivement servi sera relevé au premier essai autorisé ; aucun accès compte n'est prétendu testé. Les arbitrages et alternatives sont justifiés dans [research.md](research.md).

La suppression du parcours 5 et de ses scénarios n'entraîne aucun retrait fonctionnel : REQ-028 à REQ-037, arrière-plan et interruption restent couverts. Les parcours 1–4 et 6–8 et les 53 identifiants d'exigences sont inchangés.

## Contexte technique

| Élément | Proposition / état de preuve |
|---|---|
| Langages/versions | Rust édition 2024, toolchain stable compatible Tauri 2 ; TypeScript 5, Node 22 LTS, Vite pour compiler les ressources. T005 fige les versions de patch réellement résolues dans les lockfiles et rust-toolchain.toml ; aucun paquet installé ici. |
| Dépendances | Tauri 2 ; ProseMirror 1 (state/view/model/commands/history/markdown), DOM TypeScript sans React/Vue ; CPAL ; Tokio, reqwest et tokio-tungstenite avec TLS vérifié ; serde et validation JSON Schema ; pulldown-cmark avec offsets source ; keyring 3.6.3 avec backends explicitement activés. |
| Stockage | Note Markdown locale uniquement comme contenu ; préférences non sensibles et chemin de dernière note ; clés au trousseau ; contexte/audio/lexique/transcription/undo manuel en mémoire. Aucune base de données requise. |
| Tests | cargo test, Vitest pour TypeScript et petits tests de propriétés des patches ; recettes desktop sur deux systèmes. Ni navigateur headless ni mocks ne valident le desktop. |
| Cibles | GNOME/Wayland, notamment Fedora, et Windows 11 dès le premier prototype et chaque jalon ; macOS objectif V1 non certifié. Hôte Linux distant : disponibilité GNOME complet et Windows 11 non établie. |
| Type et portée | Greenfield desktop, une personne et une note ouverte ; pas d'architecture existante à migrer, pas de serveur applicatif ni synchronisation. |
| Performance | Mesurer démarrage, mémoire/CPU, temps STT/LLM/application et bout en bout, coûts et conditions ; aucun seuil, prix ou résultat inventé. |
| Contraintes | Pas de capture avant démarrage explicite ; pas d'auto-reprise ; pas de fournisseur caché ; Markdown autonome ; erreurs visibles sans perte silencieuse. |

### Frontière du workflow

Projet greenfield : aucun knowledge graph ni groupe de migration n'est nécessaire. Le modèle et les interfaces utiles sont définis **dans ce plan**, puis traduits en types et tests lors du premier lot. Aucun artefact de design autonome n'est livré ou attendu comme prérequis fantôme. Aucun `tasks.md` n'est créé avant `/speckit.tasks`. T004 est une revue interne bornée, pas une nouvelle demande de cadrage.

### Architecture retenue et règles d'application

Interface retenue : fenêtre de traitement de texte simple inspirée de WordPad,
menus « Fichier », « Édition », « Aide », bandeau compact de mise en forme et de
démarrage/arrêt du vocal, note mise en forme au centre. Les commandes visuelles
complètent la voix sans remettre en cause les deux modes exclusifs. La mise en forme
reste limitée aux possibilités Markdown prévues ; aucune suite bureautique complète
n'est introduite. L'état d'écoute et les précisions restent internes à la fenêtre.

Flux retenu : micro natif → tampon mémoire borné → port STT → ordonnanceur d'intentions → port LLM → validation locale d'opérations → autorité de mutation → note source/rendu → sauvegarde contrôlée. Les services sont sollicités uniquement avec les données nécessaires et choisies ; une note contenant des instructions hostiles reste une donnée.

Une seule autorité native applique les mutations. Chaque résultat porte identité de note, session, génération, opération, séquence et révision de base. Avant application : même note active, session toujours autorisée, génération valide, opération non déjà appliquée, portée et préconditions vérifiées. Toute génération invalidée reste rejetée ; seule une opération encore active mais périmée sur la même note peut être recalculée ou signalée. Sérialiser les opérations évite duplication et écrasement. Le contrôle de validité se répète au point de commit, pas seulement au départ d'une requête.

Le mode manuel n'a pas d'audio concurrent. Pause invalide les traitements en vol sans finir implicitement toute la session ; arrêt/manuel/verrouillage/veille détruisent son contexte. Reprise toujours explicite. Le silence peut déclencher une proposition, jamais un arrêt autonome. En fermeture : arrêter/invalider, sauvegarder, puis finir/sortir ; sans destination ou en erreur, conserver la note accessible sans micro, permettre sauvegarde ou autre destination.

Préservation Markdown : source canonique, rendu dérivé, patches ciblés ; garder les octets hors cible, détecter les zones non prises en charge avant altération. Rendu sans HTML actif, protocoles dangereux ni ressources distantes automatiques. Ne pas confondre un rendu sûr avec une sérialisation sans perte.

Configuration : choix STT/LLM séparés et figés pour une session ; changements à la prochaine session explicite. Saisie masquée dans le panneau local des préférences : le secret traverse une seule commande IPC native dédiée, sans entrer dans le store d'UI, puis le champ et sa référence sont vidés. Le cœur ne renvoie que présence/succès/erreur expurgée. Une boîte native supplémentaire n'est pas nécessaire au besoin et est écartée. Trousseau indisponible = erreur, jamais fichier clair ni clé dans un log.

### Modèle minimal et interfaces internes

Ces décisions sont les entrées de T004 et des tâches suivantes, pas des fichiers applicatifs déjà présents.

| Objet | Champs et invariants |
|---|---|
| Note | id, source UTF-8 canonique, revision monotone, destination facultative, dirty, empreinte et identité disque connues, blocs avec plages d'octets. Une seule note active ; aucun droit distinct selon origine manuelle/vocale. |
| Session | id unique, generation monotone, état, configuration STT/LLM figée, intention, lexique, contexte récent borné. Aucun journal d'événements ni pile d'annulation vocale. |
| Traitement | requestId, noteId, sessionId, generation, passageId, sequence, baseRevision, état et operationId. Un commit atomique vérifie l'identité de la note active et tous ces champs ; identifiants dédupliqués en mémoire pour la génération courante. |
| Passage | audio transitoire, transcription provisoire/finalisée, bornes temporelles si exposées, statut intégré/non intégré. Données libérées dès qu'inutiles ; jamais spoulées sur disque. |
| Manuel | état ProseMirror et historique undo mémoire pour la seule période manuelle courante ; détruit à la reprise vocale et à la fermeture. |
| Préférences | fournisseurs/modèles séparés, chemin dernière note et réglages non sensibles ; références de trousseau, jamais clés. Pas de base de données. |
| Consommation | durée audio envoyée, tokens rapportés, estimation, prix/source/date facultatifs, composantes inconnues. Compteurs de session sans contenu durable. |

**Session et transitions.** Démarrage en manuel sans capture. « Démarrer vocal » est une action volontaire unique par bouton ou commande accessible ; ensuite aucun geste ni mot-clé requis par intervention. En écoute, les demandes naturelles peuvent ouvrir une note, changer son nom/destination, ouvrir les préférences, corriger, mettre en pause, arrêter ou passer en manuel. Une destination ambiguë appelle une précision affichée et répondable oralement ; aucun chemin n'est exécuté comme commande système.

États natifs : `Manual`, `Listening`, `Paused`, `Closing`, `Error`. Traitement et clarification sont des sous-états visibles de `Listening`, pas des modes autorisant une saisie concurrente. La pause arrête le micro, invalide la génération et abandonne les traitements non appliqués, mais conserve intention/lexique de la session suspendue. La reprise depuis une capture arrêtée exige une action explicite locale ; **pas d'écoute cachée pour un mot de réveil**. Arrêt, passage manuel, verrouillage et veille terminent la session, vident son contexte et ne reprennent pas automatiquement. La fermeture invalide et arrête, sauvegarde puis détruit et sort ; échec = note accessible sans capture.

**Port STT.** Entrée : flux mono PCM signé 16 bits little-endian à **16 000 Hz** normalisé depuis le périphérique. CPAL assure la capture ; downmix et rééchantillonnage hors callback audio avec rubato. Callback sans réseau ni blocage, anneau mémoire borné ; saturation ou perte micro = erreur visible et invalidation, pas de réémission silencieuse. Connexion native `wss://api.mistral.ai/v1/audio/transcriptions/realtime?model=voxtral-mini-transcribe-realtime-2602`, en-tête `Authorization: Bearer …`, jamais de secret dans l'URL.

Les messages `session.update`, `input_audio.append` (base64), `input_audio.flush`, `input_audio.end` et les événements `session.created/updated`, `transcription.text.delta`, `transcription.segment`, `transcription.done`, erreur suivent le SDK officiel cité dans la recherche. Un delta est du texte en progression, un segment horodaté **n'est pas une garantie documentée de finalité**. `done` donne le texte de fin du flux, pas une intention d'édition.

**Fin de passage choisie.** Pour ne pas attendre la fin d'une longue session et ne pas inventer une garantie de finalité, le client utilise une détection locale parole/silence (WebRTC VAD via crate Rust) pour délimiter automatiquement des passages audio ; un silence ne termine jamais la session. À frontière naturelle, terminer le flux du passage (`flush/end`), continuer la capture et borner en mémoire le passage suivant pendant réception du `done`. Le texte `done` est le seul envoyé à l'interpréteur pour mutation ordinaire ; fragments incomplets demandent continuation, pas faits inventés. Le découpage forcé d'un long passage dépend des limites effectivement constatées du service et conserve un statut « suite attendue ». Pas de seuil commercial de latence inventé : réglage technique VAD à calibrer au prototype, pas décision fonctionnelle utilisateur.

Une voie prioritaire détecte dans les deltas une **intention naturelle d'interruption** par le même interpréteur, sans commande rigide : elle ne peut qu'arrêter/invalider, jamais éditer la note sur provisoire. Pas de double facturation cachée : toutes requêtes et durées sont comptées. Le compromis faux arrêt/rapidité et la rotation de flux sans perte doivent réussir au prototype, sinon NO-GO et révision technique. Une API de tour non documentée n'est pas supposée disponible.

**Port interprétation.** HTTP natif `POST https://api.mistral.ai/v1/chat/completions`, modèle `mistral-large-latest`, messages sans conversation hébergée, `response_format.type=json_schema`, `strict=true`, propriétés obligatoires et `additionalProperties=false` sur chaque objet. Le champ filaire est `schema` ; `schema_definition` est le nom Python du SDK. Réponse complète validée avant application, jamais JSON partiel streamé. Le schéma implémenté dans `src-tauri/src/ports/interpreter.rs` décrit soit une intervention résolue avec une liste ordonnée d'actions `edit`, `session_action`, `drafting_intent` (liste vide pour `no_action`), soit une intervention ambiguë `clarify` avec message et sans action exécutable. Aucune variante undo/redo. Actions allowlistées seulement, aucun outil/shell arbitraire.

**Intentions multiples et ambiguïté.** Préparer et valider l'intervention entière avant tout effet. Une ambiguïté d'intention, de cible ou d'ordre n'exécute rien : ni édition, changement de ton, sauvegarde ni autre action de cette intervention. Afficher l'ambiguïté dans l'application ; l'utilisateur peut préciser sa demande dans une nouvelle intervention, sans file d'actions suspendues à reprendre automatiquement. Les interventions non ambiguës suivent leur ordre explicite : par exemple changement de ton, édition, puis sauvegarde. Préparer les éditions sur une copie de travail et les valider avant publication ; une sauvegarde ordonnée après une édition porte sur son état accepté. Un changement de note est une barrière terminale : aucune action destinée à l'ancienne note n'est reportée sur la nouvelle. Une combinaison dont la portée après cette barrière n'est pas claire est ambiguë et n'exécute rien. La voie d'interruption prioritaire reste indépendante : elle peut arrêter/invalider immédiatement mais n'applique aucune modification de contenu.

Pour une édition : liste ordonnée de `insert`, `replace`, `delete`, `move` portant cible locale identifiée, texte attendu/empreinte, contenu Markdown nouveau et, pour déplacement, ancre de destination. Les plages d'octets et l'enveloppe de session sont attribuées par le cœur, **pas crues sur parole par le modèle**. Une cible absente ou multiple devient clarification. Valider UTF-8, portée, révision et préconditions ; calculer tous les patches sur une copie de travail, vérifier puis commit atomique en ordre logique. Requête active sur révision ancienne : recalculer une fois à partir du courant ou signaler ; génération invalidée : rejet définitif. JSON valide ne prouve pas fidélité.

**Contexte envoyé.** Dernier passage finalisé, intention/lexique utiles et sections candidates avec identifiants locaux ; élargir explicitement au besoin à la note pour une réorganisation globale, jamais ajouter fichiers voisins. Note et contenu cité restent des données sans autorité. Les fragments utilisés pour corriger « non, je voulais dire » ne servent pas à restaurer une ancienne version.

**Contrat exécuté T010 (B03-OFFLINE).** `Status` résolu/clarify, `Interruption`
none/pause/stop/manual, contexte typé avec candidats `{block_id, source}` sans
offset ni identité de session ; le cœur conserve l'enveloppe Ticket autoritaire.
`edit.operation` : insert `{anchor,text}`, replace `{target,text}`, delete
`{target}`, move `{target,anchor}`. Target = `{block_id,expected}` : occurrence
UTF-8 exacte et unique dans un bloc candidat pris en charge. Anchor =
`{block_id,expected,edge}` : source entière du bloc et before/after, y compris
point d'insertion vide natif. IDs attribués avant filtrage des blocs opaques ;
ils suivent les décalages d'octets sur la copie de travail et ne désignent
jamais un autre paragraphe après une restructuration. Une cible détruite,
multiple, absente, opaque ou une ancre périmée refuse toute l'intervention.
Tous les objets filaires sont stricts, propriétés requises, champs inconnus
rejetés ; limites locales en octets : réponse 65536, message/intention/chemin
4096, ID 128, 32 actions. `maxLength` filaire borne les caractères ; le cœur
impose en plus la limite d'octets, sans conversion ni découpage silencieux.
`session_action.command` typée : pause/stop/manual/save/new/open(path)/
save_as(path)/preferences/close. Toute commande session est terminale dans
la tranche actuelle. Pause/stop/manual et copie exclusive save sont raccordées ;
les autres refusent explicitement sans effet jusqu'à leurs tâches d'intégration.
Le modèle en est informé ; aucune commande indisponible simulée comme succès.
Le passage manuel garde l'ordre physique du contrôleur : commit/invalidation
puis arrêt du dispositif, sans relance ni file d'actions ambiguës.

**IPC et UI.** Commandes typées et allowlistées : start/pause/stop, applyManualTransaction (manuel seulement et revision attendue), note new/open/save/saveAs/copy/export, configuration et gestion de clé, test explicite, opt-in transcription. Les événements vers l'UI contiennent état français, revision, patches appliqués, zones réelles, erreur expurgée et mesures ; transcription seulement si opt-in. Même registre d'actions pour menus, bandeau et intentions vocales. Pas de tray, notification système, service TTS ni focus imposé.

La webview ne navigue pas vers le Web et ne contacte aucun service ; ressources embarquées, CSP sans script distant ni HTML actif, pas d'API générique filesystem/shell exposée. Un lien externe éventuel passe par une action explicite et une validation de protocole ; jamais ouvert par le contenu de la note. Les contrôles de confidentialité du système d'exploitation restent sous son autorité : l'application ne crée pas d'indicateur d'écoute extérieur et ne tente pas de masquer un contrôle système.

### Éditeur visuel et sauvegarde sans dégradation silencieuse

**Un seul document central ProseMirror**, sélection et copie consultables en vocal mais `editable=false` et historique absent. En manuel : saisie et undo classique, boutons titres/paragraphes/listes/gras/italique. Pas d'éditeur source/preview à deux panneaux, pas de chat, polices arbitraires ou format Office. Les menus sont une barre DOM accessible dans la fenêtre pour garder Fichier/Édition/Aide visibles également sous GNOME.

Le Markdown source est canonique dans Rust ; ProseMirror est une projection, pas le fichier. `pulldown-cmark` fournit les plages source ; un pont associe nœuds/texte affichés aux plages UTF-8 et conserve séparateurs, marqueurs et fins de ligne d'origine. Les transactions textuelles produisent des patches ciblés ; un changement de structure sérialise **la plus petite unité prise en charge réellement affectée**, pas le document entier. T039 met en œuvre ce pont dans `src/ui/MarkdownBridge.ts`, avec contrôle natif des préconditions. Les tokens bruts hors cible sont recopiés à l'identique, y compris CRLF, accents et espaces.

**Priorité du cœur — décision U2.** Le contenu accepté dans Rust fait foi, jamais une projection locale de ProseMirror. Une proposition manuelle porte `transactionId`, `noteId`, identifiant de période manuelle, `baseRevision` et patches attendus ; une seule proposition est en vol, les suivantes sont ordonnées localement. Le cœur vérifie mode, note, période, révision et préconditions, puis répond `accepted` ou `rejected` avec l'identité de transaction et la révision canonique courante. Une acceptation fournit les patches effectivement appliqués ; un refus fournit un motif et permet de récupérer un instantané canonique. Les identifiants dédupliqués empêchent qu'un renvoi de la même transaction applique deux fois la saisie.

Une saisie locale en attente n'est pas une modification confirmée ou sauvegardée. L'acquittement correspondant confirme la transaction avant l'envoi de la suivante sur la révision acceptée. Réponses et événements sont rapprochés par transaction, note, période et révision : une notification ancienne ou dupliquée ne réapplique rien et ne fait pas régresser l'éditeur. Un trou de révisions ou une divergence arrête l'envoi des propositions ; l'UI demande l'instantané du cœur. Un changement de note ou de mode invalide également les propositions de l'ancienne période, sans les reporter sur la suivante.

Sur refus ou divergence, afficher le motif, abandonner la file de propositions non acceptées et reconstruire l'éditeur depuis l'instantané canonique ; aucune réapplication automatique. Garder la saisie locale non acceptée séparément en mémoire, consultable ou copiable explicitement pendant la période manuelle, sans la confondre avec la note ni la journaliser. Signaler sa suppression à la fin de cette période si elle n'a pas été récupérée. Sauvegarde, copie et export de la note utilisent uniquement le contenu du cœur.

L'undo/redo manuel soumet lui aussi des patches au cœur, avec les mêmes contrôles ; seules les transactions acceptées alimentent l'historique confirmé. Les acquittements et projections ne créent pas une seconde entrée d'historique. Après reconstruction sur refus ou divergence, réinitialiser l'historique de l'éditeur et le signaler, afin qu'aucune transaction refusée ou fondée sur une ancienne projection ne soit réintroduite. Hors de ce cas d'erreur, saisie, formatage et annulation classiques conservent leur comportement normal. T038/T039 vérifient ces invariants, sans ajouter d'historique vocal ou durable.

Tableaux, code, HTML et constructions non représentables dans le sous-ensemble sont des nœuds opaques en lecture seule, affichés sans exécuter HTML ni charger de ressource distante. Insertion de texte autour autorisée ; toute opération traversant une zone opaque ou exigeant de normaliser au-delà de la cible est refusée et expliquée **avant** altération. Pas de « round-trip sans perte » attribué au sérialiseur standard de ProseMirror. Copier/exporter utilise la source canonique brute. Une note UTF-8 non décodable n'est pas convertie silencieusement.

Sauvegarde automatique après commit accepté, sérialisée avec les écritures manuelles et la fermeture ; une nouvelle note reçoit une destination Markdown dans le dossier utilisateur de notes créé explicitement à la configuration, sans cache de session. Le chemin dernière note permet de retrouver la dernière sauvegarde réussie. Tout changement de note protège le dirty courant.

**Changement de note — décision U1.** Une fois le changement accepté, invalider tous les traitements précédents en cours avant de charger ou créer la note suivante : incrémenter la génération monotone, vider les files et passages en attente associés à l'ancienne note et rejeter leurs résultats tardifs. L'annulation réseau ne remplace pas le contrôle au commit de `noteId` et de la génération. Une révision identique sur deux notes, ou la réouverture du même fichier, ne rend jamais un ancien résultat valide. Si le chargement échoue, garder la note courante accessible et signaler l'erreur, sans réactiver les anciens traitements. Le changement de note n'impose pas de passer en mode manuel ; l'écoute peut continuer dans le mode vocal courant pour les nouveaux passages.

À l'ouverture : empreinte du contenu et identité fichier ; watcher `notify` comme signal, puis relecture avant sauvegarde, car le watcher seul est insuffisant. Écriture dans un fichier voisin exclusif, contenant **seulement la note**, flush/sync puis remplacement contrôlé sur le même volume. Conserver la version disque déplacée jusqu'au contrôle après remplacement : `renameat2(RENAME_EXCHANGE)` sur Linux compatible, `ReplaceFileW` avec copie de sauvegarde sur Windows. Si son empreinte diffère de la version attendue, préserver cette note externe en copie de conflit et afficher les deux destinations ; ne pas tenter un rollback destructeur. En succès sans conflit, supprimer le temporaire de note. Si le système de fichiers ne fournit pas la primitive ou si l'identité est instable, sauvegarder en nouvelle copie exclusive et signaler, **ne pas écraser**.

Liens symboliques, fichiers non ordinaires et volumes non locaux : refus de remplacement automatique et proposition d'une destination locale ordinaire. Permissions à préserver ou erreur visible. Aucun mécanisme ne garantit une exclusion universelle face à un autre programme écrivant librement : courses pendant remplacement, interruption et récupération des seuls fichiers de note temporaires sont des tests NO-GO de T011/T043. Cette limite est exposée ; aucune écriture audio, transcription ou conversation n'est permise dans ces fichiers. En conflit, l'utilisateur peut décider à la voix dans une session encore active ; en fermeture interrompue, la capture reste arrêtée.

### Intégration système et secret

CPAL natif : WASAPI sur Windows ; ALSA vers le périphérique PipeWire de la session Fedora pour le prototype non sandboxé ; CoreAudio pour l'objectif macOS. Distribution initiale : paquet RPM Fedora et installateur NSIS Windows ; pas de Flatpak imposant des portails supplémentaires. T003/T005 documentent les dépendances WebKitGTK/WebView2 et figent les versions utilisées.

GNOME : abonnement D-Bus à l'état actif de `org.gnome.ScreenSaver` pour verrouillage et à `org.freedesktop.login1.Manager.PrepareForSleep` pour veille ; initialiser l'état et surveiller disparition des services. Windows : notifications WTS de verrouillage et `WM_POWERBROADCAST` avant suspension. Échec de surveillance = vocal indisponible ou arrêté avec erreur, manuel utilisable ; perte de focus seule n'est jamais un verrouillage. Aucun besoin d'extension GNOME. Un arrêt brutal du système n'offre pas de délai garanti : aucune promesse de sauvegarde d'un contenu jamais écrit.

Trousseau : keyring **3.6.3** avec `sync-secret-service` et `crypto-rust` sous GNOME (appel hors thread UI), `windows-native` sous Windows et `apple-native` pour macOS. Ne pas utiliser `linux-native` seul ni le backend mock implicite ; un test de présence du backend réel est obligatoire. Identifiants service/compte distincts pour clés STT et LLM, éventuellement même valeur choisie par l'utilisateur. Clé présente en mémoire seulement pour son usage ; erreurs réseau expurgées, aucun dump de requête. Pas de `.env`, localStorage, IndexedDB, télémétrie de contenu, logs HTTP complets ni coffre fichier de remplacement. L'effacement logique n'est pas une garantie contre swap/crash dumps ou conservation fournisseur.

## Vérification de la constitution

Constitution actuelle lue sans modification. Contrôle avant recherche puis après décisions :

| Principe | Avant recherche | Réévaluation du plan final |
|---|---|---|
| I — Fidélité | PASS documentaire | PASS : opérations ciblées, données sans autorité, clarification, corpus quatre usages et distinction erreurs STT/LLM/application planifiés. |
| II — Deux modes | PASS documentaire | PASS : manuel exclusif avec undo mémoire, aucune pile vocale, invalidation au commit, reprise intentionnelle sur le contenu courant. |
| III — Intégrité | PASS documentaire | PASS : source canonique, zones opaques protégées, sauvegarde/conflit/fermeture définis et essais de fautes obligatoires. |
| IV — Confidentialité | PASS documentaire | PASS : mémoire temporaire, trousseau explicite, tests externes informés, coûts inconnus signalés, aucune promesse de suppression fournisseur. |
| V — Simplicité/indépendance | PASS documentaire | PASS : une fenêtre, deux ports, aucun backend serveur ni base ; choix retenus avec alternatives et critères de remise en cause au prototype. |

**Gate de planification : PASS documentaire, sans dérogation.** L'absence de prototype n'est pas une violation de constitution. La faisabilité effective et la conformité produit restent à établir aux jalons ci-dessous. Le plan est suffisant pour générer les tâches ; il ne certifie ni binaire ni compte API.

## Guidelines appliquées

Catalogue local `skills/guidelines/` inspecté : uniquement `spring-boot-scaffolding` et `struts-to-spring`, sans correspondance avec ce desktop greenfield. **Aucune règle de migration appliquée.** Ne pas introduire Spring, une base ou une topologie de migration. Les sources officielles réellement consultées et leurs limites figurent dans [research.md](research.md).

## Gates de planification et jalons de recette

Les [gates de recherche](research.md#synthèse-des-gates) distinguent décisions documentées et essais futurs. G01 est résolue ; G10 signifie revue interne de ce plan, **pas attente de fichiers externes**. G02–G09 sont des critères GO/NO-GO des prototypes/jalons, actuellement NON EXÉCUTÉS, et non des erreurs de rédaction. Prix absents = estimation inconnue, pas montant fictif ; modèle indisponible sur le compte = fonction vocale non activée, pas bascule cachée.

Q01 est résolue : l'utilisateur demande que l'indicateur d'écoute et les demandes de précision apparaissent uniquement dans la fenêtre de l'application, même si celle-ci est masquée. Aucun indicateur hors fenêtre ni notification du bureau n'est requis ou prévu. Au retour volontaire de l'utilisateur, l'état courant et la demande de précision restent consultables ; aucune lecture à voix haute ni prise de premier plan n'est imposée. L'écoute en arrière-plan et la pause rapide à la voix restent requises. Une extension GNOME, une capacité « toujours au-dessus » et un raccourci global ne sont pas nécessaires pour ces affichages. Le fonctionnement sans extension reste la cible. L'accès aux deux desktops réels et à un compte Mistral autorisé doit également être établi. Aucune donnée sensible avant vérification des conditions externes.

## Étapes d'implémentation et tâches inline

Tous les chemins de tâches ci-dessous sont relatifs à la racine du projet.
La description du plan reste la cible ; B01-R2 a validé le scaffold T005
nativement, puis réalisé une tranche T006 partielle (blocage SDK ALSA au raccordement CPAL).
Les dépendances sont les prédécesseurs directs,
explicites ; leurs prérequis sont transitifs. Un numéro inférieur ne constitue
pas à lui seul une dépendance. Les cases T001–T004 reflètent les livrables
documentaires vérifiés ; T005 est cochée sur check/test/build réels ; T006 reste décochée.

`USn` correspond au parcours n existant, sans US5. Les fonctions retirées de la présentation du parcours 5 restent dans les fondations transversales. Les **60 IDs existants sont conservés** pour ne pas casser la traçabilité ; ce sont de petites unités, pas 60 jalons. Les tâches de revue et de prototype ne relancent pas le cadrage.

### Phase 1 — Setup et premier prototype

#### P1.1 — Reprise des décisions desktop
Exigences : REQ-029, REQ-050. Entrées : spec.md, R01/R05. Sortie : Q01 consignée, affichages internes uniquement et vérification de l'écoute en arrière-plan sans extension GNOME.

- [X] T001 [Plan:1.1] Relire les décisions de ce plan et consigner leur prise en compte dans `specs/001-bloc-notes-vocal/design-review.md`, dont Q01 résolue, UI WordPad et Tauri 2 retenu, sans redemander leur validation ; dépend de : aucune.

#### P1.2 — Vérification fournisseur et audio
Exigences : REQ-001, REQ-039, REQ-042, REQ-045. Entrées : R02/R03/R09 ; ne pas confondre carte de poids et API disponible.

- [X] T002 [Plan:1.2] Préparer `docs/validation/provider-check.md` à partir de R02/R03/R09 : modèles retenus, PCM mono 16 kHz, WSS, json_schema strict et test synthétique explicitement informé ; au premier essai autorisé relever catalogue du compte, modèle réellement servi, tarif officiel daté si accessible et conditions applicables, sinon afficher inconnues et exclure données sensibles ; dépend de : aucune.

#### P1.3 — Moyens de validation réels
Exigences : REQ-050, REQ-051, REQ-052. Entrées : S8, R01/R09.

- [X] T003 [Plan:1.3] Inventorier dans `docs/validation/environments.md` l'accès à GNOME/Wayland Fedora et Windows 11, matériel, micro, trousseau, versions et formats de distribution, puis planifier macOS ; dépend de : aucune ; aucun desktop complet présumé.

#### P1.4 — Revue interne des interfaces
Exigences : REQ-017, REQ-022, REQ-024, REQ-039, REQ-041. Entrées : modèle, interfaces et invariants définis dans ce plan.

- [X] T004 [Plan:1.4] Revoir dans `specs/001-bloc-notes-vocal/design-review.md` les types et interfaces inline avant leur traduction en code : barrière de commit, schéma strict, pont Markdown et conflits, secrets et frontières de session ; définir les assertions de prototype, sans attendre ni produire d'artefact de design autonome ; dépend de : T001, T002, T003.

#### P1.5 — Socle retenu, sans Git
Exigences : REQ-039, REQ-050. Entrées : décisions de ce plan et revue interne.

- [X] T005 [Plan:1.5] Créer `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/lib.rs`, `package.json`, `rust-toolchain.toml` et verrouillages pour la pile retenue, scripts `tauri`/`test:unit` avec Vitest, cibles RPM/NSIS et capacités minimales ; figer les patches réellement résolus, sans initialiser Git ; dépend de : T004.

#### P1.6 — Faisabilité et premier prototype sur deux desktops
Exigences : REQ-028, REQ-029, REQ-032, REQ-033, REQ-035, REQ-036, REQ-037, REQ-041, REQ-050, REQ-052.

- [ ] T006 [Plan:1.6] Réaliser une tranche verticale réutilisable dans `src-tauri/src/platform/probe.rs` et `src/ui/Prototype.ts` : capture CPAL, normalisation/VAD/rotation WSS, LLM strict, génération et interruption prioritaire, ProseMirror visuel en manuel/read-only vocal, préservation d'un bloc opaque, trousseau, cycle OS et sauvegarde ; menus et boutons WordPad minimaux, aucune notification ni cache de contenu ; dépend de : T005 ; ne pas annoncer compatible avant T007.
- [ ] T007 [Plan:1.6] Recetter le prototype sur les deux systèmes dans `docs/validation/prototype-gnome.md`, `docs/validation/prototype-windows.md`, `docs/validation/prototype-metrics.md` : G02–G09, passage finalisé sans perte à la rotation, pause naturelle rapide, édition visuelle/protection opaque et invariants transversaux ; dépend de : T006 ; NO-GO exige correction ou révision technique avant stories, un essai absent reste NON EXÉCUTÉ.

### Phase 2 — Fondations transversales

Entrées : revue interne et premier prototype GO. Les invariants d'arrêt, de sauvegarde et de confidentialité ne sont pas différés à une story tardive.

#### P2.1 — Modèle minimal et autorité de mutation
Exigences : REQ-005, REQ-015, REQ-016, REQ-017, REQ-018, REQ-019, REQ-032, REQ-039, REQ-046.

- [X] T008 [Plan:2.1] Écrire d'abord de petits tests dans `src-tauri/tests/session_validity.rs` : anciennes sessions, reprise, révision périmée, doublons, ordre, pause, résultat retardé après création/ouverture/réouverture de note avec contenus et révisions identiques, et aucune pile undo vocale ; dépend de : T007.
- [X] T009 [Plan:2.1] Implémenter les entités Note/Session/Traitement et la barrière de commit dans `src-tauri/src/domain/session.rs` et `src-tauri/src/domain/note.rs`, avec noteId vérifié, génération monotone, invalidation de tous les traitements précédents au changement de note et nettoyage aux frontières ; dépend de : T008.
- [X] T010 [Plan:2.1] Formaliser les ports STT/LLM et le schéma strict inline en types validés dans `src-tauri/src/ports/stt.rs` et `src-tauri/src/ports/interpreter.rs`, puis réutiliser les composants de prototype sans duplication ; dépend de : T009 ; terminée B03-OFFLINE, contrat/résolution/tests natifs ; actions fichiers vocales complètes intégrées en T024 seulement.

#### P2.2 — Fichiers et fermeture sûre
Exigences : REQ-020, REQ-022, REQ-023, REQ-024, REQ-036, REQ-037.

- [ ] T011 [Plan:2.2] Écrire `src-tauri/tests/storage_integrity.rs` : octets hors cible, version disque, disque plein, droits, interruption et course externe avant remplacement ; dépend de : T007.
- [ ] T012 [Plan:2.2] Implémenter la note unique, ouverture/destination, sauvegarde automatique contrôlée et conflit non destructif dans `src-tauri/src/storage/markdown_file.rs` ; dépend de : T009, T011.
- [ ] T013 [Plan:2.2] Implémenter l'ordre arrêt/invalidation → sauvegarde → fin/sortie et l'échec sans fermeture silencieuse dans `src-tauri/src/app/close.rs` ; dépend de : T012.

#### P2.3 — Frontière UI et Markdown
Exigences : REQ-004, REQ-021, REQ-022, REQ-026, REQ-027, REQ-030.

- [ ] T014 [Plan:2.3] Écrire `src-tauri/tests/ipc_boundary.rs`, `src/ui/markdown.test.ts` et `src/ui/DesktopShell.test.ts` pour contenu hostile, chemins, HTML/protocoles, images distantes, absence de cible fictive et présence des menus Fichier/Édition/Aide et du bandeau de mise en forme et de démarrage/arrêt vocal (S4.5) ; dépend de : T007.
- [ ] T015 [Plan:2.3] Implémenter commandes IPC allowlistées et événements d'état français sans données secrètes dans `src-tauri/src/app/commands.rs` et `src-tauri/capabilities/main.json` ; dépend de : T009, T012, T014.
- [ ] T016 [Plan:2.3] Créer la fenêtre simple inspirée de WordPad dans `src/ui/DesktopShell.ts`, les menus Fichier/Édition/Aide dans `src/ui/MenuBar.ts`, le bandeau de mise en forme et démarrage/arrêt vocal dans `src/ui/Toolbar.ts`, la note mise en forme et les états lisibles sans vol de focus dans `src/ui/NoteView.ts` et `src/ui/StatusView.ts` ; relier les commandes aux actions partagées à mesure de leur implémentation, sans dupliquer leur logique ni autoriser l'édition manuelle pendant le vocal ; dépend de : T015.

#### P2.4 — Configuration et secrets natifs
Exigences : REQ-038, REQ-039, REQ-040, REQ-041, REQ-042, REQ-043, REQ-045.

- [ ] T017 [Plan:2.4] Écrire `src-tauri/tests/secrets_policy.rs` pour création/remplacement/suppression, commande de saisie dédiée, absence de clé dans état durable UI/config/logs et trousseau indisponible sans fallback ; dépend de : T007.
- [ ] T018 [Plan:2.4] Intégrer keyring 3.6.3 avec backends Secret Service/Credential Manager explicites et commande IPC de secret masqué à durée de vie courte dans `src-tauri/src/platform/secrets.rs`, rejeter le backend mock en production ; dépend de : T017.
- [ ] T019 [Plan:2.4] Implémenter préférences non sensibles, présence des clés, choix STT/LLM séparés et prise d'effet à la session suivante dans `src-tauri/src/config/preferences.rs` ; dépend de : T010, T018.

#### P2.5 — Adaptateurs Mistral indépendants
Exigences : REQ-001, REQ-033, REQ-039, REQ-043, REQ-045, REQ-048.

- [ ] T020 [Plan:2.5] Écrire `src-tauri/tests/provider_contracts.rs` avec transport simulé : auth expurgée, segments/corrections, JSON invalide, erreur réseau, annulation et absence de fournisseur de secours ; dépend de : T010.
- [ ] T021 [Plan:2.5] Intégrer capture CPAL, downmix/rubato 16 kHz, VAD et passages WSS dans `src-tauri/src/audio/capture.rs` et `src-tauri/src/adapters/mistral_stt.rs` : modèle retenu, bornes mémoire et erreur de saturation, réconciliation done sans édition ordinaire sur delta ni réémission indue ; dépend de : T019, T020.
- [ ] T022 [Plan:2.5] Intégrer `mistral-large-latest`, schéma strict validé localement, voie prioritaire limitée à l'interruption et compteurs de requêtes dans `src-tauri/src/adapters/mistral_interpreter.rs`, sans outils arbitraires ni conversation persistante ; dépend de : T019, T020.

#### P2.6 — Écoute, interruptions et hors fenêtre
Exigences : REQ-013, REQ-028, REQ-029, REQ-030, REQ-031, REQ-032, REQ-033, REQ-034, REQ-035, REQ-036.

- [ ] T023 [Plan:2.6] Écrire `src-tauri/tests/lifecycle.rs` avec capture simulée : lancement explicite, silence sans arrêt, pause, lock/sleep, fermeture, pertes micro/service et événements OS indisponibles ; dépend de : T009.
- [ ] T024 [Plan:2.6] Implémenter abonnement OS vérifié et invalidation immédiate dans `src-tauri/src/platform/lifecycle.rs`, sans reprise automatique au retour ; dépend de : T013, T021, T023.
- [ ] T025 [Plan:2.6] Afficher l'état courant d'écoute, les états français et l'avertissement tiers dans `src/ui/SessionStatus.ts` uniquement, sans indicateur hors fenêtre ni notification du bureau ; relier la pause vocale rapide et garder l'état consultable au retour dans l'application ; dépend de : T016, T022, T024.

#### P2.7 — Gate des fondations
Exigences : REQ-050, REQ-052.

- [ ] T026 [Plan:2.7] Rejouer la matrice desktop et mesures dans `docs/validation/foundations-gnome.md`, `docs/validation/foundations-windows.md` et `docs/validation/foundations-metrics.md` ; dépend de : T016, T019, T025 ; un échec bloque les stories.

### Phase 3 — US1 : construire une note fidèle (P1)

#### P3.1 — Intention et rédaction
Exigences : REQ-001, REQ-002, REQ-003, REQ-010, REQ-053. Entrées : S1.1–S1.3, SC-001.

- [ ] T027 [US1] [Plan:3.1] Créer corpus synthétique et tests de décisions/nuances dans `tests/fixtures/fidelity.json` et `src-tauri/tests/drafting.rs`, sur les quatre usages sans prétendre garantir tout énoncé ; dépend de : T026.
- [ ] T028 [US1] [Plan:3.1] Modéliser intention, forme, ton et contexte minimal temporaire dans `src-tauri/src/domain/intention.rs` sans les insérer automatiquement dans la note ; dépend de : T027.
- [ ] T029 [US1] [Plan:3.1] Implémenter orchestration de rédaction progressive sur informations fournies dans `src-tauri/src/app/drafting.rs` et classifier erreurs STT/LLM/application ; dépend de : T028.

#### P3.2 — Perception du résultat appliqué
Exigences : REQ-025, REQ-026, REQ-027, REQ-050, REQ-052. Entrées : S1.4.

- [ ] T030 [US1] [Plan:3.2] Afficher traitement distinct du commit, zones réelles, déplacements/suppressions perceptibles et navigation préservée dans `src/ui/ChangeView.ts` ; dépend de : T029.
- [ ] T031 [US1] [Plan:3.2] Valider S1 et mesurer sur deux desktops dans `docs/validation/us1-gnome.md`, `docs/validation/us1-windows.md`, `docs/validation/us1-metrics.md` ; dépend de : T030.

### Phase 4 — US2 : piloter et corriger naturellement (P1)

#### P4.1 — Opérations ordonnées et ciblées
Exigences : REQ-004, REQ-005, REQ-006, REQ-007, REQ-008, REQ-009, REQ-015. Entrées : S2.1, S2.2, S2.3, S2.5.

- [ ] T032 [US2] [Plan:4.1] Écrire `src-tauri/tests/voice_operations.rs` : ton puis édition puis sauvegarde, ambiguïté au début ou à la fin sans aucun effet de l'intervention, absence de reprise automatique, interruption prioritaire, récit impératif, contenu hostile, portée, correction contextuelle et refus de l'undo vocal ; dépend de : T031.
- [ ] T033 [US2] [Plan:4.1] Implémenter dans `src-tauri/src/app/operations.rs` l'enveloppe ordonnée d'intentions et sa validation complète avant effet ; en cas d'ambiguïté ne rien exécuter et signaler, sinon respecter l'ordre et les barrières de changement de note, sans confirmation ordinaire ni historique vocal ; dépend de : T032.

#### P4.2 — Clarification et épellation
Exigences : REQ-007, REQ-011, REQ-012, REQ-029, REQ-046. Entrées : S2.3–S2.4.

- [ ] T034 [US2] [Plan:4.2] Implémenter lexique éphémère, épellation casse/accents/tiret/apostrophe et tests ciblé/global dans `src-tauri/src/domain/lexicon.rs` et `src-tauri/tests/spelling.rs` ; dépend de : T033.
- [ ] T035 [US2] [Plan:4.2] Implémenter signalement d'ambiguïté sans action de l'intervention et précision vocale comme nouvelle demande dans `src-tauri/src/app/clarification.rs`, affichée dans `src/ui/ClarificationView.ts` uniquement, sans reprise automatique, notification du bureau, lecture vocale, focus imposé, cible aléatoire ni épellation insérée par défaut ; dépend de : T034.

#### P4.3 — Toutes les actions vocales
Exigences : REQ-013, REQ-020, REQ-029, REQ-031, REQ-032, REQ-038, REQ-050, REQ-052.

- [ ] T036 [US2] [Plan:4.3] Relier intentions naturelles aux actions note/session/préférences dans `src-tauri/src/app/voice_actions.rs`, y compris destination/nom, retrouver une note, pause/arrêt/manuel/fermeture, hors configuration de secrets et autorisations OS ; dépend de : T035.
- [ ] T037 [US2] [Plan:4.3] Recetter S2, actions vocales et interruptions transversales dans `docs/validation/us2-gnome.md`, `docs/validation/us2-windows.md`, `docs/validation/us2-metrics.md`, notamment ordre ton/édition/sauvegarde et zéro effet d'une intervention ambiguë ; dépend de : T036.

### Phase 5 — US3 : manuel puis reprise vocale (P1)

#### P5.1 — Transfert exclusif et undo manuel
Exigences : REQ-014, REQ-015, REQ-016, REQ-017, REQ-018, REQ-019, REQ-021, REQ-022, REQ-050, REQ-052. Entrées : S3.1–S3.4, S4.2, S4.5, SC-002.

- [ ] T038 [US3] [Plan:5.1] Écrire `src-tauri/tests/mode_handoff.rs` et `src/ui/ManualEditor.test.ts` : résultat tardif avant/après reprise, undo manuel éphémère, saisies rapprochées, acceptation/refus natif, acquittement dupliqué, événement retardé, resynchronisation sur le cœur, récupération temporaire de saisie refusée, formatage puis undo et undo après refus sans réintroduire la transaction rejetée ; dépend de : T037.
- [ ] T039 [US3] [Plan:5.1] Intégrer ProseMirror visuel, bandeau Markdown et undo mémoire uniquement en manuel dans `src/ui/ManualEditor.ts`, et pont de patches/source dans `src/ui/MarkdownBridge.ts` avec contrôle natif dans `src-tauri/src/domain/note.rs` ; implémenter transactionId/noteId/période/révision, acquittement accepted/rejected, envoi ordonné, priorité du cœur et récupération canonique sans rejeu automatique, historique confirmé et reset signalé après refus ; tester dans `src/ui/MarkdownBridge.test.ts` l'égalité UI/source après stabilisation, octets hors cible inchangés, opaque non éditable et consultation read-only sans historique vocal ; dépend de : T038.
- [ ] T040 [US3] [Plan:5.1] Intégrer transfert immédiat, signalement du non-intégré, reprise explicite et modification intentionnelle du contenu manuel dans `src-tauri/src/app/mode_handoff.rs` ; dépend de : T039.
- [ ] T041 [US3] [Plan:5.1] Recetter S3 et SC-004 dans `docs/validation/us3-gnome.md`, `docs/validation/us3-windows.md`, `docs/validation/us3-metrics.md` ; dépend de : T040.

### Phase 6 — US4 : fichiers locaux intègres (P1)

#### P6.1 — Parcours de note unique
Exigences : REQ-017, REQ-020, REQ-021, REQ-023, REQ-024. Entrées : S4.1, S4.4.

- [ ] T042 [US4] [Plan:6.1] Intégrer créer/ouvrir/retrouver/enregistrer/enregistrer sous/copier/exporter en vocal et manuel, avec protection de la note sale et invalidation de tous les traitements précédents avant changement de note, dans `src/ui/NoteActions.ts` et `src-tauri/src/app/note_actions.rs` ; dépend de : T041.

#### P6.2 — Préservation et conflits réels
Exigences : REQ-017, REQ-022, REQ-023, REQ-024, REQ-036, REQ-037, REQ-050, REQ-052. Entrées : S4.1–S4.4, SC-003.

- [ ] T043 [US4] [Plan:6.2] Ajouter fixtures Markdown unsupported/Unicode/CRLF et essais d'échec/conflit externe dans `tests/fixtures/preservation.md` et `src-tauri/tests/file_workflows.rs` ; vérifier aussi les résultats retardés après création/ouverture/réouverture avec mêmes contenus et révisions, et après échec d'ouverture, sans modification tardive d'aucune note ; dépend de : T042.
- [ ] T044 [US4] [Plan:6.2] Exécuter S4.1–S4.5 et les recettes disque plein, fermeture sans destination, conflit pendant sauvegarde et interruption dans `docs/validation/us4-gnome.md`, `docs/validation/us4-windows.md`, `docs/validation/us4-metrics.md` ; dépend de : T043.

### Phase 7 — US6 : services, secrets et coûts (P1)

#### P7.1 — Préférences transparentes
Exigences : REQ-038, REQ-039, REQ-040, REQ-041, REQ-042, REQ-043.

- [ ] T045 [US6] [Plan:7.1] Intégrer panneau FR, modèles intégrés et accès vérifié distincts, champ secret masqué vidé après IPC, présence/suppression des clés et test explicite informé dans `src/ui/PreferencesView.ts` ; dépend de : T044 ; pas d'OpenAI intégré affiché.

#### P7.2 — Consommation et conditions externes
Exigences : REQ-044, REQ-045, REQ-046, REQ-050, REQ-052. Entrées : S6.1–S6.5.

- [ ] T046 [US6] [Plan:7.2] Écrire `src-tauri/tests/usage_estimates.rs` pour composantes STT/LLM, tarifs absents/périmés, métriques manquantes et différence estimation/facture ; dépend de : T045.
- [ ] T047 [US6] [Plan:7.2] Implémenter compteurs mémoire, source/date de tarif et inconnues explicites dans `src-tauri/src/domain/usage.rs` et `src/ui/UsageView.ts` ; dépend de : T046.
- [ ] T048 [US6] [Plan:7.2] Documenter dans `docs/privacy.md` et l'écran `src/ui/PrivacyView.ts` les envois minimaux, conditions vérifiées, limites externes et absence de bascule cachée ; dépend de : T047.
- [ ] T049 [US6] [Plan:7.2] Exécuter S6 et recherche de traces avec secrets synthétiques dans `docs/validation/us6-gnome.md`, `docs/validation/us6-windows.md`, `docs/validation/us6-metrics.md` ; dépend de : T048.

### Phase 8 — US8 : portabilité et mesures (P1)

#### P8.1 — Matrice complète et objectif macOS
Exigences : REQ-050, REQ-051, REQ-052, REQ-053. Entrées : S8.1–S8.3, SC-006/SC-007.

- [ ] T050 [US8] [Plan:8.1] Préparer `docs/validation/macos-status.md` : moyens macOS, Keychain, capture et même recette ; laisser objectif non confirmé si non exécuté, ne pas annoncer un support ; dépend de : T049.
- [ ] T051 [US8] [Plan:8.1] Consolider sans remplacer les recettes par jalon dans `docs/validation/p1-platforms.md` les deux desktops, quatre usages, mesures et limites, avec statut séparé macOS ; dépend de : T050.

### Phase 9 — US7 : diagnostic temporaire (P2)

#### P9.1 — Vue de transcription et purge
Exigences : REQ-046, REQ-047, REQ-048, REQ-049, REQ-050, REQ-052. Entrées : S7.1–S7.3.

- [ ] T052 [US7] [Plan:9.1] Écrire `src-tauri/tests/diagnostic_privacy.rs` : défaut désactivé, aucune console persistante, corrections/provisoire/final si exposés, purge et nouvelle session vide ; dépend de : T051.
- [ ] T053 [US7] [Plan:9.1] Implémenter opt-in par session, avertissement, segments qualifiés et vidage dans `src/ui/TranscriptView.ts` et `src-tauri/src/app/diagnostics.rs`, sans enregistrement audio ; dépend de : T052.
- [ ] T054 [US7] [Plan:9.1] Exécuter S7 puis matrice transversale dans `docs/validation/us7-gnome.md`, `docs/validation/us7-windows.md`, `docs/validation/us7-metrics.md` ; dépend de : T053.

### Phase 10 — Validation finale et sobriété

#### P10.1 — Robustesse et confidentialité
Exigences : REQ-017, REQ-024, REQ-035, REQ-036, REQ-037, REQ-041, REQ-046, REQ-049.

- [ ] T055 [Plan:10.1] Ajouter stress déterministe de courses, transport retardé, doublons et fautes d'écriture dans `src-tauri/tests/fault_matrix.rs` sans élargir le produit ; dépend de : T054.
- [ ] T056 [Plan:10.1] Vérifier fichiers/cache/temp/logs/crash et limites OS/fournisseur dans `docs/validation/privacy-audit.md`, sans conserver de vrai secret ou contenu personnel dans la preuve ; dépend de : T055.

#### P10.2 — Fidélité et recette finale sur deux systèmes
Exigences : REQ-002, REQ-007, REQ-050, REQ-051, REQ-052, REQ-053.

- [ ] T057 [Plan:10.2] Évaluer quatre usages, hésitations, chiffres, accents, noms, longues interventions, ambiguïtés et épellation dans `docs/validation/fidelity.md`, en séparant STT/LLM/application et sans fait ajouté accepté ; dépend de : T056.
- [ ] T058 [Plan:10.2] Exécuter toute la matrice SC-001 à SC-007 sur les deux desktops dans `docs/validation/final-gnome.md`, `docs/validation/final-windows.md`, `docs/validation/final-metrics.md` ; dépend de : T057.

#### P10.3 — Décision de livraison et recontrôle constitutionnel
Exigences : REQ-039, REQ-045, REQ-050, REQ-051, REQ-052.

- [ ] T059 [Plan:10.3] Proposer après mesures seulement des objectifs chiffrés à décision utilisateur et mettre à jour le guide `specs/001-bloc-notes-vocal/quickstart.md` avec les commandes réellement validées ; dépend de : T058.
- [ ] T060 [Plan:10.3] Vérifier couverture, contrats, preuves par jalon et constitution dans `docs/validation/release-gates.md` ; marquer ERROR si une gate reste ouverte, macOS non vérifié restant objectif sans annonce ; dépend de : T059.

## Dépendances et jalons

T001, T002 et T003 préparent indépendamment la revue T004 ; aucune dépendance externe de design n'est attendue. T005–T007 réalisent le premier prototype : **aucun jalon purement Linux n'est accepté**. T002 organise la vérification compte au premier essai, sans bloquer la création du socle sur un appel API préalable. Un changement de pile après NO-GO exige une révision motivée du plan, pas un nouveau cadrage fonctionnel.

Après T007, tests des domaines indépendants peuvent être préparés, mais T026 impose l'intégration complète des fondations et la recette des deux systèmes. Les stories sont ensuite volontairement séquentielles ; chacune possède son test indépendant défini dans spec.md et ses rapports GNOME, Windows et mesures. US8 est P1, donc avant le diagnostic US7 P2. Les fonctions d'arrière-plan, interruption, trousseau et fermeture se rejouent **à chaque jalon**, même si leur code ne change pas.

Le « go » d'un jalon couvre seulement les fonctions effectivement livrées et la matrice transversale ; il ne valide pas les stories futures. Toute limite incompatible avec une exigence livrée entraîne ERROR, non une coche « connu ». SC-001 à SC-007 sont des objectifs de recette et restent non exécutés.

## Structure de projet retenue

Documentation effectivement produite ici :

```text
specs/001-bloc-notes-vocal/
  plan.md
  research.md
  quickstart.md
  checkpoints/spec-to-plan.yaml
  checkpoints/plan-to-tasks.yaml
```

Entrées préexistantes inchangées : spec.md et checklists/requirements.md. Modèle et interfaces sont inline ci-dessus ; aucun fichier de design autonome ni tasks.md livré.

Structure applicative **future**, non créée :

```text
src-tauri/
  Cargo.toml, tauri.conf.json, capabilities/main.json
  src/lib.rs
  src/domain/       # note, session, intention, lexique, consommation
  src/ports/        # STT et interprétation distincts
  src/adapters/     # Mistral STT et LLM
  src/audio/        # capture mémoire native
  src/platform/ # cycle OS, trousseau
  src/storage/      # Markdown local et conflits
  src/app/          # orchestration, actions, fermeture, IPC
  tests/            # petits tests cœur et intégration ciblée
src/ui/             # texte/rendu, états, préférences et diagnostic
tests/fixtures/     # uniquement données synthétiques
docs/validation/    # preuves desktop et mesures par jalon
docs/privacy.md
package.json
```

## Stratégie de test

**Type : desktop.** Petits tests automatiques demandés : machine de session, révisions/idempotence, validation d'opérations, lexique, fichiers/conflits, estimations, purge et secrets synthétiques. Écrire les tests déterministes avant l'implémentation correspondante ; ne pas traiter le LLM comme une fonction sémantiquement déterministe.

Stack retenue : `cargo test` pour cœur et adaptateurs simulés, Vitest pour TypeScript (script futur `npm run test:unit`), essais de l'application empaquetée dans les webviews réelles. Un outil navigateur est éventuellement complémentaire pour l'UI, pas substitut à WebKitGTK, WebView2, aux permissions ni aux événements OS.

Environnements : sessions interactives GNOME/Wayland et Windows 11, microphone, réseau autorisé, trousseaux réels, accès Mistral et consentement au coût ; macOS séparé. Pas de service Docker à introduire par défaut. Les mocks couvrent les fautes et courses du cœur uniquement ; **pas de fallback qui ferait passer une recette desktop manquante**.

Données : notes et paroles synthétiques, identifiants d'essai distincts, répertoires isolés, aucune clé réelle dans les preuves, nettoyage des seuls fichiers d'essai après contrôle. Les mesures conservent uniquement métadonnées non sensibles, pas transcription ou conversation.

Parcours critiques : création/rédaction fidèle ; édition naturelle et ambiguïté affichée uniquement dans l'application, y compris au retour après masquage ; manuel/reprise/résultat retardé ; fichiers/conflit/fermeture en échec ; préférences/trousseau/fin de session. Vérifier l'absence de notification du bureau et d'indicateur d'écoute hors fenêtre. Couvrir aussi écoute continue en arrière-plan, indicateur dans l'application, pause, lock/sleep et diagnostic.

Critères : zéro résultat invalidé appliqué, zéro perte/altération silencieuse, zéro auto-reprise après lock/sleep, aucun fait ajouté accepté dans les cas de recette, aucun secret en clair ni contexte durable créé par l'application ; toutes les fonctions livrées observées sur les deux desktops. Mesures contextualisées, aucune latence ou économie inventée. Voir [guide de validation](quickstart.md).

## Complexité et dérogations

Aucune dérogation constitutionnelle. Les ports STT/LLM et l'autorité native répondent aux invariants ; pas de microservices, event store, historique vocal, ORM ou collection de notes. DOM TypeScript sans framework UI, saisie des secrets locale et absence de tray réduisent le socle. Le pont Markdown et les primitives de sauvegarde répondent au besoin d'intégrité et doivent être éprouvés tôt, pas présentés comme garanties natives des bibliothèques.

## Requirement Mapping — couverture des exigences

Chaque ligne décrit une **preuve d'implémentation attendue**, pas un fichier déjà livré. Les checkpoints distinguent couverture et préparation réelle.

Pour alléger cette table seulement : `ui/` signifie `src/ui/` ; `tests/*.rs` signifie `src-tauri/tests/*.rs` ; `audio/`, `domain/`, `app/`, `storage/`, `platform/`, `adapters/`, `ports/` et `config/` sont sous `src-tauri/src/`. `tests/fixtures/` et `docs/` restent à la racine. Les tâches donnent toujours les chemins complets projet-relatifs ; le contrôle documentaire résout ces abréviations vers ces mêmes chemins.

| REQ ID | Description | Plan Items | Implementation Evidence |
|---|---|---|---|
| REQ-001 | Français continu naturel | P1.2, P2.5, P3.1 | T002, T021, T027–T029 ; audio/capture.rs, tests/drafting.rs |
| REQ-002 | Fidélité sans faits ajoutés | P3.1, P10.2 | T027–T029, T057 ; docs/validation/fidelity.md |
| REQ-003 | Forme, ton et détail | P3.1 | T028–T029 ; domain/intention.rs |
| REQ-004 | Idée versus instruction | P2.3, P4.1 | T014–T015, T032–T033 ; tests/voice_operations.rs |
| REQ-005 | Intentions ordonnées | P2.1, P4.1 | T008–T010, T032–T033 ; domain/session.rs |
| REQ-006 | Édition ciblée naturelle | P4.1 | T032–T033 ; app/operations.rs |
| REQ-007 | Ambiguïté signalée sans action | P4.1, P4.2, P10.2 | T032–T033, T035, T057 ; app/operations.rs, app/clarification.rs |
| REQ-008 | Changements directs perceptibles | P4.1 | T032–T033 ; app/operations.rs |
| REQ-009 | Correction contextuelle | P4.1 | T032–T033 ; tests/voice_operations.rs |
| REQ-010 | Intention libre et changeante | P3.1 | T028–T029 ; domain/intention.rs |
| REQ-011 | Épellation précise | P4.2 | T034–T035 ; tests/spelling.rs |
| REQ-012 | Lexique de session ciblé | P4.2 | T034 ; domain/lexicon.rs |
| REQ-013 | Actions vocales complètes | P2.6, P4.3 | T025, T036–T037 ; app/voice_actions.rs |
| REQ-014 | Manuel exclusif classique | P5.1 | T038–T041 ; ui/ManualEditor.ts |
| REQ-015 | Aucun undo/redo vocal | P2.1, P4.1, P5.1 | T008, T032–T033, T038 ; tests/mode_handoff.rs |
| REQ-016 | Arrêt et invalidation immédiats | P2.1, P5.1 | T008–T009, T040 ; app/mode_handoff.rs |
| REQ-017 | Résultats invalidés/périmés et changement de note | P1.4, P2.1, P5.1, P6.1, P6.2, P10.1 | T004, T008–T009, T038–T040, T042–T043, T055 ; tests/session_validity.rs, tests/file_workflows.rs, tests/fault_matrix.rs |
| REQ-018 | Reprise explicite sur tout texte | P2.1, P5.1 | T009, T038–T040 ; app/mode_handoff.rs |
| REQ-019 | Undo manuel non persistant | P2.1, P5.1 | T009, T038–T039 ; ui/ManualEditor.test.ts |
| REQ-020 | Note unique Markdown locale | P2.2, P4.3, P6.1 | T012, T036, T042 ; app/note_actions.rs |
| REQ-021 | Interface simple, menus, bandeau, rendu et copie/export bruts | P2.3, P5.1, P6.1 | T014, T016, T039, T042 ; ui/DesktopShell.ts, ui/MenuBar.ts, ui/Toolbar.ts, ui/NoteView.ts, ui/NoteActions.ts, ui/ManualEditor.ts |
| REQ-022 | Unsupported préservé | P1.4, P2.2, P2.3, P5.1, P6.2 | T004, T011–T012, T014, T039, T043 ; tests/fixtures/preservation.md, ui/MarkdownBridge.ts |
| REQ-023 | Autosauvegarde retrouvable | P2.2, P6.1, P6.2 | T012, T042–T044 ; storage/markdown_file.rs |
| REQ-024 | Conflits sans écrasement | P1.4, P2.2, P6.1, P6.2, P10.1 | T004, T011–T012, T042–T044, T055 ; tests/file_workflows.rs |
| REQ-025 | Affichage progressif | P3.2 | T030–T031 ; ui/ChangeView.ts |
| REQ-026 | Traitement et zones réelles | P2.3, P3.2 | T014–T016, T030 ; ui/StatusView.ts |
| REQ-027 | Lisibilité sans vol de focus | P2.3, P3.2 | T016, T030–T031 ; ui/ChangeView.ts |
| REQ-028 | Entrée volontaire et permission | P1.6, P2.6 | T006–T007, T023–T024 ; tests/lifecycle.rs |
| REQ-029 | Arrière-plan, indicateur interne, pause | P1.1, P1.6, P2.6, P4.2, P4.3 | T001, T006–T007, T025, T035–T037 ; ui/SessionStatus.ts |
| REQ-030 | États FR distincts | P2.3, P2.6 | T015–T016, T025 ; ui/StatusView.ts |
| REQ-031 | Silence sans arrêt imposé | P2.6, P4.3 | T023, T036 ; tests/lifecycle.rs |
| REQ-032 | Pause/arrêt invalidants | P1.6, P2.1, P2.6, P4.3 | T006, T008–T009, T023–T025, T036 ; domain/session.rs |
| REQ-033 | Pertes micro/réseau explicites | P1.6, P2.5, P2.6 | T006–T007, T020–T021, T023–T025 ; adapters/mistral_stt.rs |
| REQ-034 | Avertissement captation tiers | P2.6 | T025 ; ui/SessionStatus.ts |
| REQ-035 | Lock/sleep sans auto-reprise | P1.6, P2.6, P10.1 | T006–T007, T023–T024, T055 ; platform/lifecycle.rs |
| REQ-036 | Fermeture arrêt/sauvegarde/fin | P1.6, P2.2, P2.6, P6.2, P10.1 | T006, T013, T024, T044, T055 ; app/close.rs |
| REQ-037 | Échec fermeture sans perte | P1.6, P2.2, P6.2, P10.1 | T006–T007, T011–T013, T044, T055 ; tests/storage_integrity.rs |
| REQ-038 | Préférences accessibles | P2.4, P4.3, P7.1 | T019, T036, T045 ; ui/PreferencesView.ts |
| REQ-039 | Ports/fournisseurs distincts | P1.2, P1.4, P1.5, P2.1, P2.4, P2.5, P7.1, P10.3 | T002, T004–T005, T010, T019–T022, T045, T060 ; ports/stt.rs, ports/interpreter.rs |
| REQ-040 | Gestion masquée de clés | P2.4, P7.1 | T017–T019, T045 ; platform/secrets.rs, ui/PreferencesView.ts |
| REQ-041 | Trousseau sans fallback clair | P1.4, P1.6, P2.4, P7.1, P10.1 | T004, T006–T007, T017–T018, T045, T056 ; platform/secrets.rs |
| REQ-042 | Test explicite informé | P1.2, P2.4, P7.1 | T002, T019, T045 ; ui/PreferencesView.ts |
| REQ-043 | Prise d'effet et fournisseur choisi | P2.4, P2.5, P7.1 | T019–T022, T045 ; config/preferences.rs |
| REQ-044 | Coûts estimés qualifiés | P7.2 | T046–T047 ; domain/usage.rs |
| REQ-045 | Données/services transparents | P1.2, P2.4, P2.5, P7.2, P10.3 | T002, T019, T021–T022, T048, T060 ; docs/privacy.md |
| REQ-046 | Données temporaires purgées | P2.1, P4.2, P7.2, P9.1, P10.1 | T009, T034, T048, T052–T053, T056 ; tests/diagnostic_privacy.rs |
| REQ-047 | Diagnostic opt-in sensible | P9.1 | T052–T054 ; ui/TranscriptView.ts |
| REQ-048 | Segments qualifiés et vidage | P2.5, P9.1 | T020–T021, T052–T053 ; app/diagnostics.rs |
| REQ-049 | Aucune transcription persistante | P9.1, P10.1 | T052–T054, T056 ; docs/validation/privacy-audit.md |
| REQ-050 | Deux desktops à chaque jalon | P1.1, P1.3, P1.5, P1.6, P2.7, P3.2, P4.3, P5.1, P6.2, P7.2, P8.1, P9.1, P10.2, P10.3 | T003, T007, T026, T031, T037, T041, T044, T049, T051, T054, T058, T060 ; docs/validation/prototype-gnome.md, docs/validation/prototype-windows.md |
| REQ-051 | macOS objectif à vérifier | P1.3, P8.1, P10.2, P10.3 | T003, T050–T051, T058, T060 ; docs/validation/macos-status.md |
| REQ-052 | Mesures contextualisées | P1.3, P1.6, P2.7, P3.2, P4.3, P5.1, P6.2, P7.2, P8.1, P9.1, P10.2, P10.3 | T007, T026, T031, T037, T041, T044, T049, T051, T054, T058–T059 ; docs/validation/final-metrics.md |
| REQ-053 | Qualité quatre usages | P3.1, P8.1, P10.2 | T027–T029, T051, T057–T058 ; tests/fixtures/fidelity.json |

## État de livraison de la planification

Setup effectivement exécuté depuis la racine via feature.json, sans Git et sans écraser le plan existant. Les cinq livrables de planification sont finalisés ; les tâches restent non exécutées. Couverture : **53/53 exigences, 28 items de plan, 60 tâches inline conservant leurs IDs et 29 scénarios de recette référencés**, dont S4.5. Le plan est suffisant pour `/speckit.tasks`, pas une certification du produit.

La constitution, la spécification, la checklist, le cahier historique et la pièce jointe restent protégés. Aucun nouveau cadrage ni arbitrage fonctionnel n'est demandé. Les interfaces nécessaires sont dans ce plan ; aucune dépendance à des fichiers de design absents n'est créée. Hooks avant/après : fichier d'extensions absent lors des contrôles, donc aucun hook à exécuter.

### Contrôle documentaire effectivement exécuté

Contrôle documentaire Python/PyYAML et JSON Schema local, sans installation ni test produit : identifiants canoniques et couverture exacte, items et tâches uniques, dépendances inline/YAML identiques et acycliques, concordance de la table et des checkpoints, chemins et liens locaux/ancres, 29 scénarios dont S4.5, absence de placeholders non expliqués et d'artefacts interdits. Les champs `verification` des checkpoints consignent le résultat. Les chemins de code et rapports de recette restent **attendus**, jamais preuves acquises.

Empreintes SHA-256 identiques avant/après :

| Source protégée | SHA-256 |
|---|---|
| Constitution | `50dbb3a5258e3d506aaea03540a57be415eb105b9136537734b598b6132882dc` |
| Spécification actuelle | `145e0f8abbf57a92d507083d7b61798cc7854ca2cd66a9e8828c92a56e78a96a` |
| Checklist | `73c595eb338cc7fcf1dd5d32c998f2ad6ab90ec331c2255a8521e47d71ae6004` |
| Cahier historique | `43e769143914baef892dcfcd386602461e96f61a9a2a528e7cf74908007ac9c5` |
| Pièce jointe en lecture seule | `2c97bbaf918565830628176b44a7010f009977f7118ba80464e4b2f3fc9dfb1e` |

Ce contrôle valide les documents et leur traçabilité, **pas** l'accès compte, la qualité des modèles, les tarifs/conditions fournisseur, la future application ou les plateformes. L'alias LLM reste mutable ; coûts inconnus tant que les tarifs officiels ne sont pas vérifiés. Ces limites sont visibles et affectées aux premiers essais, sans masquer une validation runtime manquante.

> **Migration OpenAI — 2026-11-11.** Pour la configuration et la recette courantes,
> suivre le [guide OpenAI](../002-remplacer-services-openai/quickstart.md).
> Les bilans Bxx/Txxx ci-dessous documentent les versions antérieures Mistral,
> pas une validation OpenAI. Les modèles actifs par défaut sont
> `gpt-realtime-whisper` et `gpt-6-luna`, avec une clé OpenAI distincte au trousseau.
> `store:false` ne garantit pas l'absence de conservation fournisseur.

**Bilan T031 — passage de recette terminé, tâche encore ouverte.** Application
fermée par l’utilisateur, absence du processus vérifiée. Contrôles ciblés réussis
sur les versions successives corrigées ; deux éditions refusées pour ancre périmée
et texte attendu absent restent à examiner. Déplacement/suppression, position de
lecture et qualification complète des contrôles/mesures restent à terminer.
Aucun GO T031 ni passage à T032. [Bilan](../../docs/validation/us1-gnome.md).

**T031 — repérage avec surlignage temporaire (demande utilisateur).**
Défilement confirmé ; ajout d’un fond et d’un contour pendant deux secondes sur
le bloc repéré au clic, sans modifier le texte ni la sélection. Sources vérifiées
(54 tests JS, build web et check embarqué PASS) ; nouvelle recette visuelle requise.
Ancienne fenêtre fermée par l’utilisateur ; version avec surlignage reconstruite et ouverte en manuel. T031 reste ouverte.

**T031 — changement d’intention corrigé, essai ciblé confirmé.**
L’utilisateur valide e-mail/cordial → synthèse concise/neutre et conservation de
la réserve sur le binaire `1a98ba35…bd81c`. Le reste de la recette T031 reste ouvert.

**T031 — changement d’intention en échec natif, correction à recetter.**
Premier passage traité en éditions seules, sans action de profil. Consigne et
exemple renforcés pour exiger le changement explicite de forme/ton ; bilan
français distinct et diagnostic technique modifié/inchangé, sans contenu.
148 tests Rust PASS ; test de changement puis réouverture du fichier inclus.
La version corrigée est reconstruite et ouverte en manuel pour nouvel essai ; T031 reste ouverte.

**T031 — intention attachée à la note (demande du 2026-10-31).** Le profil
reste après arrêt et nouveau démarrage vocal ; il est enregistré dans un commentaire
Markdown versionné, séparé du corps, avec la même transaction et les mêmes conflits.
147 tests Rust et 54 JS PASS. [Format et garanties](../../docs/validation/note-intention-format.md).
Cette règle remplace les mentions historiques « intention temporaire/purgée à l’arrêt ».
Nouvelle recette native en cours ; ancienne fenêtre fermée par l’utilisateur, nouveau binaire ouvert en manuel.

**Complément T031 — intention visible (2026-10-31).** Premier essai e-mail/ajout
confirmé par l’utilisateur. Champ d’intention en lecture seule ajouté au-dessus
de la note sur demande explicite ; profil exposé uniquement à la fenêtre, sans
persistance. 144 tests Rust et 54 tests JS PASS. Build web et natif prêts ;
ancienne fenêtre fermée par l’utilisateur, nouvelle version ouverte en manuel.
T031 reste ouverte ; nouvel affichage pas encore recetté.

**B12-US1, 2026-10-31 — T028–T030 terminées, T031 en recette GNOME.**
Profil d’intention structuré et éphémère, orchestration de rédaction commune,
échecs techniques séparés et repères des changements réellement appliqués.
144 tests Rust et 52 tests JS PASS ; build web et natif embarqué PASS.
[Preuves et recette en cours](../../docs/validation/us1-gnome.md).
La fenêtre est ouverte en manuel, microphone arrêté au lancement. Aucun GO T031
avant les confirmations utilisateur. Windows et veille réelle restent exclus.
Ce bilan prévaut sur les états historiques ci-dessous.

**T027 terminée — corpus synthétique (2026-10-31).** Dix scénarios, quatre
usages, six tests de décisions/nuances sur le décodeur et le commit réels.
135 tests Rust PASS. Réponses scriptées, sans micro ni fournisseur ; aucune
validation générale de précision STT/LLM. [Corpus et limites](../../tests/fixtures/README.md).
Prochaine étape : T028 (modèle d’intention). Ce bilan B11 prévaut sur les états historiques.

**T026 — recette locale terminée (2026-10-31) :** fondations validées sur
GNOME/Wayland avec le micro NoMachine. Application fermée, modèle STT habituel
rétabli et note sauvegardée. Réserve : précision de transcription ; aucune
garantie générale de fidélité. Windows et veille réelle exclus par l’utilisateur.
Les fautes matérielles non exercées restent qualifiées dans la
[matrice](../../docs/validation/foundations-gnome.md) et les
[mesures](../../docs/validation/foundations-metrics.md). Prochaine tâche : T027.
Ce bilan B10 prévaut sur les états historiques ci-dessous.

**Reprise B09-LIFECYCLE.** L’état visible distingue initialisation, écoute réelle,
activité du traitement et pause/manuel. Le bouton « Mettre l’écoute en pause »
utilise l’arrêt partagé ; une reprise reste explicite. Après reconstruction,
recetter T026 sur GNOME/Wayland (Windows et veille réelle exclus), en suivant
[les points de contrôle](../../docs/validation/lifecycle-gnome.md).
Les recettes historiques ci-dessous ne valident pas automatiquement ce nouveau lot.

# Guide de validation — prototypes et jalons futurs

**Mise à jour B08-ADAPTERS (sources).** La capture et la normalisation partagées
se trouvent sous `src-tauri/src/audio/`, les adaptateurs Mistral sous `adapters/`.
Les anciens chemins `platform/` sont des réexports de compatibilité. Aucun
nouveau réglage ni fournisseur. Après une erreur ou un envoi annulé, le passage
est abandonné sans réémission automatique. Une réponse LLM doit être complète,
sans outil et conforme au schéma local avant toute édition. Les limites locales
restent des paramètres d'ingénierie, pas des garanties de service.
[Rapport](../../docs/validation/adapters-gnome.md). Binaire existant inchangé.

**Recette des préférences B07 :** scénarios de fenêtre PASS utilisateur et disque :
panneau, choix séparés, présence de clé, refus d'identifiant invalide,
enregistrement et persistance après redémarrage. Modèles habituels rétablis,
application ouverte ; aucun nouvel essai vocal. [Rapport](../../docs/validation/preferences-gnome.md).
Les mentions B07 antérieures de recette/binaire en attente sont historiques.

**Mise à jour B07-PREFERENCES (sources).** Après reconstruction native,
ouvrir Édition → Préférences : choix Mistral séparés pour STT et LLM, identifiants
intégrés suggérés et modèle personnalisé non vérifié. Enregistrer ne contacte
aucun fournisseur. Les nouveaux réglages prennent effet à la prochaine nouvelle
session explicite ; une reprise conserve la configuration suspendue. La clé est
masquée, envoyée par une commande dédiée, puis le champ est vidé ; sa fermeture
vide aussi une saisie non envoyée. Présence ne signifie pas accès au modèle.
Préférences JSON dans le dossier applicatif de configuration fourni par Tauri,
sans clé ni contenu de note ; dernière destination réellement sauvegardée mémorisée,
sans ouverture automatique. Fichier invalide = erreur visible et vocal refusé,
édition manuelle accessible. [Rapport](../../docs/validation/preferences-gnome.md).

**Recette manuelle GNOME B05/B06 :** scénarios exercés PASS utilisateur et disque :
affichage, édition/formatage/annulation, autosave, avertissement externe, copies
de conflit, résolution et fermeture normale. Correctif d'affichage chargé et testé ;
application fermée. [Preuves et limites](../../docs/validation/ipc-ui-gnome.md).
Les mentions antérieures de recette/binaire en attente sont historiques.

**Mise à jour B06-IPC-UI (sources).** L'entrée native utilise maintenant
`DesktopShell`, avec menus Fichier/Édition/Aide, bandeau, note et état séparés.
Les liens des notes restent du texte sans navigation ; les images ne sont pas
chargées ; les blocs HTML/opaques restent littéraux. Le collage insère uniquement
le texte brut, et le dépôt de contenu par glisser-déposer est désactivé.
Mise en forme et saisie désactivées en vocal. Les commandes natives ont des
permissions explicites limitées à la fenêtre principale ; aucun accès générique
filesystem/shell/réseau n'est ajouté. [Preuves](../../docs/validation/ipc-ui-gnome.md).
L'ancienne instance n'a pas été remplacée.

**Mise à jour B05-STORAGE (sources, 2026-10-31).** Après reconstruction native,
une note ouverte ou enregistrée vers une destination connue est autosauvegardée
après chaque modification acceptée. « Enregistrer » utilise cette destination ;
« Enregistrer sous » crée toujours un nouveau fichier exclusif. Les changements
externes ne sont jamais importés automatiquement : un conflit garde les versions,
les chemins de récupération et la note modifiée accessible. Pour le résoudre,
choisir une nouvelle destination. Sans destination ou en cas d'échec, la fermeture
est suspendue après arrêt/invalidation ; aucun redémarrage vocal automatique.
Les descriptions de copies systématiques ci-dessous concernent l'ancien prototype.
[Recette stockage](../../docs/validation/storage-gnome.md). Binaire actuel inchangé.

**Bilan courant de validation locale, 2026-10-31 :** [preuves et matrice G02–G09](../../docs/validation/prototype-gnome-final.md).
GNOME/Wayland uniquement, veille exclue par l’utilisateur, binaire direct sans
nouveau paquet. Dictée/correction/arrière-plan/pause/verrouillage/enchaînement
D6 PASS utilisateur. 74 Rust + 32 JS, build web/format, vrais fichiers (droits,
écriture partielle, opaque), trousseau isolé et perte de connexion du moniteur
GNOME PASS. Dernière recette visuelle opaque/copie/refus d’écrasement PASS utilisateur et
comparaison d’octets PASS. T006/T007 cochées pour la tranche GNOME/Wayland
convenue ; aucune certification Windows, veille, paquet ou fonction future.
Les états plus anciens ci-dessous sont historiques ; ce bilan courant prévaut.


## Reprise actuelle B04-GNOME — 2026-10-31

L'utilisateur reprend T006/T007 sur Fedora 44 GNOME/Wayland, sans travail
Windows et sans installation système supplémentaire. Binaire natif lancé depuis
`/home/laurent/projects/vocalnotes/src-tauri/target/debug/vocalnotes`, interface
embarquée sans Vite ; commencer par la recette manuelle, vocal non autorisé.
67 tests Rust/31 JS, build frontend/typecheck/format et build natif PASS ;
sondage natif GNOME/login1 PASS. Recette manuelle (paragraphes/formatage/undo, copie et nouvelle note) confirmée
PASS par l’utilisateur ; écriture et présence de clé au trousseau confirmées.
Micro NoMachine choisi ; premier essai vocal FAIL utilisateur : aucun texte.
Diagnostic D1 ajouté/testé hors ligne : bilan des modifications, clarifications,
refus et passages sans modification, conservé après arrêt sans transcription.
D1 reconstruit et relancé après fermeture sûre confirmée. Pour le prochain essai,
dicter du contenu dans la note existante ; ne pas demander « Crée une note »,
car `new` n'est pas encore intégré au parcours vocal de cette tranche.
Après essai D1 non concluant, D2 ajoute à la demande utilisateur une trace
stderr technique et des niveaux audio numériques par seconde. L’agent relit
la console de lancement (session outil D2 : 47020 ; démarrage confirmé) ;
aucun texte de note/transcription/clé ni audio brut
n’est journalisé. D2 prouve une réception audio non silencieuse et un final STT
non vide ; le commit échoue sur Texte attendu absent. D3 corrige le schéma des
notes vides (insert seul, ancre native exacte), 69 Rust ciblés PASS ; chargement
effectué après fermeture D2 code0, relance D3 session45903. Première dictée D3
PASS utilisateur et console (insert appliqué, aucun refus), arrêt micro confirmé.
Correction et pause PASS utilisateur ; console replace appliqué, puis pause et
fermeture micro confirmées. Essai arrière-plan FAIL : remplacement au lieu d’ajout.
D4 renforce le prompt ; build PASS et chargé en session5444 après fermeture
D3 code0. Retour D4 : correction réussie et idées fusionnées ; conservation
rapportée, nuance et changement de fenêtre non certifiés. Évaluer la fidélité
du contenu plutôt que le seul choix technique insert/replace. Ne pas utiliser un outil de capture audio ou lire le trousseau
pour recopier ces données dans les preuves.
Résultats et étapes : [rapport GNOME](../../docs/validation/prototype-gnome.md).
Ne pas relancer/recompiler tant qu'une note peut être ouverte. Les instructions
de priorité Windows et de report GNOME ci-dessous sont historiques.

## B03-OFFLINE — développement actuel, pas recette vocale

T010 terminée et validée hors ligne : 67 Rust Linux, 64 Rust Windows ciblés
(9 lib + 55 dans le harnais partagé), 31 JS sur chaque hôte. Aucune API,
capture, sondage réseau, installation ou modification du trousseau.
L'application existante reste sur son ancien exe : aucun nouveau code chargé.
T006/T007 ouvertes, GNOME différée ; ces tests ne certifient aucun jalon.

Pour rejouer sans remplacer un exe Windows ouvert :
```powershell
.\tooling\windows-native.ps1 -Action OfflineTests
```
Ce mode interdit auto-installation et accès fournisseur ; les tests existants
passent par `--lib` puis `--example offline_contract_tests`, avec garde
Windows interdisant écriture/suppression de l'exe et comparaison de hash.
Ne pas utiliser `Validate`, `Build`, `cargo test --tests` ou `cargo rustc
--test` pendant qu'une note peut être ouverte : Cargo peut relinker le binaire.
Le harnais réutilise les fichiers de tests, sans duplication ni mocks disque.
T011–T013 restent non commencées : vrais tests de stockage/courses/permissions
avant leur implémentation. La copie exclusive du prototype n'est pas un
remplacement atomique contrôlé. Les attentes B02 suivantes sont historiques.

## B02-W05 — priorité sur les attentes historiques ci-dessous

Premier passage vocal Windows **FAIL utilisateur**, clé au trousseau et
consentement court déjà confirmés. **Arrêter / Manuel, ne pas réessayer.**
Probe WSS sans clé : HTTP/1.0 incompatible (`WrongHttpVersion`), TCP/TLS
vérifiés établis ; statut et cause du premier appel authentifié inconnus.
Correctif de diagnostic testé source/copie Windows, **pas encore dans
l'exe PID16144 actif**. Aucun changement TLS/proxy, micro ou appel
authentifié par l'agent. Sauvegarde/fermeture confirmées avant link,
handshake conforme sans clé avant nouvelle recette explicitement consentie.
T006/T007 ouvertes, GNOME différée.
[Diagnostic, tests et limites](../../docs/validation/prototype-windows.md).

## Reprise vocale Windows B02-W04 — état actuel, 2026-10-31 à 02:23

Les PID et attentes W01/W02 ci-dessous sont historiques. Manuel W03
Entrée/deux paragraphes/copie/nouvelle note PASS utilisateur à 02:20 ;
build/link MSVC custom-protocol W03 réussi. Ancienne instance terminée code0.
Préflight Windows actuel `windows-native.ps1 -Action Probe` **PASS code0** :
surveillance WTS initialisée sans micro/clé/API, pas preuve lock/veille.
Permissions microphone lues Allow, pas preuve capture.

Fenêtre corrigée native **PID16144**, titre **Bloc-notes vocal**,
`Responding=True`, session détachée **windows-voice-recipe**, sans Vite.
Conservée pour l'utilisateur. NodeWindows25.8.2 hors cible22 inchangé.
**Ne pas cliquer Démarrer ni cocher l'accord avant consentement explicite
micro ET appels Mistral, coût inconnu potentiellement non nul.**
Nouvelle clé uniquement au champ masqué puis Credential Manager, jamais
ancienne clé exposée, fichier/CLI/env/chat. Préparation et étapes graduelles :
[prototype-windows.md](../../docs/validation/prototype-windows.md).
Le prochain échange demande d'abord l'accord, puis le résultat trousseau ;
premier passage synthétique ≤30s seulement ensuite, arrêt manuel obligatoire.
Vocal, trousseau réel et événements OS NON EXÉCUTÉS ; T006/T007 ouvertes,
GNOME différée et aucun GO. Constitution inchangée.
Sortie : Arrêter vocal, attendre les acquittements, sauvegarder si nécessaire,
Fichier → Fermer après sauvegarde ; ne pas forcer un refus de sauvegarde.

**2026-10-31 — B02-W01 : prototype compilé et lancé nativement Windows,
44 tests Rust/22 JS PASS sur Windows et source Linux. T006 partielle,
T007 ouverte, T010 partielle ; recette interactive à faire.**
Le protocole de recette ci-dessous reste inchangé.

**Mise à jour B02-W02 :** menus horizontaux corrigés dans la source et la
copie Windows, 25 tests JS/typecheck/build frontend et check MSVC PASS.
Le binaire **n'a pas encore été relinké** : PID 25360 reste B02-W01.
Sauvegarder/récupérer les saisies et fermer proprement, puis confirmer avant
build et relancement par l'agent. Aucun arrêt forcé ni micro pour ce correctif.
La recette de la nouvelle barre reste NON EXÉCUTÉE dans WebView2 ; suivre
la première étape mise à jour du guide Windows. Ne pas utiliser la clé exposée :
révocation conseillée, aucun test API avec celle-ci.

## Windows natif — instance actuelle et première étape

Guide détaillé, gates et limites :
[prototype-windows.md](../../docs/validation/prototype-windows.md).
Copie : `C:\Users\lmirguet\Development\vocalnotes-native-B02-W01`.
Binaire : `src-tauri\target\debug\vocalnotes.exe`, assets embarqués.
Fenêtre **Bloc-notes vocal**, PID **25360**, WebView2 enfant **9376**,
session outil détachée **windows-ui** conservée. Aucun serveur dev.
MSVC/SDK réels, Rust 1.99.1 MSVC, WebView2 156.0.4314.50.
Node Windows 25.8.2 hors plage Node 22 : tests verts avec écart signalé,
pas de certification Node 22 ni changement des manifestes.

```powershell
Set-Location 'C:\Users\lmirguet\Development\vocalnotes-native-B02-W01'
.\tooling\windows-native.ps1 -Action Build
.\tooling\windows-native.ps1 -Action Probe
# Après fermeture de l'instance actuelle seulement :
.\tooling\windows-native.ps1 -Action Run
```

Build = npm test/build/typecheck et cargo fmt/test/check/build
`--features tauri/custom-protocol` pour le binaire embarqué, sans NSIS.
Probe initialise WTS sans micro/trousseau/API ; code 0 ici, pas recette
lock/suspend. Environnement uniquement local au processus, aucune installation
système ou global PATH. Auto-installation rustup implicite lors du premier
sondage documentée ; les scripts suivants la refusent.

**Commencer sans micro/clé :** confirmer fenêtre/menus/note et état Manuel,
essayer texte synthétique, Gras/Titre/Annuler et copie Markdown exclusive
à un chemin neuf. Attendre les acquittements. Ensuite seulement, recette
trousseau par champ masqué et services/micro après accord explicite.
Ne jamais fournir la clé au chat, CLI, env ou fichier d'import.
Premier constat utilisateur attendu ; aucune manipulation interactive
déclarée PASS à partir d'un simple HWND.

Pour sortir : Fichier → Fermer après sauvegarde, récupérer toute saisie
refusée, puis arrêt du helper windows-ui s'il reste actif.
X seul peut suspendre la fermeture ; ne pas forcer pour contourner la sauvegarde.
Commande d'arrêt d'urgence ciblée avec contrôle chemin/PID dans le guide.
T006/T007 non cochées. GNOME/Wayland différée, sans blocage du développement.

## Lancement natif WSLg réellement exécuté — B02

**Historique seulement : cette instance a été fermée avant B02-W01,
PID 126037/126008/125926 disparus. Ne pas la relancer à la place de Windows.**

Depuis la session WSL2 avec WSLg actif, commande réellement lancée :

```bash
cd /home/laurent/projects/vocalnotes
export RUSTUP_HOME=/home/laurent/.local/share/vocalnotes-build/rustup
export CARGO_HOME=/home/laurent/.local/share/vocalnotes-build/cargo
export PATH=/home/laurent/.local/share/vocalnotes-build/cargo/bin:$PATH
npm run tauri -- dev --no-watch
```

La fenêtre **Bloc-notes vocal** doit apparaître sur le bureau Windows via
WSLg, pas dans un navigateur. Le programme est un binaire **Linux natif**,
pas une application Windows. Association WSLg `vocalnotes`/fenêtre `0x6`
constatée dans Weston ; contenu visuel et gestes UI non inspectés à distance.
Vite `http://127.0.0.1:1420/` retourne HTTP 200 et sert la webview native,
il ne constitue pas une preuve de recette à lui seul.

La sortie native contient des avertissements EGL/Mesa (`ZINK: failed to choose
pdev`, écran `dri2` non créé). Aucun crash constaté : application,
WebKitWebProcess `126247` et WebKitNetworkProcess `126155` restent vivants.
Cela ne prouve pas que le rendu est correct : confirmer visuellement la note
sur le bureau WSLg et signaler une éventuelle fenêtre vide. Aucun réglage de
contournement sandbox ou de remplacement forcé du pilote n'a été ajouté.

Processus laissés actifs en session outil **détachée `wslg-ui`** :
application `126037`, Vite `126008`, superviseur Tauri Node `125926`.
Ne pas lancer une seconde instance tant qu'ils sont présents.
Le processus UI a été lancé avant les modifications B02 avec `--no-watch` ;
il reste sur B01-R3 pour préserver les éventuelles saisies de l'utilisateur.
B02 est compilé/testé ; fermer proprement puis relancer cette commande
pour l'exercer. Pas de redémarrage automatique.
Pour arrêter : préférer Fichier → Fermer après sauvegarde, puis Ctrl+C
dans le terminal de lancement ; dans cette session outil, `stop_bash`
avec `shellId: wslg-ui` arrête tout l'arbre, **uniquement après sauvegarde**.
Depuis un autre terminal, après fermeture propre de la fenêtre :
`kill 125926 126008` (vérifier d'abord ces PID).

Aucune capture au démarrage. Ne pas démarrer le microphone pour cette recette
d'interface. Le sondage `probe_lifecycle` constate la surveillance native
indisponible : vocal refusé avant CPAL/trousseau, aucune option de bypass.
Aucun micro, appel fournisseur ou accès à la clé racine pendant B02.
Manuel, menus, formatage, undo et copie Markdown peuvent être explorés.
Sauvegarde actuelle : **nouvelle copie exclusive**, pas remplacement contrôlé.
Windows natif et GNOME complet restent NON EXÉCUTÉS, T007 ouverte.

Validation B02 : 44 tests Rust (dont 7 sessions, 5 ports), 22 JS,
`cargo fmt/check --tests/build`, `npm run typecheck/build` PASS.
T008–T009 cochées, T010 partielle (contrat complet à formaliser),
T011–T013 non commencées. Aucun taux de couverture annoncé.
Les interdictions historiques de commencer T008 dans les sections suivantes
ne s'appliquent plus au développement autorisé ; elles ne deviennent pas une
validation du gate.

**Organisation décidée par l'utilisateur le 2026-10-31 :** commencer par la
recette native Windows 11 ; différer GNOME/Wayland à une session ultérieure.
Le développement peut continuer pendant ce report, conformément à la
dérogation décrite dans [tasks.md](tasks.md). T007 et les certifications
multiplateformes restent ouvertes ; chaque résultat est attribué à son OS.
Les mentions ci-dessous interdisant T008 en l'absence des deux desktops
décrivent le gate initial et sont remplacées, pour le développement uniquement,
par cette décision. Le protocole et les exigences Linux restent à exécuter.

## État réellement vérifié du socle T005

Depuis `/home/laurent/projects/vocalnotes`, reprise T005 **avant modification T006** :
12 tests JS, 1 test Rust, `cargo check/build`, format et frontend build/typecheck
PASS. Les SDK desktop installés par l'utilisateur sont réellement détectés.
T005 est cochée, sans certification desktop. Aucun Git initialisé.

Rust 1.99.1 est installé localement, sans modification du PATH global :

```bash
export RUSTUP_HOME=/home/laurent/.local/share/vocalnotes-build/rustup
export CARGO_HOME=/home/laurent/.local/share/vocalnotes-build/cargo
export PATH=/home/laurent/.local/share/vocalnotes-build/cargo/bin:$PATH
cargo fmt --manifest-path /home/laurent/projects/vocalnotes/src-tauri/Cargo.toml -- --check
cargo check --locked --manifest-path /home/laurent/projects/vocalnotes/src-tauri/Cargo.toml
cargo test --locked --manifest-path /home/laurent/projects/vocalnotes/src-tauri/Cargo.toml --lib
```

Le format et les métadonnées verrouillées passent sur l'arbre final.
**État final B01-R3 avec ALSA installé par l'utilisateur :**
`cargo check --locked --tests`, `cargo test --locked --tests` (**32 cas**),
`cargo build --locked` et format passent, CPAL compris. Ajouter les options
de chemin du bloc ci-dessus. Frontend : **22 tests JS**, typecheck/build PASS.
`windows_lifecycle_types.rs` compile les types Windows sur Linux : **0 test
runtime Windows**, ni MSVC/SDK ni binaire Windows certifiés.
`cargo run --locked --manifest-path src-tauri/Cargo.toml --example probe_lifecycle`
ne touche ni micro ni trousseau : sous WSL2, **code 2**, surveillance
indisponible. Ce diagnostic n'est pas une recette GNOME. Voir
[environments.md](../../docs/validation/environments.md) et
[batch-report.yaml](batch-report.yaml). Ne pas lancer le parcours T008 ni annoncer
le prototype après un simple build Vite.

`src/ui/Prototype.ts` monte maintenant une **tranche T006 partielle** dans Tauri :
édition de paragraphes/titres, structure simple/undo confirmé, menus/bandeau,
saisie masquée/keyring et copie Markdown exclusive.
Contrôleur et abonnements OS raccordés : ne pas démarrer de micro WSL comme
substitut à T007. Accord contextualisé de la fenêtre, clé au trousseau et
surveillance initialisée exigés ; absence = erreur avant capture.
Rien ne démarre au lancement. Une liste créée devient opaque ; pont complet
hors de cette tranche. X arrête/invalide et suspend la fermeture. Attendre les
acquittements, récupérer la saisie non acceptée si nécessaire, puis Fichier →
Fermer après sauvegarde. Une copie `vocalnotes-fermeture-<identifiant>.md`
est créée auprès de la destination connue ; sans destination ou en échec,
note accessible sans reprise micro. Save courant/vocal crée
`vocalnotes-copie-<identifiant>.md`, sans remplacer la note ouverte.
Pas de fichier audio/transcription/cache ; tarifs et conservation inconnus.
Trousseau réel non exercé, aucun micro démarré, aucun appel Mistral, aucune lecture
de la clé locale. Aucun helper laissé en cours.

Le blocage ALSA est levé. T006 reste non cochée faute de vérification complète
de la tranche sur les cibles ; T007 NON EXÉCUTÉE ; aucun passage à T008+.
Fournir les vrais desktops puis suivre les recettes synthétiques informées.

Le reste du guide est le protocole de recette future, pas la preuve d'un binaire
natif ou des deux environnements. Les entités, interfaces, transitions et règles
de sauvegarde sont définies dans [plan.md](plan.md) et revues en T004, sans
prérequis de design autonome.

## 1. Prérequis et autorisation

1. Lire [spec.md](spec.md), la [constitution](../../.specify/memory/constitution.md), [research.md](research.md) et les gates du [plan](plan.md#gates-de-planification-et-jalons-de-recette).
2. Vérifier Q01 : indicateur d'écoute et précision affichés uniquement dans l'application et consultables au retour volontaire ; aucune extension GNOME pour ces affichages, aucun indicateur hors fenêtre, notification du bureau, lecture vocale ni prise de premier plan imposée.
3. Obtenir deux sessions desktop interactives réelles, GNOME/Wayland et Windows 11. Un hôte Linux distant ne les fournit pas implicitement. Noter distribution/build, shell, affichage, CPU, mémoire, micro, backend audio, version WebKitGTK ou WebView2 et mode de distribution.
4. Préparer Rust stable édition 2024, Node 22 LTS, Tauri 2 et dépendances système WebKitGTK/ALSA/Secret Service Fedora ou WebView2/outillage Windows. T005 fige patches et lockfiles ; cibles RPM/NSIS. Aucune installation réalisée ici. Ne pas supposer qu'une compilation Linux suffit.
5. Pour la version migrée, utiliser OpenAI : `gpt-realtime-whisper`, PCM mono s16le transmis à 24 kHz (VAD local conservé à 16 kHz), et `gpt-6-luna` via Responses avec schéma strict et `store:false`. Suivre le [contrat OpenAI](../002-remplacer-services-openai/contracts/openai-services.md). Distinguer modèle configuré/acquitté et modèle effectivement servi quand la réponse l'atteste. Relever tarifs officiels datés si accessibles ; sinon coût « inconnu », jamais zéro. Conditions externes non vérifiées : aucun contenu sensible. Obtenir un nouvel accord sur microphone, contact externe et coût possible avant une recette réelle, sans reprendre les anciennes autorisations Mistral.
6. Saisir une clé dans le champ masqué des préférences locales, envoyé au trousseau via la commande dédiée puis vidé sans copie dans l'état d'UI. Ne pas la mettre en variable d'environnement, commande, `.env`, note ou log. Vérifier backend réel, jamais mock ou fallback clair.

## 2. Commandes futures proposées

**Commandes à exécuter seulement après les tâches créant manifestes, scripts et tests.** T005 fournit scripts et verrouillages ; T059 confirme les commandes réellement validées. Aucune installation, compilation ou exécution de ces commandes pendant la planification.

Sur GNOME/Wayland, depuis un terminal de la session interactive :

```bash
cd /home/laurent/projects/vocalnotes
npm --prefix /home/laurent/projects/vocalnotes ci
cargo test --manifest-path /home/laurent/projects/vocalnotes/src-tauri/Cargo.toml --test session_validity
cargo test --manifest-path /home/laurent/projects/vocalnotes/src-tauri/Cargo.toml
npm --prefix /home/laurent/projects/vocalnotes run test:unit
npm --prefix /home/laurent/projects/vocalnotes run tauri -- dev
```

Le premier test ciblé n'existe qu'après T008 ; la suite complète seulement après les tâches correspondantes. Le mode dev sert au diagnostic, **pas** à certifier installation, distribution, permissions ou absence de traces du paquet final. Les paquets de distribution devront être installés et lancés selon le format retenu dans `docs/validation/environments.md`, futur fichier de T003, puis soumis à la même recette. Aucun format/package n'est annoncé testé.

Sur Windows 11, PowerShell dans la session desktop, choisir le **chemin absolu réel** de cette même version du projet (pas un chemin Linux fictivement transposé) :

```powershell
$ProjectRoot = Read-Host "Chemin absolu du projet sur cette machine Windows"
if (-not [IO.Path]::IsPathFullyQualified($ProjectRoot)) { throw "Chemin absolu requis" }
Set-Location -LiteralPath $ProjectRoot
npm --prefix $ProjectRoot ci
cargo test --manifest-path (Join-Path $ProjectRoot "src-tauri/Cargo.toml") --test session_validity
cargo test --manifest-path (Join-Path $ProjectRoot "src-tauri/Cargo.toml")
npm --prefix $ProjectRoot run test:unit
npm --prefix $ProjectRoot run tauri -- dev
```

Résultat attendu des petits tests : sessions invalidées toujours rejetées, opérations idempotentes, modes exclusifs, conflits et échecs non destructifs, secrets expurgés, purge et coûts incomplets explicitement qualifiés. Un mock n'apporte aucune preuve sur l'indicateur, le microphone, le trousseau ni le cycle OS.

## 3. Recette obligatoire à chaque jalon, dès le premier prototype

**Pas de repli implicite :** une case non exécutée vaut « non vérifié » et bloque le go si elle couvre une fonction livrée ou un invariant transversal. Un test impossible sur l'un des deux OS n'est pas remplacé par le résultat de l'autre.

| Séquence | Manipulation future | Résultat attendu / référence |
|---|---|---|
| A — installation | Installer le paquet retenu ; lancer à froid ; noter versions et ressources. | Installation et démarrage réels sur chaque cible ; S8.1/S8.3, REQ-050/052. |
| B — entrée volontaire | Démarrer en manuel, refuser puis autoriser le micro, lancer explicitement le vocal. | Aucune capture avant entrée ; permission/erreur visible ; REQ-028/033. |
| C — continuité | Parler plusieurs fois sans réactivation, déplacer le focus puis minimiser. | Flux continu, pas de raccourci/pression à chaque intervention ; REQ-001/029. |
| D — arrière-plan et affichages internes | Masquer l'application, vérifier l'écoute continue puis demander pause à la voix. Revenir consulter l'état courant. Provoquer aussi une ambiguïté avec l'application masquée, puis revenir volontairement et répondre à la demande affichée. | Pause vocale rapide en arrière-plan, indicateur et précision dans l'application uniquement ; aucun indicateur hors fenêtre, notification du bureau ni focus volé ; sinon G04 no-go. |
| E — interruption | Retarder un résultat, demander pause, arrêt ou manuel ; reprendre explicitement et recevoir l'ancien résultat. | Ancien résultat sans effet, passage non intégré signalé ; S3.1/S3.3, SC-004. |
| F — verrouillage/veille | Avec requête en vol, verrouiller puis déverrouiller, puis veille/réveil ; inclure perte d'abonnement OS. | Capture arrêtée, session invalidée, manuel au retour, aucune auto-reprise ; REQ-035. |
| G — trousseau | Saisir/remplacer/supprimer un secret de test par champ masqué local ; vérifier son vidage, verrouiller/rendre indisponible le service de secrets. | Présence seule après saisie, erreur sans stockage clair ni mock, aucun secret en logs ; REQ-040/041. |
| H — fichiers/fermeture | Modifier une note, provoquer disque plein/droits/conflit externe, fermer sans destination puis avec sauvegarde possible. | Arrêt/invalidation avant sauvegarde, erreur laisse note accessible sans micro, sortie seulement après sauvegarde ; REQ-036/037. |
| I — perte service | Couper réseau ou retirer le micro ; simuler erreur du fournisseur et format invalide. | Erreur explicite, non-intégré distingué, pas de fournisseur caché ni prétendue récupération de paroles absentes ; REQ-033/043. |
| J — silence/tiers | Laisser un silence long et rappeler le risque de tiers avant arrière-plan. | Pas d'arrêt pour silence seul, avertissement et pause vocale accessibles ; REQ-031/034. |
| K — mesures | Mesurer démarrage, mémoire/CPU, temps parole complète→STT→LLM→commit et bout en bout ; qualifier coût STT/LLM. | Conditions et limites notées, inconnues explicites, aucun seuil ou prix inventé ; REQ-044/052. |

Au premier prototype, les essais sont limités à sa tranche minimale mais tous les invariants ci-dessus doivent déjà être démontrés. Ajouter : rotation automatique de passages STT avec résultat `done`, capture continue et tampon borné, interruption naturelle prioritaire sans édition sur delta, correction du provisoire dans le diagnostic si activé, et échec explicite sans modèle de secours. Vérifier ProseMirror visuel en manuel, read-only sans history en vocal, menus/bandeau S4.5 et conservation d'un bloc opaque. Les fonctions de story non encore livrées ne sont pas annoncées disponibles. Chaque nouveau jalon rejoue A–K et ses parcours.

## 4. Parcours fonctionnels à ajouter en progression

- **US1 / S1.1–S1.4 :** partir d'une note vide ; dicter en français idées, chiffres et réserves, indiquer un ton puis changer d'intention. Observer progression, traitement et zones réelles, sans saut de navigation. Refaire pour e-mail, compte rendu rapporté, synthèse et document professionnel. Comparer aux propos synthétiques : aucun fait ajouté accepté ; signaler omissions et nuances altérées.
- **US2 / S2.1–S2.5 :** ajouter, déplacer, supprimer, reformuler et restructurer ; mélanger ton, édition et sauvegarde ; rapporter « il faut supprimer les fichiers temporaires » comme contenu ; demander « supprime ce point » avec deux cibles plausibles, seul puis en fin d'intervention mixte ; épeler avec casse/accents/tiret/apostrophe ; demander une annulation d'historique vocal. Attendre zéro action de l'intervention ambiguë, message interne sans reprise automatique, ordre des intentions non ambiguës, édition ciblée sans confirmation ordinaire et refus du mécanisme undo vocal, pas refus de toute correction.
- **US3 / S3.1–S3.4 :** passer en manuel avec requête retardée, écrire/annuler classiquement, reprendre explicitement et reformuler le texte manuel. Tester aussi saisies rapprochées, formatage puis undo, refus natif, acquittement dupliqué et événement retardé : après stabilisation, l'éditeur représente le contenu du cœur, sans double application ni régression. Après refus, message explicite, restauration canonique, saisie refusée récupérable temporairement et reset signalé de l'historique ; undo ne réintroduit pas le refus. Ancien résultat toujours rejeté ; résultat actif mais périmé signalé ou recalculé. Fermer/relancer : pas d'undo manuel restauré.
- **US4 / S4.1–S4.5 :** créer/ouvrir/retrouver/enregistrer sous/copier/exporter, une note à la fois ; retarder un résultat puis créer, ouvrir ou rouvrir une note avec contenu et révision similaires : aucun ancien traitement ne modifie la note active. Répéter avec ouverture en échec, sans réactivation des anciens résultats. Comparer octets hors cible avec tableaux/code/CRLF/Unicode ; édition et formatage visuels, opaque protégé avant altération ; modifier le fichier dans une autre application pendant sauvegarde et vérifier copie de conflit ; tester interruption et primitive indisponible sans remplacement aveugle. Dernière sauvegarde effective retrouvable, pas de promesse de récupération du non-sauvegardé. Menus Fichier/Édition/Aide et bandeau, pas source/preview imposé ni chat.
- **US6 / S6.1–S6.5 :** ouvrir préférences à la voix, vérifier modèles intégrés et accès réel distingués, trousseau et champ masqué vidé, test externe informé, réglages à session suivante et estimation avec composantes inconnues. Pas d'OpenAI annoncé intégré.
- **US8 / S8.1–S8.3 :** vérifier tous les jalons sur les deux desktops ; macOS reste objectif non confirmé tant que sa propre recette n'est pas faite.
- **US7 / S7.1–S7.3 :** diagnostic inactif par défaut ; l'activer pour une seule session ; observer provisoire/final/corrections seulement si exposés ; terminer et redémarrer. Vue, transcription, conversation et lexique vidés ; seules corrections de la note restent. Ni audio enregistré ni console persistante.

Il n'y a **aucun scénario S5 ni US5** à recréer. Ses fonctions demeurent dans A–K et les fondations.

## 5. Preuves à consigner

Pour prototype, fondations, US1, US2, US3, US4, US6, US7 et final : deux rapports `docs/validation/<jalon>-gnome.md` et `docs/validation/<jalon>-windows.md` et un rapport de mesures, selon les noms exacts des tâches du plan. Cette notation décrit une famille de futurs rapports, pas des fichiers existants. US8 consolide la matrice sans supprimer les preuves par jalon.

Chaque rapport contient : date, version du binaire et des dépendances/modèles, matériel et OS/session, méthode, scénario/REQ, attendu, observé, statut PASS/FAIL/NON EXÉCUTÉ, limite et décision go/no-go. Les métriques comportent nombre d'observations et dispersion, début/fin des chronomètres et conditions réseau ; coût mesuré ou estimé avec source/date ou inconnu. Ne conserver ni vrai secret, audio, transcription ou conversation dans le rapport. Comparer les erreurs STT, compréhension et application séparément.

Contrôle final : les **29 scénarios** S1.1–S1.4, S2.1–S2.5, S3.1–S3.4, S4.1–S4.5, S6.1–S6.5, S7.1–S7.3 et S8.1–S8.3, ainsi que SC-001 à SC-007, sont recettés dans les jalons correspondants. Critères G02–G09 satisfaits pour le périmètre livré, revue G10 réalisée, constitution recontrôlée, limites macOS honnêtes. **À ce jour : aucune de ces recettes n'est passée ; cela ne remet pas le plan documentaire en ERROR.**

Recette D4 suivante : réserve en arrière-plan PASS utilisateur ; fermeture
code0. Même version relancée session4710, micro arrêté. Prochain test :
verrouillage GNOME, contrôle arrêt micro et absence de reprise automatique.

Verrouillage GNOME D4 PASS utilisateur/console session4710 : mode manuel,
flux micro fermé, aucune auto-reprise observée. Prochaine recette : sauvegarde,
démarrage explicite puis suspension/reprise ; retour manuel sans auto-reprise.
La connexion NoMachine peut être interrompue par la suspension.

Périmètre utilisateur actualisé : ne pas proposer de test de veille sur cet
ordinateur ; exclu de la recette locale, non validé, sans blocage local.
Poursuivre par trois passages distincts dans une même session vocale.

Enchaînement D4 FAIL : réponses fournisseur hors contrat, aucun effet.
D5 améliore message/diagnostics et consigne ; build PASS, chargé session56340
après fermeture code0. Résultat réel non validé : reprendre trois passages
distincts dans une même session. Veille exclue de la recette locale.

D5 deuxième passage FAIL : clarify avec actions et sans message. D6 aligne
le schéma JSON sur les invariants natifs (deux alternatives exclusives).
D6 chargé session26421 après fermeture code0 et build PASS ; compatibilité
fournisseur et enchaînement désormais PASS sur cet essai utilisateur/console
(applied5, aucun refus, pause et fermeture micro confirmées). Session26421
ouverte en pause ; autres gates T006/T007 encore à compléter.

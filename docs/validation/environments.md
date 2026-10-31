# T003 — moyens réellement disponibles

## État actuel B02-W04 — 2026-10-31 à 02:23

Les sections W01/W02 ci-dessous sont historiques. Source autoritaire WSL :
`/home/laurent/projects/vocalnotes`. Copie Windows inchangée :
`C:\Users\lmirguet\Development\vocalnotes-native-B02-W01`.

- Recette manuelle W03 deux paragraphes/copie/nouvelle note **PASS utilisateur
  à 02:20** ; build/link MSVC custom-protocol W03 déjà PASS. Barre visuelle
  PASS à 01:57. Suites W03 précédentes : 31 JS/46 Rust Linux et Windows,
  typecheck/build/check PASS, non rejouées pendant cette préparation.
- `windows-native.ps1 -Action Probe` exécuté via PowerShell Windows natif :
  **code 0**, WTS initialisé. Aucun événement lock/veille provoqué,
  aucun micro, trousseau ni API. Rust 1.99.1 MSVC, VS18.11.3/SDK10.0.26100.0.
- Permissions microphone ConsentStore Value HKCU/HKLM/HKCU NonPackaged :
  **Allow**, lecture seule ; capture WASAPI/CPAL **non vérifiée**.
- Aucune instance avant lancement ; nouvelle fenêtre native répondante,
  titre Bloc-notes vocal, PID **16144**, HWND658332, début **02:22:50 CET**,
  session détachée **windows-voice-recipe** conservée, sans Vite/Start-Process.
  Exe corrigé W03 écrit 02:14:28, SHA-256
  `CC67C1CCC3E1917E3086A00F2C1FA062A514E5D23E0549064B5E49A52FC25923`.
- Parité source/copie UI/controller/capture/secrets/lifecycle_windows/script
  PASS ; aucun correctif, dépendance ou installation nécessaire.
  Node Windows25.8.2 reste hors cible22. Constitution inchangée.
- Consentement micro/API **à demander**, aucun accord vocal acquis.
  Vrai trousseau, micro/services et événements Windows **NON EXÉCUTÉS** ;
  GNOME différée, T006/T007 ouvertes, T010 partielle, aucun GO.

Procédure graduelle, messages exacts, consentement et sortie sûre :
[état actuel de la recette Windows](prototype-windows.md).

## Correctif B02-W02 — 2026-10-31, Windows 11 natif

Source autoritaire et copie B02-W01 synchronisées uniquement sur
`src/ui/Prototype.ts`, `src/ui/Prototype.test.ts`, `src/style.css`,
après vérification d'égalité préexistante ; aucun conflit/écrasement inconnu.
25 tests JS et typecheck/build frontend PASS sur Windows et Linux.
`cargo +1.99.1 check --locked --bin vocalnotes --features tauri/custom-protocol`
MSVC PASS (cl/link/rc du DevShell x64 constatés). Aucun test Rust rejoué :
Rust inchangé, 44 tests B02-W01 restent une preuve historique.
Node Windows reste 25.8.2 hors cible 22 ; aucun outil/paquet installé.

**Nouveau link/build exe et relancement : NON EXÉCUTÉS**, attente de confirmation
sauvegarde/fermeture propre de l'ancienne fenêtre PID 25360/ HWND 396550,
toujours répondante après les checks. Empreinte exe B02-W01 inchangée.
Session détachée `windows-ui` conservée, aucune session WSLg relancée.
Nouveau rendu WebView2/gestes NON EXÉCUTÉS ; GNOME différée sans blocage
de développement. Micro, API et accès clé/trousseau par l'agent : zéro.

## État actuel — B02-W01, Windows natif, 2026-10-31

Ce résultat remplace les disponibilités négatives des sondages historiques
ci-dessous ; ceux-ci ne décrivent plus l'outillage installé.
Guide/commandes/preuves : [prototype-windows.md](prototype-windows.md).

- Windows 11 Pro 10.0.26200/build 26200 accessible via PowerShell 7 natif :
  `/mnt/c/Users/lmirguet/AppData/Local/Microsoft/WindowsApps/pwsh.exe`.
- `C:\Users\lmirguet\.cargo\bin` : Rust/Cargo 1.99.1,
  **x86_64-pc-windows-msvc**. Le premier sondage rustup depuis le projet a
  déclenché l'auto-installation implicite de 1.99.1 ; téléchargée, non
  intentionnelle. Les commandes suivantes interdisent l'auto-installation.
- vswhere réel sous `C:\Program Files (x86)\Microsoft Visual Studio\Installer` :
  VS Build Tools 2026 18.11.3 installé par l'utilisateur,
  `C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools`.
  DevShell x64 fournit cl 19.51.36260/MSVC 14.51.36231, link.exe et
  rc.exe/SDK **10.0.26100.0**. Aucune modification du PATH global.
- Node/npm natifs `C:\Program Files\nodejs` : **25.8.2 / 11.8.0**.
  Écart explicite avec `engines >=22.23.2 <23`, avertissements EBADENGINE
  lors de npm ci ; pas de changement de manifeste ni installation de Node.
- WebView2 **156.0.4314.50** déjà présent, aucune installation runtime.
- Copie exclusive : `C:\Users\lmirguet\Development\vocalnotes-native-B02-W01`,
  55 fichiers allowlist vérifiés ; aucun secret, note, attachment, cache,
  node_modules/target/dist Linux copié.
- Dépendances projet restaurées nativement ; 44 tests Rust et 22 JS,
  fmt/check/build MSVC et frontend/typecheck **PASS**, codes 0.
  Même source Linux finale : 44/22 tests et fmt/check/build PASS.
- `probe_lifecycle` **code 0** sous Windows, abonnement/état WTS initial
  disponibles ; aucun événement lock/suspend provoqué ou certifié.
- Application native Windows avec assets embarqués lancée à 01:39:23 CET :
  PID **25360**, HWND **396550**, fenêtre « Bloc-notes vocal » répondant,
  WebView2 enfant **9376**, session outil détachée **windows-ui** conservée.
  Aucun serveur Vite ni nouvelle UI WSLg. Le sondage WaitForInputIdle initial
  a échoué pendant le démarrage ; vérifications HWND/réponse ultérieures PASS.
- Ancienne session WSLg arrêtée avant cette reprise (preuve du tour précédent,
  PID 126037/126008/125926 disparus), aucune instance à conserver ou relancer.
- Rendu visuel/gestes, micro WASAPI, Credential Manager et appels Mistral :
  **NON EXÉCUTÉS**. Aucun micro, aucune clé lue, aucun appel fournisseur.
  CPU/RAM/micro et mesures de recette restent à consigner par l'utilisateur.
- **T006 partielle/T007 ouverte** ; GNOME/Wayland **NON EXÉCUTÉ, différé**.
  Développement autorisé sous dérogation, autres dépendances obligatoires,
  aucune certification desktop/multiplateforme ou NSIS.

Les sections WSLg « laissée active » ci-dessous sont historiques : leur
instance est désormais fermée. Ne pas appliquer leurs commandes de relancement
à la recette Windows demandée.

Sondage ciblé du 2026-10-30, sans micro ni lecture de secrets.

| Élément | Observé |
|---|---|
| Hôte | Ubuntu 24.04.4 LTS x86_64, noyau 6.18.54.1-microsoft-standard-WSL2 |
| Affichage déclaré | DISPLAY=:0, WAYLAND_DISPLAY=wayland-0 ; pas de XDG_SESSION_TYPE/XDG_CURRENT_DESKTOP |
| GNOME | gnome-shell absent du PATH ; session Fedora/GNOME complète NON ÉTABLIE |
| Windows 11 | Aucun accès desktop/outillage Windows établi ; WSL2 ne prouve ni version ni recette Windows |
| Node/npm | v22.23.2 / 12.2.1 |
| Rust/Cargo/rustup | Absents du PATH et pas de cargo dans ~/.cargo/bin au sondage initial |
| Compilation C | cc/gcc présents |
| Dépendances natives | pkg-config absent ; versions GTK/WebKitGTK/ALSA/D-Bus non établies |
| Micro/matériel audio | NON TESTÉ ; aucune capture ni ouverture de périphérique |
| Trousseau | NON TESTÉ ; aucune clé saisie/lue et aucun fallback |
| RPM/NSIS | rpm/makensis absents du PATH ; distribution non testée |
| Matériel de recette (CPU/RAM/micro) | À relever sur chacun des deux desktops réels |

## Préparation des cibles, sans installation système implicite

### Résultat du socle B01

Toolchain utilisateur installée dans `/home/laurent/.local/share/vocalnotes-build/` :
Rust/Cargo 1.99.1, rustup 1.29.2, rustfmt. Aucun profil shell modifié.
`cargo generate-lockfile` et `cargo metadata --locked --no-deps` passent :
Tauri 2.12.5, tauri-build 2.7.4, serde 1.0.229 ; lockfile de 406 paquets.
CLI/API Tauri npm 2.12.4, TypeScript 5.9.3, Vite 7.3.7, Vitest 4.1.12.

`cargo check --locked` bloque sur libdbus-sys : pkg-config introuvable.
`cargo test --locked --lib` bloque sur GLib/GObject pour le même exécutable absent
(glib-sys initialement, gobject-sys lors du dernier essai après ajout des icônes) ;
aucun test Rust exécuté. `cargo fmt --check` passe.
dpkg confirme pkg-config en état config-files seulement et GLib/GTK dev non
installés. D-Bus dev, WebKitGTK dev, rsvg dev, ALSA dev et patchelf ne sont pas
rapportés installés par la requête ciblée. Le diagnostic CLI ne peut donc pas
certifier la présence de leurs bibliothèques via pkg-config.

Sur cet Ubuntu, faire approuver puis fournir au minimum `pkg-config`,
`libdbus-1-dev`, `libglib2.0-dev`, `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`,
`librsvg2-dev` et les prérequis Tauri de distribution ; ALSA dev sera également
nécessaire à la capture T006. Pas de commande système exécutée pour les installer.

Les métadonnées publiques npm, l'index Cargo et static.rust-lang.org sont
accessibles avec TLS vérifié. Les requêtes d'information à crates.io/api
échouent en TLS UnknownIssuer ; aucun contournement TLS/proxy n'a été ajouté.
Cela n'a pas empêché le verrouillage Cargo via son index public.

- **Fedora GNOME/Wayland** : hôte interactif, permissions micro, ALSA/PipeWire,
  WebKitGTK 4.1 et GTK3, pkg-config et outillage C/Rust ; Secret Service réel et
  services ScreenSaver/login1. Installer un RPM puis exécuter A–K du quickstart.
  Aucune dépendance Fedora n'est présumée fournie par cet Ubuntu WSL.
- **Windows 11** : hôte interactif, MSVC/SDK, Rust cible MSVC, Node 22, WebView2,
  WASAPI, Credential Manager, notifications WTS/alimentation et paquet NSIS.
  Recette sur le même état de code, pas simple cross-compilation.
- **macOS** : objectif V1 séparé, machine/micro/CoreAudio/Keychain et recette
  complète à prévoir ; aucune compatibilité annoncée.

Pas de paquets système installés dans cet environnement partagé. La toolchain
Rust éventuelle est utilisateur uniquement ; son résultat et les commandes
effectives figurent dans batch-report.yaml. Aucun succès desktop déduit d'un
test Vitest, d'un navigateur ou d'un build frontend.

## Gate

### B02 — interface Linux native sous WSLg, 2026-10-31

Demande utilisateur actuelle : rester sous WSLg, sans installation Windows.
Commande `npm run tauri -- dev --no-watch` avec toolchain locale du quickstart,
exécutée en async détaché (`wslg-ui`), pas nohup/disown ni double instance.
Vite prêt, HTTP 200 sur `127.0.0.1:1420`, binaire natif Linux
`target/debug/vocalnotes` PID `126037` vivant ; Vite PID `126008`,
superviseur Tauri Node `125926`. Weston associe `appId: vocalnotes`,
`appWindowId: 0x6` au pont RDP WSLg à 01:03:31 CET.
Fenêtre sur le bureau Windows via WSLg ; aucun contenu de fenêtre inspecté
à distance, aucune recette interactive UI déclarée réussie.
La sortie native comporte des avertissements EGL/Mesa : pilote non identifié
pour fd -1, ZINK sans pdev, échec de création dri2. Pas de crash constaté :
WebKitWebProcess `126247`, WebKitNetworkProcess `126155`, application et Vite
toujours vivants. Rendu effectif non certifié à distance ; confirmer la fenêtre
visuellement, sans désactiver la sandbox ni forcer un pilote en contournement.
Pas d'erreur fatale dans la sortie de lancement ; aucun contournement sandbox,
TLS, autorisation ou surveillance OS. Application et serveur laissés actifs
pour l'utilisateur. Le processus pré-lot B01-R3 n'est pas redémarré afin de
préserver sa saisie ; nouveau code B02 compilé/testé seulement.

Le sondage OS reste indisponible ; mode vocal DOIT rester refusé avant capture.
0 micro, 0 appel Mistral, clé racine non lue, aucun appel keyring réel.
Pas d'installation ni changement de dépendances. La constitution approuvée
reste inchangée (SHA-256 ci-dessous).
44 tests Rust / 22 JS, fmt/check/build PASS. T008–T009 réalisées sous la
dérogation de développement ; T010 partielle, T011–T013 non commencées.
**Windows natif : NON EXÉCUTÉ ; GNOME/Wayland : NON EXÉCUTÉ ; T007 ouverte.**
Commande reproductible et arrêt après sauvegarde :
[quickstart](../../specs/001-bloc-notes-vocal/quickstart.md#lancement-natif-wslg-réellement-exécuté--b02).

### Décision utilisateur et sondage Windows — 2026-10-31

L'utilisateur demande une recette Windows 11 en premier et reporte
GNOME/Wayland, sans arrêter les développements. La seule absence de recette
GNOME ne bloque donc plus les tâches de développement dépendant de T007.
T007 reste ouverte et aucun GO multiplateforme n'est accordé ; les autres
dépendances et corrections nécessaires restent applicables. Cette décision
prévaut sur les interdictions de poursuivre formulées dans les états historiques
ci-dessous. Les exigences Linux sont conservées.

Sondage natif via `pwsh.exe` depuis WSL, sans installation ni accès au micro :
Windows 11 Professionnel, version 10.0.26200, build 26200 ; `node.exe` et
`npm.cmd` disponibles. `cargo.exe`, `rustup.exe` et `cl.exe` absents du PATH
Windows ; `vswhere.exe` absent de son emplacement habituel. Une toolchain
MSVC complète n'est pas établie. Aucun build ni essai applicatif Windows
effectué par ce sondage. Préparer Rust natif MSVC et Visual Studio Build Tools
avec outils C++ et SDK Windows avant de compiler le projet sur Windows.

T007 reste NON EXÉCUTÉ tant que T006 n'est pas entièrement vérifiée et les deux
desktops indisponibles. L'accord Mistral synthétique/coût a été reçu, sans appel ;
un démarrage micro réel exige encore son accord contextualisé. Rapports avec versions,
matériel, conditions, résultats G02–G09, traces expurgées et coût inconnu si absent.
Ne pas franchir T008 sur la base de cet inventaire.

## Reprise T006 — B01-R3, 2026-10-31

L'utilisateur a installé `libasound2-dev`. Vérification sans installation :
`pkg-config --modversion alsa` = **1.2.11**. `cargo check --locked`
sur CPAL passe immédiatement ; blocage ALSA levé. Rust/Cargo 1.99.1 restent
locaux, sans PATH/profil global modifié.

Code final raccordé : contrôleur continu, files bornées, VAD/pré-roll,
rotation `flush/end`/`done` seulement, LLM ordonné indépendant de la capture
et priorité sémantique coalescée (750 ms, **pas SLA**).
Bornes : pré-roll 300 ms, file native 64 trames, contrôleur 128 trames,
quatre finalisés en attente, passage 60 s, provisoire 4096 octets.
Saturation/erreur invalide et signale le non-intégré ; aucun spool ni rejeu.
Enveloppe native note/session/génération/requête/opération/passage/séquence/révision
contrôlée au commit. Pause suspend l'intention ; arrêt/OS la purgent.
Save ton→édition→copie exclusive disponible ; une action après save est refusée
entièrement dans cette tranche. Le premier X arrête/invalide, fermeture en attente ;
la commande dédiée attend les acquittements UI, puis sauvegarde l'accepté
avant destruction. Sans destination/échec, note accessible et micro arrêté.

GNOME : ScreenSaver/login1, abonnements avant état initial, propriétaires D-Bus
et santé des services. Windows : récepteur invisible WTS/WM_POWERBROADCAST,
état initial et santé WTS. `windows_lifecycle_types.rs` vérifie la **forme des
API sur Linux seulement**, pas SDK/MSVC, linkage ni événements Windows.

Final : **22 tests JS, 32 tests Rust**, `npm run typecheck/build`,
`cargo check --locked --tests`, `cargo build --locked` et format PASS.
Sondage réel `cargo run --locked --manifest-path src-tauri/Cargo.toml
--example probe_lifecycle` : **code 2**, surveillance OS indisponible.
Pas de capture, réseau fournisseur ni lecture trousseau dans ce diagnostic.
WSLg annoncé ≠ GNOME/Wayland recetté ; aucune preuve Windows 11.

T006 partielle/décochée : vrais microphones/WSS/LLM/trousseaux, rotation sans
perte/qualité/pause rapide et lock/sleep/perte d'abonnement non vérifiés sur cibles.
T007 NON EXÉCUTÉE. Voir `prototype-gnome.md`, `prototype-windows.md`,
`prototype-metrics.md` : essais absents, **aucun GO**.
Pas de SDK supplémentaire déclaré nécessaire aux parties compilées ici.
Prochaine action : vrais desktops, recettes synthétiques informées, pas une
nouvelle installation système implicite sous WSL2.

0 appel Mistral, 0 micro démarré, 0 appel réel keyring ; clé racine non lue.
Constitution SHA-256 inchangé :
`50dbb3a5258e3d506aaea03540a57be415eb105b9136537734b598b6132882dc`.

## Reprise T005 — B01-R1, 2026-10-30

Nouvelle validation réelle, sans installation système ni changement de manifeste.
L'hôte reste Ubuntu 24.04.4 LTS x86_64 sous WSL2. `build-essential`, GCC et
make sont installés ; Rust/Cargo 1.99.1 fonctionnent avec les variables locales
`CARGO_HOME`, `RUSTUP_HOME` et PATH décrites dans le quickstart.

### Diagnostic différentiel

- `ldconfig -p` trouve les bibliothèques **d'exécution** D-Bus, GLib/GObject,
  GTK3, WebKitGTK 4.1 et ALSA. Elles ne constituent pas un SDK de développement.
- Les répertoires `/usr/include/dbus-1.0`, `/usr/include/glib-2.0`,
  `/usr/include/gtk-3.0`, `/usr/include/webkitgtk-4.1` et `/usr/include/alsa`
  sont absents.
- Aucun exécutable `pkg-config`/`pkgconf` trouvé dans `/usr/bin`, `/bin`,
  `/usr/local/bin` et `~/.local/bin`.
- Recherche bornée dans les répertoires pkgconfig de `/usr`, `/usr/local`,
  `/opt` et la toolchain utilisateur : aucun fichier `.pc` correspondant
  aux bibliothèques ci-dessus ni outil pkgconf trouvé dans le périmètre lisible.
  `/opt/containerd` n'est pas accessible ; aucune conclusion sur son contenu.
- Le cache `/var/cache/apt/archives` ne fournit aucune archive pkg-config/pkgconf
  ou dev D-Bus/GLib/GTK/WebKitGTK/ALSA recherchée.
  Aucun SDK local complet n'est donc établi. Ajouter seulement pkg-config,
  fabriquer des `.pc` ou des liens vers les `.so` runtime ne validerait pas le produit.
  Un sysroot complet provisionné séparément pourrait être une alternative ;
  aucun n'a été téléchargé/assemblé ni prétendu impossible.

`cargo check --locked --manifest-path src-tauri/Cargo.toml` échoue à nouveau
dans `libdbus-sys 0.2.7` (code 101, pkg-config absent).
`cargo test --locked --manifest-path src-tauri/Cargo.toml --lib` échoue
(code 101) dans `gdk-sys`, puis GIO/GObject/GLib et autres dépendances GTK :
pkg-config absent ; **0 test Rust exécuté**. Aucun build natif réussi.
Le format Rust et les métadonnées verrouillées passent.
`npm test` passe 12/12 cas, `npm run typecheck` et `npm run build` passent
(codes 0). L'intégration `runTests` ne découvre toujours pas ces tests ;
le résultat Vitest vient bien du script npm exécuté.

### Autorisation système à obtenir, pas accordée

Commande proposée pour T005, **NON EXÉCUTÉE** :

```bash
sudo apt-get --no-install-recommends install \
  pkg-config libdbus-1-dev libglib2.0-dev libgtk-3-dev \
  libwebkit2gtk-4.1-dev librsvg2-dev patchelf
```

Les six premiers paquets fournissent outil et SDK desktop ; `patchelf` prépare
le packaging Tauri et n'est pas la cause du premier échec Cargo.
Pas de dépendance tray/AppIndicator demandée. RPM/NSIS restent non construits ;
aucune recette de distribution n'est déduite de ce lot.

Simulation uniquement : même commande avec `apt-get --simulate` et sans sudo,
code 0, **96 nouveaux paquets, 12 mises à jour, 0 suppression** selon les index
APT présents. Les mises à jour touchent GLib et Mesa ; l'impact n'est donc pas
limité à sept noms de paquets sur cet hôte partagé. Aucun `apt update` effectué,
le plan pourra évoluer après actualisation des index.

`libasound2-dev` sera requis pour CPAL en T006, mais est séparé de la demande
T005. Simulation du lot ci-dessus augmenté d'ALSA : 97 nouveaux paquets,
12 mises à jour, 0 suppression. Aucune des deux installations n'est autorisée
par la seule demande de reprise.

Pas d'outil `ask_user` exposé après recherche ; demande à présenter en réponse.
T005 reste décochée à la fin de B01-R1, T006 non commencée, T007 NON EXÉCUTÉ
et T008+ non engagées. Micro, trousseau et API Mistral non utilisés ; clé non lue,
constitution inchangée.

## Reprise T005 puis T006 — B01-R2, 2026-10-30

L'utilisateur annonce avoir installé les dépendances système T005. Vérification
réelle : `dpkg-query` confirme installés pkg-config, libdbus-1-dev,
libglib2.0-dev, libgtk-3-dev, libwebkit2gtk-4.1-dev, librsvg2-dev et patchelf.
`pkg-config --modversion` renvoie D-Bus **1.14.10**, GLib **2.80.0**,
GTK **3.24.41**, WebKitGTK **2.52.6**. Pas d'installation système par l'agent.

Avec l'environnement Rust local ci-dessus et dans le quickstart :

| État validé | Résultat |
|---|---|
| T005 avant modification T006 | cargo check, test --lib **1/1**, build, fmt PASS ; npm test **12/12**, typecheck/build PASS |
| T006 avant ajout CPAL au graphe compilé | cargo check/build PASS ; **17 tests Rust PASS**, 0 échec/ignoré (1 socle + 9 cœur + 4 services + 3 audio) |
| Frontend T006 final | **19 tests JS PASS**, 0 échec/ignoré ; typecheck/build PASS |
| T006 après ajout CPAL | cargo check **101**, `alsa-sys 0.3.1`, `alsa.pc` absent |
| Format et verrouillage finaux | cargo fmt --check et metadata --locked --no-deps PASS |

Le nouvel `platform/capture.rs` n'a donc **pas été compilé par rustc**. Ne pas
attribuer les 17 tests précédents au code de capture final. Pas de nouvelle
tentative test/build après cet échec de prérequis connu. CPAL n'est ni retiré,
ni remplacé par un mock ou une feature sans audio.
Le binaire généré avant CPAL n'est pas le build de l'arbre final.

`libasound2-dev` n'est pas installé et `pkg-config --modversion alsa` échoue.
La demande T005 précédente ne l'incluait pas. **Autorisation séparée nécessaire**
pour une installation par l'agent, ou installation par l'utilisateur :

```bash
# Proposition seulement, NON EXÉCUTÉE :
sudo apt-get --no-install-recommends install libasound2-dev
```

Simulation uniquement, sans sudo ni apt update :
`apt-get --simulate --no-install-recommends install libasound2-dev`, code 0,
**1 nouveau paquet** `1.2.11-1ubuntu0.3`, **0 mise à jour, 0 suppression**
selon les index actuels. Pas d'outil `ask_user` exposé après recherche :
question dans la réponse utilisateur.

Du travail concret indépendant du SDK ALSA a été effectué avant ce blocage :
autorité native et tests U1/U2/U3 partiels, éditeur existant-blocks et blocs opaques,
menus/bandeau, saisie masquée/keyring réel, copie exclusive de note, fermeture
protégée ; transports TLS natifs Mistral et downmix/rubato/VAD testés sans réseau.
Le contrôleur continu capture→STT→LLM→commit et les abonnements OS restent à
assembler ; le bouton vocal est désactivé explicitement. Les limites d'édition
et de sauvegarde sont affichées, aucun succès de remplacement automatique annoncé.

T005 est terminée ; T006 reste partielle et décochée. Les desktops GNOME/Wayland
et Windows 11 ne sont toujours pas établis/recettés. T007 NON EXÉCUTÉE, T008+
non engagées. Aucun micro, accès trousseau réel ou appel API ; clé locale non lue.
Constitution SHA-256 inchangé :
`50dbb3a5258e3d506aaea03540a57be415eb105b9136537734b598b6132882dc`.

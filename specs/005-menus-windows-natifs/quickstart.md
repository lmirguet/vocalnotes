# Quickstart: Valider les menus et le cycle de document Windows et Linux

Ce guide décrit les recettes restant à réaliser et consigne séparément les vérifications déjà exécutées dans les bilans d'incrément.

## État implémenté — reprise T017–T047

- Les menus natifs de `main` sont la seule surface documentaire : Fichier, Edition et Aide. La barre HTML, le champ de destination libre et les anciennes commandes IPC de sauvegarde/fermeture sont retirés.
- Enregistrer est explicite. Une nouvelle note est enregistrée en `.von` ; un `.md` ouvert reste associé tant que son contexte est inchangé. Ajouter, modifier ou vider ce contexte impose une destination `.von`, sans toucher au Markdown initial.
- Exporter écrit exactement les octets UTF-8 du contenu accepté, y compris vide, sans enveloppe ni adoption du chemin. Une destination identique au document courant, notamment par lien physique, est refusée.
- Nouveau/Ouvrir/Quitter et la croix de `main` partagent préparation, drainage, décision Enregistrer/Ne pas enregistrer/Annuler et destruction autorisée hors verrou. Une saisie non acquittée reste récupérable en mémoire, y compris dans les champs ; elle bloque `ready`. Après copie éventuelle, seul le bouton explicite d'abandon permet de la supprimer.
- Annuler/Rétablir utilisent les historiques Rust des trois zones, globalement bornés à 256 entrées/16 Mio de deltas UTF-8. Sauvegarde/export les préservent ; nouvelle note/ouverture les effacent. Une indication signale l'éviction sans tronquer le texte.
- Préférences est une fenêtre locale distincte, choisie par son vrai label Tauri, à cadre système et propriétaire natif. OK attend la persistance ; Annuler/Echap/croix abandonnent le brouillon. Une écriture déjà acceptée bloque temporairement la fermeture ; toutes les commandes de note restent refusées sous modalité côté Rust. Les opérations de trousseau sont explicitement immédiates et réservées à cette fenêtre.
- A propos utilise nom/version natifs, sans troisième webview.

**Portée des preuves** : tests automatiques Windows et Linux Ubuntu/WSL, fichiers synthétiques sur les systèmes de fichiers locaux correspondants, services/dialogues simulés. Des interactions Windows réelles sont également consignées ci-dessous, sans certifier la recette native complète. Cela ne certifie pas GNOME/Wayland ni Fedora. T022 et T048–T052 restent des recettes séparées ; aucun échange SC-011, participant, microphone, fournisseur réel ou accès à une clé n'est revendiqué.

### Bilan automatisé de cette reprise

T017–T021 et T023–T046 sont cochées pour leur périmètre code/tests automatisés. **T022 reste ouverte** : recette sur les deux bureaux natifs non réalisée, hôte Fedora absent. Le blocage Vitest Linux de T047 a été résolu par une exécution sur le système de fichiers Linux natif ; les essais historiques incomplets sont conservés dans le tableau. T048 est partiellement exécutée sous Windows ; T049–T051 restent non exécutées et T052 n'est qu'un bilan provisoire.

| Vérification réellement exécutée | Résultat |
|----------------------------------|----------|
| Windows MSVC : `cargo test --locked --manifest-path src-tauri/Cargo.toml --quiet` | **263 réussis**, aucun échec/ignoré, 27 bilans de suites, dont 61 tests de bibliothèque. Exécution complète finale après les gardes de modalité/fermeture. |
| Ubuntu WSL : même suite Rust, `CARGO_TARGET_DIR="$PWD/target/linux-storage"` | **265 réussis**, aucun échec/ignoré, 28 bilans de suites, dont 57 tests de bibliothèque. Branches GTK/Linux réellement compilées/exécutées ; fichiers temporaires sous le système de fichiers Linux natif, pas `/mnt/c`. |
| Windows et Linux : `cargo build --locked --manifest-path src-tauri/Cargo.toml --quiet` | **Réussite sur les deux OS**, liaison des vrais binaires sans les lancer. `cargo check --locked` Windows avait également réussi. |
| Windows Node 22.23.2 : `npm test` puis `npm run build` | **96/96 tests**, 13 fichiers ; TypeScript et Vite réussis après la régression Echap des préférences. |
| Linux Node 22.23.2 : tests et build sur une copie de travail frontend ext4 temporaire | **96/96 tests**, 13 fichiers avec `--maxWorkers=2` ; TypeScript et Vite réussis après la même correction. |
| Linux Vitest, pool par défaut | **84 assertions réussies**, mais deux erreurs non gérées de démarrage de workers ; exécution non verte. |
| Linux Vitest, `--maxWorkers=2` | **75 assertions réussies**, mais six erreurs de démarrage de workers ; exécution non verte. Le fichier DesktopShell actuel passe ses 20 tests. |
| Linux Vitest, `--pool=threads --maxWorkers=2` | Progression partielle, puis interruption du helper après absence prolongée d'achèvement ; pas de succès complet revendiqué. |
| Linux Vitest, `--pool=forks --maxWorkers=1 --no-isolate` | **13 assertions réussies**, une erreur de démarrage pour le groupe des fichiers UI ; exécution non verte. Les délais et assertions des tests n'ont pas été dilués. |
| Dépendances et propreté | `rfd 0.17.2`/plugin dialogue `2.8.4` avec GTK3, sans `xdg-portal` ; une seule version `gtk`/`gtk-sys`/`gdk`/`gdk-sys` 0.18.2, WebKitGTK 2.0.2. `git -c core.safecrlf=false diff --check` réussi. |

Les anciennes attentes de sauvegarde vocale ont été remplacées par une vérification plus stricte : l'intervention contenant Save est refusée atomiquement, les modifications vocales acceptées seules ne touchent pas le disque, puis la sauvegarde native explicite écrit l'état accepté. Un writer de fixture est fermé avant réouverture Windows, sans assouplissement du stockage.

**Comptages exécutés, pas déduits du nombre de tests** :

- `document_workflow` boucle réellement sur **20 cycles par OS** et effectue **40 exports par OS** : 20 avant toute association de destination, puis 20 après sauvegarde, avec contenu vide, Unicode/CRLF, préfixe d'enveloppe littéral, texte du corps égal au contexte et contenu non enregistré. Octets, dirty, chemin, format, version disque et historiques sont comparés. Les autres cas couvrent migration `.md`, refus/échec/annulation, alias courant, remplacement confirmé, candidat écrit exact et fermeture hors verrou.
- `document_history` exécute **20 séquences par OS**, chacune avec deux sauvegardes réelles, dictée, reformulation annulée en une entrée, abandon du redo, retour au savedState, puis 300 mutations de contexte/256 undo/256 redo pour l'éviction. Le test unitaire distinct vérifie aussi le budget global de 16 Mio et l'intégrité UTF-8 d'un grand delta.
- `document_workflow` exécute **20 cycles logiciels de modalité par OS** : unicité, rejet des commandes de note, jetons de nettoyage périmés, réservation d'écriture et refus de fermeture pendant persistance. Ce ne sont pas 20 ouvertures de fenêtres natives.
- `ModelPreferencesView.test.ts` exécute **dix parcours de brouillon** dans JSDOM ; `Startup.test.ts` vérifie le vrai label, l'absence de privilège par query string et l'échec de montage expurgé. Aucune opération réelle de trousseau n'est effectuée.

Commandes de reprise Linux utilisées depuis ce worktree :

```powershell
wsl.exe --cd "$($PWD.Path)" --exec sh -lc 'export PATH="$HOME/.cargo/bin:$PATH"; export CARGO_TARGET_DIR="$PWD/target/linux-storage"; cargo test --locked --manifest-path src-tauri/Cargo.toml --quiet && cargo build --locked --manifest-path src-tauri/Cargo.toml --quiet'
```

Le premier essai frontend Linux a signalé le module optionnel Rollup Linux manquant dans les dépendances restaurées sous Windows. Seuls les deux binaires optionnels aux versions déjà résolues ont été installés, dans `target/` ignoré, sans changer les manifests/verrous ni supprimer les dépendances Windows :

```bash
npm install --prefix target/linux-node-tools --no-package-lock --no-save --ignore-scripts @rollup/rollup-linux-x64-gnu@4.66.0 @esbuild/linux-x64@0.28.2
NODE_PATH="$PWD/target/linux-node-tools/node_modules" npm test
NODE_PATH="$PWD/target/linux-node-tools/node_modules" npm run build
```

**Blocage Vitest résolu** : les délais `[vitest-pool-runner]: Timeout waiting for worker to respond` concernaient le checkout monté depuis Windows. Une copie temporaire ext4 des seuls sources, tests, configurations et dépendances frontend, avec les deux paquets optionnels Linux déjà résolus, a exécuté intégralement `npm test -- --maxWorkers=2` puis `npm run build` : d'abord 95/95, puis **96/96** après ajout de la régression Préférences. Aucun délai/assertion n'a été assoupli. Cela reste une validation Ubuntu/WSL, pas une recette Fedora.

### Recette Windows réelle du 2026-11-14 — bilan provisoire T048/T052

Exécution du vrai binaire Tauri/WebView2 avec `TAURI_CONFIG={"identifier":"fr.vocalnotes.acceptance.20261114"}` à la compilation, un profil distinct et des notes synthétiques sous `target/native-acceptance`, sans microphone, fournisseur ni trousseau.

- F10/flèches/Entrée et Alt+E ouvrent les menus natifs. Ctrl+O ouvre le sélecteur système ; Ctrl+S écrit le contenu modifié et retire l'astérisque. Ctrl+Z puis Ctrl+Y modifient puis rétablissent le savedState.
- Le premier essai a montré que WebView2 absorbait les accélérateurs et que le focus clavier de Tao ne représentait pas le HWND propriétaire actif. Correction Windows : contrôle du HWND réellement au premier plan et routage natif `AcceleratorKeyPressed` vers la même autorité Rust, sans raccourci documentaire JavaScript concurrent, sans réentrance COM ni AltGr capturé.
- **10/10 parcours natifs Préférences réussis** : Echap, Annuler, Alt+F4, OK et erreur de validation suivie d'Echap, chacun deux fois. Vérification du propriétaire réellement désactivé pendant la modalité puis réactivé, du brouillon persisté/abandonné et des octets de note inchangés.
- **12/12 décisions natives réussies** sur Nouveau, Ouvrir, Quitter et croix, chacune avec Annuler, Ne pas enregistrer et Enregistrer. Le dialogue de sélection Windows a réellement choisi le fichier `.von` synthétique ; après chaque cas, le titre/état de note et les octets disque attendus ont été vérifiés. Alt+F4 et le menu Quitter terminent le processus isolé.
- L'essai après erreur a révélé un focus revenu sur `body` : Echap n'atteignait plus le formulaire. Une régression a échoué avant correction ; la fenêtre traite maintenant Echap au niveau document, sans double fermeture, et nettoie son écouteur à la destruction.
- La réouverture du fichier courant après « Enregistrer » avait été préparée sur les anciens octets. Une régression Rust reproduit le contexte perdu ; le candidat est désormais relu et validé après sauvegarde, avant publication.
- Validation Windows révisée : **259 tests Rust réussis** en sélectionnant `--lib` et toutes les intégrations sauf `prototype_audio`; frontend **96/96** et build frontend réussis. Le build Tauri Windows verrouillé réussit avec l'identifiant d'acceptation isolé. T047 reste ouverte : le nouveau binaire `prototype_audio` est absent après une compilation, et Windows refuse son lancement (`Accès refusé`, jamais exécuté), y compris lors de l'essai ciblé ; le binaire historique distinct réussit 6/6 mais ne remplace pas le test compilé actuel.

La recette Windows native reste partielle : les dix effets de menu, A propos/Export, les parcours complets de remplacement/conflit et la récupération native ne sont pas tous vérifiés. T048 demeure donc non cochée malgré les 10 parcours Préférences et 12 décisions ci-dessus. Fedora/GNOME/Wayland (T049), échanges entre les deux bureaux (T050), mesures et participants réels (T051) ne sont pas réalisés dans cette portée Windows. **Aucune validation globale ni clôture T052 n'est revendiquée.**

Pour la recette Windows : utiliser le binaire reconstruit, un dossier temporaire NTFS synthétique et aucun service vocal ; vérifier Alt/F10, les sept accélérateurs, les dix effets de menu, la croix/Alt+F4, les décisions et annulations, le parentage des sélecteurs, puis OK/Annuler/Echap/croix des préférences avec retour de focus dans les trois zones. Les échecs de création/modification de modalité doivent être contrôlés sur le bureau réel. Répéter ensuite sous Fedora/GNOME/Wayland ; les tests WSL n'en tiennent pas lieu.

## Prérequis

- Deux hôtes natifs : Windows 11 / WebView2 / outils C++ Tauri et Fedora / GNOME / Wayland / GTK3 / WebKitGTK. Sur chacun : Node.js `>=22.23.2 <23`, npm et Rust `1.99.1`.
- Sous Fedora : SDK et outils existants du projet pour GTK3 3.24, WebKitGTK 4.1, GLib, ALSA, D-Bus et compilation C/C++, avec une session Secret Service utilisable pour les parcours de trousseau. Le formulaire de modèles et la recette manuelle de menus ne requièrent pas une clé fournisseur. Ne pas installer de nouveau toolkit ni modifier une session de bureau pour contourner un échec.
- Dépendances du dépôt restaurées ; après modification des manifests pendant l'implémentation, mettre à jour/verrouiller les dépendances avant les vérifications.
- Un dossier temporaire local ordinaire par hôte : NTFS sous Windows, Btrfs ou ext4 sous Fedora. Ne pas utiliser de notes réelles comme données de test, ni un volume réseau ou un dossier synchronisé.
- Les tests automatiques remplacent dialogues et services vocaux par des doubles ; aucune clé, activation de microphone ou requête fournisseur n'est requise.

Références : [modèle de données](data-model.md), [commandes desktop](contracts/desktop-commands.md), [format de note](contracts/note-format.md).

### Relevé initial du 2026-11-13 (T002)

| Prérequis | Observation dans cette session | Suite nécessaire |
|-----------|--------------------------------|------------------|
| Windows | Windows 11 Professionnel, build `10.0.26200`, x64 ; volume `C:` NTFS sain. | Recette sur le binaire de cette fonctionnalité encore à réaliser. |
| WebView2 | Microsoft Edge WebView2 Runtime `157.0.4341.46`. | Vérifier les interactions natives après compilation. |
| Outils C++ | Installation Visual Studio avec outils C++ détectée, version `18.11.12328.437`. | Vérifier compilation et liaison réelles ; la présence seule ne les valide pas. |
| Rust | `rustc 1.99.1`, `cargo 1.99.1`, cible Windows MSVC. | Utiliser le toolchain épinglé par le dépôt. |
| Node/npm | Node `25.8.2`, npm `11.8.0` ; aucun gestionnaire nvm/fnm/Volta détecté. | Node ne satisfait pas `>=22.23.2 <23` ; fournir un runtime 22.x compatible avant validation frontend. |
| Linux | Ubuntu sous WSL est installé ; aucun hôte Fedora/GNOME/Wayland de recette n'est vérifié. | Provisionner ou rendre accessible le bureau natif de référence ; WSL ne le remplace pas. |

Ce relevé n'est une validation ni de la fonctionnalité, ni des recettes natives. Aucun microphone ni fournisseur n'a été sollicité.

### Premier incrément du 2026-11-13 (T001–T006)

T001 à T006 sont terminées. Le coordinateur, les snapshots, le bridge injecté et les interfaces natives sont présents. Le plugin de dialogue est initialisé ; les dialogues concrets relèvent encore de T016. Le bridge n'est pas encore monté dans le shell et la commande IPC `document_action` reste à enregistrer en T007. La barre HTML, l'autosauvegarde, les actions de fichiers et les préférences inline existantes ne sont pas encore remplacées : aucune story complète n'est livrée.

| Vérification exécutée sous Windows | Résultat et portée |
|-----------------------------------|--------------------|
| Rust `1.99.1`, `cargo check --locked` | Réussite avec le manifeste et le verrou modifiés. |
| `cargo test --locked --lib --test ipc_boundary` | 42 tests de bibliothèque et 7 tests IPC réussis ; contrôleur avec périphériques simulés. Aucune recette graphique. |
| Arbre Cargo ciblant Linux | `tauri-plugin-dialog 2.8.4`, `rfd 0.17.2` avec `gtk3`, sans `xdg-portal`, GTK partagé `0.18.2`. Résolution de dépendances seulement, pas compilation ou exécution Linux. |
| Node `22.23.2` fourni par `npm exec` | Runtime requis utilisable sans remplacement de l'installation globale Node 25 ; `npm ci` exécuté après échec initial faute de TypeScript installé. |
| Tests frontend ciblés | 65 tests réussis dans DocumentCommands, DesktopShell, Prototype.editor, Prototype, RenderEvidence et markdown. |
| `npm run build` avec Node 22 | TypeScript et Vite réussissent. |

Pour reproduire les commandes npm avec le runtime utilisé dans cette session PowerShell :

```powershell
$npmCli = Join-Path (Split-Path -Parent (Get-Command npm).Source) 'node_modules\npm\bin\npm-cli.js'
npm exec --yes --package=node@22.23.2 -- node "$npmCli" test -- DocumentCommands.test.ts DesktopShell.test.ts Prototype.editor.test.ts Prototype.test.ts RenderEvidence.test.ts markdown.test.ts
npm exec --yes --package=node@22.23.2 -- node "$npmCli" run build
```

Ces résultats décrivent seulement le premier incrément. Les recettes Windows/Linux, les échanges SC-011, les mesures et les essais utilisateurs ne sont pas réalisés. Aucun hôte Fedora/GNOME/Wayland de recette n'est vérifié et aucun participant n'est fourni ; cela ne valide ni T049 ni T051. T052 reste ouverte malgré ce bilan provisoire.

### Reprise du 2026-11-13 (T007–T011)

T001 à T011 sont terminées. `document_action` est enregistré avec arguments stricts, identifiants d'opération textuels, contrôle du label natif et seules permissions événementielles listen/unlisten. Le build utilise la liste commune d'IPC pour générer la permission applicative. Le validateur partagé accepte `.von`/`.md` sans modifier la casse des chemins et refuse chemins relatifs, réseau, traversal, NUL, dépassements et flux alternatifs Windows ; les suffixes ne sont ajoutés que s'ils manquent.

Le domaine conserve une référence sauvegardée exacte. L'édition manuelle, les changements d'intention/contexte, la dictée et la reformulation ne déclenchent plus d'écriture. Revenir aux valeurs enregistrées retire `dirty` ; les instructions temporaires n'y participent pas. Le codec vérifie la taille du fichier après échappement sans normaliser les octets du corps. Le contrôleur suit désormais les terminaisons capture/reformulation déjà engagées dès la demande : un échec pendant la préparation UI n'est pas perdu, une annulation invalide les résultats tardifs et laisse le micro arrêté. Annuler une sauvegarde non destructive ne stoppe pas la capture.

**Limite importante de cet état intermédiaire Windows** : l'ancien stockage ne sait toujours pas remplacer un fichier de manière contrôlée (T012–T015 à faire). Sa préparation de secours n'est désormais plus adoptée comme un enregistrement réussi : la sauvegarde au même chemin est explicitement refusée, le document reste modifié et la destination ne change pas. Une nouvelle copie choisie explicitement reste possible par l'ancienne commande. Les assertions Windows de sauvegarde ordonnée et fermeture vérifient ce refus et la conservation des données ; elles ne certifient pas un remplacement natif réussi.

La nouvelle IPC acquitte pour l'instant la préparation ; son moteur de fichiers et ses décisions ne sont pas encore raccordés (T017–T020). L'historique est décodé mais refuse explicitement les demandes avec `history_unavailable` jusqu'à T032–T034. Le drainage est disponible côté contrôleur mais l'ancienne croix utilise encore le parcours historique. Le bridge n'est pas encore monté ; menus HTML et préférences inline restent en place. Aucune story complète n'est livrée.

| Vérification de la reprise sous Windows | Résultat et portée |
|-----------------------------------------|--------------------|
| Suite Rust complète verrouillée | 224 tests réussis, dont 48 de bibliothèque. Les cas sous `cfg` Linux ne s'exécutent pas sur cet hôte et ne sont pas validés. |
| Frontend ciblé et build Node 22 | 34 tests réussis (`DocumentCommands`, `DesktopShell`, `Prototype.editor`), puis typecheck et build réussis. Une régression reproduit la réponse `ready` tardive qui réactivait une opération annulée ; le bridge l'ignore désormais. Aucune recette graphique native. |

### Contrat de stockage commun (T012)

T012 est terminée : candidat immuable conservant état, octets sérialisés, format, destination complétée et versions ; consentement consommable lié au chemin et à la version observée ; préparation exclusive synchronisée dans le même dossier avec le suffixe du format. L'export prépare les octets exacts du corps, même s'il commence par le namespace des métadonnées. Une préparation abandonnée reste récupérable et ne modifie pas la destination.

`WriteResult` distingue `Saved`, `Conflict` et `Failed`, transporte les versions et chemins de récupération, et ne possède aucune variante de copie adoptable. `Saved` porte aussi le candidat réellement écrit pour la future référence sauvegardée. L'observation Windows de `DiskVersion` refuse explicitement `unsupported_storage` tant que T014 n'apporte pas de véritable identité native ; elle ne réutilise pas l'identité factice de l'ancien stockage. Les primitives finales d'installation/remplacement et leur branchement restent à faire (T013–T015).

Vérification après T012 : **75 tests Rust ciblés réussis**, dont 50 de bibliothèque, 14 de stockage, 9 IPC et 2 de sauvegarde ordonnée. Les cinq nouveaux cas couvrent consentement périmé/autre chemin, apparition d'une destination, préparation exclusive récupérable, refus avant écriture et export exact. La suite complète de 224 tests ci-dessus précède T012 ; elle n'est pas présentée comme une nouvelle exécution complète.

L'inspection T016 a aussi identifié que les constructeurs GTK de `rfd 0.17.2` passent un parent nul. `research.md`, le plan et T016 sont corrigés : l'implémentation Linux devra utiliser les dialogues GTK3 directs parentés à `main`, pas considérer le `set_parent` du plugin comme une garantie. Aucun dialogue concret n'a encore été implémenté.

### Stockage natif et intégration (T013–T015)

T001 à T015 sont terminées. Linux utilise les installations exclusives `RENAME_NOREPLACE` et les échanges contrôlés existants. Windows utilise les identités natives de volume/fichier à 128 bits, vérifie les points de réanalyse, liens, attributs et permissions, puis emploie `ReplaceFileW` avec une sauvegarde déplacée distincte ou une installation sans écrasement. Une DACL protégée ou non reproductible est refusée avant remplacement ; aucune ACL n'est normalisée pour contourner ce refus.

Le stockage commun ne conserve plus d'identité Windows factice. `PreparedWrite::install` retourne le candidat effectivement écrit et sa version, un conflit, ou un échec avec récupération. La préparation est liée à son identité, à ses métadonnées et aux octets du candidat ; une modification avant ou pendant l'installation ne peut être adoptée comme une réussite. Les contrôles après remplacement utilisent le vrai chemin déplacé de chaque OS et conservent les versions utiles sans rollback.

| Vérification exécutée pour cet incrément | Résultat et portée |
|-----------------------------------------|--------------------|
| Windows 11 / NTFS, sélecteurs ciblés | **110 tests réussis** : 55 de bibliothèque (dont 5 primitifs Win32), 25 de stockage, 11 de pipeline, 9 IPC, 6 de préférences, 2 de sauvegarde ordonnée et 2 de fermeture. Les cas communs exercent réellement créations concurrentes, courses avant/après remplacement, liens et interruption de processus sur Windows. |
| Ubuntu WSL / stockage temporaire Linux local, mêmes sélecteurs | **109 tests réussis** : 50 de bibliothèque, 28 de stockage, 11 de pipeline, 9 IPC, 7 de préférences, 2 de sauvegarde ordonnée et 2 de fermeture. Les différences de nombre correspondent aux branches propres aux OS, pas à des cas certifiés sans exécution. |

La passe Linux a révélé une attente fixe de 30 ms dans le test de drainage : elle est remplacée par le signal de terminaison de capture, sans changement du comportement applicatif. La fixture de préférences utilise désormais le dossier temporaire local ; le checkout Windows monté dans WSL est refusé comme destination de note, sans affaiblissement du contrôle de volume.

Pour reproduire les sélecteurs sous PowerShell :

```powershell
cargo test --locked --manifest-path src-tauri\Cargo.toml --lib --test storage_integrity --test prototype_ordered_save --test prototype_pipeline --test openai_preferences --test close_workflow --test ipc_boundary --quiet
wsl.exe --cd "$($PWD.Path)" --exec sh -lc 'export PATH="$HOME/.cargo/bin:$PATH"; export CARGO_TARGET_DIR="$PWD/target/linux-storage"; cargo test --locked --manifest-path src-tauri/Cargo.toml --lib --test storage_integrity --test prototype_ordered_save --test prototype_pipeline --test openai_preferences --test close_workflow --test ipc_boundary --quiet'
```

Il s'agit de tests ciblés, pas d'une nouvelle suite complète ni d'une recette graphique. WSL valide ici les appels système et parcours simulés Linux ; il ne valide pas Fedora/GNOME/Wayland, les dialogues, la modalité, les menus ni les interactions utilisateur. Aucun microphone ni fournisseur réel n'a été sollicité.

### Dialogues natifs asynchrones (T016, 2026-11-14)

`NativeDialogs` implémente les sélecteurs `.von`/`.md` et les confirmations à trois issues. Sous Windows, le plugin reçoit `main` après contrôle de son handle. Sous Linux, les dialogues GTK3 sont créés et détruits sur le thread principal avec un vrai parent transitoire, modalité vérifiée et destruction avec le propriétaire. Les réponses et destructions alimentent des canaux asynchrones ; l'abandon de l'attente ferme le dialogue GTK sans boucle de dialogue imbriquée.

Les réponses typées et `Custom` sont reconnues explicitement ; une réponse inconnue, un callback perdu ou un chemin non local produit une erreur contrôlée. Les propositions de noms suivent le format de l'action, mais une extension sélectionnée incompatible n'est jamais renommée silencieusement. La confirmation de remplacement rend un consentement consommable lié au chemin et à la version observée, pas un booléen désactivant les vérifications.

| Vérification exécutée pour T016 | Résultat et portée |
|--------------------------------|--------------------|
| Windows | 58 tests de bibliothèque et 9 IPC réussis, dont 6 cas de l'adaptateur desktop. Aucun dialogue graphique n'a été ouvert par ces tests. |
| Linux WSL | `cargo check --locked --lib` réussi et 7 tests desktop réussis, incluant la compilation des objets et signaux GTK3 ainsi que le mapping des réponses GTK. Aucun bureau GNOME/Wayland de référence n'est validé. |

Les cas couvrent annulation distincte d'un callback perdu, réponses invalides, URL locales/non locales, casse des suffixes, suffixes absents et refus d'extension incompatible. La confirmation éventuellement intégrée au sélecteur Windows ne remplace pas l'accord versionné ; l'interaction réelle reste à vérifier lors de son raccordement et de la recette native.

**État historique de cet incrément : 16/52 tâches terminées.** T017–T020 (moteur documentaire et branchement UI/croix) restaient à implémenter. L'adaptateur était disponible mais les nouveaux dialogues n'étaient pas encore appelés par ce moteur. Le bridge n'était pas encore monté ; les menus HTML et préférences inline étaient encore présents. Ce bilan ne décrit pas l'état intégré actuel, consigné en tête du document.

Avant la recette Linux, relever dans le terminal de la session native :

```bash
printf '%s\n' "$XDG_CURRENT_DESKTOP" "$XDG_SESSION_TYPE"
gnome-shell --version
pkg-config --modversion gtk+-3.0 webkit2gtk-4.1 alsa dbus-1
findmnt -T "$HOME"
```

Les résultats doivent identifier GNOME, Wayland et les SDK attendus ; contrôler aussi le volume du dossier de notes effectivement choisi. Conserver les versions observées avec la version de l'application. Un SDK manquant est un prérequis non satisfait, pas une preuve d'incompatibilité de la fonctionnalité.

## Commandes ciblées

Exécuter les mêmes sélecteurs sur chacun des deux hôtes, depuis la racine de son checkout de recette et après implémentation des fichiers de tests prévus. Sous Windows PowerShell :

```powershell
npm test -- DocumentCommands.test.ts DesktopShell.test.ts Prototype.editor.test.ts ModelPreferencesView.test.ts Startup.test.ts
npm run typecheck
cargo test --locked --manifest-path src-tauri\Cargo.toml --test document_workflow --test document_dialog_lifetime --test document_history --test close_workflow --test storage_integrity --test prototype_envelope --test prototype_ordered_save --test drafting --test ipc_boundary
cargo test --locked --manifest-path src-tauri\Cargo.toml platform::controller::tests
```

Sous Linux, dans le terminal de la session Fedora :

```bash
npm test -- DocumentCommands.test.ts DesktopShell.test.ts Prototype.editor.test.ts ModelPreferencesView.test.ts Startup.test.ts
npm run typecheck
cd src-tauri
cargo test --locked --test document_workflow --test document_dialog_lifetime --test document_history --test close_workflow --test storage_integrity --test prototype_envelope --test prototype_ordered_save --test drafting --test ipc_boundary
cargo test --locked platform::controller::tests
cargo tree --locked -e features -i tauri-plugin-dialog
cargo tree --locked -d
cd ..
```

Vérifier que le plugin de dialogue utilise GTK3, sans `xdg-portal`, et qu'une seule version compatible de GTK 0.18 est résolue. L'absence de doublon ne prouve pas la modalité ; la présence des features ne prouve pas le parentage Wayland.

Ces exécutables de tests sont maintenant présents. `document_dialog_lifetime` vérifie séparément le maintien de la garde mono-opération jusqu'au callback Windows et l'abandon possible d'un dialogue fermable au drop ; ses doubles ne constituent pas une recette graphique.

Les tests de stockage Windows doivent s'exécuter réellement sur Windows et les branches Linux sur Linux. Consigner les tests ignorés par `cfg` ; un cas obligatoire sauté n'est pas validé. Les tests de contrôleur utilisent les périphériques simulés existants.

Quand les commandes ciblées passent :

```powershell
npm run build
cargo check --locked --manifest-path src-tauri\Cargo.toml
```

Sous Linux :

```bash
npm run build
cd src-tauri
cargo check --locked
cd ..
```

Si les résultats ciblés révèlent un impact plus large sur les snapshots, l'IPC ou l'édition, élargir avec `npm test` puis `cargo test --locked` depuis `src-tauri` sur l'hôte concerné. Ne pas transformer une recette native non réalisée en succès de tests simulés.

## Jeux de fichiers

Utiliser des fixtures `.von` et `.md` comprenant : contenu/contexte distincts, accents et emoji, Unicode non normalisé, LF/CRLF et espaces finaux, bloc opaque, contexte vide, anciens champs d'intention, corps commençant par le namespace, contexte contenant `-->`, version d'enveloppe inconnue et fichier invalide. Inclure les limites de taille de l'intention et du fichier complet après encodage, ainsi que `.VON` et `.MD`. Le transport des fixtures ne doit pas passer par une conversion de fins de ligne Git ou un réenregistrement externe.

Les fichiers générés par les tests doivent être nommés exclusivement et nettoyés uniquement dans leurs dossiers temporaires connus. Les versions utiles à une récupération simulée restent vérifiables jusqu'à la fin du scénario.

## Parcours automatisés attendus

Les nombres de cycles ci-dessous s'appliquent séparément à chaque OS. Les doubles couvrent la logique ; les tests de fichiers réels et recettes natives complètent cette preuve.

1. **Sauvegarde explicite** : ouvrir/enregistrer une note, changer contenu et contexte, vérifier qu'aucune écriture n'a lieu à la frappe ou à la réception d'une dictée. Enregistrer écrit au même chemin et conserve l'historique. Une modification arrivée après capture d'écriture reste modifiée après acquittement.
2. **Protection des quatre actions** : Nouvelle note, Ouvrir, Quitter et croix × Enregistrer, Ne pas enregistrer, Annuler. Répéter avec note propre, neuve modifiée, contexte seul modifié, annulation du choix de fichier, refus de remplacement et échec d'écriture. Aucune fermeture/remplacement après annulation ou échec.
3. **Ouverture non destructive** : annuler le sélecteur, charger un fichier invalide ou inaccessible après Ne pas enregistrer. L'ancienne note reste en mémoire. Un candidat valide devient courant seulement après résolution de la protection. Les résultats de l'ancienne note ne l'atteignent jamais.
4. **Compatibilité .md** : modifier seulement le contenu d'un `.md` sans contexte ; Enregistrer garde exactement ce chemin sans conversion. Ajouter/modifier/vider un contexte par rapport à celui chargé ; Enregistrer propose un `.von` complet et garde le `.md` initial intact. Annuler un ajout de contexte jusqu'à l'état initial permet à nouveau de conserver le `.md`.
5. **Allers-retours .von** : 20 cycles couvrant les fixtures ; contenu et contexte exactement restaurés, fichier consultable hors application dans un éditeur texte et rendu Markdown sans contexte visible.
6. **Export** : 20 exports de notes enregistrées ou non, contenu non sauvegardé inclus, résultat identique à `source`, contexte/instructions exclus en tant que champs. Destination, état modifié et historique inchangés. Tester fichier vide, remplacement confirmé/refusé et alias du fichier de travail.
7. **Stockage Windows** : écriture au même chemin sans adoption d'une copie, destination créée pendant préparation, cible modifiée avant/après confirmation, course avant remplacement, fichier verrouillé, droits insuffisants et échec partiel simulé. Contrôler copies récupérables, version externe préservée et absence de rollback destructif. Utiliser des fichiers natifs pour identité, ACL, liens et points de réanalyse ; ne pas se limiter à des doubles Win32.
8. **Stockage Linux** : effectuer les écritures au même chemin par échange et les créations par installation sans écrasement ; injecter les courses avant/après échange, les changements d'inode de même contenu, les interruptions et échecs de synchronisation. Contrôler permissions, ACL particulières, propriétaire, liens physiques/symboliques, répertoire déplacé, primitive et volume non pris en charge. Ces refus doivent conserver le document modifié et expliquer Enregistrer sous, sans adopter une copie. Vérifier deux noms distincts par la casse et les suffixes majuscules. Le watcher reste indicatif : aucune écriture à la frappe et relecture avant chaque sauvegarde.
9. **Attente de traitement** : utiliser des signaux contrôlés pour simuler capture et reformulation en cours. La capture s'arrête avant la décision, les résultats attendus sont traités avant la confirmation, Annuler l'attente/échec laisse la note ouverte et le micro arrêté. Une seconde demande ne crée pas un second dialogue ; un ancien acquittement n'agit pas sur l'opération suivante.
10. **Historique** : 20 séquences sur les zones et origines prévues, dont dictée et reformulation ; cette dernière s'annule en une action. Les autres zones restent intactes, retour à la sauvegarde retire `dirty`, sauvegarde/export conservent les piles, nouvelle note et sortie d'Instructions effacent les historiques appropriés. Vérifier la cohérence lors d'éviction d'anciennes entrées.
11. **Préférences** : 10 parcours avec OK, Annuler, Echap, croix, validation invalide et échec de persistance ; brouillon conservé sur erreur et abandonné sur annulation. Aucune requête fournisseur. Une demande répétée réutilise la fenêtre ; les chemins d'erreur/destruction libèrent la modalité. Avec doubles de dialogues, vérifier les réponses `Custom`, Cancel et inconnues sans jamais confondre annulation et autorisation d'écrire.
12. **Frontière IPC** : une fenêtre Préférences ne peut pas changer/sauver la note ; `main` ne peut pas se faire passer pour le dialogue dans la fermeture de celui-ci. Rejeter labels, opérations, préconditions ou champs inconnus ; aucune ancienne commande ne contourne les confirmations. Les garanties restent identiques avec l'adaptateur natif de chaque OS.

## Recette native commune, obligatoire sur chaque bureau

Sous Windows, lancer l'application réelle :

```powershell
npm run tauri -- dev
```

Sous Fedora, depuis le terminal GNOME de la session Wayland :

```bash
GDK_BACKEND=wayland npm run tauri -- dev
```

La fenêtre doit être réellement rendue par GTK/WebKitGTK en Wayland ; ne pas remplacer un échec par X11/XWayland, WSLg ou le seul serveur Vite dans un navigateur. Les anciennes validations du projet, dont `docs/validation/foundations-gnome.md`, ne valident pas ce nouveau binaire.

Au démarrage, ne sélectionner aucun mode vocal : cette recette peut être menée au clavier et à la souris sans réseau fournisseur.

1. Vérifier une seule barre native, sans ancien panneau HTML ni champ de chemin. Contrôler les 10 libellés, leur ordre et les 7 raccourcis de la spécification.
2. Parcourir les menus par Alt/F10, flèches, Entrée et Echap. Vérifier bascule au survol, fermeture par clic extérieur, noms accessibles et état grisé. Une activation produit une seule action.
3. Tester Ctrl+Z/Ctrl+Y depuis Contenu, Contexte et Instructions puis via Edition ; le focus menu ne change pas la cible. Une commande de format ou un refresh canonique ne doit pas détruire l'historique.
4. Sauvegarder une note et rouvrir le `.von` dans un éditeur texte ordinaire ; distinguer contexte et corps. Ouvrir son export dans un lecteur Markdown et constater l'absence de contexte ajouté.
5. Fermer par Quitter, Alt+F4 et la croix ; mêmes trois décisions. Vérifier la barre de titre et « Sans titre » sans chemin.
6. Ouvrir Préférence : cadre système, titre, contrôles familiers du système, focus visible, Tab/Maj+Tab confinés au dialogue. La fenêtre principale ne reçoit ni clics ni commandes de note. Tester aussi Ctrl+Z dans un champ du dialogue, sans effet sur la note.
7. Vérifier que OK attend la persistance, Annuler/Echap/croix abandonne le brouillon, une seconde ouverture n'ajoute pas de fenêtre et le focus revient à l'éditeur. Contrôler la réactivation du propriétaire après échec de création ou destruction forcée du seul dialogue dans un test dédié.
8. Ouvrir A propos ; contrôler nom/version réels, Echap et restitution du focus.

Les scénarios de capture en cours utilisent les doubles pour l'automatisation. Une recette vocale réelle supplémentaire n'est effectuée que si l'opérateur veut explicitement utiliser le microphone et les services configurés.

### Contrôles Linux supplémentaires

1. Partir du focus dans chacune des trois zones WebKitGTK ; tester F10, Alt et les mnémoniques, puis la fermeture du menu et le retour à la dernière zone. Vérifier une seule exécution de Ctrl+S/Ctrl+Z/Ctrl+Y, même après réouverture d'un menu.
2. Vérifier le parentage des sélecteurs GTK et des confirmations dans la session Wayland : trois décisions attendues, Echap/croix sans écriture ni fermeture de note, aucun dialogue caché derrière `main`.
3. Ouvrir Préférence et tenter clic, clavier, raccourcis et activation de `main` par la barre des tâches. Le dialogue reste seul interactif ; aucune commande de note ne passe par un événement global. Vérifier Tab/Maj+Tab et l'absence de fenêtre documentaire dupliquée.
4. Répéter OK, Annuler, Echap, croix et destruction contrôlée du seul dialogue ; `main` redevient utilisable et la dernière zone retrouve le focus. Injecter aussi un échec de création/préparation. Vérifier le nettoyage de l'état Tauri lors de la destruction du propriétaire dans un test dédié, sans fermer une note réelle.
5. Vérifier le thème clair/sombre et la familiarité GNOME du formulaire, sans faux cadre Windows. Ne pas exiger une coordonnée de placement précise ni un vol de focus sur une autre application ; cela ne dispense pas de la modalité et de la restitution locale.

## Échanges Windows et Linux — SC-011

Préparer un jeu synthétique de 20 notes par hôte à partir des cas du [contrat des fichiers](contracts/note-format.md). Conserver, avant chaque transfert, le corps et les quatre champs d'intention attendus, ainsi que les octets de l'export ; ne stocker ces preuves qu'avec les fixtures synthétiques.

1. Transférer les 20 notes Windows vers le dossier de recette Linux sans transformation d'octets. Les ouvrir puis les réenregistrer dans l'application Linux réelle ; comparer source/intention exactes et export brut, puis rouvrir les résultats sous Windows.
2. Refaire le même parcours avec les 20 notes créées sous Linux, transférées et réenregistrées sous Windows, puis rouvertes sous Linux.
3. Sur chaque hôte récepteur, vérifier que le chemin courant est le chemin local choisi et non un chemin du système d'origine, que les `.md` sans contexte restent enregistrables en `.md` et qu'un contexte changé impose la sauvegarde complète sans toucher le `.md` initial.

Le résultat attendu est 20/20 échanges réussis par sens, avec source/contexte identiques et export égal à `source`, sans enveloppe d'application ajoutée. Les identités disque natives diffèrent normalement. Un échec est attaché au cas et au sens précis ; un simple test du codec sur un seul OS ne valide pas cette matrice.

## Mesures et critère de sortie

| Preuve | Windows 11 | Fedora/GNOME/Wayland |
|--------|------------|----------------------|
| SC-001 à SC-010 | Résultats séparés, version et configuration du binaire consignées. | Mêmes critères, résultats propres à cet hôte. |
| SC-007 | 30 essais, au moins 29 réponses visibles sous 300 ms. | 30 autres essais, même seuil ; pas de moyenne entre hôtes. |
| SC-008 | Au moins 4/5 utilisateurs familiers de Windows réussissent sans aide. | Au moins 4/5 utilisateurs familiers du bureau Linux réussissent sans aide. |
| Stockage et modalité | Exécution native, y compris les refus et nettoyages. | Exécution native, y compris échange de fichiers et parentage Wayland. |
| SC-011 | 20 échanges émis vers Linux et réception du retour. | 20 échanges émis vers Windows et réception du retour. |

Mesurer l'interaction réelle pour SC-007, pas seulement l'invocation d'un handler, avec configuration disponible et conditions de mesure consignées. Conserver une distinction entre tests unitaires, tests de stockage de chaque OS, recette native, échanges et essai utilisateur. Une étape non effectuée reste « non exécutée », pas « réussie ».

Le plan seul ne prouve aucune conformité d'exécution. La livraison exige absence de perte dans les matrices d'erreur et validation des interactions sur les deux bureaux. Aucun résultat Windows ne remplace un résultat Linux, et aucune compilation, simulation ou ancienne recette ne certifie le nouveau comportement natif.

# T014–T016 — frontière IPC et interface

Lot B06-IPC-UI du 2026-10-31, Linux/GNOME/Wayland, après T011–T013.
Implémentation validée hors ligne ; recette de la nouvelle fenêtre non effectuée.
Windows et veille hors périmètre de cette reprise.

## Changements

`DesktopShell.ts` orchestre les commandes partagées, acquittements, historique
manuel et synchronisation avec le cœur. `MenuBar.ts` conserve les panneaux
Fichier/Édition/Aide et leur navigation clavier. `Toolbar.ts` porte la mise en
forme et les commandes vocales ; `NoteView.ts` construit la projection ProseMirror ;
`StatusView.ts` affiche l'état français sans déplacer le focus. Les anciens tests
du prototype utilisent un simple alias vers cette même implémentation.

Les liens de la note ne créent aucun `href` et les images aucun `src` : leur
contenu reste représenté sans navigation ni chargement automatique. Les îlots
opaques gardent leurs octets et s'affichent littéralement. Le collage consomme
uniquement `text/plain` avant toute analyse HTML du presse-papiers. Le dépôt de
contenu par glisser-déposer est désactivé. Aucune zone de traitement fictive n'est
ajoutée ; la perception des modifications réelles relève de T030.

Les boutons de mise en forme et l'édition sont désactivés pendant le vocal ; les
gardes natives et la barrière des propositions restent en place. L'état utilise
une région `aria-live="polite"`, actualisée uniquement lorsque le texte change.

La liste des huit commandes applicatives figure dans `src/app/ipc_commands.rs`,
utilisée par le script de build Tauri pour générer leurs permissions individuelles.
`capabilities/main.json` les accorde uniquement à la fenêtre principale locale.
Aucune permission générique filesystem, shell, réseau ou notification n'est ajoutée.
Les schémas et permissions générés sont mis à jour avec cette configuration.

Le dispatch natif partagé refuse les actions et champs inconnus, exige des chemins
absolus Markdown sans NUL/traversée parent et bornés à 4096 octets, et ne traite
jamais un chemin comme une commande shell. Les fichiers restent contrôlés par
le stockage natif de T012. Un démarrage sans consentement est refusé avant tout
accès micro/fournisseur. Les instantanés français restent le canal d'état existant,
sans nouvelle API générique ni retour de clé secrète.

Un filtre natif de navigation accepte les ressources embarquées locales et, en
mode développement Tauri seulement, l'origine exacte `http://127.0.0.1:1420`.
Les URL distantes, protocoles actifs, fichiers et identifiants dans l'URL sont refusés.

## Tests et vérifications

Tests écrits d'abord dans `ipc_boundary.rs`, `markdown.test.ts` et
`DesktopShell.test.ts`. Premiers échecs : modules et dispatch absents. Les tests
ont ensuite révélé que les variantes Serde sans champ ignoraient des paramètres
supplémentaires malgré `deny_unknown_fields` ; les variantes à structure vide
corrigent ce comportement sans changer le JSON utilisé par l'interface.

| Vérification | Résultat |
| --- | --- |
| Bibliothèque Rust | 10 PASS |
| Harnais partagé, dont 6 tests IPC | 86 PASS |
| JavaScript, dont rendu hostile et fenêtre | 44 PASS |
| Typecheck/build web | PASS |
| Check Rust toutes cibles | PASS |
| Check avec ressources embarquées `tauri/custom-protocol` | PASS |
| Format Rust et `git diff --check` | PASS |

Les tests couvrent commandes/champs interdits, chemins hostiles et bornes,
absence d'effet d'une proposition à cible inexistante, ouverture échouée avec
invalidation, HTML et protocoles actifs, images distantes/locales, collage brut,
menus/bandeau, édition interdite en vocal et conservation du focus pendant les
mises à jour d'état. Les suites existantes sur stockage, voix et acquittements
restent exécutées. ProseMirror peut créer un séparateur `<img>` interne sans
source pour son curseur ; ce n'est pas une image issue de la note ni une requête.

Logs temporaires : `/tmp/vocalnotes-ui-red-js.log`,
`/tmp/vocalnotes-ui-red-rust.log`, `/tmp/vocalnotes-ui-js.log`,
`/tmp/vocalnotes-ui-rust.log`, `/tmp/vocalnotes-ui-web.log`,
`/tmp/vocalnotes-ui-check.log`, `/tmp/vocalnotes-ui-embedded-check.log`.

## Limites et suite

Le binaire principal n'est ni reconstruit ni remplacé par ce lot. SHA-256 inchangé :
`5b441b0be92924b6516b77574b066b116488c475b29e591e7401b9a39732f321`.
La recette réelle de l'ACL et de la navigation dans WebKitGTK reste à effectuer
après fermeture volontaire de l'ancienne instance puis reconstruction native.
Les tests DOM ne constituent pas une certification du comportement de la webview.
Aucune installation, aucun appel micro/fournisseur/trousseau pendant ce lot.

Recette native : vérifier saisie/formatage/undo, menus au clavier, état sans vol
de focus, note synthétique contenant HTML/liens/images, sauvegarde et conflit
B05, puis mode vocal sans édition simultanée après démarrage explicite informé.
La suite du plan est T017–T019 : politique des secrets et préférences.

## Début de recette native

À la demande utilisateur : absence d'ancienne instance vérifiée hors bac à sable,
build web puis build natif `--features tauri/custom-protocol --bin vocalnotes`
réussis. Nouvelle fenêtre lancée directement, session outil 81390 ; trace de
lancement confirmant microphone arrêté. Note synthétique de recette préparée dans
`/tmp/vocalnotes-recette-gnome/note-source.md`. Résultat utilisateur encore attendu.
Les mentions de binaire inchangé plus haut décrivent la phase d'implémentation.

### Première étape native — PASS

Utilisateur : note affichée sans image chargée ni fenêtre externe, modification
et actions manuelles concluantes. Contrôle disque : texte initial modifié,
HTML/lien/image Markdown/bloc protégé conservés, un seul fichier Markdown,
UTF-8 lisible. SHA-256 après modification :
`470a6c9fdb65cee54a58b6c427128948f9be2417bb122d3712281ed3d7e35757`.
Binaire chargé : `6d6ed980b1495b9bb1b8ac8af6047b4007cd4d8c456c76c427f0f7410527ae3f`.
Ajout externe synthétique effectué ensuite dans le fichier de recette pour
exercer la surveillance et le conflit ; retour utilisateur attendu.

### Conflit natif — conservation disque PASS, avertissement visuel non validé

Utilisateur : aucun avertissement « Modification externe détectée » observé ;
pense ne pas avoir sauvegardé explicitement. La première modification avait
pourtant été enregistrée par autosave, confirmée par lecture du fichier.
Contrôle suivant : trois fichiers Markdown présents, destination contenant
l'ajout externe, copie locale sans cet ajout et copie externe identique à la
destination. Les deux versions ont donc été préservées par le conflit réel.
La visibilité des messages dans la fenêtre reste à vérifier avec l'utilisateur ;
ne pas certifier la surveillance visuelle à partir des seules copies disque.

### Correctif de lisibilité après refus d’enregistrement

Retour utilisateur : pas de conflit visible, message « Échec de l’action » après
une tentative d'enregistrement. Le détail persistant était placé après tous les
compteurs et le refus passait par une erreur générique. Nouveau test de scénario
rouge puis vert : refus save → resynchronisation → conflit et chemins en premier,
sans remplacement par le diagnostic générique. Le stockage dispose d'un encadré
sur plusieurs lignes, avant les statistiques. Traces console statiques ajoutées
pour changement externe/conflit/erreur, sans chemin ni contenu de note.
45 tests JS et build/typecheck PASS. Code non chargé dans la fenêtre courante ;
préserver la note par copie exclusive avant fermeture et reconstruction.

Vérification du correctif : bibliothèque Rust 10 + harnais 86 PASS, aucun échec.

Utilisateur : copie de récupération puis fermeture effectuées. Contrôle :
`/tmp/vocalnotes-recette-gnome/note-recuperee.md` présent, UTF-8 lisible,
246 octets ; absence de processus VocalNotes vérifiée hors bac à sable avant
reconstruction du correctif. Les versions de conflit précédentes sont conservées.

Build natif embarqué du correctif PASS (15,08 s), nouvelle fenêtre lancée en
session outil 46227, trace initiale microphone arrêté. SHA-256 chargé :
`42c015d651a532aa116a6eba4451a5bcfcdc9df3bef3c81277a61e77200fd904`.
Recette suivante : ouvrir la copie de récupération, puis provoquer un nouveau
changement externe après confirmation d'ouverture ; avertissement visuel et
trace console à vérifier sur ce binaire.

Utilisateur : copie de récupération ouverte sur le binaire corrigé. Ajout externe synthétique synchronisé ; avertissement visuel et console en attente.

### Recette du correctif — avertissement et conflit PASS

Utilisateur : avertissement externe visible dans l'encadré, texte de la note
inchangé ; après modification manuelle, conflit visible avec les chemins des
versions préservées. Console : changement externe détecté sur le binaire corrigé.
Contrôle disque : destination avec ajout externe, copie externe identique et
copie locale récente modifiée depuis l'ouverture, sans cet ajout. Résolution vers
une nouvelle destination et fermeture restent à recetter.

### Résolution du conflit — PASS

Utilisateur : enregistrement dans `note-resolue.md` réussi, encadré de conflit
disparu et note conservée à l'écran. Contrôle : fichier UTF-8 présent ; destination précédente avec ajout externe conservée.
La nouvelle copie diffère de la copie créée à la détection du conflit ; une
égalité d’octets avec cet instantané antérieur n’est donc pas revendiquée.
Fermeture normale après résolution encore à vérifier.

### Fermeture normale après résolution — PASS

Utilisateur : fermeture normale confirmée. Absence de processus vérifiée hors
bac à sable ; note résolue toujours présente, UTF-8 lisible, HTML et bloc protégé
conservés. Recette manuelle GNOME terminée pour affichage, modification/formatage/
annulation, autosave, avertissement externe, conflit avec copies, résolution vers
une nouvelle destination et fermeture normale. Les limites précédentes restent
applicables : pas de certification exhaustive ACL/navigation WebKit, de nouvelle
recette vocale, de clavier/focus complet, de fermeture en erreur, Windows ou veille.

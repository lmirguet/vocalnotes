# Validation T006/T007 — bilan GNOME/Wayland courant

2026-10-31. Ce bilan fait autorité pour la reprise locale ; les sections B01–B04
des autres rapports conservent l'historique et ne décrivent pas toutes l'état courant.

## Périmètre et verdict

Fedora 44, GNOME 50.6, Wayland, microphone NoMachine `nx_remapped_out`, lancement
direct avec assets embarqués, sans serveur Vite. Windows n'est pas dans cette
reprise. Veille exclue sur instruction utilisateur, non testée et sans blocage
local. Aucun nouveau paquet généré ou installé ; aucune certification RPM/NSIS.

**T006/T007 terminées pour la tranche prototype GNOME/Wayland convenue.
GO local borné ; aucune certification Windows, veille ou distribution.**
Le périmètre initial de T007 sur deux OS est réduit pour cette clôture sur
instruction explicite de l’utilisateur ; ces exclusions restent non validées.
Les fonctions de stockage atomique, conflits et pont Markdown complet des tâches
T011–T013/T039 ne sont pas livrées par cette tranche et ne sont pas certifiées ici.

## Matrice de preuves

| Gate | Résultat local et limites |
|---|---|
| G02 modèles/protocole | PASS borné : WSS/STT réels, `transcription.done`, modèle STT servi rapporté `voxtral-mini-transcribe-realtime-2602`, LLM `mistral-large-latest`, schéma D6 accepté par le compte. Catalogue complet du compte non interrogé ; alias LLM mutable, aucune version figée déduite. |
| G03 capture/rotation | PASS sur les passages recettés : CPAL 44 100 Hz stéréo NoMachine, normalisation mono PCM s16le 16 kHz, plusieurs passages sans réactivation, idées conservées selon l'utilisateur. Trois tests audio et les tests de bornage/finalité passent. Pas de garantie de transcription de toute parole, ni essai prolongé. |
| G04 arrière-plan/pause | PASS utilisateur pour arrière-plan, réserve et pause ; fermeture native du micro constatée. Affichages internes vérifiés par tests UI et revue CSP/capacités ; pas de tray/notification/prise de focus ajouté. Délai parole→pause non mesuré. |
| G05 cycle/session | PASS verrouillage/déverrouillage avec retour manuel sans reprise ; perte réelle de la connexion D-Bus d'une sonde et indisponibilité au démarrage vérifiées. Tests des tickets périmés/annulations et mode manuel réutilisable PASS. La coupure de sonde ne supprime pas le service GNOME global. Veille exclue. |
| G06 trousseau | PASS backend réel : création/lecture/remplacement/suppression d'une entrée synthétique distincte ; entrée supprimée en fin de test. Indisponibilité du bus : erreur réelle, sans stockage de substitution. Clé applicative saisie et présence déjà confirmées par l'utilisateur ; champ masqué/purge et erreur IPC testés en JS. Remplacement/suppression de la vraie clé applicative non effectués. |
| G07 intégrité | PASS natif : sauvegarde exclusive, octets Unicode/CRLF/opaque, échecs droits/écriture partielle, cible existante/lien, UTF-8 invalide, ouverture/fermeture en échec. Manuel/formatage/undo/copie/nouvelle note déjà PASS utilisateur. PASS utilisateur pour le dernier contrôle visuel opaque/copie/refus ; comparaison des octets après sauvegarde PASS. |
| G08 qualité/services | PASS utilisateur sur dictée, correction, réserve et enchaînement D6. Réponses D4/D5 invalides refusées sans modification ; D6 interdit la combinaison fautive dans le schéma. Aucun fallback, aucun contenu personnel utilisé par les sondes. Pas de garantie sémantique générale. Prix/coût effectif et conditions du compte inconnus. |
| G09 plateforme/mesures | Binaire natif GNOME réellement lancé et manipulé ; mesures contextualisées dans `prototype-metrics.md`. Pas d'installation de paquet ni de Windows. Démarrage à froid jusqu'au rendu, charge vocale CPU et délai parole→affichage non mesurés. Aucun seuil de performance revendiqué. |

## Version réellement recettée

Binaire D6 `src-tauri/target/debug/vocalnotes`, SHA-256
`5b441b0be92924b6516b77574b066b116488c475b29e591e7401b9a39732f321`,
session de console 26421. Le dernier essai a donné `applied=5`,
`clarifications=0`, `rejected=0`, `no_change=0`, puis pause et fermeture native
du flux. Les sondes ajoutées ensuite ne modifient pas le code de production
et n'ont pas remplacé ce binaire pendant qu'une note pouvait être ouverte.

## Vérifications exécutées à cette reprise

- Checklist des exigences : 28/28, inchangée ; hooks avant/après absents.
- Node 22.23.2 : 32 tests JS PASS, typecheck et Vite build PASS.
- Rust Fedora 1.99.1 : 10 tests bibliothèque + 64 tests du harnais PASS,
  dont les trois tests audio désormais réutilisés dans le harnais. Aucun
  test runtime Windows revendiqué. Formatage et `git diff --check` PASS.
- Schéma D6 : 18 cas indépendants déjà vérifiés avec jsonschema 4.23.0,
  pour notes vide/non vide, réponses resolved/clarify, message blanc et ancre.
- `validate_gnome files` : vrais fichiers locaux temporaires, lecture après
  sauvegarde et comparaison d'octets, dossier non inscriptible, lien symbolique,
  fichier préexistant, entrée UTF-8 invalide ; nettoyage des seuls fichiers de sonde.
- `validate_gnome write-limit` : vraie limite de taille du processus enfant,
  erreur après écriture partielle ; note dirty complète conservée. Ce n'est pas
  un volume physiquement plein. Une destination partielle peut subsister et
  le message l'annonce ; aucun remplacement d'un fichier existant.
- `validate_gnome keyring` hors sandbox : backend Secret Service réel,
  namespace `fr.vocalnotes.validation`, identifiant unique et valeurs fictives.
  Aucun accès à l'entrée `fr.vocalnotes.desktop/mistral` par la sonde.
- `keyring-unavailable` et `lifecycle-unavailable` : adresse D-Bus inexistante
  uniquement dans le processus enfant, aucune modification de l'environnement
  desktop. Le contrôleur revient en manuel avec compteurs audio/STT/LLM à zéro.
- `validate-gnome-bus-loss.py` : relayage de la connexion de la sonde vers le vrai
  bus de session, initialisation auprès de GNOME/login1, puis fermeture du relais.
  Le vrai moniteur signale la perte. Aucun mock GNOME, microphone ou fournisseur.
- Ignore files, CSP embarquée et capacités relus : pas d'API générique shell,
  filesystem, notification ou réseau webview. L'écriture de contenu en production
  reste limitée à la copie Markdown explicite ; diagnostics numériques/staticisés.

Une assertion initiale de la sonde d'indisponibilité attendait le mot « GNOME » ;
le contrôleur a correctement retourné son message commun de perte de surveillance.
L'assertion a été corrigée et la sonde rejouée PASS ; aucun changement produit.

## Reproduction sans toucher à l'application ouverte

Depuis la racine du projet, avec les dépendances déjà installées :

```bash
CARGO_HOME=/tmp/vocalnotes-cargo cargo test --offline --locked --manifest-path src-tauri/Cargo.toml --lib --example offline_contract_tests
CARGO_HOME=/tmp/vocalnotes-cargo cargo build --offline --locked --manifest-path src-tauri/Cargo.toml --example validate_gnome
src-tauri/target/debug/examples/validate_gnome files
DBUS_SESSION_BUS_ADDRESS=unix:path=/tmp/vocalnotes-validation-no-session-bus src-tauri/target/debug/examples/validate_gnome keyring-unavailable
DBUS_SESSION_BUS_ADDRESS=unix:path=/tmp/vocalnotes-validation-no-session-bus src-tauri/target/debug/examples/validate_gnome lifecycle-unavailable
```

Dans un terminal de la vraie session GNOME, sans sudo :

```bash
src-tauri/target/debug/examples/validate_gnome keyring
python3 tooling/validate-gnome-bus-loss.py
bash -c 'trap "" XFSZ; ulimit -f 1; exec src-tauri/target/debug/examples/validate_gnome write-limit'
```

Ces commandes n'ouvrent pas le micro, n'appellent pas Mistral et ne lisent pas
la clé de l'application. La sonde trousseau est explicite, jamais lancée par
la suite de tests par défaut. Ne pas lancer les tests Cargo qui relinkent le
binaire principal pendant qu'une note peut être ouverte.

## Dernière recette visuelle — PASS

L’utilisateur confirme « cela fonctionne » pour les étapes suivantes :

1. Sauvegarder la note actuelle si nécessaire ; ouvrir le fichier synthétique
   `/tmp/vocalnotes-validation-gnome/source.md` en mode manuel.
2. Modifier seulement le paragraphe ordinaire ; vérifier que le bloc OPAQUE est
   affiché, protégé et conservé.
3. Enregistrer une nouvelle copie vers `/tmp/vocalnotes-validation-gnome/copie-ui.md`.
4. Tenter une copie vers `/tmp/vocalnotes-validation-gnome/existant.md` : refus
   attendu et note accessible. Le fichier sentinelle doit rester inchangé.

Comparaison après retour utilisateur : modification du paragraphe constatée,
bloc opaque conservé octet pour octet sans duplication, suffixe intact, fichier
sentinelle inchangé. Copie de 118 octets, SHA-256
`75cae5f134c6479058cafda2a1db63cf5c81d138ccc384670cda7fe57b45477f`.
Aucun contenu édité par l’utilisateur n’est recopié dans les preuves.

Cette clôture autorise la suite du prototype local. Elle ne certifie pas
les fonctions futures, l’installation d’un paquet, Windows ou les mesures
explicitement non réalisées. Aucune tâche T011+ commencée dans cette reprise.

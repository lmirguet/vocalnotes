# T017–T019 — secrets et préférences

Lot B07-PREFERENCES, 2026-10-31, Linux/GNOME/Wayland. Implémentation et tests
réussis, dont un test sur une entrée isolée du vrai trousseau. Nouvelle recette
de la fenêtre non effectuée ; application laissée fermée.

## Comportement

`SecretStore` construit explicitement une entrée Secret Service sur Linux et
Credential Manager sur Windows. Le type concret du backend est vérifié : un
backend mock ou autre type est refusé. Les identifiants de l'entrée applicative
restent identiques au prototype, sans migration ni lecture de sa clé par l'agent.
Une panne du trousseau renvoie une erreur statique, sans stockage fichier/mémoire
de substitution. Les valeurs transitent dans des buffers `Zeroizing`, y compris
l'attente du travail bloquant après l'entrée IPC. Aucun getter de clé n'est exposé
à la webview ; présence, saisie et suppression restent des commandes dédiées.

`config/preferences.rs` conserve seulement les choix STT/LLM séparés et le chemin
de la dernière note effectivement ouverte/sauvegardée. Le JSON strict refuse les
champs supplémentaires, notamment une clé ajoutée. Identifiants bornés et validés,
fournisseur intégré Mistral seulement. Une absence de fichier utilise les valeurs
initiales documentées ; un fichier invalide ou inaccessible produit une erreur,
refuse le vocal et conserve l'accès à l'édition manuelle.

Les préférences sont chargées depuis `app_config_dir()/preferences.json`, chemin
fourni par Tauri. Écriture exclusive dans un fichier voisin, permissions 0600 sous
Unix, synchronisation puis renommage ; liens symboliques et fichiers non ordinaires
à la destination refusés. Un échec garde les réglages actifs en mémoire. Il ne
s'agit pas d'une garantie universelle contre un écrivain concurrent non coopératif.
Les tests n'écrivent que dans des dossiers temporaires ; aucun fichier de
configuration de l'utilisateur n'a été créé pendant cette implémentation.

Les modèles sont copiés au démarrage explicite, puis transmis aux requêtes STT,
LLM ordinaires et prioritaires de cette session. Une modification de réglage ne
change pas ces copies. Une reprise conserve les choix suspendus ; un nouveau
démarrage prend les nouveaux réglages. Hôtes et protocoles restent ceux de
l'intégration Mistral : aucun repli vers un autre fournisseur. Le modèle réellement
servi demeure distinct du modèle demandé dans les mesures.

Édition → Préférences propose ces choix sans édition manuelle du JSON. Les modèles
personnalisés sont indiqués comme non vérifiés ; enregistrer un réglage ne fait
aucun appel fournisseur. Le panneau rassemble aussi la saisie masquée, la présence
et la suppression de clé déjà disponibles. Le champ est vidé avant envoi IPC,
à la fermeture du panneau et à la destruction de l'interface. Le chemin de dernière
note est mémorisé sans réouverture automatique dans ce lot.

Deux commandes locales de préférences s'ajoutent aux permissions Tauri explicites
de la fenêtre principale, soit dix commandes applicatives au total. Le panneau
complet, le test explicite d'accès au modèle et les mesures de coût de T045–T049
ne sont pas déclarés achevés par ce lot.

## Vérifications

Tests d'abord : absence de `SecretStore` et de `config` constatée avant code,
puis absence du composant de préférences. Suites finales :

| Vérification | Résultat |
| --- | --- |
| Bibliothèque Rust | 10 PASS |
| Harnais partagé, dont 11 tests de politique/configuration | 97 PASS |
| JavaScript | 48 PASS |
| Typecheck/build web | PASS |
| Check Rust toutes cibles avec `tauri/custom-protocol` | PASS |
| Format et `git diff --check` | PASS |
| Entrée synthétique du vrai Secret Service | PASS, nettoyée |

Les tests couvrent refus du mock, entrées invalides, trousseau indisponible dans
un sous-processus avec adresse D-Bus invalide et absence de fichiers de repli,
persistance/rechargement, champs secrets interdits, fournisseur non intégré,
modèles invalides, échec d'écriture, liens symboliques, configuration figée en
reprise, nouvelle configuration au prochain démarrage, chemin de dernière note
sans contenu, URL STT sans secret et hôte fixe, panneau chargé seulement à son
ouverture, erreurs expurgées et effacement d'une clé non envoyée.

Le test natif opt-in a créé une entrée `fr.vocalnotes.validation` avec identifiant
unique, écrit une valeur synthétique, remplacé cette valeur, relu le remplacement,
puis supprimé l'entrée et vérifié son absence. La clé `fr.vocalnotes.desktop`
n'a pas été consultée ou modifiée. Le test utilise le même `SecretStore` que
l'application et ne se limite pas à un backend simulé. Le helper opt-in ne fait
aucune opération trousseau lors de la suite ordinaire ; son exécution réelle
séparée a donné 1 PASS.

Logs temporaires : `/tmp/vocalnotes-preferences-red.log`,
`/tmp/vocalnotes-preferences-rust.log`, `/tmp/vocalnotes-preferences-js-red.log`,
`/tmp/vocalnotes-preferences-js.log`, `/tmp/vocalnotes-preferences-web.log`,
`/tmp/vocalnotes-preferences-check.log`. Le test réel du trousseau est également
consigné dans la sortie de l'outil de cette session.

## Limites et suite

Pas d'appel fournisseur ni micro, pas d'installation ni nouvelle dépendance.
Windows n'est pas testé dans cette reprise. Les tests de gel de configuration
n'attestent pas l'accès à un modèle personnalisé sur un compte fournisseur.
Le binaire principal reste inchangé :
`42c015d651a532aa116a6eba4451a5bcfcdc9df3bef3c81277a61e77200fd904`.
La fenêtre n'a pas été relancée ; ces nouveautés ne sont pas encore chargées
dans le binaire existant.

Recette native restante : reconstruire, ouvrir Édition → Préférences, vérifier les
choix séparés et leur persistance après fermeture/relance, présence de la clé
sans révélation, erreur de réglage invalide sans perte des valeurs précédentes.
Un nouveau test fournisseur exige le démarrage explicite informé habituel.
Suite du plan : T020–T022, contrats et extraction des adaptateurs.

## Début de recette native à la demande utilisateur

Ancienne instance absente, vérifiée hors bac à sable. Build web puis build natif
embarqué réussis ; nouvelle fenêtre lancée directement pour la recette des
préférences. Résultats utilisateur en attente. Les mentions de binaire inchangé
ci-dessus décrivent la phase d'implémentation précédente.

### Panneau et présence de clé — PASS utilisateur

Utilisateur confirme Édition → Préférences, modèles STT
`voxtral-mini-transcribe-realtime-2602` et LLM `mistral-large-latest`, ainsi que
présence de clé sans révélation. Binaire chargé :
`b549a055da0e2668840ea43f93f92806fe2ed216f52e8fafe2c08dcfad10f6b9`,
session outil 33511, microphone arrêté au lancement. Vérification de présence
initiée explicitement par l'utilisateur ; aucune valeur de clé récupérée par
l'agent. Validation d'un réglage invalide et persistance restent à recetter.

### Réglage invalide puis valide — PASS utilisateur

Utilisateur confirme refus de `mistral modèle invalide`, puis réussite après
rétablissement de `mistral-large-latest`. Contrôle du fichier applicatif non
sensible : présence, champs STT/LLM/lastNote uniquement, valeurs habituelles
STT/LLM conservées. Aucun appel fournisseur pendant ces actions. Persistance
avec une valeur distinctive après redémarrage à exercer ensuite.

Utilisateur confirme enregistrement du modèle temporaire distinctif puis fermeture.
Contrôle disque : LLM `mistral-recette-persistance` présent, STT habituel conservé.
Absence de processus vérifiée hors bac à sable avant relance du même binaire.

### Persistance après redémarrage et rétablissement — PASS

Utilisateur : valeur temporaire retrouvée dans le panneau après relance, puis
modèle `mistral-large-latest` rétabli et enregistré avec succès. Contrôle disque :
LLM habituel rétabli, STT `voxtral-mini-transcribe-realtime-2602` conservé.
Relance du même binaire en session outil 29307, microphone arrêté au lancement.
Recette de l'interface concluante pour panneau, modèles séparés, présence de clé
sans révélation, refus d'identifiant invalide, enregistrement valide et persistance.
Aucun appel fournisseur/micro pour ces tests. Le gel des modèles en session reste
validé par les tests automatisés, sans nouvelle recette vocale. L'application
reste ouverte à l'issue de cette étape ; aucun arrêt effectué par l'agent.

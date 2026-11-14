# Data Model: Menus et cycle de document Windows et Linux

Date : 2026-11-13. Modèle de conception ; les noms sont les contrats cibles, pas des types déjà implémentés.

## 1. Document canonique

Le domaine Rust reste l'unique autorité. Les champs existants `noteId`, `revision`, `inputGeneration`, `manualPeriod`, source et intention sont conservés.

| Champ | Sens / invariant |
|-------|------------------|
| `noteId` | Identité monotone de la note en mémoire ; change uniquement après Nouvelle note ou Ouvrir réussi. |
| `revision` | Révision monotone des changements persistants acceptés, y compris Annuler/Rétablir ; ne revient jamais à une ancienne valeur. |
| `inputGeneration` | Génération d'entrée des traitements existants ; protège aussi les instructions et les résultats de reformulation. |
| `source` | Markdown UTF-8 exact de la note, indépendant de son rendu ProseMirror. |
| `intention` | `DraftingIntention` existante : `purpose`, `form`, `tone`, `context`. Seul le contexte est éditable dans les nouveaux parcours ; préserver les autres champs historiques. |
| `instructions` | Texte temporaire, hors état sauvegardé et hors calcul de `dirty`. |
| `destination` | Chemin absolu local éventuellement associé ; change seulement après ouverture ou sauvegarde complète réussie. Jamais sérialisé dans le fichier de note ni interprété sur l'autre système. |
| `format` | `von`, `markdown` ou absence de format sans destination ; déterminé par le chemin sélectionné, extension insensible à la casse. |
| `savedState` | État persistant exact capturé par la dernière sauvegarde réussie, ou contenu/contexte vides et intention vide au départ. |
| `openedContext` | Contexte chargé à l'ouverture du fichier courant, pour la règle de migration des `.md` ; remis à jour lors de l'adoption d'une nouvelle destination. |
| `diskVersion` | Version du fichier associé validée à l'ouverture/écriture ; distincte des compteurs de note. |
| `history` | Historiques en mémoire des trois zones, liés à ce `noteId`. |
| `workflow` | Aucune opération ou référence de l'opération active. |

### État modifié

`dirty` est dérivé de la différence entre l'état persistant courant et `savedState`, pas d'une révision ni d'un booléen irréversible. Le contexte et le contenu y participent ; les préférences, la sélection, les instructions, l'historique et l'état du micro n'y participent pas.

Les autres champs historiques d'intention sont conservés à l'identique ; tout chemin existant qui peut encore les modifier doit maintenir le même invariant d'intégrité. Une mutation ne déclenche jamais une écriture disque implicite.

Une sauvegarde de la révision R écrit l'état capturé en R. Si le document a déjà changé lors de l'acquittement disque, `savedState` devient l'état réellement écrit et le document peut rester `dirty`. Ne pas recopier le dernier snapshot mémoire comme s'il avait été écrit.

Une note neuve sans contenu ni contexte est propre. Saisir puis annuler jusqu'à cet état la rend à nouveau propre, même sans destination.

### Extension et transition de format

| Situation | Comportement |
|-----------|--------------|
| Nouvelle note sans chemin, Enregistrer | Choix de fichier, `.von` par défaut. |
| `.von` associé, Enregistrer | Même chemin, contenu et intention complets. |
| `.md` associé, aucun contexte défini | Même chemin `.md`, aucun dialogue de conversion imposé. |
| `.md` historique, contexte égal à celui chargé | Même chemin en conservant les métadonnées existantes. |
| `.md`, contexte différent de `openedContext` | Enregistrer devient Enregistrer sous en `.von` ; `.md` initial inchangé. |
| Enregistrer sous | Toujours demander un chemin ; proposer `.von` et n'adopter le chemin qu'après réussite. |
| Exporter | `.md`, source seule ; aucun changement de format ou de chemin de travail. |

Vider un contexte chargé non vide constitue bien une différence. Ajouter puis annuler un contexte jusqu'à l'état chargé annule cette différence : aucune conversion n'est forcée pour une modification abandonnée.

## 2. Version disque et transaction de sauvegarde

| Élément | Champs / rôle |
|---------|---------------|
| `DiskVersion` | Empreinte SHA-256 des octets, identité native obtenue par handle Windows ou `(dev, ino)` sous Linux, caractéristiques nécessaires au contrôle de liens et de permissions. Recalculée à l'ouverture locale, jamais persistée dans `.von`. |
| `WriteCandidate` | État persistant capturé, octets sérialisés, chemin demandé, format, version de note et version disque attendue. |
| `OverwriteConsent` | Accord sur une destination et une version disque précise ; jamais un booléen global désactivant les vérifications futures. |
| `WriteResult` | `saved`, `conflict` ou `failed`, avec versions et chemins de récupération si présents ; aucun résultat de copie n'est assimilé à `saved`. |

### Transitions

1. Valider le chemin/format, la taille et la sérialisation ; les échecs n'affectent pas le document.
2. Observer la destination. Si elle existe et doit être remplacée après Enregistrer sous ou export, demander confirmation. Pour un fichier courant, comparer avec `diskVersion` ; demander une décision en cas de différence externe.
3. Préparer exclusivement un fichier du même dossier et synchroniser son contenu.
4. Installer sans écrasement si la destination était absente, ou remplacer avec sauvegarde de la version déplacée si la version attendue/confirmée existe.
5. Vérifier la version installée et la version déplacée ; détecter une nouvelle course et préserver les copies utiles sans rollback destructif.
6. Sur succès complet, avancer `diskVersion` et `savedState` pour une sauvegarde de travail, éventuellement adopter le nouveau chemin, puis nettoyer les seuls fichiers transitoires vérifiés.
7. Sur conflit/échec, laisser la destination logique antérieure et l'état modifié inchangés ; expliquer les actions de récupération. Ne pas adopter la copie de préparation.

Les fichiers de récupération contiennent seulement le document sérialisé ou la version externe déplacée. Ils ne constituent pas une fonctionnalité nouvelle de récupération de session, ne contiennent ni historique en mémoire ni instructions temporaires, et ne sont pas supprimés si leur utilité reste incertaine.

### Réalisation native et refus explicites

| Système | Remplacement existant | Destination absente | Éléments contrôlés |
|---------|-----------------------|---------------------|--------------------|
| Windows | `ReplaceFileW`, version déplacée récupérable et vérifications. | Installation de la préparation sans écrasement d'un fichier apparu entre-temps. | Identité par handle, volume/dossier, points de réanalyse, liens, ACL et échecs partiels. |
| Linux | `renameat2(RENAME_EXCHANGE)` dans le répertoire déjà ouvert. | `renameat2(RENAME_NOREPLACE)` après synchronisation de la préparation. | `openat`/`O_NOFOLLOW`, `(dev, ino)`, liens, propriétaire, permissions/ACL, volume local, synchronisation du répertoire. |

Une primitive indisponible ou des métadonnées ne permettant pas un remplacement sûr produisent `unsupported_storage` ou `write_failed` avec explication et éventuels chemins récupérables. Le résultat historique `Copy` ne change plus implicitement `destination`, `savedState` ou `dirty`, sur aucun système. Une autre destination relève d'un nouvel Enregistrer sous explicitement choisi.

Le suffixe est insensible à la casse, pas le chemin complet. Deux chemins Linux différant seulement par la casse ne sont pas fusionnés ; l'identité effective reste nécessaire pour refuser un export sur le document courant. Une copie transférée possède une nouvelle identité locale sans changer sa source ni son intention.

## 3. Opération documentaire

Une seule opération de premier plan est active dans le coordinateur. Son identifiant monotone ne peut être choisi par la webview.

| Champ | Description |
|-------|-------------|
| `operationId` | Identifiant unique de cette demande, utilisé pour acquitter/annuler sans agir sur une autre demande. |
| `kind` | `new`, `open`, `save`, `save_as`, `export_markdown`, `quit`. |
| `phase` | `awaiting_ui`, `draining`, `choosing_file`, `confirming_save`, `confirming_overwrite`, `writing`, `committing`, `idle`. |
| `noteId` | Document d'origine de l'opération ; un changement inattendu invalide l'opération. |
| `uiAcknowledgement` | Révision et génération confirmant que les saisies déjà proposées ont été traitées sans divergence. |
| `candidate` | Candidat de lecture ou d'écriture, gardé séparé de la note active tant que non validé. |
| `outcome` | Terminé, annulé ou échec typé ; aucune annulation n'est présentée comme une réussite. |

### Nouvelle note, Ouvrir et Quitter

`idle → awaiting_ui → draining → [choosing_file pour Ouvrir] → [confirming_save si dirty] → [choosing_file / writing si Enregistrer] → committing → idle`.

La lecture/validation d'un candidat Ouvrir peut précéder la confirmation de sauvegarde, mais sa publication ne la précède jamais. Annuler le sélecteur Ouvrir évite toute destruction de la note courante.

- `awaiting_ui` gèle les nouvelles éditions et attend les propositions et champs différés déjà engagés. Un texte refusé/récupérable non résolu bloque une action destructive ; l'UI ne l'annonce pas comme enregistré.
- `draining` arrête la capture, reçoit les derniers segments et attend une reformulation déjà engagée ; pas de nouvelle requête ni d'invalidation prématurée de leurs préconditions. Chaque résultat reste soumis aux gardes existantes.
- Toute attente peut être annulée via `operationId`. L'annulation rend les traitements restants non applicables, conserve les résultats déjà acceptés et laisse le micro arrêté. Un échec produit le même abandon de fermeture/remplacement avec un diagnostic explicite.
- Enregistrer poursuit uniquement après un résultat `saved`. Ne pas enregistrer autorise le commit demandé sans modifier la version disque courante. Annuler interrompt l'opération.
- Nouvelle note/Ouvrir publient atomiquement leur nouvel état et changent `noteId`. Quitter détruit `main` une fois les verrous libérés.
- Une seconde demande pendant l'opération reçoit l'opération active, sans second dialogue. Une demande périmée ne peut pas valider une opération suivante.

### Enregistrer, Enregistrer sous et Exporter

`idle → awaiting_ui → [choosing_file] → [confirming_overwrite] → writing → committing → idle`.

Ces actions n'arrêtent pas la dictée et n'attendent pas une requête réseau : elles utilisent l'état courant accepté à la capture d'écriture. Les résultats ultérieurs restent dans la même note et peuvent la rendre modifiée. Les changements de document ne peuvent pas dépasser cette opération.

L'export n'avance ni `savedState` ni `diskVersion` du document et ne modifie aucun historique. Si sa destination désigne le fichier de travail courant, y compris par alias, il est refusé.

## 4. Historique d'édition

Chaque zone possède ses piles undo/redo et une valeur canonique. Les sélections visuelles restent une information UI ; aucune sélection ne donne le droit de réécrire un autre champ.

| Champ d'entrée | Rôle |
|----------------|------|
| `entryId`, `noteId`, `area` | Identité et portée : `content`, `context` ou `instructions`. |
| `origin` | `manual`, `formatting`, `dictation`, `reformat` ; information en mémoire seulement. |
| `before`, `after` | Deltas réversibles avec texte attendu, sur frontières UTF-8 valides ; contexte et instructions peuvent utiliser un remplacement de champ complet. |
| `transactionId` / passage | Déduplication d'une proposition réessayée ou regroupement d'un segment finalisé. |

Règles :

- Une saisie ou mise en forme acceptée ajoute une entrée ; un segment finalisé de dictée une entrée ; une reformulation réussie une entrée de contenu.
- Annuler/Rétablir vérifie `noteId`, révision/génération courante et valeur attendue de la zone. Il ne repasse pas par un appel fournisseur ni par une sauvegarde.
- Un changement d'une autre zone n'efface pas cette pile. Une nouvelle édition après Annuler efface le redo de la zone modifiée.
- Les compteurs de version restent monotones ; `dirty` est recalculé après la mutation.
- Nouvelle note/Ouvrir vident toutes les piles. Une sauvegarde ou un export ne les vide pas. La sortie d'Instructions efface sa valeur et son historique, pour empêcher sa récupération après effacement volontaire.
- Capacité globale : 256 entrées et 16 Mio de deltas. Retirer les plus anciennes entrées en gardant des piles cohérentes et indiquer la limite dans l'état utilisateur ; ne jamais perdre le texte courant.

Le snapshot expose `canUndo` et `canRedo` par zone. La dernière zone active est mémorisée par le bridge avant le focus menu ; la disponibilité grisée vient du domaine.

## 5. Préférences et modalité

| Élément | Description |
|---------|-------------|
| `PreferencesStore` | Magasin existant de modèles STT/LLM et dernière destination locale ; jamais de secret. Les clés restent dans le trousseau natif, Windows Credential Manager ou Secret Service Linux. |
| `PreferencesDraft` | Copie locale des modèles à l'ouverture ; modifications temporaires jusqu'à OK. |
| `PreferencesWindow` | Label fixe `preferences`, propriétaire `main`, cadre système, unique, sans menu de note. |
| `ModalGuard` | État de modalité attaché à l'instance réelle : propriétaire désactivé par Win32 ou fenêtre GTK transitoire/modale. Nettoyage idempotent sur fermeture, destruction du propriétaire ou échec. Aucun objet GTK transféré hors du thread principal. |

`fermée → création → ouverte → validation/écriture OK → fermeture`.

- Annuler/Echap/croix passent directement de ouverte à fermeture sans persister le brouillon.
- Une validation ou écriture échouée revient à ouverte avec le brouillon et le diagnostic.
- Une seconde demande d'ouverture focalise l'instance présente.
- Créer la fenêtre cachée ; établir le lien de propriétaire et la modalité avant de l'afficher. Sous Linux, `gtk_window()` et `GtkWindowExt::{set_transient_for,set_modal,set_destroy_with_parent}` sont exécutés sur le thread GTK ; sous Windows, le garde utilise `EnableWindow`.
- À la fermeture, réactiver le propriétaire Windows ou libérer la modalité GTK, puis restituer la dernière zone d'édition de `main`. Fermer explicitement les préférences avant destruction validée de `main` ; le lien GTK `destroy_with_parent` ne remplace pas le nettoyage de l'état Tauri. Une demande documentaire pendant la modalité ne peut pas contourner le garde.
- Un échec de préparation détruit l'instance incomplète et libère la modalité ; la note reste accessible avec un diagnostic. La recette Wayland vérifie le focus réel sans exiger coordonnées absolues ou priorité globale sur les autres applications.
- Les actions de trousseau gardent leurs boutons de validation explicite existants ; elles sont distinguées du brouillon de modèles et n'exposent jamais une clé stockée. Une clé en cours de saisie est effacée à la fermeture.
- Aucune transition de préférences ne modifie le document, `savedState` ou les historiques ; les règles existantes de prise d'effet des modèles demeurent.

## 6. Validations transversales

- Conserver les limites existantes : fichier sérialisé de 1 Mio au maximum, intention cumulée de 4096 octets, instructions de 16 384 octets. Vérifier aussi les limites après échappement de l'enveloppe, pas seulement la longueur du corps.
- Les chemins sont absolus, locaux, sans NUL ni traversal, limités comme les chemins actuels ; contrôler les destinations par identité native, pas par simple chaîne insensible à la casse. Refuser les liens/points de réanalyse ou volumes dont les garanties de remplacement ne sont pas établies, avec une erreur actionable.
- Ne pas tronquer de contenu pour faire entrer un fichier ou un historique dans une limite.
- Les snapshots reflètent erreurs, attente et fin réelle ; une erreur de capacité, de verrou, de disque ou de fournisseur n'est pas remplacée par une valeur de succès.
- Le codec et les limites sont communs aux deux systèmes : aucun champ de plateforme, chemin, historique ou préférence dans le document ; aucune normalisation de Unicode ou des fins de ligne du corps lors d'un échange.

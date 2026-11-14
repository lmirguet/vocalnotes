# Contract: Commandes desktop, synchronisation et fenêtres

Statut : surface cible à implémenter, commune à Windows 11 et Fedora/GNOME/Wayland. Les commandes de dictée et les propositions manuelles actuelles restent en place, sauf les anciennes voies documentaires qui doivent rejoindre ce protocole.

## 1. Menu de `main`

| Menu | Identifiant stable | Texte | Accélérateur |
|------|---------------------|-------|--------------|
| Fichier | `file.new` | Nouvelle note | Ctrl+N |
| Fichier | `file.open` | Ouvrir | Ctrl+O |
| Fichier | `file.save` | Enregistrer | Ctrl+S |
| Fichier | `file.save_as` | Enregistrer sous | Ctrl+Maj+S |
| Fichier | `file.export_markdown` | Exporter en markdown | Aucun |
| Fichier | `file.quit` | Quitter | Alt+F4 |
| Edition | `edit.preferences` | Préférence | Aucun |
| Edition | `edit.undo` | Annuler | Ctrl+Z |
| Edition | `edit.redo` | Rétablir | Ctrl+Y |
| Aide | `help.about` | A propos | Aucun |

Les séparateurs ne sont pas des commandes. Les mnémoniques sont distincts dans leur portée et n'altèrent pas le texte visible demandé. Titres Tauri : `&Fichier`, `&Edition`, `&Aide`. Dans Fichier : `&Nouvelle note`, `&Ouvrir`, `&Enregistrer`, `Enregistrer &sous`, `E&xporter en markdown`, `&Quitter` ; dans Edition : `&Préférence`, `&Annuler`, `&Rétablir` ; dans Aide : `A &propos`. L'adaptateur GTK traduit ces marqueurs via muda.

La navigation doit satisfaire Alt/F10, flèches, Entrée, survol et Echap sur les deux bureaux, y compris quand le focus initial est dans WebView2 ou WebKitGTK. Installer le menu sur `main` seulement ; ne pas dépendre d'un menu global GNOME. Les accélérateurs natifs possèdent seuls les raccourcis de note et ne doublonnent pas des handlers DOM. Les callbacks pouvant recevoir des événements d'autres fenêtres, le répartiteur vérifie identifiant, fenêtre active et modalité avant toute action.

`CloseRequested` de `main`, Alt+F4 et `file.quit` demandent tous `quit` au même coordinateur. Une double notification système/menu est fusionnée tant que l'opération est active. Aucun appel de destruction directe depuis la webview.

Annuler/Rétablir sont grisés selon la dernière zone d'édition active et l'historique canonique ; indisponibles pendant une transition incompatible. Pendant les préférences, les commandes de note ne peuvent pas s'exécuter, même depuis un handler global.

## 2. Appels renderer → Rust

Ajouter une commande IPC applicative `document_action` avec une union typée, `deny_unknown_fields`, et le label de fenêtre appelant vérifié côté Rust. Pas de chemins libres dans cette union.

| Variante `type` | Données | Effet |
|----------------|---------|-------|
| `request` | `action`: `new`, `open`, `save`, `save_as`, `export_markdown` ou `quit` | Créer une opération ; si occupé, retourner l'opération active sans créer un autre dialogue. |
| `ready` | `operationId`, `noteId`, `revision`, `inputGeneration` | Signaler que les propositions/champs UI ont été acquittés sans divergence ; reprendre l'opération correspondante. |
| `cancel` | `operationId` | Annuler une attente ou une préparation avant commit ; ne peut pas interrompre rétroactivement une écriture déjà engagée. |
| `focus` | `area`: `content`, `context` ou `instructions` | Mémoriser la cible d'historique ; ne modifie aucun texte et ne démarre pas le micro. |
| `history` | `direction`: `undo`/`redo`, `area`, `transactionId`, `noteId`, `baseRevision`, `inputGeneration` | Appliquer une mutation d'historique une seule fois après validation des préconditions. |

Les commandes de menu Rust créent les mêmes intentions ; elles ne se réinjectent pas via un deuxième `request` JS. La réponse comprend le snapshot à jour et l'état d'opération typé. Une répétition de `ready` pour une opération déjà terminée ne répète pas son écriture ; elle est signalée comme périmée.

Les compteurs d'identité suivent les conventions actuelles ; le contrat n'introduit pas de perte de précision nouvelle dans les DTO JavaScript. L'implémentation doit garder les identifiants dans la plage entière sûre ou les sérialiser sans perte.

### Préconditions de `ready`

- L'opération et la note doivent encore correspondre.
- La révision/génération envoyée doit être celle de l'état canonique ; si une dictée a avancé le domaine, demander une nouvelle réconciliation, pas accepter une saisie ancienne par défaut.
- Le bridge ne doit envoyer `ready` qu'après vidage des propositions manuelles, champs différés et commandes de champs déjà engagées.
- Une saisie refusée, une queue divergente ou une saisie récupérable non résolue empêche `ready` pour une action destructive et laisse la note accessible.
- L'UI gèle les nouvelles éditions pendant cette préparation, mais doit continuer à afficher les résultats canoniques et les boutons d'annulation d'attente.
- Après `ready`, le coordinateur revalide l'état au moment de capturer l'écriture ou de prendre la décision de sauvegarde. Aucun `dirty` fourni par le renderer ne fait autorité.

### Préconditions de l'historique

Le changement porte seulement sur `area`, avec préconditions de note et de version. Les commandes de menu utilisent la dernière zone ayant le focus avant l'ouverture du menu ; elles ne prennent pas le focus courant du titre de menu comme nouvelle cible.

Même transaction rejouée : acquittement cohérent, pas de deuxième annulation. Version périmée ou résultat attendu différent : conflit explicite et réconciliation, pas de restauration arbitraire.

Le renderer n'exécute pas en plus l'undo natif du `textarea` ou celui d'un plugin ProseMirror. La pile canonique reçoit saisie, mise en forme, dictée et reformulation. Les items undo/redo prédéfinis Tauri ne sont pas utilisés sur Windows ou Linux. Dans la fenêtre Préférences, les raccourcis d'édition de ses propres champs ne sont pas des commandes d'historique de note.

## 3. Événements Rust → renderer

Un événement ciblé `document-command` sur `main` transporte seulement l'intention à préparer ou une action d'historique, avec son identifiant de dispatch. Les opérations documentaires possèdent aussi `operationId`.

- L'événement ne contient ni chemin, ni texte, ni clé ; le contenu provient du snapshot existant.
- Le renderer enregistre son listener avant d'activer les commandes. Une opération reste visible dans le snapshot si l'événement a précédé l'abonnement ; elle n'est pas perdue.
- Les dispatchs déjà traités sont dédupliqués. `dispose` désabonne le listener.
- La webview n'est pas autorisée à émettre elle-même cet événement de contrôle ; une notification JS ne constitue de toute façon pas une autorisation d'écrire.
- Ajouter uniquement les permissions d'écoute/désabonnement nécessaires à `main`, pas le jeu complet d'API événementielles.

## 4. Compléments de snapshot

Étendre les DTO Rust/TypeScript ensemble et conserver les champs existants des tests de dictée :

| Champ | Description |
|-------|-------------|
| `documentName` | Nom de fichier ou « Sans titre », calculé côté natif. |
| `documentPath` | Chemin courant facultatif pour affichage local, sans autorité pour écrire. |
| `documentFormat` | `von`, `markdown` ou null. |
| `dirty` | Dérivé de l'état sauvegardé ; exclut les instructions. |
| `history` | `canUndo` / `canRedo` pour chacune des trois zones. |
| `documentOperation` | Opération facultative : identifiant, action, phase, attente annulable ou non. |
| `documentError` | Code utilisateur borné et diagnostic autorisé ; pas de payload fournisseur ou contenu arbitraire d'exception. |

Le titre natif est mis à jour avec nom de note, indicateur de modification et nom d'application. L'état disque et l'état d'opération ne remplacent pas les états vocaux existants.

## 5. Préférences et A propos

Ajouter `preferences_open` et `preferences_close` :

- `preferences_open` est réservé à `main` et crée ou focalise `preferences` côté Rust ; aucune URL ou label libre n'est accepté.
- `preferences_close` est réservé à `preferences`. Le bouton OK appelle d'abord `preferences_set` avec les modèles validés ; seul son succès autorise cette fermeture. Annuler/Echap et la croix ferment sans cet appel.
- Conserver `preferences_get` pour les modèles affichés dans `main` et le dialogue. Retourner aux préférences seulement les données utiles ; la dernière destination n'est pas nécessaire au formulaire.
- Réserver l'écriture des modèles et les opérations explicites de trousseau au dialogue. Conserver les commandes de clés existantes et leur absence de valeur de retour secrète.
- Le brouillon de modèles reste local à la fenêtre et n'est pas conservé après Annuler. Les boutons explicites de gestion du trousseau gardent leurs effets immédiats, clairement distingués du brouillon ; une clé simplement saisie puis abandonnée ne quitte pas le champ.
- Le label réel détermine le montage UI ; un paramètre de navigation n'accorde aucun droit. La fenêtre utilise le cadre système et un formulaire sobre suivant thème, typographie et focus du système, sans tableau de bord ni imitation d'un cadre Windows sous GNOME.
- Les chemins OK, Annuler, Echap, croix, destruction inattendue et échec de création réactivent le propriétaire ; le focus revient à l'éditeur après fermeture. Pour la croix, intervenir sur la fenêtre concernée, jamais sur le cycle de sauvegarde de `main`.
- A propos est un dialogue système propriétaire affichant le nom et la version de `app.package_info()` ; il ne modifie pas la note.

La fenêtre possédée est construite cachée et sans blocage du thread de messages, puis affichée seulement après préparation de la modalité :

| Système | Adaptation native et cycle de vie |
|---------|-----------------------------------|
| Windows | Propriétaire `main`, cadre système, `EnableWindow` pour désactiver puis réactiver le propriétaire avant destruction. Vérifier son état, pas un faux code de succès du retour Win32. |
| Linux | `parent`/relation transitoire avec `main`, `gtk_window()` puis `GtkWindowExt::set_transient_for`, `set_modal(true)` et `set_destroy_with_parent(true)` sur le thread GTK via `run_on_main_thread`. Libérer la modalité et restaurer le focus à la fermeture. |

Une relation parent/enfant, `always_on_top` ou une désactivation CSS n'est pas une modalité. Le nettoyage est idempotent même si la fenêtre ou son propriétaire est détruit de façon inattendue ; la fermeture validée de `main` ferme explicitement le dialogue. Sous Wayland, ne pas forcer une position ou une activation globale ; la restitution de focus après fermeture utilisateur doit être validée sur GNOME.

Les sélecteurs et confirmations utilisent `tauri-plugin-dialog` côté Rust sous Windows, avec contrôle du handle de `main`. Sous Linux, ils utilisent directement GTK3 sur le thread principal : `rfd 0.17.2` verrouillé ignore le parent de ses dialogues GTK. Définir et vérifier le parent transitoire réel et la modalité, activer la destruction avec le propriétaire, puis adapter les signaux de réponse/destruction en attente asynchrone. Ne jamais utiliser `Dialog::run` ni transférer un objet GTK entre threads. L'abandon de l'attente ferme le dialogue GTK restant.

Les trois issues Enregistrer / Ne pas enregistrer / Annuler et les variantes `Custom` reconnues sont mappées explicitement. Croix/Echap/Cancel annulent ; réponse inconnue, callback perdu ou chemin non local échouent avec diagnostic, sans poursuivre l'action destructive. Ouvrir accepte `.von`/`.md` ; Enregistrer sous propose `.von` et Exporter `.md`. Ne compléter que les suffixes absents des chemins sélectionnés, sans modifier une extension incompatible. Aucun appel de dialogue bloquant sur le thread UI ni verrou du domaine conservé pendant l'attente.

La confirmation de remplacement reçoit le chemin et la `DiskVersion` déjà observée, puis rend soit un `OverwriteConsent` consommable lié à cette paire, soit Choisir un autre fichier ou Annuler. Le moteur recontrôle ce consentement à l'écriture. La confirmation éventuelle intégrée au sélecteur Windows n'est pas considérée comme un consentement versionné ; celle de GTK est désactivée au profit de ce parcours explicite.

## 6. Erreurs et compatibilité des commandes

Codes attendus : `forbidden` (fenêtre ou origine non autorisée), `busy`, `stale_operation`, `unsynchronized_ui`, `stale_revision`, `invalid_path`, `unsupported_format`, `unsupported_storage`, `invalid_document`, `size_limit`, `external_conflict`, `write_failed`, `drain_failed`, `window_failed`, `dialog_failed`, `preferences_invalid`, `preferences_write_failed`, `core_unavailable` (état interne inaccessible, distinct d'une opération occupée), `history_unavailable` (aucune mutation disponible dans la zone).

Une annulation utilisateur est un résultat `cancelled`, pas une erreur et pas une réussite. Les codes sont associés à des messages français contrôlés et à des possibilités concrètes de reprise. Les journaux n'incluent ni source, ni contexte, ni instructions, ni secrets.

Migration :

- Les variantes New/Open/SaveCopy/Save de `prototype_action` ne doivent plus contourner les dialogues, l'attente ou les confirmations. Les retirer de l'IPC publique ou les faire rejoindre le coordinateur ; les méthodes de bas niveau restent des détails internes testables.
- `prototype_finish_close` ne peut plus détruire une fenêtre à partir d'une simple demande du renderer. Le retirer de l'exposition ou le faire déléguer au même protocole sécurisé.
- Les changements de mode et de champs restent soumis aux gardes d'opération dans le domaine, pas seulement à des boutons désactivés.
- Chaque nouvelle commande est enregistrée dans `generate_handler!`, `ipc_commands.rs` et les permissions Tauri. Une capability `preferences` distincte autorise uniquement les commandes nécessaires ; aucune permission générique de navigation distante, fichiers, shell ou création de fenêtres n'est ajoutée.
- Les DTO et capabilities applicatifs restent identiques sur les deux OS ; les handles, objets GTK et choix de backend restent internes à Rust. Les erreurs de stockage ne sont jamais converties en adoption automatique d'une copie, même lorsque l'ancien adaptateur Linux produit `SaveOutcome::Copy`.

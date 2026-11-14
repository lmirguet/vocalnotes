# Research: Menus et cycle de document Windows et Linux

Date : 2026-11-13. Recherche et décisions de conception, sans implémentation. Références : Windows 11 / WebView2 et Fedora / GNOME / Wayland / WebKitGTK. Les anciennes recettes GNOME du dépôt attestent seulement le socle historique, pas cette fonctionnalité.

## 1. Socle conservé et écarts constatés

**Decision** : conserver Tauri 2, le domaine Rust comme autorité du document, le frontend TypeScript et l'éditeur ProseMirror. Modifier le cycle de document, pas les fournisseurs vocaux.

**Rationale** :

- `src-tauri/src/domain/note.rs` détient le texte, le contexte, les révisions, la destination et le stockage. `publish`, `set_context` et certains changements d'intention déclenchent actuellement `save_current` : cette autosauvegarde doit disparaître pour rendre les décisions Enregistrer / Ne pas enregistrer effectives.
- `src-tauri/src/app/close.rs` invalide actuellement les traitements avant de fermer et sauvegarde sans choix d'abandon. Il ne satisfait pas l'attente annulable confirmée lors de la clarification.
- `src-tauri/src/storage/markdown_file.rs` refuse les extensions autres que `.md`. Hors Linux, `replace_allowed` renvoie false : `SaveOutcome::Copy` peut changer la destination après une demande d'enregistrement au chemin courant. Ce résultat ne pourra plus être interprété comme un Enregistrer réussi.
- `src/ui/MenuBar.ts` construit des panneaux de divulgation ; `DesktopShell.ts` y place un champ de chemin. Les préférences sont une section de la page, et l'historique manuel du contenu est réinitialisé lors de certains rafraîchissements.
- Les trois contrôles de chemin de `commands.rs`, `markdown_file.rs` et `config/preferences.rs` doivent adopter la même politique `.von` / `.md`, sans relâcher les contrôles de chemins locaux.

**Alternatives considered** : changer de framework ou remplacer tout l'éditeur ; rejeté car les défauts sont localisés au cycle de document et à son intégration desktop.

## 2. Menus réellement natifs sur les deux systèmes

**Decision** : construire le menu Rust avec `Menu`, `Submenu` et des `MenuItem` identifiés ; un seul répartiteur traite menu, raccourcis et fermeture. Installer le menu par fenêtre sur `main`, pas sur les préférences, sous Windows comme sous Linux. Supprimer la barre HTML dans l'application Tauri.

**Rationale** : Tauri s'appuie sur le menu Windows et sur GTK3 via muda sous Linux, et non sur une imitation HTML ou un menu global GNOME. Les commandes et raccourcis restent ceux de la spécification. `PredefinedMenuItem::undo` et `redo` sont documentés non pris en charge sur Windows et Linux : utiliser des items ordinaires reliés à notre historique.

Le répartiteur conserve la dernière zone d'édition active avant l'entrée dans le menu. Les accélérateurs natifs sont la seule source des raccourcis applicatifs en production ; ne pas leur superposer un second handler DOM qui exécute la même commande. Les tests de montage utilisent un adaptateur injecté, pas une deuxième implémentation de menus. Les raccourcis propres aux champs de préférences restent locaux à cette fenêtre, sans atteindre l'historique de la note.

Les libellés Tauri utilisent `&` pour les mnémoniques, que muda traduit pour GTK ; le texte visible ne change pas. Vérifier F10, Alt et les accès par lettre lorsque WebKitGTK détient le focus : la documentation Tauri ne certifie pas ce parcours complet. Une correction éventuelle reste dans l'adaptateur natif GTK, sans second menu ni déclencheur DOM concurrent. Les callbacks de menu ne prouvent pas à eux seuls la fenêtre d'origine : filtrer les identifiants et vérifier aussi fenêtre active, dialogue modal et état du coordinateur.

**Alternatives considered** : réimplémenter un menu ARIA complet, exiger un menu global GNOME ou utiliser les items Undo/Redo prédéfinis ; ces options multiplient les écarts ou ne couvrent pas les deux cibles.

**Sources** :

- https://v2.tauri.app/learn/window-menu/
- https://docs.rs/tauri/latest/tauri/menu/struct.MenuItemBuilder.html
- https://docs.rs/tauri/latest/tauri/menu/struct.PredefinedMenuItem.html
- https://docs.rs/tauri/latest/tauri/struct.AppHandle.html#method.on_menu_event
- https://raw.githubusercontent.com/tauri-apps/muda/dev/src/platform_impl/gtk/mod.rs

## 3. Fichiers et confirmations système

**Decision corrigée après inspection des dépendances verrouillées** : utiliser le plugin Rust officiel `tauri-plugin-dialog = "=2.8.4"` pour les dialogues Windows, avec sélecteurs et `YesNoCancelCustom`. Sous Linux, créer les sélecteurs `gtk::FileChooserDialog` et confirmations GTK3 directement sur le thread GTK, avec la vraie fenêtre `main` comme parent, `transient_for`, modalité et destruction avec le propriétaire. Adapter leurs signaux en attente asynchrone sans `Dialog::run`. La dépendance GTK `0.18.2` est déjà disponible ; aucun nouveau framework ni portail requis.

**Rationale** : les dialogues utilisent Windows sur Windows et GTK sur le bureau Linux ; le renderer ne reçoit pas de permission filesystem générique. L'inspection de `rfd 0.17.2` effectivement verrouillé montre que les constructeurs GTK de message et de sélecteur passent un parent nul et n'utilisent pas le parent fourni au plugin. Cette branche ne garantit donc pas le parentage Wayland requis. Un simple `set_parent` du plugin ne suffit pas ; ne pas assimiler GTK modal sans parent à une vraie modalité propriétaire. L'adaptateur GTK direct doit transporter seulement chemins et décisions dans le canal de réponse, jamais un objet GTK entre threads. Contrôler aussi les features Cargo pour qu'une autre dépendance n'active pas involontairement `xdg-portal`.

Les appels par callback sont adaptés en attente asynchrone ; aucune attente bloquante sur le thread UI ni garde `Mutex` tenue pendant un dialogue. Mapper explicitement les réponses typées et les variantes `Custom` correspondant aux trois libellés. La croix, Echap et la réponse Cancel signifient Annuler ; une réponse inconnue abandonne l'opération avec diagnostic, jamais avec autorisation d'écrire. Un résultat de sélecteur qui ne peut pas être converti en chemin local est une erreur visible, pas une annulation supposée.

Les extensions sont comparées sans tenir compte de la casse. Ajouter `.von` ou `.md` seulement si aucune extension n'a été saisie. Une extension incompatible avec l'action est expliquée et refusée, jamais changée silencieusement. Ouvrir accepte `.von` et `.md`. Enregistrer conserve `.md` sans contexte défini ; un contexte différent de celui chargé impose Enregistrer sous en `.von`. Les nouvelles notes et Enregistrer sous proposent `.von`.

**Alternatives considered** : champ de chemin HTML ; plugin filesystem utilisable librement depuis JavaScript ; dialogue binaire incapable d'exprimer Annuler. Ces alternatives ne satisfont pas le comportement ou ouvrent inutilement les accès.

**Sources** :

- Inspection locale du paquet verrouillé `rfd 0.17.2` : `src\backend\gtk3\message_dialog.rs:85–88` et `src\backend\gtk3\file_dialog\dialog_ffi.rs:28–35` (parent nul dans les deux constructeurs). L'ancien argument fondé sur la branche publique courante ne s'applique pas à cette version.
- https://v2.tauri.app/plugin/dialog/
- https://docs.rs/tauri-plugin-dialog/latest/tauri_plugin_dialog/struct.FileDialogBuilder.html
- https://docs.rs/tauri-plugin-dialog/latest/tauri_plugin_dialog/struct.MessageDialogBuilder.html
- https://docs.rs/tauri-plugin-dialog/latest/tauri_plugin_dialog/enum.MessageDialogButtons.html
- https://docs.rs/tauri-plugin-dialog/latest/tauri_plugin_dialog/enum.MessageDialogResult.html
- https://raw.githubusercontent.com/tauri-apps/plugins-workspace/v2/plugins/dialog/Cargo.toml
- https://raw.githubusercontent.com/tauri-apps/plugins-workspace/v2/plugins/dialog/src/desktop.rs
- https://raw.githubusercontent.com/PolyMeilex/rfd/master/src/backend/gtk3/utils.rs
- https://raw.githubusercontent.com/PolyMeilex/rfd/master/src/backend/gtk3/message_dialog.rs

## 4. .von reste du Markdown local

**Decision** : conserver le codec existant de `storage/note_document.rs` : un contenu Markdown précédé, si nécessaire, du commentaire HTML versionné `vocalnotes:intention:v1`. `.von` est une extension de travail, pas un format binaire, une archive ou un chiffrement propriétaire.

**Rationale** : le contexte est déjà isolé dans une enveloppe textuelle lisible. Les rendus Markdown ordinaires masquent le commentaire et affichent la note ; un éditeur texte donne accès aux deux champs sans Vocalnotes. Les champs historiques `purpose`, `form` et `tone` restent préservés pour ne pas dégrader les fichiers existants. Aucun stockage du contexte dans un fichier compagnon n'est nécessaire.

Réutiliser les règles d'échappement de `note_document::encode` pour empêcher `-->` de fermer le commentaire. Une version inconnue ou des métadonnées corrompues font échouer Ouvrir sans remplacer la note en mémoire. Un fichier sans enveloppe représente un contenu avec intention vide, même en `.von`.

L'export n'utilise jamais ce codec : il écrit les octets UTF-8 de `source` seulement. Il ne supprime pas un commentaire ou une chaîne du corps parce qu'ils ressemblent au contexte : il exclut le champ, pas les occurrences de texte.

**Alternatives considered** : JSON autonome, ZIP, métadonnées Markdown visibles dans le corps, fichiers compagnons. Rejetés pour incompatibilité avec la propriété Markdown, duplication de formats ou risque de désynchronisation.

**Constitution** : le principe III est respecté par un `.von` qui demeure un fichier Markdown exploitable hors application. L'extension seule n'impose pas de révision constitutionnelle. Le contexte attaché à la note est distinct du contexte temporaire de session cité au principe IV ; aucun audio, transcription de session ou secret n'est ajouté au fichier.

**Sources locales** : `src-tauri/src/storage/note_document.rs`, `src-tauri/src/domain/intention.rs`, `.specify/memory/constitution.md`.

## 5. Écriture contrôlée et conflit externe sur les deux systèmes

**Decision** : étendre le stockage contrôlé existant avec une branche Windows dédiée : préparation exclusive dans le même dossier, synchronisation du fichier, vérification de version et d'identité, puis remplacement `ReplaceFileW` avec sauvegarde de la version déplacée et contrôles après remplacement. Pour une destination absente, installer sans autoriser l'écrasement d'un fichier apparu entre-temps. Conserver la branche Linux existante.

**Rationale** : un simple `std::fs::rename` documente le remplacement, mais pas les garanties de conflit, de préservation des ACL et de récupération déjà exigées par le dépôt. `ReplaceFileW` permet de conserver la version remplacée et ses attributs ; ses échecs partiels doivent être traités explicitement. Les options qui ignorent les erreurs ACL sont exclues. `REPLACEFILE_WRITE_THROUGH` est documenté non pris en charge : ne pas s'en servir comme garantie de durabilité.

L'adaptateur Windows doit remplacer l'identité factice `(0, 0)` par une identité issue du handle fichier, contrôler liens physiques et points de réanalyse, et vérifier que le dossier/destinataire observé est encore le bon. Le fichier préparé et sa sauvegarde ne sont supprimés qu'après vérification ; en cas d'incertitude, conserver les versions utiles et signaler leurs emplacements dans l'interface, pas leur contenu dans les logs.

**Résultat d'implémentation NTFS** : l'identité utilise le numéro de volume et les 128 bits de `FILE_ID_INFO`. La version inclut aussi liens, attributs et empreinte du descripteur propriétaire/groupe/DACL. Le remplacement refuse avant écriture une DACL effectivement protégée (`SE_DACL_PROTECTED`) ou un descripteur différent de celui de la préparation. Il ne tente ni de normaliser les ACL, ni de réparer les permissions après remplacement. Les fichiers ordinaires héritant des mêmes permissions dans le même dossier sont remplaçables ; les permissions particulières non reproductibles nécessitent Enregistrer sous. Le test de DACL protégée relit le contrôle après sa configuration, sans déduire la protection du seul succès d'un setter.

Le répartiteur de stockage commun conserve le chemin réel de la version déplacée : la préparation initiale sous Linux, une sauvegarde distincte avec `ReplaceFileW` sous Windows. Il vérifie l'identité et les métadonnées de la préparation ainsi que ses octets contre le candidat immuable, puis contrôle les versions installée/déplacée avant adoption. Une interruption après remplacement conserve aussi une copie explicite du candidat si une écriture externe a déjà changé la destination. Le nettoyage Windows vérifie et supprime la sauvegarde par handle.

Un conflit détecté avant écriture demande une nouvelle destination, une annulation ou une confirmation de remplacement portant sur la version externe observée. La confirmation ne neutralise pas les contrôles : une nouvelle course retourne un conflit. Après un remplacement dont les vérifications détectent une course, garder les versions, ne jamais restaurer aveuglément par-dessus un autre écrivain.

Une copie de secours n'est ni une nouvelle destination adoptée automatiquement ni une sauvegarde réussie du fichier demandé. Aucune promesse de durabilité absolue lors d'une coupure matérielle ; les garanties portent sur les résultats observables, les versions récupérables et l'absence de réussite trompeuse.

**Linux** : réutiliser la traversée par descripteurs de répertoire avec `openat`/`O_NOFOLLOW`, l'identité `(dev, ino)`, les contrôles de volume local et de liens, les empreintes et `renameat2(RENAME_EXCHANGE)`. Étendre les mêmes validateurs à `.von` et `.md`, le consentement de remplacement à une version observée et l'installation d'une préparation absente par `RENAME_NOREPLACE`. Synchroniser fichier préparé et répertoire, puis contrôler version déplacée, version installée et identité du répertoire avant de conclure.

Les refus existants de remplacement (liens physiques, ACL particulières, propriétaire différent, bits spéciaux ou primitive indisponible) ne doivent plus être transformés par le domaine en adoption réussie de `SaveOutcome::Copy`. Renvoyer un refus explicite avec versions récupérables et proposer Enregistrer sous à l'utilisateur. Conserver les protections existantes plutôt que les contourner par troncature ou suppression/renommage. La recette Fedora utilise un dossier ordinaire sur Btrfs ou ext4 ; un volume non pris en charge est refusé avec explication, pas présenté comme un Linux fonctionnellement dépourvu de sauvegarde.

Comparer les extensions sans casse, mais jamais tous les chemins Linux : deux noms différant seulement par la casse peuvent être deux fichiers différents. L'export est protégé par l'identité native du fichier courant, pas par une comparaison globale de chaînes en minuscules. Les identités de fichiers et les chemins sont locaux à chaque ouverture et ne voyagent pas dans le `.von`.

**Alternatives considered** : écriture directe/troncature ; effacement de la cible suivi d'un renommage ; autosauvegarde dans une copie choisie automatiquement ; réécriture complète du stockage Linux. Toutes sont écartées.

**Sources** :

- https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-replacefilew
- https://doc.rust-lang.org/std/fs/fn.rename.html
- `src-tauri/src/storage/markdown_file.rs`
- `src-tauri/tests/storage_integrity.rs`

## 6. Cycle de document et arrêt gracieux

**Decision** : un coordinateur Rust de document possède une opération active identifiée et un protocole de préparation avec l'UI. Retirer les autosauvegardes des chemins d'édition. Calculer `dirty` par comparaison de l'état persistant à la dernière sauvegarde réussie.

**Rationale** : les menus natifs et la croix arrivent hors de la file de propositions du renderer. Le coordinateur doit d'abord demander le gel des nouvelles commandes/éditions et l'acquittement des saisies existantes. Pour Nouvelle note, Ouvrir et Quitter, il arrête ensuite la capture sans invalider prématurément les traitements déjà attendus, attend la fin de la transcription et de la reformulation déjà engagées, traite leurs résultats, puis évalue les modifications et ouvre la confirmation éventuelle. Enregistrer, Enregistrer sous et Exporter capturent l'état courant accepté sans arrêter l'écoute ni attendre de résultat réseau ; un résultat reçu plus tard peut laisser la note modifiée.

Extraire de `Controller::stop_active_mode` le drainage utile, mais différer l'invalidation de session, le changement de cible et l'effacement des instructions jusqu'à la fin des traitements attendus. L'`AtomicBool` actuel de reformulation doit être complété par un signal de terminaison portant succès ou échec ; pas de boucle occupée ni nouvelle requête LLM.

L'attente est visible et annulable. Une annulation invalide l'opération et ses tickets restants, conserve les résultats déjà reçus, et retourne à une note ouverte avec micro arrêté. Un échec abandonne aussi la fermeture/remplacement. Les protections existantes `noteId`, révision, génération et intention d'entrée continuent de refuser les résultats devenus périmés.

Pour Ouvrir, la note candidate est chargée/validée avant sa publication et le refus Ne pas enregistrer ne vide jamais le document courant par anticipation. Une sauvegarde antérieure explicitement validée reste acquise si le chargement suivant échoue. Pour fermer, utiliser `destroy` seulement au terme de l'opération validée, hors de toute garde du cœur.

**Alternatives considered** : appeler l'ancien `stop(false)` dès la demande ; invalider les résultats pour gagner du temps ; confirmation uniquement dans le renderer ; sauvegarde implicite à la fermeture. Toutes contredisent les clarifications ou rendent les files concurrentes fragiles.

## 7. Historique par zone, canonique et non persistant

**Decision** : gérer dans le domaine Rust un historique par zone Contenu / Contexte / Instructions, alimenté par les changements acceptés, quelle que soit leur origine. Réutiliser les préconditions de transaction et la représentation exacte de source ; retirer le second historique autonome du contenu dans `DesktopShell.ts`.

**Rationale** : le tableau manuel de l'UI ne couvre pas la dictée, la reformulation ni les champs texte. Une autorité unique permet de calculer la disponibilité des menus et `dirty` après annulation. Une reformulation constitue une entrée ; les lots de dictée sont groupés par passage finalisé. L'historique d'Instructions est effacé avec les instructions à la sortie du mode.

Les entrées contiennent des deltas avant/après et les préconditions de la zone. La révision globale augmente même quand Annuler retrouve un ancien texte : ne jamais remettre les compteurs en arrière. Limiter l'historique à 256 entrées et 16 Mio de deltas par document ; retirer les entrées les plus anciennes avec indication de limite atteinte, sans toucher au texte courant. Conserver des positions de sélection dans l'adaptateur UI pour rendre le focus, sans en faire l'autorité du contenu.

**Alternatives considered** : ajouter seulement `prosemirror-history` ou s'appuyer sur l'historique natif des `textarea`. Ces solutions divergent des changements appliqués par Rust et des zones mises à jour par dictée. La dépendance ProseMirror déjà présente n'oblige pas à maintenir deux historiques.

## 8. Préférences : fenêtre possédée et réellement modale sur chaque bureau

**Decision** : créer une seule `WebviewWindow` nommée `preferences`, à partir de la page locale existante dans un mode de rendu dédié, avec cadre système, propriétaire `main`, sans menu documentaire. Réutiliser la validation des modèles et les commandes de trousseau existantes. Un même petit adaptateur de modalité expose deux branches natives, sans nouvelle infrastructure de fenêtres.

**Rationale** : `WebviewWindowBuilder::parent` établit un propriétaire sur Windows, pas une modalité complète documentée. Ne pas supposer l'existence de `set_enabled`. Un petit adaptateur Windows utilise `EnableWindow` sur le thread de fenêtre pour désactiver le propriétaire et le réactiver avant destruction du dialogue ; vérifier l'état effectif plutôt que d'interpréter le retour de `EnableWindow` comme un code de succès, car il décrit l'état précédent.

Le dialogue est construit depuis un contexte asynchrone pour éviter le blocage Windows documenté des créations de webview synchrones. En cas d'échec de création, réactiver `main`. OK attend la persistance des modèles puis ferme ; Annuler, Echap et la croix abandonnent le brouillon. Toutes les voies de destruction, y compris inattendues, libèrent la modalité et rétablissent le focus.

**Linux** : `WebviewWindowBuilder::parent` rend la fenêtre transitoire, mais ne la rend pas modale. Après construction cachée, récupérer les `gtk::ApplicationWindow` de Tauri par `gtk_window()` et appliquer `GtkWindowExt::set_transient_for`, `set_modal(true)` et `set_destroy_with_parent(true)` avant affichage. Exécuter ces appels et le nettoyage sur le thread GTK via `run_on_main_thread`, sans transférer d'objets GTK entre threads ni attendre leur résultat de manière bloquante sur ce thread. Ajouter directement `gtk = "=0.18.2"` avec `v3_24` dans les seules dépendances Linux : c'est la version déjà présente dans Cargo.lock pour Tauri, pas un deuxième toolkit.

Le garde natif retire la modalité lors de la fermeture et restaure le focus de l'éditeur de `main`. La fin du propriétaire ferme explicitement le dialogue ; `destroy_with_parent` est une protection complémentaire, pas le seul nettoyage de l'état Tauri. Une demande documentaire pendant la modalité est refusée côté Rust et ramène au dialogue, même si un événement de menu global parvient au handler. Un échec de préparation détruit la seule fenêtre incomplète, libère le garde et affiche une erreur dans `main` ; ne jamais laisser une fenêtre principale désactivée.

Le cadre et la relation transitoire sont natifs ; le formulaire web réutilisé suit le thème clair/sombre, la typographie, les contrôles et les indicateurs de focus familiers du système. Aucun faux cadre Windows sous GNOME. Le choix n'affirme pas que chaque contrôle HTML est un widget GTK : l'apparence observable est à qualifier par la recette et SC-008. Ni position absolue, ni forçage global de focus, ni solution X11 ne sont imposés à Wayland ; la restitution lors de la fermeture utilisateur doit fonctionner dans le compositeur de référence et reste un critère bloquant de livraison.

Le frontend distingue les fenêtres via leur label réel ; une query string ne confère aucun droit. Les capabilities de `preferences` n'autorisent ni mutation de note, ni accès générique aux fichiers. Les actions existantes de trousseau demeurent des actions explicitement validées et identifiées comme immédiates, distinctes du brouillon de modèles annulable ; aucune valeur de clé n'est lue en retour ni persistée dans les préférences.

Les secrets restent dans Windows Credential Manager ou Secret Service sous Linux selon l'adaptateur `keyring` déjà sélectionné. Une erreur de trousseau reste visible ; pas de stockage de remplacement dans un fichier de préférences.

A propos peut utiliser un message système avec le nom et la version fournis par Tauri, sans troisième webview.

**Alternatives considered** : panneau dans la page, fenêtre simplement always-on-top, désactivation CSS du seul éditeur, usage d'`EnableWindow` sur Linux ou ajout de GTK4/libadwaita. Ces choix ne satisfont pas la modalité des deux cibles ou introduisent un second toolkit inutile.

**Sources** :

- https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html
- https://docs.rs/tauri/latest/tauri/struct.CloseRequestApi.html
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enablewindow
- https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindow.html#method.gtk_window
- https://docs.rs/gtk/0.18.2/gtk/prelude/trait.GtkWindowExt.html
- https://docs.gtk.org/gtk3/method.Window.set_modal.html
- https://docs.gtk.org/gtk3/method.Window.set_transient_for.html
- https://docs.gtk.org/gtk3/method.Window.set_destroy_with_parent.html
- `src-tauri/Cargo.lock` (`gtk 0.18.2`, `muda 0.20.0`, `webkit2gtk 2.0.2`).

## 9. Interopérabilité et preuve multiplateforme

**Decision** : garder un unique codec et un unique protocole documentaire ; exécuter les mêmes matrices fonctionnelles sur les deux plateformes et une matrice d'échanges de fichiers dans les deux sens.

**Rationale** : un changement de bureau n'est pas un changement de format. UTF-8, les chaînes d'intention, les espaces et les fins de ligne du corps ne sont pas normalisés. L'enveloppe conserve ses délimiteurs LF et le corps conserve ses propres LF/CRLF ; aucun chemin, identifiant disque, réglage ou marqueur de plateforme n'est sérialisé. L'autre système reconstruit sa propre identité disque à l'ouverture du fichier transféré.

SC-001 à SC-010 s'évaluent séparément sous Windows 11 et Fedora/GNOME/Wayland. SC-011 ajoute 20 échanges par sens : comparer source et intention exactes, relire les exports bruts et vérifier que réenregistrer cible le fichier local choisi. Les historiques d'anciennes recettes Linux, les doubles de fenêtre, WSLg et une compilation croisée ne valident pas la nouvelle modalité, les menus ni le stockage natif.

**Alternatives considered** : deux codecs, normalisation automatique des fins de ligne ou validation d'un seul OS extrapolée à l'autre. Rejetées car elles introduiraient des différences de données ou de fausses preuves.

## Résultat de recherche

Les choix techniques sont arrêtés pour les deux systèmes ; aucune décision fonctionnelle supplémentaire n'est requise. Les API ont été vérifiées dans la documentation publique disponible et les dépendances existantes, dont certaines sources suivent la dernière version : compiler et contrôler l'arbre de features des versions verrouillées reste une obligation de l'implémentation, pas une validation déjà réalisée. Les points dépendant de WebView2, WebKitGTK, du système de fichiers, des menus et du focus Wayland ont des scénarios de recette explicites dans le quickstart. Aucune dépendance n'a été installée et aucune recette native n'a été exécutée pendant cette recherche.

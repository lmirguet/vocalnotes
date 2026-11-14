# Implementation Plan: Menus et cycle de document Windows et Linux

**Branch**: `lmirguet-capgemini-menus-windows-natifs` | **Date**: 2026-11-13 | **Spec**: [spec.md](spec.md)

**Input**: Spécification active `specs/005-menus-windows-natifs/spec.md`, dont les cinq clarifications du 2026-11-13 et FR-001 à FR-020.

**Status**: Conception révisée pour Windows et Linux, phases 0 et 1 terminées. Les artefacts associés couvrent FR-019, FR-020 et SC-011. Il s'agit d'un plan à implémenter, pas d'une validation d'exécution sur les deux systèmes.

## Summary

Remplacer les panneaux de menu HTML par une barre Tauri native Windows ou GTK3 sous Linux. Unifier Nouvelle note, Ouvrir, Enregistrer, Enregistrer sous, Exporter et Quitter avec la croix de fenêtre autour d'un coordinateur Rust qui attend les saisies et traitements en cours, protège les modifications et distingue sauvegarde complète et export. La logique métier et le protocole de commandes restent communs.

Le format `.von` réutilise le Markdown et l'enveloppe textuelle de contexte existants ; un `.md` sans contexte défini conserve son chemin. L'export `.md` écrit seulement le contenu. Les fichiers voyagent entre les deux systèmes sans conversion ni normalisation du corps. Supprimer l'autosauvegarde implicite ; fiabiliser le remplacement Windows et réutiliser l'échange Linux, sans adoption automatique d'une copie sur aucun système. Étendre l'historique aux changements acceptés des trois zones. Déplacer les préférences dans une fenêtre possédée réellement modale, à cadre et apparence du système, avec OK/Annuler ; modalité Win32 sous Windows et GTK sous GNOME/Wayland.

## Technical Context

**Language/Version**: Rust edition 2024, toolchain `1.99.1` ; TypeScript `5.9.3` ; Node.js `>=22.23.2 <23`.

**Primary Dependencies**: Tauri Rust `2.12.5`, API JS `2.12.4`, Tokio `1.54.1`, Serde/serde_json, SHA-256 ; ProseMirror existant ; Vite `7.3.7`. Ajout prévu du plugin Rust officiel `tauri-plugin-dialog` compatible Tauri 2 (version étudiée `2.8.4`, à verrouiller), backend `gtk3` explicite sans `xdg-portal`, mais dialogues Linux à créer directement en GTK3 pour garantir le parent réel : la branche GTK de `rfd 0.17.2` verrouillé ignore le parent fourni au plugin. Dépendance directe Linux `gtk = "=0.18.2"` avec `v3_24`, déjà transitive dans le verrou ; conserver `libc` et `dbus`. Étendre seulement les features Windows de `windows-sys 0.61.2` nécessaires aux fichiers. Pas de plugin filesystem frontend, de GTK4/libadwaita ni de nouveau framework.

**Storage**: fichiers Markdown UTF-8 locaux `.von` ou `.md`, enveloppe existante `vocalnotes:intention:v1` ; préférences non sensibles dans leur fichier existant ; clés dans Windows Credential Manager ou Secret Service Linux via `keyring` existant. Limites existantes : 1 Mio par fichier sérialisé, 4096 octets cumulés d'intention, 16 384 octets d'instructions temporaires. Identités et garanties de remplacement natives à chaque système ; codec commun.

**Testing**: Vitest `4.1.12` + jsdom `30.1.3`, tests Rust existants et doubles de dialogues/périphériques sur les deux hôtes ; tests de stockage natifs sur Windows et Linux, puis recettes Tauri/WebView2 et Tauri/WebKitGTK pour menus, focus et modalité. Matrice de 20 échanges de fichiers par sens. Aucun service externe requis pour les tests automatisés.

**Target Platform**: Windows 11 / WebView2 / NTFS et Fedora / GNOME / Wayland / GTK3 / WebKitGTK, sur dossier local ordinaire Btrfs ou ext4 pour la recette Linux. Le projet fournit déjà ce socle Linux. macOS, autres distributions, KDE et X11 ne sont pas ajoutés au périmètre garanti. WSLg, XWayland ou une compilation croisée ne remplacent pas la recette GNOME/Wayland.

**Project Type**: application desktop Tauri, renderer local, domaine Rust déjà canonique.

**Performance Goals**: réponse visible des menus/préférences en moins de 300 ms dans au moins 95 % des 30 essais prescrits sur chaque hôte de référence. Les opérations réseau en cours ont une attente distincte visible et annulable ; aucune cible de latence fournisseur n'est inventée.

**Constraints**: un document courant par fenêtre ; une opération documentaire active ; aucune perte silencieuse ; pas de nouvelle requête LLM implicite ; pas de secrets/contenus dans les logs ; aucune capacité générique filesystem/shell ; pas d'attente de dialogue, disque ou réseau sous verrou du cœur. Historique borné à 256 entrées et 16 Mio de deltas en mémoire.

**Scale/Scope**: 3 menus, 10 commandes, 7 raccourcis ; 3 zones éditables ; 1 fenêtre principale et au plus 1 fenêtre Préférences. Pas d'onglets, de fichiers récents, d'association de fichiers système, de récupération automatique ou de nouveaux réglages.

## Constitution Check

*GATE avant Phase 0 : PASS avec les contraintes de conception ci-dessous. Relecture après Phase 1 : PASS ; aucune dérogation constitutionnelle.*

| Principe | Contrôle avant recherche | Contrôle après conception |
|----------|--------------------------|----------------------------|
| I. Fidélité | Ne pas transformer le contenu lors des opérations de fichier. | Le codec conserve les octets de source ; l'export ne reformule rien ; Annuler restaure une version acceptée. |
| II. Dictée et contrôle explicite | Garder les destinations et l'action LLM explicite. | Drainage des traitements déjà engagés avant la décision de fermeture ; aucune nouvelle requête déclenchée ; tickets périmés refusés ; Annuler ne redémarre pas le micro. |
| III. Propriété et intégrité | `.von` doit rester du Markdown local exploitable hors Vocalnotes sur les deux systèmes. | L'enveloppe est un commentaire HTML versionné existant, non un conteneur propriétaire ; sauvegarde contrôlée par OS, conflit explicite et export brut ; `.md` sans contexte inchangé ; échanges sans altération de source/intention. |
| IV. Confidentialité | Conserver seulement la note et son intention attachée ; aucun audio, historique de transcription ni secret dans les fichiers de note. | Instructions et historiques uniquement en mémoire ; capabilities par fenêtre ; trousseaux système existants ; aucune télémétrie ou exposition réseau ajoutée. Le contexte persistant de note n'est pas le contexte temporaire de session. |
| V. Simplicité | Réutiliser domaine, codec, validation et stockage ; pas de nouvelle architecture générale. | Un coordinateur documentaire, un adaptateur de menu et de dialogue/fenêtre avec branches Win32/GTK ; stockage Linux réutilisé et petite branche Windows, sans duplication des fournisseurs ni du renderer complet. |

La réserve de gouvernance signalée dans la spécification est résolue par le choix d'un `.von` qui demeure du Markdown avec commentaire HTML. Aucune édition de la constitution n'est nécessaire. L'écart entre « contexte de note » et « contexte de session » reste explicitement respecté.

## Project Structure

### Documentation (this feature)

```text
specs\005-menus-windows-natifs\
  spec.md
  plan.md
  research.md
  data-model.md
  quickstart.md
  checklists\requirements.md
  contracts\
    desktop-commands.md
    note-format.md
```

`tasks.md` sera créé par `/speckit-tasks`, pas par cette phase.

### Source Code (repository root)

Chemins existants sauf mention « nouveau » ; le découpage final reste limité à ces responsabilités.

```text
src\
  main.ts
  style.css
  ui\
    DesktopShell.ts
    NoteView.ts
    ModelPreferencesView.ts
    MenuBar.ts                  retrait de la barre HTML en production
    contracts.ts
    DocumentCommands.ts         nouveau : bridge natif, file UI, focus
src-tauri\
  Cargo.toml
  Cargo.lock
  build.rs
  capabilities\
    main.json
    preferences.json           nouveau : permissions minimales du dialogue
  src\
    lib.rs
    app\
      commands.rs
      close.rs
      ipc_commands.rs
      document.rs              nouveau : coordination de document
    domain\
      note.rs
      history.rs               nouveau : historique par zone
    platform\
      controller.rs
      desktop.rs               nouveau : menus et dialogues Tauri ; modalité Win32/GTK
    storage\
      markdown_file.rs
      note_document.rs
      windows_file.rs           nouveau : primitives Win32 contrôlées
    config\preferences.rs
  tests\
    close_workflow.rs
    storage_integrity.rs
    drafting.rs
    prototype_envelope.rs
    prototype_ordered_save.rs
    ipc_boundary.rs
    document_workflow.rs        nouveau : matrice de commandes
    document_history.rs         nouveau : sources et zones d'édition
```

**Structure Decision** : prolonger les couches présentes ; isoler les appels Tauri/Win32/GTK dans `platform` et les primitives de fichiers dans `storage`, conserver les transitions de texte dans `domain`, et rendre le coordinateur testable avec des doubles Rust sans fenêtre réelle. Les appels GTK restent sous compilation conditionnelle Linux, sans fuite de type GTK dans le domaine. Le renderer continue d'utiliser propositions/acquittements ; il ne devient pas propriétaire de la sauvegarde.

## Phase 0 — Recherche et décisions

Résultats détaillés dans [research.md](research.md) :

1. Menus Rust natifs Windows/GTK et accélérateurs centralisés ; Undo/Redo applicatifs car les items prédéfinis ne sont pris en charge sur aucune des deux cibles.
2. Dialogues système côté Rust, backend GTK explicite sous Linux, résultats à trois issues, aucun chemin libre fourni par la webview.
3. Réutilisation du codec Markdown textuel existant pour `.von` et lecture des anciens `.md` ; export sans codec.
4. Remplacement Windows contrôlé et échange Linux existant, avec version externe, sauvegarde récupérable et vérifications ; aucun `Copy` automatiquement adopté sur les deux systèmes.
5. Attente coordonnée des propositions UI, de la capture et de la reformulation ; état modifié comparé à la sauvegarde.
6. Historique natif par zone, alimenté par toutes les origines de modifications.
7. Préférences dans une fenêtre possédée ; vraie modalité via Win32 ou GTK3 sur le thread natif, pas une API Tauri supposée.
8. Un seul format et des preuves séparées pour Windows, Fedora/GNOME/Wayland et les échanges entre systèmes.

Toutes les inconnues de choix technique sont résolues. Les tests de compilation des API épinglées et les comportements natifs font partie de l'implémentation ; cette phase n'annonce pas leur exécution.

## Phase 1 — Conception

### Flux de document

- Une intention native crée une opération identifiée et notifie `main`. Le bridge gèle les nouvelles éditions, termine les propositions en file et les champs différés, et acquitte la préparation avec les révisions attendues. Un refus de proposition garde le texte récupérable et interrompt l'action destructive.
- Pour Nouvelle note, Ouvrir et Quitter, le coordinateur arrête la capture sans invalider les résultats attendus, attend leurs signaux de terminaison puis évalue `dirty`. Une annulation ou un échec abandonne l'opération et laisse la note ouverte, micro arrêté. Enregistrer, Enregistrer sous et Exporter ne stoppent pas l'écoute et capturent seulement l'état accepté à écrire, sans attendre de réseau. Aucun verrou du cœur n'est gardé pendant une attente.
- Nouvelle note/Ouvrir/Quitter passent tous par la même décision Enregistrer / Ne pas enregistrer / Annuler. Ouvrir publie un candidat seulement après lecture/validation réussie. La croix ne concerne que `main` ; celle de `preferences` signifie Annuler le brouillon de préférences.
- Enregistrer capture une version de contenu/intention, écrit au chemin demandé et n'avance le point de sauvegarde qu'après vérification du résultat. Une édition acceptée après la capture d'écriture reste modifiée. Les changements éditoriaux ne déclenchent plus d'écriture disque.
- Enregistrer sous adopte le chemin seulement après succès. Exporter écrit `source` seul, contrôle l'identité de la destination pour ne pas remplacer le document de travail et n'adopte aucun chemin.

### Domaine, fichiers et historique

Le [modèle de données](data-model.md) définit l'état persistant de référence, l'identité disque, les transitions et les deltas d'historique. Le [contrat des fichiers](contracts/note-format.md) fixe le format `.von`, la compatibilité `.md`, les limites et l'export exact.

L'historique Rust remplace les piles manuelles autonomes du renderer. Chaque annulation est une nouvelle mutation acquittée avec révision monotone ; seule la zone ciblée est restaurée. Un changement de note vide tous les historiques, une sauvegarde ne les vide pas, et quitter Instruction vide son historique temporaire. Préserver la sélection et les blocs Markdown opaques lors des réconciliations UI.

### Fenêtres, commandes et permissions

Le [contrat desktop](contracts/desktop-commands.md) décrit les identifiants de menu, la préparation UI, les préconditions et les erreurs. Les anciennes commandes documentaires de `prototype_action` et `prototype_finish_close` doivent être migrées vers le coordinateur ou supprimées de la surface publique pour qu'aucune voie ne contourne la protection.

`main` reçoit les commandes de note et l'écoute d'événements ciblés. `preferences` reçoit seulement les commandes de modèles, de trousseau déjà existantes et de fermeture de son propre dialogue. Ni filesystem générique, ni création arbitraire de fenêtre, ni émission libre d'événement de commande. Les handlers vérifient aussi le label de fenêtre appelant. Mettre à jour ensemble `generate_handler!`, `ipc_commands.rs`, permissions générées et capabilities.

Construire les préférences cachées puis préparer la modalité avant affichage. Sous Windows, désactiver/réactiver le propriétaire par `EnableWindow`. Sous Linux, obtenir les fenêtres GTK sur le thread principal, poser relation transitoire et `set_modal(true)`, puis nettoyer et restituer le focus à la fermeture. `set_destroy_with_parent(true)` complète, sans remplacer, la fermeture explicite et le nettoyage Tauri. Les contrôles suivent le système et le thème ; pas de faux cadre Windows sous GNOME ni de positionnement/focus forcé dépendant de X11. Une création échouée ou destruction inattendue libère toujours le garde.

### Stockage et passage d'un système à l'autre

Le format ne sérialise aucun chemin ou identifiant disque. À chaque ouverture sur l'autre hôte, reconstruire une identité locale et conserver les octets de source, les chaînes d'intention et les fins de ligne. Comparer les suffixes sans casse, mais respecter la sensibilité à la casse du chemin du système ; protéger l'export par l'identité native.

Sous Linux, garder `openat`/`O_NOFOLLOW`, les contrôles de volume/permissions et `RENAME_EXCHANGE`, et ajouter l'installation sans écrasement `RENAME_NOREPLACE` pour une destination absente. Un refus de primitive ou de métadonnées particulières devient une erreur explicite avec possibilité d'Enregistrer sous, pas l'adoption d'une copie. Sous Windows, ajouter identité par handle et remplacement avec version déplacée récupérable. Sur les deux systèmes, aucune réussite logique avant les contrôles et aucune restauration aveugle sur un autre écrivain.

Conformément au comportement d'éditeur retenu après analyse, un conflit connu exige confirmation avant remplacement. Une course découverte seulement après installation est un échec avec versions récupérables et note laissée ouverte, jamais une sauvegarde annoncée comme réussie. La conservation d'une version déplacée ne constitue pas une garantie de prévention absolue de tout remplacement concurrent.

### Vérification prévue et documentation directement liée

Le [quickstart](quickstart.md) fournit les commandes et scénarios ; il distingue tests automatisés, stockage natif, recettes Windows et GNOME/Wayland réelles, et échanges de fichiers. Mettre à jour pendant l'implémentation les messages d'aide et notices qui annoncent encore l'autosauvegarde et les anciens tests qui l'attendent. Les rapports historiques tels que `docs/validation/storage-gnome.md` restent des preuves de leurs anciennes versions ; ne pas les réécrire en validations de cette fonctionnalité.

| Exigences | Vérification prévue |
|-----------|----------------------|
| FR-001 à FR-003 | Menu exact, disponibilité, accélérateurs uniques ; recettes clavier/souris Windows et GTK/WebKitGTK. |
| FR-004 à FR-009 | Matrice Nouvelle/Ouvrir/Quitter/croix × décisions, annulations, erreurs et sauvegarde capturée ; plus aucune écriture à la frappe. |
| FR-006, FR-007, FR-018 | Allers-retours .von et .md, enveloppe historique, limites UTF-8, casse d'extension, contexte changé ou absent. |
| FR-010, FR-011 | Export source seule, invariants d'identité/dirty/historique, alias de fichier, conflit externe et courses d'écriture. |
| FR-012, FR-013 | Annuler/Rétablir par zone et origine, retour à la sauvegarde, reformulation en une entrée, limite d'historique. |
| FR-014 à FR-016 | Modèles OK/Annuler/erreur, clavier, focus, propriétaire réactivé sur les deux bureaux ; A propos et version réelle. |
| FR-017 | Capture drainée, LLM attendu, attente annulée/échouée, doubles demandes et retours périmés. |
| FR-019 | Toutes les lignes précédentes sur chaque OS ; stockage Linux réel, modalité GTK, parentage Wayland et contrôle des features de dialogue. SC-001 à SC-010 ne sont jamais validés par extrapolation. |
| FR-020 | SC-011 : 20 échanges dans chaque sens, source/contexte identiques et exports exacts ; aucune conversion de chemin, d'encodage ou de fins de ligne dans le fichier. |

### Relecture constitutionnelle après conception

PASS : format toujours Markdown ouvert ; aucun stockage supplémentaire sensible ; propriété des modifications, absence de requête implicite et intégrité des fichiers préservées sur les deux systèmes. Les tests natifs de remplacement et de modalité, la recette Wayland et les échanges de fichiers sont des critères de livraison obligatoires, pas des dérogations ni des inconnues fonctionnelles restantes.

## Complexity Tracking

Aucune violation constitutionnelle à justifier. Les ajouts de coordination, de primitives Windows et de modalité GTK corrigent directement les défauts identifiés ; aucune couche générique de workflow, nouveau fournisseur ou mécanisme de plugin applicatif n'est introduit.

# Contrat UI et IPC — Dictée multimode

## Surface et autorité

Les commandes Tauri existantes restent la frontière entre l'interface et le
domaine natif. Aucun contenu de note ni aucune consigne de remplacement n'est
accepté directement comme commande LLM. Le Rust natif reste l'autorité pour la
note courante, le contexte, la révision, l'enregistrement, l'état de capture et
la validation des résultats asynchrones.

### Snapshot

Conserver les champs existants et ajouter des champs typés :

- `mode`: état de capture existant (`manual`, `voice`, `paused`) ; ne pas le
  réinterpréter comme destination de dictée.
- `dictationMode`: `content`, `context` ou `instructions`, valeur transitoire.
- `context`: texte associé à la note et restauré depuis ses métadonnées.
- `instructions`: texte temporaire, vide après sortie du mode Instructions.
- Un statut de transition peut indiquer que la sortie d'Instructions attend la
  finalisation/routage d'un ou plusieurs segments.
- `inputGeneration`: génération incrémentée par les modifications du contenu,
  contexte ou instructions, utilisée pour invalider une reformulation périmée.
- Conserver les révisions, identifiants, erreurs et blocs Markdown existants
  nécessaires à l'éditeur et à la protection contre les réponses périmées.

## Actions UI

`prototype_action` demeure typé et refuse les propriétés inconnues. Étendre ses
actions avec :

| Action | Entrée | Effet attendu |
|--------|--------|---------------|
| `set_dictation_mode` | `mode: content \| context \| instructions` | Change la destination et la visibilité. Une sortie d'Instructions avec des segments d'instructions en attente est différée. |
| `set_context` | Texte du contexte | Met à jour le contexte de la note, incrémente sa révision et utilise le chemin d'enregistrement existant. |
| `set_instructions` | Texte d'instructions | Met à jour uniquement l'état transitoire ; ne persiste pas. |

Les actions vocales explicites existantes restent disponibles. Les zones de
contexte et de contenu restent manuellement éditables quel que soit le mode.
L'éditeur de contenu conserve les commandes ProseMirror pour les styles
Markdown pris en charge.

## Reformulation

Ajouter `prototype_reformat` comme commande asynchrone explicite :

- Entrée : `{ noteId, baseRevision }`, servant uniquement à rejeter un clic
  obsolète. Le backend lit contenu, contexte et instructions à partir de l'état
  natif verrouillé.
- Aucun appel réseau lors de la saisie, de la dictée, d'un changement de mode
  ou d'une sauvegarde.
- Une invocation utilisateur produit au plus une requête de reformulation ;
  aucun retry automatique ni repli silencieux.
- Résultat : un snapshot après application réussie. Erreur, sortie invalide,
  résultat périmé ou limites dépassées produisent un échec visible et ne
  modifient pas le contenu.
- Pendant l'attente, désactiver les déclenchements dupliqués pour la même
  opération et afficher un état en cours ; conserver l'édition manuelle.
- Les erreurs et diagnostics ne contiennent ni le contenu, ni le contexte, ni
  les instructions.

Le contenu, le contexte et les instructions envoyés au modèle ne quittent
l'application qu'à la suite de l'action explicite de reformulation. Les envois
audio/STT restent couverts par le consentement vocal existant.

## Routage des segments vocaux

- Au démarrage d'un segment, capturer `dictationMode` dans l'identifiant local
  du passage.
- La finalisation STT est routée vers cette cible capturée, jamais vers la
  valeur courante au retour réseau.
- En quittant Instructions avec des segments d'instructions en cours, maintenir
  cette cible active jusqu'au routage de ces segments, puis vider Instructions
  et appliquer la transition demandée.
- Le changement de note, l'arrêt ou l'invalidation de session conserve les
  barrières et le rejet fail-closed existants.

## Markdown et intégrité

- Les snapshots exposent le même modèle de blocs et de révisions existant.
- Les modifications manuelles du contenu utilisent `prototype_manual` et ses
  protections de transaction ; les modifications de contexte suivent le
  chemin natif des métadonnées.
- La reformulation ne modifie que la source de contenu, jamais le contexte ni
  les instructions.
- Les blocs opaques qui ne sont pas éditables doivent être préservés à
  l'identique ; sinon rejeter le résultat entier.
- Les liens et images restent inertes dans l'interface. L'édition de tableaux
  et d'images est hors périmètre v2.

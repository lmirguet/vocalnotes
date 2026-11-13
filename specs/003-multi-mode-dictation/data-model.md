# Modèle de données — Dictée multimode et reformulation de notes

## État de dictée

### DictationTarget

Enumération transitoire et typée, indépendante de l'état de capture existant :

| Valeur | Zone de destination | Visibilité |
|--------|---------------------|------------|
| `content` | Contenu Markdown de la note | Toujours |
| `context` | Contexte de la note | Toujours |
| `instructions` | Instructions temporaires | Uniquement en mode Instructions |

- Valeur initiale : `content`.
- Non persisté dans la note ni dans les préférences.
- Un changement de cible n'efface ni le contenu ni le contexte.
- L'état actuel `manual` / `voice` / `paused` reste distinct et conserve son
  comportement de capture.

### InstructionText

- Champ de texte temporaire associé à la session d'édition courante.
- Éditable et visible uniquement lorsque `DictationTarget = instructions`.
- Limité à 16 KiB ; un dépassement est refusé sans troncature.
- Effacé à la sortie du mode Instructions, au remplacement de note et à la
  fermeture.
- Si un segment ciblant Instructions est déjà en cours, différer la sortie
  jusqu'à sa finalisation et son routage ; appliquer ensuite l'effacement.
- Non sérialisé et non ajouté aux journaux.

## Données de note persistantes

### NoteContent

- `source`: texte Markdown de la note ; limite existante de 1 MiB.
- Contient la structure et les styles supportés (titres, gras, italique, listes).
- Seul champ remplacé après une reformulation valide.
- Conserve les blocs opaques inchangés ; les tableaux et images ne sont pas de
  nouvelles fonctions d'édition dans cette version.

### NoteContext

- Texte facultatif persistant, éditable dans tous les modes.
- Stocké dans le champ `context` du commentaire de métadonnées versionné
  `vocalnotes:intention:v1`, puis restauré à l'ouverture.
- Conserve l'encodage et les limites existantes du type
  `DraftingIntention` (4096 octets cumulés sur ses champs legacy et contexte).
- N'est inclus dans une requête de reformulation que s'il contient du texte
  non blanc ; la valeur stockée n'est pas tronquée ni normalisée à cette fin.
- Une modification incrémente la révision et passe par le chemin d'enregistrement
  existant.

### LegacyDraftingMetadata

- Champs conservés pour préserver la compatibilité des notes existantes :
  `purpose`, `form`, `tone` et `context`.
- `context` est exposé comme NoteContext.
- `purpose`, `form`, `tone` restent masqués et inchangés lors des opérations
  v2 ; ils ne sont pas envoyés au nouveau contrat LLM.
- Une métadonnée inconnue, invalide ou trop volumineuse continue d'échouer de
  manière visible sans réécriture silencieuse.

## Transcription différée

### SpeechSegment

- `passage_id`, ticket de session, bornes temporelles : réutilisent les
  identifiants natifs existants.
- `target_at_start`: valeur immuable de `DictationTarget` capturée au début de la
  parole, avant l'attente de la finalisation STT.
- `state`: en capture, finalisé, routé ou invalidé.
- Le texte final est ajouté uniquement à `target_at_start`; aucun LLM ne
  l'interprète automatiquement.
- Les segments d'instructions en attente bloquent uniquement la transition
  demandée hors du mode Instructions jusqu'à ce qu'ils soient finalisés et
  routés. La transition efface ensuite les instructions.
- Les segments liés à une note/session invalidée sont rejetés, selon les
  barrières existantes.

## Reformulation explicite

### ReformatRequest (éphémère)

- `note_id`, `base_revision`: identité de la note et révision au départ.
- `input_generation`: invalide le résultat après modification du contexte ou
  des instructions, même si le contenu Markdown est inchangé.
- `content_markdown`: état courant non vide du contenu.
- `context`: présent uniquement si le contexte n'est pas blanc.
- `instructions`: présent uniquement si les instructions ne sont pas blanches.
- Construction côté Rust à partir de l'état natif ; l'UI n'envoie ni contenu
  arbitraire ni champs à la place du modèle.
- Limite de requête totale : 16 KiB ; dépassement refusé sans troncature.
- Non persisté et non journalisé.

### ReformatResult (éphémère)

- Un seul champ `content_markdown` non vide, validé par le contrat strict.
- Taille de réponse maximale : 64 KiB ; la note résultante reste sous sa limite
  de 1 MiB.
- Markdown analysable ; les blocs opaques présents dans l'entrée doivent rester
  identiques, sinon le résultat est rejeté.
- Appliqué uniquement si `note_id`, `base_revision` et `input_generation`
  correspondent toujours à l'état courant.
- Échec, dépassement ou résultat périmé : contenu et contexte existants
  inchangés, avec un état d'erreur visible.

## Transitions principales

```text
DictationTarget(content | context | instructions)
  ├─ segment vocal commencé → SpeechSegment(target_at_start)
  ├─ changement vers content/context → nouveau DictationTarget
  └─ quitter instructions avec segments d'instructions en attente
       → transition en attente → finalisation/routage → effacement → cible demandée

Note content/context/instructions
  ├─ édition manuelle → nouvelle révision/génération
  ├─ reformulation explicite → ReformatRequest
  │    ├─ résultat valide et toujours courant → remplacer le contenu
  │    └─ échec ou résultat périmé → conserver l'état courant
  └─ fermeture/remplacement de note → invalider travail en attente et effacer instructions
```

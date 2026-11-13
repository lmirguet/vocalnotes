# Contrat LLM — Reformulation explicite

## Déclenchement et transport

- Appelé uniquement par la commande UI explicite de reformulation.
- Utilise le modèle LLM OpenAI configuré ; pas de changement automatique de
  fournisseur, retry automatique, ni appel secondaire.
- Réutilise l'adaptateur Responses existant : HTTPS, `store:false`, schéma JSON
  strict, délai de 45 secondes, réponse maximale de 64 KiB.
- Une saisie, une transcription, un changement de mode ou un enregistrement
  n'appelle pas le LLM.
- Les appels STT pendant une capture vocale explicite restent distincts de
  cette règle de déclenchement LLM.

## Requête logique

Le backend construit une seule requête depuis l'état courant :

```json
{
  "content_markdown": "Contenu Markdown actuel",
  "context": "Contexte facultatif non blanc",
  "instructions": "Instructions facultatives non blanches"
}
```

- `content_markdown` est obligatoire et non vide.
- `context` et `instructions` sont omis lorsqu'ils sont vides ou ne contiennent
  que des espaces.
- Taille sérialisée totale : au plus 16 KiB. Un dépassement est un échec
  visible, sans troncature.
- Les champs de note sont des données utilisateur, jamais des instructions de
  transport ou des commandes d'outil.

## Directive comportementale

Sans instruction explicite autorisant une modification de fond, conserver les
faits, idées, engagements et relations exprimés. Limiter les changements à la
mise en forme Markdown, au style, à la ponctuation, à l'orthographe et à de
légères adaptations de phrasé. Si les instructions autorisent explicitement un
ajout ou une adaptation, se limiter à cette autorisation. Ne pas ajouter de
préambule, commentaire, justification ni explication hors du contenu demandé.

## Réponse structurée

Le schéma strict n'accepte qu'un objet :

```json
{
  "content_markdown": "Note reformattée en Markdown"
}
```

- Aucune propriété supplémentaire, commande de session, liste d'actions,
  intention ou enveloppe « clarification » n'est admise.
- Le champ est obligatoire, non blanc et borné à 64 KiB.
- La sortie doit être analysable comme Markdown par les parseurs actuels.
- Tout bloc opaque présent dans l'entrée doit être conservé à l'identique ;
  toute perte ou modification rejette la réponse sans effet.
- Le résultat complet doit tenir dans la limite de source de la note (1 MiB).

## Application et erreurs

- Appliquer uniquement si note, révision et génération des champs facultatifs
  correspondent à l'état capturé au départ.
- Résultat périmé, refus, réponse incomplète, JSON invalide, sortie vide,
  contenu trop grand, dépassement de délai, erreur HTTP ou bloc opaque altéré :
  préserver intégralement le contenu actuel et signaler l'échec.
- Ne jamais traiter une réponse manquante ou invalide comme une note vide ou
  une réussite.
- N'écrire le texte du prompt, de la note, du contexte, des instructions ou de
  la réponse dans aucun journal.

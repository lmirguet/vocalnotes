# Modèle de données — remplacement par OpenAI

## Préférences persistantes

Le format existant est conservé :

```json
{
  "stt": {"provider": "openai", "model": "gpt-realtime-whisper"},
  "llm": {"provider": "openai", "model": "gpt-6-luna"},
  "lastNote": null
}
```

- `provider` : seule la valeur active `openai` est acceptée après migration.
- `model` : chaîne non vide, au plus 128 octets ASCII alphanumériques, tiret,
  point ou soulignement ; existence et capacités non déduites de cette validation.
- `lastNote` : règles existantes inchangées, chemin absolu Markdown ou absence.
- Aucun secret, passage, contexte ni historique ajouté au fichier.
- Les sélections sont figées au démarrage explicite ; une reprise de session
  conserve les choix capturés, conformément au comportement existant.

### Migration

1. Lire avec les limites et protections existantes (16 384 octets, fichier
   ordinaire non symbolique, validation du chemin).
2. Reconnaître explicitement le format historique Mistral avant la validation
   réservée aux fournisseurs actifs. Ne pas traiter une configuration corrompue
   comme une installation neuve.
3. Remplacer chaque sélection historique Mistral par la sélection OpenAI
   correspondante demandée ; conserver chaque sélection déjà OpenAI valide
   et `lastNote`. Refuser un fournisseur inconnu.
4. Sauvegarder atomiquement avec le mécanisme existant ; ne publier les valeurs
   migrées en mémoire qu'après succès. En cas d'échec, signaler le problème
   et ne pas démarrer le vocal.
5. Afficher une notification de migration explicite. Une configuration déjà
   migrée n'est ni réécrite ni signalée comme nouvellement migrée.

La notification peut être un état de démarrage temporaire ; aucun journal
persistant de migration n'est nécessaire.

## Secret fournisseur

- Service du trousseau : `fr.vocalnotes.desktop`, conservé.
- Compte actif : `openai`, distinct du compte historique `mistral`.
- Valeur : clé protégée en mémoire, non sérialisable et jamais affichée.
- Actions : présence, écriture, remplacement, suppression du seul compte OpenAI.
- Aucune lecture de l'ancienne clé Mistral pour une requête OpenAI ; aucune
  suppression de cette entrée durant la migration.
- Aucun import automatique depuis un fichier ou une variable d'environnement.

Les scripts de diagnostic ponctuels autorisés par l'utilisateur ne constituent
pas un mécanisme d'import pour l'application.

## Session vocale et passage

Les identifiants de note, génération, période manuelle, révision et ticket de
session restent les barrières d'autorité du domaine. Les identifiants distants
ne les remplacent pas.

Cycle local d'un passage :

```text
attente → capture/envoi → finalisation demandée → final valide → interprétation
                 \                \                  \
                  → échec / annulation / session invalidée
```

- Une séquence de passage locale identifie l'application unique d'un résultat.
- Une connexion de transcription reste ouverte durant l'écoute ; un seul passage
  distant non finalisé est autorisé à la fois. Après le commit, les trames
  suivantes restent dans la file audio bornée jusqu'au final, sans bloquer
  physiquement le callback de capture ; une saturation arrête la session.
- Le passage associe ticket natif, séquence locale, `item_id` distant et
  `content_index`. Un delta peut précéder l'acquittement du commit : son identité
  est liée provisoirement au passage actif puis vérifiée à l'acquittement.
  Le final exige le commit demandé et acquitté pour cette même identité.
- Le dernier identifiant finalisé permet de reconnaître un doublon sans le
  réappliquer ; une autre identité inattendue est un incident explicite.
- Le texte provisoire s'accumule de manière bornée ; il ne devient jamais une
  demande ordinaire de rédaction.
- La voie prioritaire existante peut traiter un provisoire pour pause/arrêt/
  manuel ; elle ne peut pas appliquer une édition de contenu.
- Un passage final est associé à la session et à la séquence attendues avant
  sa mise en file ; un doublon ou résultat étranger n'est pas appliqué.
- Pas de stockage disque de l'audio ni des passages en attente ; saturation
  signalée et arrêt plutôt que perte ou troncature silencieuse.

## Intervention et intention

Les entités du domaine existantes restent la référence :

- `Intervention` : statut `resolved` ou `clarify`, message et liste bornée d'actions.
- Actions : édition `insert`/`replace`/`delete`/`move`, profil complet d'intention,
  commande de session déjà admise.
- Cibles : identifiant natif de bloc et texte attendu ; jamais un chemin ou
  une position arbitraire décidée par le fournisseur.
- `DraftingIntention` : but, forme, ton, contexte ; pas un historique.
- La proposition n'est appliquée qu'après contrôle du contrat, de la note,
  de sa révision et du ticket ; l'application reste transactionnelle.
- Une clarification ne contient aucune action à appliquer.

Le schéma présenté au fournisseur peut être adapté à son sous-ensemble de schéma
JSON ; les contrôles métier locaux ne sont jamais retirés pour obtenir une réponse.

## Métadonnées de diagnostic

Autoriser uniquement les compteurs, durées, codes HTTP, catégories d'erreur et
identifiants de modèles validés. Distinguer modèle configuré, configuration
acquittée et modèle effectivement attesté par une réponse ; ne pas inventer
un modèle servi quand le protocole ne le renvoie pas.

La note et l'intention enregistrées restent les seules données de contenu
durables. Leur format ne change pas.

## Bornes conservées et conversion audio

- Capture normalisée/VAD : PCM16 mono 16 kHz, trames de 480 échantillons.
- Transmission : PCM16 mono 24 kHz ; après vidange et retrait du délai connu
  du filtre, `N` échantillons natifs donnent `3N/2` échantillons transmis pour
  les trames entières. Pré-roll et dernier reliquat inclus, aucun doublon.
- Passage : au maximum 60 secondes, soit 2 880 000 octets PCM transmis ;
  ne pas garder l'ancienne borne Mistral de 1 920 000 octets.
- File audio : 128 trames de 30 ms ; file des passages finaux : quatre entrées.
- Message distant et transcription cumulée : 65 536 octets au maximum ;
  contexte LLM : 16 384 octets ; provisoire de voie prioritaire : 4 096 octets.
- Conserver les limites métier de 32 actions et de 4 096 octets pour le message
  d'intervention ; refuser un dépassement plutôt que tronquer silencieusement.

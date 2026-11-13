# Contrat des services OpenAI

## Périmètre et preuves

Modèles retenus par décision utilisateur :

- transcription : `gpt-realtime-whisper` ;
- interprétation ordinaire et priorité : `gpt-6-luna`.

Les [preuves de recherche](../research.md) établissent un échange STT de dix
secondes et deux contrats LLM réduits. Le présent contrat complet doit être
testé lors de l'implémentation ; aucun résultat de recette n'est présumé.

Les sélections personnalisées restent locales et non vérifiées lors de leur
sauvegarde. Un modèle personnalisé incompatible produit une erreur ; il ne
déclenche pas une adaptation silencieuse, un autre modèle ou un autre fournisseur.

## 1. Transport et secrets

- STT : `wss://api.openai.com/v1/realtime?intent=transcription`.
- LLM : `https://api.openai.com/v1/responses`.
- Authentification : en-tête `Authorization: Bearer …` construit dans le backend
  à partir de l'entrée OpenAI du trousseau ; jamais dans l'URL, les corps,
  les paramètres de commande ou les diagnostics.
- Hôte fixe `api.openai.com`, certificat validé avec les racines natives ;
  pas d'URL arbitraire issue des préférences.
- Aucune redirection HTTP authentifiée suivie ; un upgrade WebSocket non conforme
  est un échec, pas une invitation à ouvrir une page de notification.
- Délai absolu de 45 s pour connexion, acquittement de configuration,
  finalisation et requête LLM, reprenant la borne existante. Les ping/pong ne
  renouvellent pas ce délai. L'attente normale pendant une écoute silencieuse
  n'est pas assimilée à une finalisation bloquée.
- Aucun retry automatique ; pas de désactivation TLS, proxy de contournement,
  fournisseur de repli ni copie de contenu dans un journal d'erreur.

## 2. Initialisation STT

Attendre `session.created`, puis envoyer :

```json
{
  "type": "session.update",
  "session": {
    "type": "transcription",
    "audio": {
      "input": {
        "format": {"type": "audio/pcm", "rate": 24000},
        "transcription": {
          "model": "gpt-realtime-whisper",
          "delay": "minimal"
        },
        "turn_detection": null
      }
    }
  }
}
```

Attendre `session.updated` et contrôler le type de session, le modèle, la
fréquence et la désactivation du VAD serveur. Le modèle doit correspondre
exactement à celui demandé. Le délai acquitté peut être absent ou être l'un des
enum documentés `minimal`, `low`, `medium`, `high` ou `xhigh`; toute autre valeur
est une erreur de configuration. Signaler dans le diagnostic expurgé si le délai
est absent ou différent de `minimal`, sans journaliser sa valeur. L'acquittement
`session.updated` reste obligatoire et aucun audio ne peut être envoyé avant sa
validation.

- Ne pas envoyer `language:"fr"` : détection automatique requise.
- Ne pas envoyer de `prompt` à ce modèle.
- Ne pas envoyer le paramètre `languages` d'un autre modèle par analogie.
- Le VAD local demeure responsable de la fin de passage ; l'utilisateur n'a
  pas à valider sa parole par un bouton.

L'essai confirme le modèle acquitté et le comportement de streaming. Les tests
d'implémentation devront contrôler toute la configuration, pas seulement le modèle.

## 3. Audio et fin de passage

Envoyer les trames PCM16 little-endian mono 24 kHz sous forme :

```json
{"type": "input_audio_buffer.append", "audio": "<base64 des octets PCM>"}
```

Le marqueur entre chevrons décrit le format, pas une valeur à envoyer.
Chaque paquet reste borné à 32 000 octets PCM au maximum, comme l'envoi actuel ;
la limite serveur plus élevée ne justifie pas un buffer applicatif non borné.
Les 960 octets des trames locales 16 kHz ne sont pas directement des trames 24 kHz.

À la fin de passage détectée localement :

1. vider la conversion audio avec compensation du délai de filtre ;
2. envoyer les derniers échantillons correspondant au passage, sans troncature ;
3. envoyer une seule fois `{"type":"input_audio_buffer.commit"}` ;
4. attendre commit acquitté et final correspondant avant d'ouvrir le passage
   distant suivant sur la même connexion.

Ne pas committer un buffer vide. Conserver les bornes du
[modèle de données](../data-model.md). Le volume audio et les durées sont calculés
à partir des fréquences effectives, pas de l'ancien diviseur de 32 octets/ms.
Pas de `input_audio.end`, `transcription.done` ou autre événement Mistral.

## 4. Événements et finalité

| Événement | Traitement |
|---|---|
| `conversation.item.input_audio_transcription.delta` | Lire `delta`, `item_id`, `content_index`, accumuler l'aperçu borné |
| `input_audio_buffer.committed` | Vérifier le commit demandé ; lier `item_id`, vérifier le prédécesseur attendu lorsqu'il est présent |
| `conversation.item.input_audio_transcription.completed` | Lire `transcript`, accepter une seule fois après commit acquitté pour le même item |
| `conversation.item.input_audio_transcription.failed` | Échec de reconnaissance explicite, aucun passage au LLM |
| `error` | Échec classé, aucun effet métier |
| `conversation.item.added` / `conversation.item.created` / `conversation.item.done` | Auxiliaire : ignorer sans lire ni conserver l'item ; ne remplace aucun acquittement ni résultat |
| Événement auxiliaire connu | Ignorer sans conserver son contenu |
| JSON invalide, type absent ou identité contradictoire | Erreur de protocole explicite |

Une matrice de tests couvre le delta avant acquittement de commit, plusieurs
deltas par item, identité contradictoire, doublon de final, erreur distante,
déconnexion et résultat périmé. Les identifiants sont validés et bornés ;
aucun identifiant distant ne donne l'autorité d'appliquer une édition.

`conversation.item.added` est un événement serveur officiel, notamment émis après
commit du buffer audio ([référence OpenAI](https://developers.openai.com/api/reference/resources/realtime/server-events#conversation.item.added)).
Il peut être ignoré pendant l'attente de configuration et pendant le streaming ;
`session.updated`, le commit acquitté et le final corrélé restent obligatoires.

`conversation.item.done` signale la finalisation d'un item de conversation
([référence OpenAI](https://developers.openai.com/api/reference/resources/realtime/server-events#conversation.item.done)),
pas la finalisation de sa transcription. Son contenu est ignoré de la même façon ;
seul `conversation.item.input_audio_transcription.completed` fournit le final STT.

`completed` fournit le texte complet ; ne pas lui concaténer une seconde fois
les deltas. Un final vide ne déclenche pas d'interprétation ordinaire.
Un doublon du dernier final déjà traité peut être compté et ignoré ; ne pas
accumuler un historique illimité d'identifiants.

La valeur `model` n'est pas exigée dans le final si elle n'appartient pas à ce
contrat. Le modèle configuré/acquitté peut être affiché comme tel, mais ne doit
pas être présenté comme un modèle effectivement renvoyé dans chaque final.
Le port neutre conserve les informations de provenance nécessaires sans forger
de champ fournisseur.

## 5. Requête LLM ordinaire

Requête non streamée pour garder la validation transactionnelle existante :

```json
{
  "model": "gpt-6-luna",
  "store": false,
  "stream": false,
  "max_output_tokens": 4096,
  "instructions": "<règles de fidélité et de langue>",
  "input": "<JSON du passage final, de l'intention et des sections candidates>",
  "text": {
    "format": {
      "type": "json_schema",
      "name": "intervention",
      "strict": true,
      "schema": {}
    }
  }
}
```

Les chaînes et le schéma vide sont des emplacements descriptifs ; le schéma
réel d'intervention est obligatoire. L'essai réduit utilisait une limite
de 1 500 tokens ; la limite de 4 096 et le schéma complet restent à tester.
Ne pas ajouter `temperature`, `reasoning` ou autre paramètre non nécessaire
par simple analogie avec l'ancien fournisseur.

### Schéma d'intervention

- Racine objet : `status`, `message`, `actions`, tous requis.
- `status` : `resolved` ou `clarify`.
- `actions` : union imbriquée des actions du port existant (édition, intention,
  commande de session). Conserver noms et propriétés du domaine.
- Chaque objet utilise `additionalProperties:false` et tous ses champs sont requis.
- Ne pas reprendre le `anyOf` au niveau racine de l'ancien générateur.
- Spécialiser les cibles/ancres avec les candidats natifs comme aujourd'hui.
- Les restrictions de longueur, nombre et cohérence sont toujours vérifiées
  localement. Un sous-ensemble fournisseur plus simple n'affaiblit pas ces gardes.

Le contexte transmis reste plafonné à 16 384 octets. L'intention mémorisée
n'est ni un historique ni une commande à rejouer. Le passage est une donnée ;
le texte d'une note ne devient pas une instruction de confiance.
Le prompt préserve la langue de rédaction et reconnaît les demandes dans les
deux langues, au lieu d'imposer systématiquement le français.

### Extraction et validation

1. Exiger HTTP 2xx, corps borné à 65 536 octets et JSON valide.
2. Exiger `status:"completed"` sans erreur ni `incomplete_details` non nul.
3. Parcourir `output` ; accepter seulement les métadonnées de raisonnement
   sans effet et un message assistant avec un contenu `output_text` attendu.
   Refuser outils, refus et sorties inattendues ; ne pas supposer que le champ
   de confort `output_text` d'un SDK existe au niveau racine HTTP.
4. Lire l'identifiant de modèle présent dans la réponse, le borner et l'exposer
   comme modèle servi sans s'en servir comme preuve de qualité.
5. Décoder le JSON d'intervention et exécuter les validations existantes :
   cohérence de statut, 32 actions maximum, ordre terminal, cibles, intention.
6. Vérifier ticket, note et révision au commit transactionnel. Une réponse
   parfaitement structurée ne dispense d'aucune validation métier.

## 6. Voie prioritaire

Même transport Responses et modèle ; schéma strict réduit :

```json
{
  "type": "object",
  "properties": {
    "interrupt": {"type": "string", "enum": ["none", "pause", "stop", "manual"]}
  },
  "required": ["interrupt"],
  "additionalProperties": false
}
```

Un seul appel en vol, fusion des provisoires récents, cadence existante de 750 ms
conservée. Le contrôle ne peut pas produire d'édition. Les propos cités ne sont
pas pris pour une demande immédiate ; couvrir les deux langues.
Ne pas masquer un dépassement du contexte par troncature silencieuse.

## 7. Catégories d'erreur

Maintenir la distinction réseau, timeout, TLS/certificat, handshake WebSocket,
configuration, authentification (401), accès/modèle (403/404), quota/limitation
(429), service (5xx), réponse invalide, refus et saturation locale.
Un 404 ne prouve pas universellement l'inexistence d'un modèle ; un 429 ne
distingue pas à lui seul quota financier et limitation de débit.

Les codes distants peuvent être traduits par liste fermée de catégories ;
ne jamais afficher le message fournisseur brut, le corps ou les en-têtes.
Toute erreur suit la barrière d'arrêt existante, préserve la note et interdit
une réémission automatique.

`store:false` ne constitue pas une garantie de non-conservation par le
fournisseur. Les informations de compte et de conservation doivent être
vérifiées et annoncées avant les recettes réelles.

# Recherche — OpenAI

**Date** : 2026-11-11.
**État** : décisions de conception résolues ; faisabilité établie sur essais
bornés, recette produit complète encore requise.

## 1. Modèle de langage retenu

**Decision** : `gpt-6-luna`, choisi explicitement par l'utilisateur après
consultation du catalogue authentifiée.

**Rationale** : l'identifiant initial `gpt-6.1-luna` retourne 404, sans distinction
entre modèle inexistant et accès refusé. Le catalogue complet retourne 98
identifiants, dont `gpt-6-astra`, `gpt-6-luna`, `gpt-6-sol`, `gpt-6.1-sol`,
`gpt-6.1-terra`. Le premier filtre utilisé excluait à tort les suffixes.
Deux requêtes Responses autorisées ont ensuite vérifié accès et sortie structurée.

**Alternatives considered** : GPT-5 non retenu ; modèles GPT-6.1 proposés au
catalogue mais non choisis. Aucun autre modèle de génération n'a été appelé.

## 2. Streaming avant fin de parole : preuve et décision utilisateur

**Decision** : `gpt-realtime-whisper`, après approbation conditionnelle explicite
de l'utilisateur et essai satisfaisant la condition de texte pendant la parole.
Conserver `gpt-6-luna`, les langues et les seuils de la spécification.

**Rationale** : le modèle initial ne délivrait du texte qu'après commit lors de
l'essai. La segmentation forcée produisait un aperçu précoce mais dégradait le
nombre de mots reconnus. Le modèle temps réel produit des deltas avant commit,
avec un seul flux, sans devoir couper les mots ni doubler les transmissions.

### Protocole expérimental autorisé

- Audio anglais synthétique généré avec FFmpeg/flite, en mémoire, sans microphone,
  note, propos d'un tiers ni fichier audio persistant.
- Même séquence fictive de dix secondes à 24 kHz mono PCM16, envoyée à cadence
  réelle dans trois essais distincts ; 30 secondes audio envoyées au total.
- Connexions authentifiées directement à OpenAI, certificats validés ;
  pas de redirection, fournisseur tiers, retry automatique ni repli.
- Langue non spécifiée ; un seul test anglais ne démontre pas la qualité française.
- Premier temps mesuré depuis le début de l'envoi audio, après configuration
  acquittée. Ce n'est pas une mesure de rendu UI depuis un microphone physique.
- Aucun audio ni texte de transcription conservé dans les preuves ; seulement
  compteurs, délais et résultats booléens.

| Essai | Configuration | Premier texte | Commit initial | Deltas / finals | Mots reconnus |
|---|---|---|---|---|---|
| A | `gpt-4o-transcribe`, commit à la fin | 10,563 s | 10,001 s | 45 / 1 | 44 |
| B | `gpt-4o-transcribe`, commit chaque seconde audio | 1,176 s | 0,902 s | 37 / 10 | 30 |
| C | `gpt-realtime-whisper`, `delay:minimal`, commit à la fin | 0,799 s | 10,001 s | 46 / 1 | 44 |

L'essai B envoyait des paquets de 100 ms et validait après la dixième trame,
d'où un premier commit à 0,902 s de temps mural. Ces mesures n'incluent pas
la normalisation native, le pré-roll ni le rendu dans Vocalnotes.
Le nombre de mots n'est pas un taux d'erreur : ni 44 mots ni un seul essai
n'établissent la fidélité sémantique ou SC-001.

Pour C : modèle acquitté, texte reçu avant le premier commit et avant la fin
du flux, commit acquitté et un final correspondant reçu.

**Alternatives considered** :

- Attendre la fin de passage avec le modèle initial : rejeté par l'utilisateur.
- Commits périodiques : écart de reconnaissance observé, non retenus.
- Deux flux (aperçu court et final continu) : proposition discutée, non adoptée ;
  aucune double transmission simultanée réalisée.
- Modèle temps réel proposé par l'utilisateur : retenu après essai C.

## 3. Contrat STT retenu

**Decision** : une connexion Realtime de transcription, session de type
`transcription`, PCM16 mono 24 kHz, `model:gpt-realtime-whisper`, `delay:minimal`,
`turn_detection:null`, sans paramètre `language` ni `prompt`.

**Rationale** : configuration et deltas pré-commit confirmés par C ; le SDK
officiel impose `turn_detection:null` pour ce modèle et expose le réglage
`delay`. Le prompt n'est pas pris en charge selon le type officiel.
Les finalités et événements génériques décrits comme « après commit » ne
doivent pas être extrapolés aux deltas effectivement observés avant commit.

**Alternatives considered** : VAD serveur non admis par le modèle, WebRTC
inutile pour les transports natifs existants, SDK fournisseur supplémentaire inutile.

Sources officielles figées à la révision
`141a89aa7fbb7f13a979bda930831a5fe38bc3a4` d'OpenAI Python :

- [Paramètres de transcription](https://github.com/openai/openai-python/blob/141a89aa7fbb7f13a979bda930831a5fe38bc3a4/src/openai/types/realtime/audio_transcription.py) :
  modèle, langue facultative, délai pour `gpt-realtime-whisper`, prompt non admis.
- [Configuration audio](https://github.com/openai/openai-python/blob/141a89aa7fbb7f13a979bda930831a5fe38bc3a4/src/openai/types/realtime/realtime_transcription_session_audio_input.py) :
  PCM 24 kHz et VAD désactivé pour ce modèle.
- [Append](https://github.com/openai/openai-python/blob/141a89aa7fbb7f13a979bda930831a5fe38bc3a4/src/openai/types/realtime/input_audio_buffer_append_event.py) :
  audio base64, absence d'acquittement par paquet, limite serveur 15 MiB.
- [Commit](https://github.com/openai/openai-python/blob/141a89aa7fbb7f13a979bda930831a5fe38bc3a4/src/openai/types/realtime/input_audio_buffer_commit_event.py) :
  validation du buffer, erreur sur buffer vide.
- [Commit acquitté](https://github.com/openai/openai-python/blob/141a89aa7fbb7f13a979bda930831a5fe38bc3a4/src/openai/types/realtime/input_audio_buffer_committed_event.py) :
  `item_id` et `previous_item_id`.
- [Delta](https://github.com/openai/openai-python/blob/141a89aa7fbb7f13a979bda930831a5fe38bc3a4/src/openai/types/realtime/conversation_item_input_audio_transcription_delta_event.py)
  et [final](https://github.com/openai/openai-python/blob/141a89aa7fbb7f13a979bda930831a5fe38bc3a4/src/openai/types/realtime/conversation_item_input_audio_transcription_completed_event.py) :
  corrélation `item_id`/`content_index`, `delta` et `transcript`.

Les guides OpenAI directs restaient inaccessibles via la récupération utilisée ;
aucun contournement TLS n'a été effectué. La recherche déléguée a obtenu une
[fiche AWS Bedrock](https://docs.aws.amazon.com/bedrock/latest/userguide/model-card-openai-gpt-6-luna.html),
source partenaire uniquement : aucune propriété propre à cet hébergement n'est
présumée pour l'API OpenAI directe. Les contrats ci-dessus sont fondés sur le
SDK officiel et les essais directs, pas sur cette fiche partenaire.

Le SDK annonce par ailleurs un arrêt au 2027-02-26 de plusieurs modèles
historiques, dont `gpt-4o-transcribe`. Aucun changement supplémentaire n'en
est déduit ; la disponibilité du modèle retenu devra être surveillée et
aucune migration future automatique n'est autorisée.

## 4. Audio et responsabilité de fin de passage

**Decision** : conserver VAD local et gate à 16 kHz, convertir les trames
sélectionnées en continu vers 24 kHz dans l'adaptateur, hors callback temps réel.
Un seul commit par fin de passage métier, pas de segmentation artificielle.

**Rationale** : normalisation existante à 16 kHz, trames de 480 échantillons,
30 ms, pré-roll dix trames, silence 810 ms, passage borné à 60 secondes.
Le VAD WebRTC n'accepte pas 24 kHz ; changer sa constante ne serait pas une solution.
L'adaptateur Mistral ne peut pas être simplement pointé vers une autre URL.

**Alternatives considered** : rééchantillonnage de tout le pipeline vers une
autre fréquence admise par le VAD : davantage de modifications sans besoin.
Le choix rubato réutilise la dépendance existante ; tester délai de filtre,
continuité, nombre exact d'échantillons et vidange du dernier bloc.

## 5. Sorties structurées LLM

**Decision** : Responses HTTPS avec `store:false`, schéma strict dans
`text.format`, `max_output_tokens` explicite et sans copie des paramètres
Mistral `temperature`/`max_tokens`.

**Rationale** : deux appels directs autorisés à `gpt-6-luna`, maximum 1 500
tokens de sortie chacun, ont donné HTTP 200, état `completed`, modèle attendu,
pas de refus et contenu JSON conforme aux résultats attendus.

| Contrat réduit | Temps mesuré | Résultat |
|---|---|---|
| Intervention : statut/message, union imbriquée intention ou insertion dans bloc vide | 2,612 s | insertion anglaise attendue, sans fait ajouté |
| Priorité : enum `none`/`pause`/`stop`/`manual` | 2,031 s | `pause` attendu |

Les prompts étaient fictifs et ne contenaient ni note ni secret. Les deux appels
ne prouvent pas le contrat complet, les 12 cas de rédaction, la latence native
ou la qualité bilingue. Le coût exact n'a pas été vérifié.

**Alternatives considered** : Chat Completions proche de Mistral, mais Responses
désormais testé avec le modèle retenu ; inutile de multiplier les transports.
Le schéma existant contient un `anyOf` racine : présenter au fournisseur un objet
racine avec unions imbriquées et conserver les invariants localement.
Chaque champ est requis et chaque objet interdit les propriétés supplémentaires.
Ne pas retirer les contrôles du domaine pour accommoder le service.

Un refus ou `incomplete` n'est jamais parsé comme une proposition utilisable ;
les outils et sorties inattendues sont refusés. `store:false` désactive le
stockage de réponse demandé par cette option ; il ne garantit pas une
absence de conservation fournisseur ou de journaux de sécurité.

## 6. Migration et limites de validation

**Decision** : format de note inchangé, migration explicite des sélections,
entrée `openai` du trousseau, notification et consentement mis à jour.

**Rationale** : validation actuelle limitée à Mistral ; migrer avant validation
active tout en conservant protections de fichiers et chemin. Ne pas traiter une
configuration corrompue comme neuve. Ne ni lire ni supprimer l'ancienne clé.

**Alternatives considered** : import de fichier de clé, récupération automatique
de configuration corrompue et nouveau registre multifournisseur refusés.

Les essais payants ont respecté l'accord : trente secondes audio synthétique,
deux petites générations, aucune relance automatique et aucun microphone.
Les scripts temporaires sont supprimés ; les preuves persistantes sont les
métadonnées synthétiques ci-dessus, sans contenu sensible.
Les essais Rust, UI, langues réelles et plateformes ne sont pas déclarés réussis :
voir [quickstart.md](./quickstart.md) pour les obligations de livraison.

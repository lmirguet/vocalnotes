# Recette OpenAI — Windows 11

**Statut : NON EXÉCUTÉE / NON VALIDÉE — 2026-11-11.**

L'implémentation est réalisée sur Linux. Aucun poste Windows 11, Credential
Manager, microphone, WebView2 ou cycle verrouillage/veille Windows n'a été testé.
Les contrôles Linux et le type-check ne qualifient pas cette plateforme.

## Blocages et mesures absentes

- Machine Windows 11 et configuration matérielle/réseau non disponibles ici.
- Nouvelle autorisation microphone, envois OpenAI et coûts non obtenue.
- SC-001 à SC-008 : **non validés sur Windows** ; aucun WER, comptage
  18/20, résultat 12/12, récupération 100 % ou mesure de coût.
- Référence synchronisée de parole/rendu non calibrée.
- Effet actuel de Zscaler sur le binaire et les certificats Windows : inconnu.

Modèles prévus : `gpt-realtime-whisper` et `gpt-6-luna`, sans substitution.
Coût futur inconnu ; budget de planification déjà consommé.
Tout refus réseau/certificat doit rester un blocage explicite, sans contournement.
Suivre le [guide](../../specs/002-remplacer-services-openai/quickstart.md)
après autorisation et publier les observations réelles. T045 reste ouverte,
aucun GO.

## Diagnostic expurgé des événements STT

Le stderr peut afficher `STT streaming event rejected event_type_code=N`.
Ce diagnostic ne change pas le refus de protocole et ne contient aucun payload.
Les codes appartiennent à une liste fermée :

| Code | Type |
|---|---|
| 0 | Token de type inconnu valide, accompagné d'une empreinte |
| 1 | Champ `type` absent |
| 2 | Champ `type` non textuel |
| 3 | Chaîne vide, trop longue ou alphabet invalide |
| 10 | `session.created` |
| 11 | `session.updated` |
| 20 | `input_audio_buffer.cleared` |
| 21 | `input_audio_buffer.speech_started` |
| 22 | `input_audio_buffer.speech_stopped` |
| 30 | `conversation.item.input_audio_transcription.segment` |
| 31 | `conversation.item.done` (désormais auxiliaire accepté, non rejeté) |
| 32 | `conversation.item.deleted` |
| 33 | `conversation.item.retrieved` |
| 34 | `conversation.item.added` (désormais auxiliaire accepté, non rejeté) |

Relever uniquement le code de l'essai utilisateur, sans afficher le reste du
journal ni une chaîne de type distante arbitraire. Ces codes ne constituent pas
une preuve de recette ou de compatibilité des événements.

Pour le code 0 uniquement, `event_type_fingerprint` est l'entier non signé
big-endian des huit premiers octets du SHA-256 du seul champ `type`. Le token
doit contenir de 1 à 128 octets ASCII parmi `A-Z`, `a-z`, `0-9`, `_`, `.` et `-`.
Aucune empreinte n'est produite pour les catégories 1, 2 ou 3.

L'empreinte est stable entre exécutions et permet de comparer hors ligne des
types candidats sans journaliser le texte reçu. La troncature à 64 bits permet
des collisions : une correspondance n'est pas une preuve d'identité. Ce hash
ne constitue pas un chiffrement et des tokens devinables peuvent être retrouvés
par comparaison ; ne jamais l'appliquer à du contenu utilisateur.

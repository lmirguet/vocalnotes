# T002 — contrôle fournisseur préparé

2026-10-30. Sources : research R02/R03/R09 et contrats du plan courant.
**Aucun appel authentifié, compte, audio réel ou test payant exécuté.**
Autorisation utilisateur reçue pendant ce lot : usage de la clé locale permis
pour quelques vérifications Mistral strictement nécessaires et synthétiques,
coût possible inclus. Aucun fichier de clé n'a été lu à ce stade. Cette permission
ne change ni la configuration produit par trousseau, ni la gate deux OS.

| Port indépendant | Contrat retenu | État réel |
|---|---|---|
| STT | `voxtral-mini-transcribe-realtime-2602` ; WSS `wss://api.mistral.ai/v1/audio/transcriptions/realtime` avec paramètre model | Intégration à écrire, accès compte inconnu |
| Interprétation | `mistral-large-latest` ; POST `https://api.mistral.ai/v1/chat/completions` ; json_schema strict | Intégration à écrire, modèle servi inconnu |

STT : PCM mono s16le 16 000 Hz sans WAV, base64 append après session créée et
format configuré. Rotation locale VAD avec flush/end ; continuer la capture
bornée du passage suivant pendant finalisation. `transcription.done` seul permet
le traitement ordinaire. Delta/segment ne sont pas une preuve de finalité.
Voie prioritaire sur provisoire : interruption/invalidation seulement.

LLM : `response_format.type=json_schema`, `strict=true`, champ filaire `schema`
(pas `schema_definition`), tous champs obligatoires, `additionalProperties=false`
sur chaque objet. Réponse complète validée localement avant effets ; enveloppe
résolue ordonnée ou clarify sans action, aucun outil arbitraire/undo/redo vocal.
`latest` est mutable : ne pas le présenter comme Large 3/4 ou version figée.

## Premier essai réel : protocole borné

1. Accord externe/coût reçu pour ce lot : catalogue, une réponse structurée et un
   court STT synthétique seulement si utiles ; aucune note ni voix réelle.
2. Produit : saisie masquée puis trousseau natif uniquement. Exception de test
   autorisée pour ce lot : un processus local peut charger directement le fichier
   original en mémoire, sans afficher/copier/persister sa valeur, sans argument
   CLI/env/URL et sans import dans l'application. TLS vérifié, API officielle
   uniquement, aucun proxy/relais ni bypass. Pièce jointe inutile et non consultée.
3. Vérifier catalogue/droits du compte, acceptation PCM/WSS/schéma ; relever le
   modèle effectivement servi et la date. Ne conserver que métadonnées expurgées.
4. Utiliser uniquement des phrases synthétiques sans données personnelles.
   Vérifier rotation, correction du provisoire et interruption naturelle.
5. Vérifier tarifs officiels datés/applicables et conditions du compte avant
   usage sensible ; absence de preuve = composante inconnue, pas zéro.
6. Documenter séparément les essais GNOME et Windows en T007, sans fallback.

| Métadonnée attendue au premier essai autorisé | Valeur actuelle |
|---|---|
| Autorisation API/coût | REÇUE pour le lot, vérifications synthétiques bornées seulement |
| Catalogue du compte | INCONNU |
| Modèles réellement servis STT/LLM | INCONNUS |
| Usage audio/tokens rapporté | NON MESURÉ |
| Prix STT/LLM, source officielle, date | INCONNUS |
| Conservation et conditions applicables | NON VÉRIFIÉES |
| Latence/qualité/acceptation schéma | NON EXÉCUTÉES |

Références publiques issues de research :
[SDK figé](https://github.com/mistralai/client-python/tree/90e300904b64c36e0630f72ef475dbab7912068e),
[cookbook figé](https://github.com/mistralai/cookbook/tree/0ce219a9ac212d5521c22df47bbc889527416bc6),
[tarifs à vérifier](https://mistral.ai/pricing),
[conditions à vérifier](https://mistral.ai/terms).
Ces liens ne constituent pas une vérification du compte ou des conditions.

# Contrat d'interface et IPC

## Surface conservée

Les commandes Tauri existantes restent les points d'entrée :

- `preferences_get` et `preferences_set` : sélections de service séparées,
  `lastNote` conservé, aucun appel réseau lors de l'enregistrement.
- `prototype_set_key`, `prototype_delete_key`, `prototype_key_presence` :
  même parcours sécurisé, ciblant exclusivement l'entrée OpenAI du trousseau.
- `prototype_action`, `prototype_snapshot`, `prototype_manual`,
  `prototype_finish_close`, `startup_status` : responsabilités existantes
  conservées, aucune nouvelle permission générale.

Le secret n'est jamais renvoyé par une commande. Les erreurs ne contiennent ni
requête distante, ni corps fournisseur, ni en-tête d'authentification.

## Préférences

Les deux fournisseurs affichés deviennent OpenAI. Les suggestions de modèles
sont `gpt-realtime-whisper` et `gpt-6-luna`. Les champs de modèles personnalisés restent
distincts ; leur sauvegarde n'atteste pas leur disponibilité.

La langue est détectée automatiquement, sans nouveau choix obligatoire.
La notice conserve la distinction entre nouvelle session et reprise.

Lors d'une migration, une notification visible annonce le changement de
fournisseur et demande de renseigner la clé OpenAI si nécessaire. Une notification
temporaire dans l'état de démarrage peut être ajoutée de façon typée et testée
sans exposer les préférences historiques complètes.

## Transparence et consentement

Avant le démarrage autorisé du vocal :

- nommer OpenAI comme destinataire de l'audio et du contexte textuel nécessaire ;
- afficher les modèles configurés sans prétendre qu'ils ont été testés ;
- expliquer les coûts et la conservation connus ou explicitement inconnus ;
- ne pas réutiliser une autorisation libellée Mistral pour annoncer des envois OpenAI.

Le formulaire de clé, la barre d'outils, les messages d'état, les préférences
et l'aide doivent être cohérents. L'interface reste française ; le contenu dicté
conserve sa langue.

## Échecs et édition

Le stade `recognition`, `interpretation` ou `application` reste distinct de la
catégorie d'erreur. Un refus ne se transforme pas en succès vide.
Pause/arrêt/manuel et les événements système invalident les traitements.
La note, l'intention, les retours de modification et l'annulation manuelle
conservent leurs contrats actuels.

L'état « écoute prête » n'est annoncé qu'après connexion et configuration de
transcription acquittées. L'utilisateur ne doit pas être invité à dicter pendant
une phase où la capture ou le transport n'accepterait pas encore sa parole.
Les deltas alimentent l'aperçu et la voie prioritaire, jamais la rédaction ordinaire.

## Vérifications

- Installation neuve, migration, configuration déjà migrée, fichier invalide,
  migration impossible et trousseau indisponible.
- Clé absente/ajoutée/remplacée/supprimée sans secret dans les snapshots.
- Libellés actifs OpenAI seulement ; les rapports historiques Mistral restent
  clairement historiques.
- Changement de modèles appliqué à la prochaine nouvelle session seulement.
- Aucun appel réseau depuis `preferences_set` ni lecture automatique d'un fichier de clé.

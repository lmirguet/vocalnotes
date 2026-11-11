# Guide de validation — migration OpenAI

Ce guide est exécuté après implémentation. Il ne prétend pas que les futurs
adaptateurs, tests ou comportements existent déjà. Voir le
[plan](./plan.md), les [contrats](./contracts/openai-services.md) et le
[modèle de données](./data-model.md).

## 1. Prérequis

- Node conforme à la plage déclarée et dépendances du lockfile.
- Rust 1.99.1 et composants natifs Tauri sur la plateforme cible.
- Sous GNOME/Wayland : microphone et Secret Service utilisables ;
  sous Windows 11 : microphone et Credential Manager utilisables.
- Accès OpenAI autorisé, clé ajoutée via le formulaire sécurisé de l'application,
  quota et conditions de conservation vérifiés pour les données testées.
- Modèles : `gpt-realtime-whisper` et `gpt-6-luna`.
- Certificats et accès réseau validés normalement ; aucun `-k`, désactivation TLS,
  proxy de contournement ou envoi de clé à un autre hôte.

La simple présence d'un fichier de clé à la racine n'est pas une configuration
de l'application. Ne jamais coller une clé dans les commandes, les rapports ou
la conversation. Les scripts temporaires de planification ne sont pas un outil
à conserver ou un mécanisme d'import à implémenter.

Dans l'environnement de planification, Cargo était absent. L'installation locale
de Rustup et Rust 1.99.1 a ensuite été autorisée et réalisée, sans modification du
PATH ni installation système ; utiliser `~/.cargo/bin/cargo` si nécessaire.
Les résultats automatisés sont consignés dans
[le rapport de migration](../../docs/validation/openai-automated.md).

## 2. Contrôles automatisés sans réseau fournisseur

Depuis la racine du dépôt, après implémentation :

```sh
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml
```

Restaurer les dépendances uniquement si elles manquent ou si le manifeste a
changé ; utiliser alors `npm ci`, sans réécriture du lockfile.
Les commandes Rust exigent une toolchain et les dépendances natives disponibles.

La suite doit couvrir :

1. Requêtes et réponses simulées conformes au contrat STT, delta pré-commit,
   commit/final corrélés, doublons, erreurs, limite de taille et annulation.
2. Conversion 16→24 kHz : pré-roll, durée, amplitude, délai, vidange et borne 60 s,
   sans trous ni duplication ; VAD local inchangé.
3. Responses ordinaire et priorité : schéma complet, refus, réponse incomplète,
   outils interdits, contenu inattendu, anglais préservé, contexte borné.
4. Ticket/révision/note périmés, ordre d'application, pause/veille/verrouillage,
   saturation des files, arrêt de capture et absence de spool.
5. Préférences neuves, historiques, déjà migrées, fournisseur inconnu, fichier
   corrompu/non ordinaire, migration non enregistrable ; pas de valeurs de secours.
6. Isolation des entrées de trousseau et secrets absents des snapshots/logs.
7. Corps Markdown, intention, sauvegarde, édition manuelle, annulation et fermeture.
8. Interface, consentement, messages et outils actifs : aucun usage Mistral
   résiduel ; les tests/rapports historiques ne sont pas des validations OpenAI.

Les tests contractuels utilisent des transports simulés : `cargo test` et
`npm test` ne doivent pas déclencher des appels fournisseur payants.

## 3. Démarrage natif et préparation

```sh
npm run tauri dev
```

Avant le test, préparer une note de référence non confidentielle et un corpus
fictif. Autoriser explicitement microphone et envois OpenAI après lecture des
informations de coût et de conservation. Une nouvelle recette payante nécessite
un nouvel accord ; les trente secondes de planification ont été consommées.

Vérifier :

- préférences OpenAI, modèles retenus et état de clé distinct de Mistral ;
- vocal refusé avant capture si la clé manque ;
- configuration acquittée avant l'indication « écoute prête » ;
- pas d'envoi vers Mistral ni de fichier audio/transcription durable ;
- interface de transcription visible pour mesurer le premier rendu.

Ne pas utiliser les diagnostics détaillés de réseau s'ils exposent des en-têtes
d'authentification ou du contenu. Rapports : durées, compteurs, catégories
et observations sur corpus fictif uniquement.

## 4. Recette de migration

Sur un profil de test séparé, sans toucher aux notes ou secrets de production :

1. Préparer une configuration historique valide et une note avec intention.
2. Démarrer : notification, remplacement des sélections Mistral, chemin de note
   conservé, ancienne clé laissée intacte et jamais réutilisée.
3. Ajouter/remplacer/supprimer une clé de test via l'application ; vérifier
   uniquement présence et succès des actions.
4. Redémarrer : pas de nouvelle migration ni perte de sélection OpenAI compatible.
5. Éditer manuellement, sauvegarder et rouvrir ; comparer corps et intention.
6. Provoquer un échec d'enregistrement de préférences sur ce profil : erreur
   explicite et aucun démarrage vocal sur une migration non persistée.

Un fichier corrompu ou fournisseur inconnu ne doit pas être « migré » en écrasant
le problème avec des valeurs par défaut.

## 5. Corpus et seuils exacts

Le [corpus OpenAI](../../tests/fixtures/openai-fidelity.json) fixe les passages,
demandes et interruptions. Fixer le corpus et les attentes avant le premier essai, sans modifier la
référence pour transformer un échec en succès. Les enregistrements éventuels
de fixtures doivent être synthétiques/non confidentiels et explicitement
autorisés, distincts de l'audio temporaire de l'application.

### Transcription et latence — SC-001 et SC-002

- Dix passages français et dix anglais, alternés dans une même session, sans
  sélection de langue ni arrêt du vocal entre passages.
- Inclure noms, nombres et négations pré-identifiés comme critiques ; prévoir
  des passages de plus de deux secondes sans silence intermédiaire, pour ne
  pas confondre texte après fin de parole et texte pendant la parole.
- Taux d'erreur de mots : `(substitutions + suppressions + insertions) / mots
  de référence`, calculé séparément par langue. Fixer avant recette la
  tokenisation et la normalisation de casse/ponctuation ; ne pas effacer
  les nombres ou négations pour améliorer le score.
- Exiger ≤ 10 % par langue et aucun contresens sur un élément critique.
- Mesurer depuis le début réel de parole capté jusqu'au premier texte non vide
  effectivement visible ; inclure normalisation, réseau, événements et rendu.
  Exiger ≤ 2 s pour au moins 18/20 passages.
- Mesurer depuis la fin réelle de parole jusqu'au rendu de note validé ;
  inclure le silence VAD, la finalisation et le LLM. Exiger ≤ 5 s pour 18/20.
- Les deltas ne doivent pas modifier la note ; le final n'est intégré qu'une fois.

Ces seuils ne sont pas validés par les 0,799 s mesurées pendant le diagnostic
de protocole, qui n'avait ni microphone ni interface native.

#### Corrélation de rendu et référence de parole

L'interface émet `vocalnotes-render` après deux `requestAnimationFrame`, avec
uniquement `type` (`transcript-partial` ou `note-commit`), `noteId`, `sessionId`,
`generation`, `passageId` et `renderedAt` (millisecondes,
`performance.timeOrigin + performance.now()`). Ces métadonnées ne sont ni
journalisées ni persistées par l'application. Le premier provisoire et le commit
sont dédupliqués par passage ; les callbacks périmés sont annulés. Le snapshot
expose `transcriptPassageId` et `appliedPassageId` pour la corrélation.

Il s'agit d'une **occasion de peinture**, pas d'une preuve de pixels effectivement
visibles : les fenêtres masquées sont exclues mais une zone hors viewport ou
occultée n'est pas détectée. Garder les deux zones visibles et confirmer le rendu
sur une référence externe synchronisée. Ne pas compter un événement masqué ou
un changement non visible comme succès.

La référence doit fournir le début et la fin **réels** de la parole captée et
le premier rendu effectivement visible dans la même base de temps. Décrire le
dispositif, sa calibration, sa résolution et son incertitude avant l'essai ;
invalider la mesure en cas de saut d'horloge ou de corrélation ambiguë. Une capture
externe éventuelle du seul corpus fictif exige une autorisation distincte ; elle
ne devient pas un enregistrement de l'application. Avec une incertitude, retenir
la borne haute du délai pour comparer aux seuils inchangés.

Ne pas remplacer ces repères par le début VAD, le commit (qui inclut déjà le
silence), `lastSttFinalMs` ou `lastLlmMs`. Les tests hors ligne vérifient les
identifiants, annulations et occasions de peinture, **pas** la référence audio
ni SC-002. L'instrumentation bout en bout T042 reste partielle jusqu'à cette
calibration et à la vérification des vrais repères.

### Rédaction — SC-003

Douze demandes fixées, six par langue, réparties entre rédaction, correction,
intention et ambiguïté. Attendre 12/12 résultats conformes à leurs attentes :
faits conservés, langue respectée, modifications ciblées, clarification sans
édition quand requise. Inclure les règles d'ordre terminal et de note vide.

### Interruption et erreurs — SC-004 à SC-007

- Dix séquences couvrant les cinq situations pause, arrêt, verrouillage, veille
  et fermeture : aucune application tardive ni duplication.
- Toutes les notes, intentions et chemins de référence restent récupérables.
- Échecs simulés : réseau/TLS, 401, 403/404, 429, 5xx, timeout, refus,
  réponse invalide, contexte trop grand, trousseau indisponible et saturation.
- L'utilisateur voit une erreur, conserve sa note et peut éditer manuellement ;
  aucune réémission automatique ou bascule de modèle.
- Aucun secret, audio ou contexte temporaire dans les préférences, logs ou
  notes enregistrées hors contenu et intention volontairement rédigés.
- Aucun trafic Mistral après migration. Utiliser des assertions de transport
  et les destinations, pas des captures contenant les clés.

## 6. Preuves par plateforme — SC-008

Publier deux rapports distincts sous `docs/validation/`, un GNOME/Wayland et
un Windows 11. Indiquer :

- version de l'application et modèles configurés/acquittés/servis selon preuve ;
- OS, matériel, fréquence micro, réseau, date et conditions de test ;
- commandes exécutées, résultats de chaque SC, nombres de succès et mesures ;
- permissions, trousseau, arrêt physique de capture et événements système ;
- erreurs et blocages, coût observé ou inconnu, limites de conservation connues.

Si une plateforme, une langue ou un seuil est bloqué/non testé, l'écrire ainsi.
Une simulation, un type-check Windows sur Linux ou un HTTP 101 ne remplace
pas cette recette. Ne pas déclarer la migration livrable tant qu'un critère
obligatoire reste hors seuil ou sans preuve.

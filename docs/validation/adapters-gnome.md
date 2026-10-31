# T020–T022 — contrats fournisseur, capture et adaptateurs

Lot B08-ADAPTERS du 2026-10-31, Linux/GNOME/Wayland. Implémentation et vérification
hors ligne. Aucune nouvelle recette micro/fournisseur ; la fenêtre existante
n'a pas été arrêtée, reconstruite ou relancée par ce lot.

## Changements

`audio/capture.rs` porte la capture CPAL et `audio/normalization.rs` la
normalisation mono PCM s16le 16 kHz, rubato et VAD du prototype. Les anciens modules
`platform/audio.rs` et `platform/capture.rs` réexportent ce même code, sans seconde
implémentation. Le callback garde son anneau borné, sans allocation, réseau,
disque, journalisation, verrou ou rééchantillonnage. Deux tests exercent directement
son entrée d'échantillons pour arrêt et saturation sans ouvrir de périphérique.

`adapters/mistral_stt.rs` partage le protocole entre les connexions WebSocket
natives et les transports en mémoire : création, configuration audio acquittée,
append borné, flush/end puis finalisation. Les fragments et segments ne deviennent
pas des éditions ordinaires ; seul `transcription.done` fournit le texte final,
y compris ses corrections. Un `done` prématuré ou déjà consommé est refusé.
Les moitiés lecture/écriture natives partagent l'état du passage.

Un envoi échoué ou annulé rend le passage terminal. Le test d'annulation a d'abord
montré une deuxième tentative possible après abandon de la future ; une garde
sur l'envoi marque désormais cet état même lorsque la future est détruite pendant
l'attente. Aucun nouvel append/flush/end n'est autorisé ensuite. Une erreur de
lecture/JSON invalide rend également le passage terminal. Pas de reconnexion ou
réémission automatique ; l'invalidation au commit reste nécessaire et conservée.

`adapters/mistral_interpreter.rs` porte le transport HTTP natif et l'interprétation
avec transport injectable. Hôte HTTPS fixe, redirections désactivées, en-tête
d'authentification sensible ; aucune clé dans l'URL, le JSON ou le Debug de la
requête synthétique testée. Les erreurs fournisseur restent statiques, sans corps
de réponse. Une réponse de succès est bornée puis contrôlée : un seul choix,
fin `stop`, modèle servi non vide, aucun appel d'outil, JSON strict validé localement.
Réponse tronquée, volumineuse ou hors contrat = aucun effet et aucun repli.

Le prompt de fidélité et la sélection du contexte restent ceux validés dans le
prototype ; deux messages system/user par requête, sans conversation persistante.
La voie prioritaire ne peut retourner qu'une interruption typée, jamais une édition.
Le contrôleur utilise directement les nouveaux adaptateurs et conserve les
compteurs STT/LLM/priorité existants, les modèles figés et les gardes de session.
`platform/mistral.rs` devient une façade de compatibilité sans transport dupliqué.

## Vérifications

Tests écrits d'abord dans `provider_contracts.rs` : premier échec sur module
absent, puis test rouge spécifique d'annulation avant correction. Les transports
simulés passent par les mêmes méthodes de protocole et de validation que les
transports natifs ; ils n'appellent ni le trousseau ni un serveur local/distant.
La suite existante inclut aussi son test de trousseau indisponible dans un
sous-processus à adresse D-Bus invalide, sans accès au trousseau utilisateur.

| Vérification | Résultat |
| --- | --- |
| Bibliothèque Rust, dont deux tests du callback | 12 PASS |
| Harnais partagé, dont dix contrats fournisseur | 107 PASS |
| JavaScript | 48 PASS |
| Typecheck/build web | PASS |
| Check Rust toutes cibles avec `tauri/custom-protocol` | PASS |
| Format Rust et `git diff --check` | PASS |

Couverture des nouveaux contrats : acquittement obligatoire avant audio,
delta/segment/final corrigé, consommation unique, fin unique, envoi réseau en
échec sans répétition, JSON illisible et erreur expurgée, annulation d'envoi audio,
limite du passage, authentification sensible, schémas distincts ordinaires/priorité,
nombre exact de requêtes, erreur HTTP/réseau sans repli, réponse tronquée,
réponse trop volumineuse, tentative d'outil et édition sur voie prioritaire,
destruction de la requête LLM en attente lors de l'annulation.

Les suites de normalisation, de validité de session et de pipeline déjà présentes
restent exécutées sur le code réextrait. Silence, bornes et invalidation ne sont
pas assimilés à une recette matérielle nouvelle.

Logs temporaires : `/tmp/vocalnotes-adapters-red.log`,
`/tmp/vocalnotes-adapters-cancel-red.log`, `/tmp/vocalnotes-adapters-rust.log`,
`/tmp/vocalnotes-adapters-js.log`, `/tmp/vocalnotes-adapters-web.log`,
`/tmp/vocalnotes-adapters-check.log`.

## Limites et suite

Les bornes locales de 60 s d'audio par passage, 65 536 octets de réponse et 45 s
d'attente sont des paramètres d'ingénierie, pas des limites fournisseur certifiées
ni des objectifs de performance ratifiés. Détruire une future abandonne son
traitement local ; cela ne retire pas des données déjà reçues par le fournisseur.
Les tests n'attestent pas la qualité de transcription ou d'interprétation réelles.

Aucun appel micro, fournisseur ou trousseau utilisateur ; aucune installation ou nouvelle
dépendance. Windows et veille hors périmètre de cette reprise. Le binaire
principal est inchangé :
`b549a055da0e2668840ea43f93f92806fe2ed216f52e8fafe2c08dcfad10f6b9`.
Le harnais exemple permet les tests sans relier/remplacer l'exécutable ouvert.

Après fermeture volontaire puis reconstruction, reprendre une recette avec le
micro NoMachine : plusieurs passages, correction, silence, pause/arrêt et rejet
des résultats tardifs. Les données et frais externes restent soumis au démarrage
explicite informé habituel. Suite du plan : T023–T025, cycle de vie et état de session.

## Début de recette native — 2026-10-31

Fermeture de l’ancienne fenêtre confirmée par l’utilisateur puis absence de
processus vérifiée. Build web et build Rust embarqué réussis, sans génération
de paquet. Source audio par défaut vérifiée : `nx_remapped_out` (NoMachine).
Nouvelle fenêtre lancée, microphone arrêté au démarrage, session outil 39014.
Binaire : `bfa882c702560f8cb94eabc8be876d9a781c4cd298d02289311929017af5eed7`.
Journal console : `/tmp/vocalnotes-adapters-native.log`.
Recette dictée/correction/silence/pause/arrêt : résultats utilisateur en attente.
Les mentions de binaire inchangé ci-dessus décrivent la phase hors ligne.

### Premier essai vocal — ajout refusé, recette non validée

Après correction du routage système par l’utilisateur, les mesures confirment
un signal non nul ; le premier passage est transcrit puis inséré. Au deuxième
passage (« prévoir aussi des stylos », selon l’utilisateur), le fournisseur
retourne deux opérations delete puis replace. Le cœur refuse une cible absente
avec « Texte attendu absent ». Les compteurs applied=1/rejected=1 sont cumulatifs,
pas une application partielle de cette réponse. Aucun contenu utilisateur ou
corps fournisseur supplémentaire journalisé.

Ajout de deux exemples JSON synthétiques complets au prompt : ajout via insert,
correction via replace, avec interdiction explicite de supprimer une cible
avant son remplacement. Cela améliore les consignes sans garantir l’obéissance
sémantique du modèle. Nouveau test : delete suivi d’un replace impossible laisse
la source et la révision intactes ; un insert ultérieur conserve les cahiers.
Tests hors ligne : 12 bibliothèque + 108 harnais PASS. Journal :
`/tmp/vocalnotes-adapters-recipe-fix.log`. Correction non encore chargée dans la
fenêtre ; fermeture volontaire puis reconstruction et nouvel essai requis.

### Relance avec exemples corrigés

Fermeture confirmée et absence de processus vérifiée. Build natif embarqué
réussi. Nouvelle fenêtre lancée en session outil 22247, microphone arrêté ;
source par défaut `nx_remapped_out`. Binaire
`f56ec81343d953871da779d209edd548633e831328167d22f45ee5ebfc6363a6`.
Journal distinct préservant le premier essai :
`/tmp/vocalnotes-adapters-native-fix.log`. Résultat de la nouvelle recette en attente.

### Deuxième essai — clarification sans édition

Utilisateur signale un échec de traitement de sa parole. Journal de la version
avec exemples : signal audio non nul pendant un passage, final STT non vide
(34 octets), appel LLM HTTP 200, réponse clarify avec zéro action ; aucune
édition ni rejet natif sur cet essai. Trois autres finals vides sont ignorés.
Pause explicite et fermeture du flux micro constatées. La transcription brute
et la question du modèle ne sont pas dans le journal ; leur exactitude ne peut
pas être déduite de ces seules métadonnées. Recette non validée. Ne pas conclure
à une correction du problème par le seul renforcement du prompt.

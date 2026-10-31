# T026 — recette des fondations GNOME/Wayland

2026-10-31, lot B10-FOUNDATIONS. **CLÔTURE LOCALE GNOME/Wayland — GO fonctionnel avec limites ci-dessous.**
Périmètre utilisateur : Fedora/GNOME/Wayland, micro NoMachine ; Windows,
mise en veille réelle et génération/installation de paquet exclus.

Version recettée : sources T023–T025 au-dessus du commit `35276d6`.
Ne pas attribuer à cette version les recettes des versions précédentes.
Environnement relevé : Fedora 44, GNOME Shell 50.6, noyau
7.2.11-200.fc44.x86_64, Ryzen 7 5800H (16 processeurs logiques),
WebKitGTK 2.54.2. Source audio par défaut `nx_remapped_out`.

## Matrice de recette

| Contrôle | État de cette version | Preuve attendue |
| --- | --- | --- |
| Construction et lancement natifs | PASS | Builds web/Rust et fenêtre réelle |
| Capture uniquement après démarrage explicite | PASS | Console et utilisateur |
| Initialisation puis écoute effective | PASS | États et ouverture native |
| Dictée et ajout sur plusieurs passages | PASS fonctionnel, réserve fidélité | Utilisateur et trois éditions appliquées ; correction vocale non rejouée |
| Silence et avertissement tiers | PASS utilisateur | Écoute conservée et avertissement visible |
| Arrière-plan et pause vocale | PASS | Utilisateur et fermeture native |
| Pause bouton et reprise explicite | PASS | Utilisateur et console ; purge diagnostic couverte hors ligne, non confirmée séparément ici |
| Verrouillage sans reprise automatique | PASS | Second essai sur capture active, utilisateur et console |
| Surveillance indisponible/perdue | PASS sondes natives isolées | Bus indisponible avant capture ; perte de connexion via relais |
| Refus fournisseur | PASS | HTTP 403 avec modèle fictif, retour manuel et micro fermé |
| Perte physique micro, coupure réseau, résultats tardifs | Hors ligne PASS ; natif NON EXÉCUTÉ | Aucune certification matérielle déduite |
| Manuel, mise en forme, annulation, opaque | PASS | Utilisateur et disque |
| Sauvegarde, changement externe, conflit et résolution | PASS | Utilisateur, console et fichiers synthétiques |
| Préférences et trousseau | PASS ciblé | Modèles rétablis sur disque ; sonde Secret Service synthétique nettoyée |
| Fermeture et note sauvegardée | PASS | Processus absent et contenu synthétique conservé |
| Mesures et limites | Documentées | foundations-metrics.md ; aucune cible ratifiée |
| Windows, veille réelle, paquet | EXCLUS | Consignes utilisateur ; aucune certification |

Les tests hors ligne du lot B09 passent (129 Rust, 50 JavaScript), avec capture,
OS et fournisseurs simulés pour les fautes. Les fichiers de tests de stockage
utilisent de vrais fichiers temporaires. Cela ne remplace pas les observations
natives de cette matrice. Les tests de faute matérielle non réalisés restent
explicitement NON EXÉCUTÉS ; toute décision de clôture locale les qualifiera.

Aucun audio, transcription brute ou conversation n’est conservé dans ce rapport.
Les journaux ne contiennent que des messages techniques et mesures expurgées.
Les envois Mistral et le micro exigent le démarrage explicitement autorisé dans
la fenêtre ; coût et conservation fournisseur inconnus.

## Début de recette native

Ancienne instance absente, source audio NoMachine vérifiée. Build web et build
Rust embarqué PASS, sans création de paquet. Nouvelle fenêtre lancée en session
outil 63288, microphone arrêté au démarrage selon la console. Binaire :
`403f9fc77a87e753bd18b32313544fdbebec81d683f8db70c73f84668df8b4ee`.
Journal technique : `/tmp/vocalnotes-foundations-native.log`.
Build : `/tmp/vocalnotes-foundations-{web,build}.log`.
État initial de recette ; résultats finaux et décision ci-dessous.

### Premier contrôle fenêtre et pause — PASS utilisateur, réserve STT

L’utilisateur confirme : état manuel/micro arrêté au lancement, avertissement
sur les tiers visible, passage à l’écoute après démarrage explicite, insertion
d’une première dictée, puis Pause avec micro arrêté et note conservée.
Il signale cependant une substitution du verbe dans la transcription par rapport
à son énoncé. La chaîne produit une édition, mais la fidélité de cette dictée est
en défaut : réserve STT à examiner séparément, pas de validation de qualité générale.
La recette des autres invariants continue ; aucun GO global à ce stade.

### Silence, arrière-plan et pause vocale — PASS utilisateur, fidélité en réserve

L’utilisateur confirme le maintien de l’écoute après silence, l’ajout en
arrière-plan, la pause vocale effective et l’absence de retour forcé de fenêtre.
Il signale de nouveau une reformulation phonétiquement proche mais incorrecte
dans la note. La transcription brute de ce passage n’a pas été comparée par
l’utilisateur : ne pas attribuer définitivement cette erreur au STT ou au LLM.
Pause vocale validée ; fidélité non validée, diagnostic à poursuivre avant GO.

### Comparaison transcription finale / note — PASS utilisateur

L’utilisateur confirme que le dernier passage finalisé et le texte de la note
sont identiques et que ce nouvel essai fonctionne. Aucune réécriture incorrecte
par l’interpréteur constatée sur ce passage. Les approximations précédentes
restent consignées ; ce succès ne démontre pas une précision générale du STT.
La vue temporaire permet désormais de distinguer reconnaissance et interprétation
pour la suite des essais. Verrouillage GNOME et autres contrôles restent ouverts.

### Verrouillage GNOME — PASS utilisateur

Après sauvegarde de la note de test et reprise explicite de l’écoute, l’utilisateur
confirme le verrouillage/déverrouillage GNOME, le retour en manuel avec micro
arrêté, l’absence de reprise automatique et la conservation de la note. La mise
en veille n’a pas été demandée ni exécutée. Les événements lock simulés de B09
ne sont plus la seule preuve pour ce contrôle, qui est exercé sur le binaire T026.

#### Réserve sur la preuve du verrouillage

Le journal courant ne contient aucun nouveau démarrage d’écoute après la pause
précédente, ni callback de verrouillage fermant un flux actif. La confirmation
utilisateur atteste l’état au retour et la conservation de note ; elle ne suffit
pas à démontrer ici l’arrêt d’une capture active par le verrouillage. Ce contrôle
reste À CONFIRMER sur une écoute effectivement active, sans invalider la réussite
observée de la pause précédente. Ne pas utiliser le paragraphe PASS ci-dessus
comme preuve technique complète de ce scénario.

### Nouvel essai ciblé de verrouillage — confirmation utilisateur

Après consigne explicite de démarrer et d’attendre « Écoute en cours » avant
Super+L (sans pause intermédiaire), l’utilisateur confirme la réussite : manuel,
micro arrêté au retour, aucune reprise automatique. Corrélation console examinée
séparément ci-après ; ce nouvel essai remplace le précédent contrôle ambigu.

### Verrouillage et surveillance OS — PASS utilisateur et natif

Console : nouvelle initialisation à 1793478572467, écoute à 1793478572531,
invalidation OS à 1793478576098, fermeture native du flux à 1793478576100.
Le nouvel essai atteste donc une écoute active arrêtée par l’événement OS, en
complément de la confirmation utilisateur. Pas de veille provoquée. Sonde
`probe_lifecycle` PASS : abonnements et état initial disponibles. Sonde isolée
`tooling/validate-gnome-bus-loss.py` PASS : vraie connexion du moniteur au bus
GNOME interrompue côté relais de test uniquement, perte détectée, aucun micro,
trousseau ou fournisseur, aucune coupure du bus de bureau.

### Surveillance absente et trousseau isolé — PASS natif

`validate_gnome lifecycle-unavailable` avec adresse de bus invalide locale au
sous-processus : retour manuel avant capture/API, compteurs STT/LLM/audio nuls,
édition manuelle utilisable. `validate_gnome keyring` : entrée synthétique dans
un service distinct, création/remplacement/lecture/suppression PASS et absence
vérifiée après nettoyage ; clé applicative non consultée. Aucun service utilisateur
coupé et aucune clé réelle copiée dans les rapports.

### Fixture manuelle préparée

Fichier synthétique `/tmp/vocalnotes-t026-b3qkuuxx/note.md` : titre, paragraphe,
bloc de code opaque et dernier paragraphe. Comparaison de sauvegarde/conflit à
réaliser après contrôle utilisateur. Aucun fichier personnel modifié par l’agent.

### Édition manuelle, sauvegarde et opaque — PASS utilisateur/disque

L’utilisateur confirme les manipulations demandées (édition, gras/annulation,
bloc opaque et sauvegarde). Le fichier synthétique sur disque contient le
paragraphe modifié et conserve exactement la séquence du bloc de code opaque.
Empreinte avant changement externe :
`74986da8b821fff43c98cdf4c180050d262e0c0e509782087fa41d23c9440a16`.
Copie de contrôle synthétique conservée dans le dossier temporaire de recette.
L’agent ajoute ensuite uniquement à la note synthétique une ligne de modification
externe. Avertissement visible et conflit avant écrasement restent à confirmer.

### Changement externe et conflit — PASS utilisateur/disque/console

L’utilisateur confirme le scénario. Console : avertissement de modification
externe, puis conflit avec nouvelle destination requise. Le fichier original
conserve exactement l’empreinte de la version externe ; une copie locale et une
copie externe distinctes sont présentes. La modification locale effectivement
constatée est l’ajout de lignes vides (pas la phrase proposée dans la consigne).
Les deux copies conservent le bloc opaque. Aucun écrasement de la version externe.
Résolution par nouvelle destination et fermeture restent à exercer.

### Résolution du conflit par nouvelle destination — PASS utilisateur/disque

L’utilisateur confirme l’enregistrement d’une nouvelle copie `note-resolue.md`,
la disparition du signalement de conflit et la note conservée. Contrôle disque :
nouvelle copie présente ; bloc opaque intact ; original externe toujours
inchangé. La copie résolue diffère de la première copie locale du conflit,
ce qui est compatible avec une édition locale supplémentaire après ce conflit. Pas de fusion silencieuse ni de
suppression des copies de récupération. Fermeture normale encore à vérifier.

Comparaison de la copie résolue : seule différence observée avec la première
copie locale de récupération, une ligne vide remplacée par le marqueur local de
recette demandé. Cette modification locale est donc bien sauvegardée dans la
nouvelle destination, sans toucher l’original externe.

### Modèle volontairement indisponible — PASS refus et arrêt natifs

L’utilisateur signale « Manuel — STT accès refusé ». Le modèle STT enregistré
est toujours l’identifiant fictif de recette ; le LLM reste `mistral-large-latest`.
Le serveur retourne HTTP 403. Les traces montrent l’arrêt natif du flux à chaque
refus et aucune nouvelle édition acceptée sur ces essais. La copie résolue
conserve le marqueur local et le bloc opaque. Plusieurs initialisations séparées
sont visibles ; ne pas les confondre avec une réémission interne de passage.
La clé n’est ni lue par l’agent ni mise en cause par ce test, qui ne permet pas de
classer le 403 plus finement qu’un refus d’accès au modèle demandé.
Rétablissement du modèle normal, sauvegarde et fermeture encore attendus.

### Fermeture et décision finale

Après confirmation utilisateur, processus absent et session de lancement terminée
avec code 0. Préférences sur disque : STT `voxtral-mini-transcribe-realtime-3-0`,
LLM `mistral-large-latest`, dernière note à la nouvelle destination. Copie résolue
présente avec marqueur local et bloc opaque intacts ; original externe préservé.

T026 clôturée pour le périmètre fonctionnel GNOME/Wayland autorisé. Les deux
approximations de reconnaissance restent une réserve de fidélité ; le passage
comparé ensuite est correct. Le corpus et les tests US1 à partir de T027 devront
qualifier l’interprétation sans masquer les erreurs de reconnaissance. Cette
clôture ne certifie ni Windows, ni veille réelle, ni fautes matérielles non
exercées, ni précision générale du STT, ni objectif de performance ou de coût.
Les mentions « en attente » des étapes chronologiques décrivent leur état initial.

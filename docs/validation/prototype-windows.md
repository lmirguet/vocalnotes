# Prototype Windows 11 — manuel validé, recette vocale préparée

## B03-OFFLINE — contrats T010 validés, application conservée

T010 terminée dans la source et copie ciblée Windows : opérations/cibles/commandes
typées, validation entière et résolution native sans offsets LLM. **31 JS et
64 Rust ciblés MSVC PASS** (9 bibliothèque + 55 issus des huit fichiers de tests,
réutilisés dans `examples/offline_contract_tests.rs`). Source Linux : 67 Rust,
31 JS/typecheck et fmt/check PASS. Format Windows et check lib custom-protocol PASS.

Commande finale effectivement exécutée depuis WSL :
```sh
pwsh.exe -NoProfile -File 'C:\Users\lmirguet\Development\vocalnotes-native-B02-W01\tooling\windows-native.ps1' -Action OfflineTests
```
Rust 1.99.1 MSVC/VS Build Tools 18.11.3/SDK10.0.26100.0 locaux ; Node25.8.2
hors plage22 inchangé. Aucun téléchargement/install, micro, clé/trousseau,
API ou sondage réseau de recette. 14 fichiers source/test/script en allowlist,
différences et hashes pré-copie vérifiés ; aucune vraie note ni exe copié.

Premier essai de tests d'intégration via `cargo rustc --test` FAIL :
Cargo tente de retirer `vocalnotes.exe`, refus OS32/101. La garde de lecture
`FileShare.Read` interdisait écriture/suppression de l'exe ; aucune atteinte à
l'application. Remédiation : harnais example réutilisant les mêmes tests,
sans duplication ni ciblage du binaire principal ; résultat final PASS code0.
Handle libéré en `finally`, hash exe avant/après identique.

**Fenêtre existante inchangée :** aucun arrêt/relink/remplacement/relancement,
code T010 non chargé. PID22548 mentionné dans le contexte de reprise, non
resondé ici. Les états/PID/attentes B02 ci-dessous restent historiques.
Le blocage STT HTTP1.0/307 décrit ci-dessous n'a pas été retesté ni contourné.
T006 partielle/T007 ouverte, GNOME différée, aucun GO.

Arrêt de ce lot à T010 complète ; T011–T013 non commencées, prochain travail
stockage TDD avant implémentation et vrais tests Windows de courses/permissions.
La copie exclusive actuelle ne démontre pas autosave/watch/ReplaceFile.
Les nouvelles commandes fichiers vocales sont seulement formalisées, refusées
sans effet jusqu'à intégration ; pas de nouvelle recette manuelle ni vocale.

## État autoritaire B02-W05 — connexion STT, 2026-10-31

### Deuxième essai et diagnostic réseau complémentaire

Retour utilisateur à 02:44 après nouvel accord : aucun texte intégré,
`STT [ws-http-version]`, compteur « tentatives STT : 2 » ; retour Manuel
ou application fermée confirmé. Coût et audio effectivement transmis inconnus.

Deux sondages Windows natifs supplémentaires, sans authentification, audio,
lecture du trousseau ni suivi de redirection, avec TLS vérifié :
requête WebSocket HTTP/1.1 vers l'URL STT officielle ; réponse **HTTP/1.0 307**.
L'en-tête Location redirige vers **https://web-notification.capgemini.com/**
(paramètres non affichés). Seuls version/statut puis destination publique
ont été affichés, aucun corps, cookie ou autre en-tête.
Cette réponse explique l'erreur de version du sondage : le handshake reçoit
une redirection vers une page de notification, pas un upgrade WebSocket Mistral.
Elle est cohérente avec une interception/filtrage réseau d'entreprise ;
la règle exacte et l'origine du dispositif restent à confirmer avec l'IT.
Aucune redirection suivie, aucun en-tête d'authentification transmis à cette
destination et aucun contournement TLS ou réseau ajouté. Suspendre les
nouveaux essais authentifiés jusqu'à résolution autorisée de ce blocage.

**Nouvel accord utilisateur à 02:44 :** essai de 30 secondes maximum avec
microphone, données fictives sans paroles de tiers et appels Mistral de coût
possible explicitement autorisé après information sur coût/conservation
inconnus et données transmises. Arrêt dès l'erreur ou au plus tard à 30 secondes,
sans boucle de relance ; uniquement la nouvelle clé au trousseau.
Résultat de cette seconde tentative à recueillir, pas de succès présumé.

**Suite après confirmation utilisateur :** application précédente déjà fermée,
texte récupéré ou aucune saisie à conserver ; remplacement et relance autorisés.
Build/link Windows MSVC avec `tauri/custom-protocol` réussi (code 0).
Nouvelle fenêtre **PID 22548**, titre **Bloc-notes vocal**, `Responding=True`,
session détachée **windows-ui-stt-diagnostic** active. Les diagnostics expurgés
sont désormais chargés ; les mentions de PID16144 conservé ci-dessous sont
historiques. Aucun nouvel essai vocal ou appel authentifié effectué lors de
cette relance ; un nouvel accord est requis avant la prochaine tentative.

**Premier passage vocal Windows : FAIL utilisateur.** Après l'accord explicite
≤30 s/données fictives/coût possible et le stockage confirmé de la nouvelle
clé (champ vidé, « Clé présente au trousseau »), le vocal démarre mais aucun
texte n'est intégré. Message : « Connexion STT impossible (réseau, TLS ou
compte) ». Ces résultats remplacent les attentes W04 ci-dessous, conservées
comme historique. Compteurs, durée réelle, audio transmis et facturation
inconnus ; aucun résultat sémantique certifié. T006/T007 restent ouvertes.

**Consigne immédiate : Arrêter / Manuel, aucun nouvel essai automatique.**
L'agent n'a ouvert ni micro ni trousseau, n'a reçu aucune valeur de clé et
n'a fait aucun appel authentifié. L'accord du premier essai n'est pas réutilisé.
PID16144 est conservé : ne pas fermer/forcer ni remplacer son exe sans
récupération/sauvegarde du texte et confirmation utilisateur.

### Preuves bornées, sans authentification

- Source et copie Windows du transport/lockfile identiques avant diagnostic.
- HTTPS Windows `Invoke-WebRequest`, vérification TLS normale, URI STT fixe,
  sans en-tête d'authentification : **HTTP 200**. Corps et en-têtes non affichés.
  Cela ne prouve pas un upgrade WebSocket ni les droits du compte.
- Trois exécutions réseau du petit exemple MSVC `probe_stt_transport`,
  chacune limitée à un handshake sans clé, sans payload audio/session.update
  et sans retry : protocole, puis catégorie expurgée, puis sous-type exact
  **`ProtocolError::WrongHttpVersion`**. Aucune session établie.
  Une tentative intermédiaire échoue à compiler (fonction insérée dans
  le mauvais bloc), donc **aucun contact réseau** ; corrigée avant continuation.
  Aucun autre sondage ensuite.
- Test Windows de la requête réelle : **GET / WSS / HTTP/1.1**, host,
  Connection/Upgrade/version13, URL/modèle corrects. Source
  `tungstenite-0.28.0/src/handshake/client.rs` : `WrongHttpVersion` est émis
  sur une requête <1.1 **ou**, après réception, sur une réponse HTTP/1.0.
  La requête 1.1 est prouvée : la réponse reçue est donc HTTP/1.0.
  Le parsing refuse cette version **avant lecture du statut HTTP**.
- Rejeu entièrement en mémoire **PASS Linux et Windows** : une réponse
  synthétique HTTP/1.0 401 produit `WrongHttpVersion`, HTTP/1.1 401 produit
  `Error::Http` → authentification. Ce **401 est synthétique**, pas le statut
  du service réellement observé. Ne pas conclure clé invalide/quota.
- La réception HTTP dans le probe WSS démontre TCP et TLS rustls vérifié
  réussis pour ces sondages natifs. Lockfile : rustls0.23.46/ring seul ;
  roots natifs Windows via schannel, pas hyper/reqwest dans ce chemin WSS.
  Aucun panic/provider ni défaut de confiance observé. Timeout non observé.
- **Confirmé :** le diagnostic générique produit confond les catégories ;
  le probe Windows rencontre un HTTP/1.0 incompatible avec WebSocket.
  **Inconnu :** origine service/intermédiaire de cette réponse, statut
  masqué, droits/clé/quota du compte, et catégorie exacte de l'essai vocal
  authentifié initial (détail éliminé par l'ancien code). Même cause plausible,
  **pas prouvée** pour cet essai ; pas de capture/log secret pour la retrouver.

Référence publique vérifiée : [SDK officiel, construction URL et
headers](https://github.com/mistralai/client-python/blob/90e300904b64c36e0630f72ef475dbab7912068e/src/mistralai/extra/realtime/transcription.py).
Endpoint, query modèle et Authorization Bearer conformes à cette source.
La présence de clé au trousseau ne prouve pas sa validité serveur.

### Correctif limité au diagnostic, pas contournement réseau

`platform/stt_transport.rs` construit la même requête et classe les erreurs
par variantes/status, jamais par texte distant. Message attendu dans le
chemin observé : **« STT [ws-http-version] : version HTTP incompatible avec
le handshake WebSocket. »** Autres catégories : réseau/IO, timeout, TLS,
certificat TLS, HTTP401, 403, 429 (quota **ou** rate limit), autres4xx/5xx,
upgrade/protocole/fermeture/configuration/capacité. Corps/en-têtes non
rapportés ; header d'authentification toujours sensible. Les erreurs rustls
encapsulées en `Error::Io` par tokio-tungstenite sont reconnues par downcast
typé, pas faussement classées réseau. Le contrôleur existant conserve son
arrêt terminal et sa barrière anti-résultat tardif ; aucun retry ajouté.

TLS, confiance système et proxy inchangés ; pas de fallback fournisseur,
pas d'acceptation d'HTTP/1.0. On ne « corrige » pas la réponse incompatible
en affaiblissant la validation du client.

### Validation et disponibilité

- Linux : **31 JS + 51 Rust PASS**, typecheck/build frontend, fmt/check
  custom-protocol PASS ; cinq nouveaux tests synthétiques de transport.
- Windows MSVC : **31 JS + 9 tests Rust de bibliothèque PASS**, dont les
  cinq nouveaux tests ; typecheck/build frontend, fmt, check et
  **build natif bibliothèque custom-protocol PASS**.
  Les 42 autres tests Rust Windows précédents n'ont **pas** été rejoués :
  les targets d'intégration Cargo peuvent reconstruire l'exe actif.
- Synchronisation ciblée de sept fichiers source/script/manifeste/lockfile,
  parité SHA-256 PASS, aucun parcours/copie de secret ou données.
  rustls déjà verrouillé : ajout comme dépendance directe pour downcast ;
  aucune mise à niveau de paquet. NodeWindows25.8.2 hors cible22 inchangé.
- **Nouveau link de vocalnotes.exe NON EXÉCUTÉ**, instance actuelle encore
  W03 : bibliothèque testée ≠ correctif chargé dans l'UI.
  Contrôle final PID16144 répondant, SHA-256 exe inchangé
  `CC67C1CCC3E1917E3086A00F2C1FA062A514E5D23E0549064B5E49A52FC25923`.
  Garde de protection respectée ; aucun kill/fermeture/relancement.
  Pas de ghost UI, succès vocal ou GO certifié. WTS PASS antérieur conservé,
  lock/veille non recettés ; GNOME différée.

### Prochaine étape sûre — ne pas redémarrer le vocal

Demander d'abord : **« Avez-vous cliqué Arrêter et retrouvé Manuel ?
Ne relancez pas le vocal. »** Ensuite récupération/sauvegarde de la note et
fermeture propre uniquement après confirmation, avant build/link du nouvel
exe. Ce déploiement ne résout pas l'origine HTTP/1.0 ; établir un handshake
conforme sans clé avant toute nouvelle recette. Aucun changement système,
certificats/proxy/réseau partagé sans nouvel accord.

Si une nouvelle recette authentifiée devient nécessaire, demander un
**nouveau consentement contextualisé** (mêmes destinations audio Voxtral,
contexte/texte et priorité Mistral Large, données fictives, ≤30s et coût
possible), puis action volontaire de l'utilisateur uniquement.
Rapporter catégorie/compteurs expurgés, jamais clé, contenu ou réponse brute.
Rapport borné : [B02-W05](../../specs/001-bloc-notes-vocal/diagnostics/B02-W05/batch-report.yaml).

**Consentement utilisateur, suite B02-W04 :** après présentation des données
envoyées, du coût inconnu potentiellement non nul et des conditions fournisseur
non vérifiées, l'utilisateur accepte explicitement un essai microphone et
Mistral de 30 secondes maximum avec données fictives, arrêté par lui-même.
Cet accord ne constitue pas une preuve de capture ou d'appel effectué.
Configuration d'une nouvelle clé au trousseau à vérifier avant le démarrage.

**Retour utilisateur, configuration sécurisée :** nouvelle clé enregistrée,
champ vidé et message « Clé présente au trousseau ». Vérification interactive
Windows du stockage et de la présence réussie ; aucune valeur de clé recueillie.
L'accès API et la capture restent à tester.

## État actuel — préparation vocale B02-W04, 2026-10-31 à 02:23

Cette section et le retour utilisateur W03 à 02:20 font autorité sur les
anciens états « link/recette en attente » conservés comme historique ci-dessous.
**T006 partielle, T007 ouverte ; aucun GO multiplateforme.**

### Préflight réellement exécuté, sans capture ni service

- Aucune instance `vocalnotes` présente avant lancement. L'ancienne W03
  PID 19624/session `windows-ui-editing` a terminé normalement (code 0,
  constat du tour précédent). Pas de fermeture forcée dans cette préparation.
- `pwsh.exe -NoProfile -File
  C:\Users\lmirguet\Development\vocalnotes-native-B02-W01\tooling\windows-native.ps1
  -Action Probe` : **PASS, code 0**, compilation/exécution Windows MSVC de
  `probe_lifecycle`, « Surveillance OS initialisée ». Abonnement WTS et état
  initial établis ; **aucun événement verrouillage/veille testé**.
  Rust 1.99.1 MSVC, VS Build Tools 18.11.3, SDK 10.0.26100.0 ;
  Node 25.8.2/npm 11.8.0, écart Node 22 maintenu.
- Lecture ciblée de `Value` seulement dans ConsentStore/microphone :
  HKCU, HKLM et HKCU/NonPackaged = **Allow**. Aucune modification de permission,
  lecture de sous-clés d'autres applications, ouverture de périphérique ou capture.
  Ce réglage ne prouve pas WASAPI/CPAL fonctionnel. Confirmer dans Windows
  **Paramètres → Confidentialité et sécurité → Microphone**, notamment
  l'accès des applications de bureau ; ne pas lancer de test d'enregistrement.
- Parité SHA-256 source/copie PASS pour UI, contrôleur, capture, secrets,
  surveillance Windows et script. Aucun correctif préflight nécessaire,
  aucun manifeste changé, aucune installation ou copie de secret.
- Fenêtre native lancée par script **au premier plan de son processus outil
  détaché**, sans Start-Process ni serveur Vite :
  session **`windows-voice-recipe`**, PID **16144**, début
  **2026-10-31T02:22:50+01:00**, titre **Bloc-notes vocal**,
  HWND **658332**, `Responding=True`. Assets embarqués/custom-protocol.
  Exe W03 : 32 254 976 octets, écrit à 02:14:28 ;
  SHA-256 `CC67C1CCC3E1917E3086A00F2C1FA062A514E5D23E0549064B5E49A52FC25923`.
  Cela prouve une fenêtre répondante, pas ses pixels ni une recette audio.
- Une première commande de lecture des permissions a échoué (syntaxe de
  pipeline PowerShell, code 1) ; corrigée, lecture seule finale code 0.
  Aucun défaut produit constaté. Les 31 JS/46 Rust et builds W03 sont des
  preuves précédentes, **pas des suites rejouées dans W04**.

### Ordre natif et états à observer

Le code actuel exige accord IPC puis démarrage volontaire, initialise la
surveillance OS, vérifie la présence au trousseau, puis ouvre CPAL sur le
microphone d'entrée Windows par défaut. Il ne teste pas la clé auprès de
Mistral lors de son enregistrement. Les connexions STT commencent à la
détection locale de parole ; ne pas prendre « Vocal » seul pour preuve API.

- Au lancement attendu : **Manuel**, accord décoché, Démarrer désactivé,
  compteurs STT/LLM/priorité et audio à zéro (à confirmer par l'utilisateur).
- Après Démarrer : « Initialisation de la surveillance OS puis du microphone ;
  coût Mistral inconnu. » puis « Écoute — silence sans arrêt… » si capture établie.
- Après silence : « Traitement du passage finalisé — aucune édition sur
  provisoire. » puis acceptation, clarification ou erreur dans la fenêtre.
  Les compteurs sont des tentatives, pas des factures ; modèles servis/délais
  absents restent inconnus/non mesurés.
- Surveillance absente/verrouillée : refus avant capture. Clé absente :
  « Clé Mistral absente du trousseau : vocal refusé avant capture. ».
  Trousseau, micro, réseau, quota ou schéma refusés : erreur expurgée et arrêt ;
  **ne pas contourner**, ne pas relancer en boucle. Rapporter seulement le
  message visible et les compteurs, sans clé, audio ni logs de requêtes.

### Consentement à obtenir — NON REÇU

L'accord UI/fichier de 02:20 ne vaut pas accord vocal/API. Aucun outil
générique `ask_user` n'est exposé : demander dans le chat, sans formulaire
de réarchitecture détourné et sans enregistrer un accord fictif.

**Question à présenter :** « Acceptez-vous l'activation volontaire du microphone
et les appels Mistral pour un essai synthétique de 30 secondes maximum ?
L'audio sera envoyé à Voxtral realtime (`voxtral-mini-transcribe-realtime-2602`) ;
les passages finalisés, l'intention et les sections éditables courantes seront
envoyés à `mistral-large-latest`. Les provisoires déclenchent aussi des requêtes
Mistral de priorité pour l'interruption. Coût inconnu, potentiellement non nul ;
tarifs et conservation fournisseur non vérifiés. Uniquement données fictives,
aucun contenu sensible ni paroles de tiers. Oui/non ? »

### Recette graduelle proposée — NON EXÉCUTÉE

Ne pas soumettre tout le protocole A–K à la fois. Après réponse explicite :

1. **Configuration seule en Manuel** : révoquer l'ancienne clé exposée et
   générer une nouvelle clé soi-même. Saisir celle-ci uniquement dans
   « Clé Mistral (trousseau uniquement) », champ masqué ; Enregistrer la clé.
   Attendu : champ vidé, « Opération trousseau effectuée. Aucune valeur
   renvoyée. ». Vérifier présence : « Clé présente au trousseau. ».
   Backend Windows Credential Manager, service `fr.vocalnotes.desktop`,
   compte `mistral`. Aucune clé au chat, CLI, env, fichier ou pièce jointe.
   Échec = « Échec du trousseau ; aucun stockage de substitution. » :
   **arrêter cette étape**, aucun fallback. Demander ce premier résultat
   avant d'inviter au démarrage. Ne remplacer un credential que volontairement.
2. **Premier passage**, après confirmation de configuration : note vide
   synthétique en Manuel, Aide lue, cocher l'accord puis cliquer soi-même
   Démarrer le vocal. Dire :
   « Le projet fictif Atlas démarre lundi. Son budget prévu est de cent euros. »
   Garder ensuite le silence, attendre traitement et note. Observer état,
   tentatives STT/LLM/priorité, audio transmis et erreurs, sans publier
   de transcription ni vrai contenu. L'exactitude n'est pas présumée.
   **Arrêter le vocal au plus tard après 30 s**, même sans note obtenue.
   Cette borne est une consigne utilisateur, pas un minuteur produit ;
   des requêtes déjà envoyées peuvent avoir un coût malgré l'arrêt.
   Rapporter uniquement succès/échec et message/compteurs expurgés.
3. **Puis seulement si le premier passage réussit et l'utilisateur souhaite
   poursuivre**, nouvelle entrée explicite courte : « Remplace le budget du
   projet Atlas par deux cents euros. ». Silence puis vérifier que seul
   le budget change. Pause : état **Pause**, micro éteint ; aucune reprise
   sans « Reprendre explicitement ». Arrêter : **Manuel**, micro éteint,
   aucun résultat tardif ne change la note. Sauvegarder une copie exclusive
   si voulu ; aucune écriture silencieuse dans un fichier existant.

Arrêter immédiatement en cas d'erreur/quota, perte de surveillance ou capture
inattendue. Le témoin OS éventuel reste sous l'autorité Windows, pas un
indicateur ajouté par l'application. Ambiguïtés, arrière-plan, pause naturelle,
lock/veille et pertes périphérique/réseau sont des recettes **séparées**,
déclenchées seulement par l'utilisateur ; GNOME reste différée.

**Résultats actuels :** manuel Entrée/copie/nouvelle note PASS utilisateur W03 ;
barre PASS à 01:57. Vocal, vrai trousseau, WASAPI, Mistral et événements OS
**NON EXÉCUTÉS**. Aucun accès clé/trousseau, micro ou appel fournisseur par
l'agent. Constitution 3.0.0 inchangée.

La fenêtre PID 16144 est laissée disponible. Sortie sûre :
**Arrêter le vocal → attendre les acquittements → sauvegarder si nécessaire →
Fichier → Fermer après sauvegarde**. Sans destination, choisir un chemin neuf
si fermeture refusée ; ne pas forcer X. Après fermeture propre seulement,
arrêter le helper `windows-voice-recipe` s'il subsiste. Ne pas réutiliser les
PID/sessions historiques ci-dessous.

2026-10-31, **B02-W01**. **T006 partielle, T007 ouverte ; aucun GO.**
Source autoritaire : `/home/laurent/projects/vocalnotes` (Ubuntu 24.04 WSL2).
Copie native créée exclusivement, sans écrasement :
`C:\Users\lmirguet\Development\vocalnotes-native-B02-W01`.

## B02-W03 — Entrée/focus et actions Fichier : corrigés et relancés

**Retour utilisateur du 2026-10-31 à 02:20 :** « OK, les tests sont
concluants », en réponse à la recette manuelle proposée : deux paragraphes
avec Entrée, enregistrement d'une copie Markdown avec destination Windows
et nouvelle note. Recette manuelle Windows validée par l'utilisateur pour
ce périmètre. Le processus de cette instance s'est terminé avec le code 0.
Ce résultat ne valide pas le parcours vocal, les événements système ni
GNOME/Wayland ; T006/T007 restent ouvertes.

**Suite après accord utilisateur :** l'utilisateur confirme que son texte
est récupéré ou qu'aucune saisie n'est à conserver et autorise l'arrêt puis
la relance. Ancienne instance PID 19316 arrêtée après vérification du chemin.
Build/link Windows MSVC avec `tauri/custom-protocol` réussi (code 0).
Nouvelle fenêtre native **PID 19624**, titre **Bloc-notes vocal**,
`Responding=True`, session détachée **windows-ui-editing** conservée.
Le correctif est désormais actif dans l'exécutable ; les mentions de
relancement suspendu plus bas décrivent l'état antérieur. La recette
interactive Entrée et actions Fichier reste à confirmer par l'utilisateur.
Aucun microphone ni appel Mistral démarré.

Retour utilisateur du 2026-10-31 après validation de la barre à **01:57** :
impossible de créer deux paragraphes ; Entrée fait perdre le focus ;
Nouvelle note et Enregistrer une nouvelle copie semblent sans effet.
**Recette manuelle Windows FAIL** pour ces interactions sur B02-W02.
Le **PASS visuel de la barre de menus reste acquis**, pas celui des commandes.

### Diagnostic et correctifs source

- `baseKeymap` reçoit bien Entrée. Le sérialiseur Markdown supprime le
  paragraphe vide final ; le diff reconnaît aussi le premier paragraphe inchangé
  et omet le séparateur lors d'une insertion vide. L'ack structurel reconstruit
  ensuite tout l'état PM, avec perte de sélection/focus. Une structure en
  attente interdit en outre la saisie et Entrée est refusée après une frappe
  non acquittée. Les nouveaux tests reproduisaient ces défauts avant correctif.
- Sérialisation **du seul segment modifié**, séparateurs `\n\n` explicites,
  plages UTF-8 de projection alignées sur les paragraphes PM, y compris vides.
  Conservation du document/sélection/focus sur ack accepté, sans annoncer la
  projection sauvegardée ; file de propositions révisionnées autorisant
  saisie → Entrée → saisie avant ack. Historique exclusivement acquitté,
  undo/redo restaurent aussi les insertions vides ; refus = resynchronisation
  canonique et projection complète récupérable, jamais rejeu.
- Les clics disclosure atteignent bien `prototype_action`. Les commandes natives
  refusent une nouvelle note modifiée non sauvegardée ou une destination absente.
  L'UI masquait leur motif dans un catch générique puis le polling l'effaçait.
  Messages statiques natifs **allowlistés exactement**, persistants jusqu'à une
  nouvelle opération/transition ; exceptions arbitraires toujours expurgées.
  Nouvelle note réussie affiche désormais « Nouvelle note créée — manuel,
  aucune capture », même depuis une note vide. Pas de contournement du refus
  de perdre une note modifiée, ni d'écrasement de fichier existant.

### Preuves et limites

- **31/31 JS**, typecheck et build Vite PASS, source Linux et **Node Windows
  natif 25.8.2** (écart Node 22 inchangé). Six nouveaux tests exécutent
  l'événement clavier Entrée dans le vrai keymap PM, sélection/focus après ack,
  saisie rapide/undo/redo, refus sans rejeu, UTF-8, îlot opaque/CRLF hors cible,
  polling et clics Fichier avec réponses/refus. Transport déterministe de test ;
  jsdom n'a pas de géométrie Range : seul le scroll est neutralisé.
  **Ce n'est pas une preuve de gestes/pixels WebView2 ni d'IPC Tauri en recette.**
- **46/46 Rust Linux**, format/check PASS. **46/46 Rust MSVC Windows** exécutés
  directement dans neuf artefacts de tests distincts compilés à 02:07–02:08,
  après synchronisation. Les 11 contrats incluent Entrée/saisie/undo avec
  octets opaques intacts et un fichier Markdown **synthétique**, créé
  exclusivement dans un répertoire temporaire natif, relu avec comparaison
  exacte `un\n\ndeux`, refus d'écrasement vérifié, puis supprimé.
  Aucune vraie note inspectée ni modifiée.
- `cargo check --locked --lib --features tauri/custom-protocol` **MSVC PASS**.
  **Pas de nouveau build/link de l'exécutable application**, pas de relancement.
  La première commande `windows-native.ps1 -Action Validate` a passé JS/build/fmt
  mais `cargo test --tests` a tenté de retirer l'exe actif : **FAIL 101,
  accès refusé Windows**. L'exe est resté inchangé. Aucun kill/fermeture forcée.
  Le script protège désormais Validate/Build **avant Cargo** si ce chemin exe
  est actif ; refus attendu vérifié sur PID 19316. Les tests distincts ci-dessus
  sont réellement exécutés, pas un cargo test global annoncé réussi.
- Cinq fichiers source synchronisés avec contrôle SHA-256/conflit préalable :
  `src/ui/Prototype.ts`, `src/ui/Prototype.editor.test.ts`,
  `src-tauri/src/domain/note.rs`, `src-tauri/tests/prototype_contracts.rs`,
  `tooling/windows-native.ps1`. Aucun secret, `.env`, dépendance ou target Linux
  copié ; aucun changement de constitution, locks ou permissions IPC.
- **Instance actuelle B02-W02 inchangée**, PID **19316**, session
  **windows-ui-menus**, chemin exe indiqué ci-dessus, `Responding=True`.
  Exe : 32 254 976 octets, dernière écriture **01:56:23**,
  SHA-256 `634663A67E76C9CCD22BAA221F7125654EFF5733E82445D9092516E6D4E36099`.
  Cette fenêtre peut contenir du texte utilisateur : elle reste disponible
  pour le copier. **Correction W03 non recettée et non active dans cette fenêtre.**

### Accord requis puis prochaine recette, sans micro

Récupérer/copier le texte actuel ou confirmer explicitement qu'il n'y a rien à
garder. Confirmer ensuite l'autorisation de fermer **cette** instance, compiler
et relancer. L'ancien accord d'arrêt de PID 25360 ne vaut pas pour PID 19316.
Si la fermeture est bloquée faute de destination, ne pas forcer avant cet accord.

Après fermeture confirmée seulement : vérifier absence du PID/chemin, terminer
le helper spécifique si nécessaire, `windows-native.ps1 -Action Build`
(build/link final MSVC avec assets embarqués), lancer via helper détaché et
relever nouveau PID/titre/réponse/empreinte sans prétendre preuve de pixels.

Recette W03 à confirmer par l'utilisateur après relancement :
1. Nouvelle note depuis une note vide : résultat explicite visible.
2. Écrire `un`, Entrée, `deux` rapidement : deux paragraphes, curseur dans le
   second, focus conservé. Tester Entrée au milieu et Annuler/Rétablir.
3. Sans destination : sauvegarde refusée avec motif toujours lisible après 2 s.
   Nouvelle note sur note modifiée : refus clair, contenu conservé.
4. Destination absolue vers **un nouveau** `.md` synthétique local : copie
   réussie, contenu Markdown relu hors application, puis Nouvelle note réussie.
   Un chemin déjà existant doit être refusé sans écrasement.

T006 partielle/T007 ouverte, T008–T009 cochées, T010 partielle inchangées.
GNOME différée avec dérogation de développement ; aucun GO multiplateforme.
Aucun micro, appel API ou accès trousseau réalisé dans ce lot.

## B02-W02 — menus corrigés, nouvel exécutable lancé

**Retour utilisateur du 2026-10-31 à 01:57 :** « j'ai vérifié la barre
de menu. C'est OK. » La barre de menus est validée visuellement sur Windows
par l'utilisateur. Ce constat ne certifie pas séparément chaque interaction
clavier ni les autres étapes de recette T006/T007.

Suite de recette : l'utilisateur indique que la croix ne ferme pas la fenêtre,
sans destination renseignée et sans saisie à conserver. L'ancienne instance
PID 25360 a été arrêtée spécifiquement après vérification de son chemin.
La croix reste soumise au protocole de sauvegarde ; aucun contournement
de cette protection n'a été ajouté.

Le build/link Windows MSVC avec `tauri/custom-protocol` a ensuite réussi
(code 0). Nouvelle instance native **PID 19316**, titre **Bloc-notes vocal**,
`Responding=True`, session détachée **windows-ui-menus** conservée pour la
recette utilisateur. Aucun serveur Vite, micro ou appel Mistral démarré.
Les paragraphes d'attente ci-dessous décrivent l'état antérieur, désormais
résolu ; la vérification visuelle des nouveaux menus reste à réaliser.

Source et copie native corrigées : barre horizontale **Fichier / Édition / Aide**,
trois boutons et panneaux déroulants superposés, jamais tous visibles.
Actions/chemin et bandeau distinct conservés. Choix disclosure DOM :
Enter/Espace activent les boutons, Tab parcourt les commandes et le champ,
flèches gauche/droite et Home/End parcourent les intitulés, Escape referme
et rend le focus à l'intitulé ; une action referme, un clic extérieur ou
une sortie par Tab referme sans reprendre le focus à la cible extérieure.
Pas de rôles ARIA menu/menubar ni de migration vers un menu Windows natif.

**Preuves actuelles :** 25/25 tests JS Windows et source Linux (13 du prototype),
typecheck/build Vite PASS ; check Cargo 1.99.1 **MSVC natif**, binaire cible
avec `tauri/custom-protocol`, PASS. CSS horizontal/positionnement/masquage
et interactions jsdom vérifiés ; **aucune preuve de pixels WebView2**.
Les trois fichiers synchronisés ont été contrôlés SHA-256 avant/après :
aucun conflit utilisateur, secret ou sortie Linux copié ; Rust/locks inchangés.
Pas de nouvelle suite Rust complète (44 tests B02-W01 historiques).

**État antérieur résolu :** nouveau link/build exe et relancement alors NON EXÉCUTÉS.
L'instance répondante PID **25360**, session détachée **windows-ui**, reste
sur l'ancien B02-W01 ; l'empreinte exe ci-dessous est inchangée.
L'outil ask_user n'étant pas exposé, confirmation à obtenir dans le chat :
**sauvegardez/récupérez toute saisie puis fermez proprement via Fichier →
Fermer après sauvegarde ; confirmez que la fenêtre est fermée.**
Ne pas forcer X si la sauvegarde est refusée. Ensuite seulement l'agent
contrôlera l'absence du processus, arrêtera ce helper s'il reste actif,
compilera/linkera et relancera l'exe natif dans une session détachée autorisée.
Pas de Vite/WSLg ni d'arrêt brutal en attendant.

**Après relancement, recette UI sans micro/clé :**
1. Confirmer les trois intitulés sur **une même ligne** et les panneaux fermés.
2. Ouvrir Fichier puis Édition puis Aide : un seul panneau, note non repoussée ;
   recliquer le même intitulé referme. Champ chemin et toutes actions accessibles.
3. Tester clic dans la note et Escape ; Tab vers/depuis le champ, Enter/Espace
   sur un intitulé et flèches/Home/End. Signaler tout focus perdu ou champ bloqué.
4. Puis tester manuel synthétique/Gras/Titre/Annuler et copie Markdown selon
   les étapes ci-dessous, sans cliquer Démarrer ni cocher l'accord.
Rapporter attendu/observé PASS/FAIL ; T006/T007 restent ouvertes jusqu'aux
recettes requises, GNOME différée. Le vocal est une étape ultérieure avec
consentement explicite distinct ; **ne jamais utiliser la clé exposée**, dont
la révocation a été conseillée. Aucune clé, vraie note ou transcription au chat.

## Historique B02-W01 — ancien binaire remplacé

## Préparation et preuves automatisées

| Élément | Observé sur Windows |
|---|---|
| OS | Windows 11 Professionnel, 10.0.26200, build 26200 (sondage natif préalable) |
| Rust/Cargo | 1.99.1, host `x86_64-pc-windows-msvc`, sous `C:\Users\lmirguet\.cargo\bin` |
| Build Tools | Visual Studio 2026 Build Tools 18.11.3, installation `C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools` |
| C++/SDK | cl 19.51.36260 x64, MSVC 14.51.36231, SDK 10.0.26100.0, link.exe/rc.exe réels |
| Node/npm | `C:\Program Files\nodejs`, 25.8.2 / 11.8.0 ; **hors plage Node 22 du projet** |
| WebView2 | Runtime 156.0.4314.50, déjà installé ; aucune installation silencieuse |
| Copie | 55 fichiers allowlist avec égalité SHA-256 ; sources/config/locks/icônes/tests uniquement |
| Tests natifs Windows | 44 Rust et 22 JS PASS, 0 échec/ignoré ; format/check/build PASS, codes 0 |
| Source Linux finale | 44 Rust et 22 JS PASS, fmt/check/build natif Linux et frontend/typecheck PASS |
| Lifecycle natif | `probe_lifecycle.exe` : initialisation WTS PASS, code 0 ; **pas de test lock/suspend** |
| Lancement | Fenêtre `Bloc-notes vocal`, PID 25360, HWND 396550, répond ; WebView2 enfant PID 9376 |

Binaire Windows PE x64, debug avec ressources web **embarquées** :
`C:\Users\lmirguet\Development\vocalnotes-native-B02-W01\src-tauri\target\debug\vocalnotes.exe`
(32 254 976 octets).
SHA-256 : `1144BB8DA9C606C2D4FEC8804537AAD210BCB0337C483AE1C02F05C6AF8546B2`.
Lancement à 01:39:23 CET ; session outil détachée **`windows-ui`** conservée.
Pas de serveur Vite, navigateur substitut, build cross-Linux ni paquet NSIS.
PID/empreinte correspondent à cette exécution, à revérifier après relancement.

Le premier `npm test` Windows a échoué (21/22) sur un séparateur `/`
codé en dur dans `tests/scaffold.test.ts`. Correction minimale : comparer au
chemin absolu natif obtenu par `fileURLToPath`/`resolve`, sans réduire le
contrôle de confinement. Source Linux et copie Windows corrigées, reruns verts.
Un premier `WaitForInputIdle` a échoué pendant l'initialisation ; contrôle
ultérieur stable du PID, HWND, titre, réponse et enfant WebView2 réussi.
Cela **ne prouve pas le contenu rendu ou les gestes utilisateur**.

Le sondage initial `rustup toolchain list` depuis le projet UNC a provoqué
l'auto-installation implicite de la toolchain épinglée 1.99.1 par rustup.
Ce téléchargement utilisateur n'était pas intentionnel ; scripts suivants :
`RUSTUP_AUTO_INSTALL=0`, toolchain explicite. Aucun SDK, Node, VS ou WebView2
installé par l'agent ; seuls `npm ci` et les dépendances Cargo du projet
ont ensuite été restaurés. Aucun PATH/profil global modifié.
Les avertissements `EBADENGINE` Node 25 restent un écart non résolu :
résultats obtenus avec cette version, pas certification Node 22 Windows.

## Commandes reproductibles

Depuis **PowerShell 7 Windows**, sans ouvrir une seconde instance :

```powershell
Set-Location 'C:\Users\lmirguet\Development\vocalnotes-native-B02-W01'
.\tooling\windows-native.ps1 -Action Build
.\tooling\windows-native.ps1 -Action Probe
# Seulement après fermeture de l'instance actuelle :
.\tooling\windows-native.ps1 -Action Run
```

`Build` utilise les exécutables Windows, vswhere et le dev shell x64,
`npm ci --no-audit --no-fund` si dépendances absentes, `npm test`,
`npm run build` (inclut typecheck), `cargo +1.99.1 fmt/test/check --locked`
(sauf fmt), puis `cargo +1.99.1 build --locked --features tauri/custom-protocol`.
Pas de génération d'installateur ni téléchargement runtime.
Pour une copie neuve depuis WSL :

```bash
python3 tooling/copy-windows-source.py \
  /mnt/c/Users/lmirguet/Development/vocalnotes-native-B02-W02
```

La copie refuse une destination existante. Aucune copie de clé, pièce jointe,
`.env`, notes, `node_modules`, `target` ou `dist` Linux ; pas de miroir destructif.
Les correctifs restent à faire dans la source Linux puis à reporter uniquement
sur les fichiers connus de la copie détenue.

## Recette guidée T006/T007 — commencer sans microphone

**Premier constat demandé :** voyez-vous la fenêtre, les menus
Fichier/Édition/Aide, la note et l'état **Manuel**, sans erreur ni écoute ?
Ne cochez pas l'autorisation et ne cliquez pas sur Démarrer le vocal.
Rapportez seulement l'état/erreur expurgée, jamais la clé ou une vraie note.

1. **Manuel et intégrité (G07)** : saisir une courte note synthétique avec
   accents (« Budget test : 12 euros, décision à confirmer. »), appliquer
   Gras/Titre, puis Édition → Annuler. Attendre la fin des acquittements :
   aucune « projection en attente ». Confirmer absence de double application.
2. **Sauvegarde réelle** : Fichier → destination Markdown locale :
   `C:\Users\lmirguet\Development\vocalnotes-native-B02-W01\recette-T006-001.md`
   **si ce nom est libre**, puis « Enregistrer une nouvelle copie ».
   Relire le fichier dans un éditeur Windows, rouvrir via le champ et Ouvrir.
   Comparer Unicode et contenu. Retenter ce même chemin doit refuser
   l'écrasement et garder la note. Ne tester droits/disque plein que dans un
   dossier isolé, sans toucher à des fichiers existants.
3. **Trousseau (G06), ultérieurement et volontairement** : révoquer la clé
   exposée, ne pas la réutiliser ; saisir soi-même une nouvelle clé dans
   « Clé Mistral (trousseau uniquement) », puis Enregistrer la clé.
   Champ immédiatement vide, retour limité au succès ou à l'erreur ;
   Vérifier présence ne renvoie que présence/absence.
   Credential Manager utilise service `fr.vocalnotes.desktop`, compte `mistral`.
   Aucune clé à transmettre au chat, fichier, CLI ou variable d'environnement.
   Ne supprimer/remplacer un credential préexistant qu'intentionnellement.
4. **Entrée et services (G02/G03/G08)** : lire Aide et l'accord
   « micro et envois Mistral, coût inconnu ». Rien avant votre consentement
   contextualisé et clic explicite. Coût, conditions et conservation non
   vérifiés : uniquement propos synthétiques courts, pas de note réelle.
   Refuser l'accord doit laisser le manuel sans capture.
   Ensuite, si vous acceptez, autoriser et Démarrer le vocal ; vérifier mode,
   erreur éventuelle et lecture seule de la note. Dire deux interventions,
   séparées par un silence, sans réactivation (« Prépare une note de test :
   budget de douze euros, rien n'est décidé. » puis « Ajoute une réserve sur
   la date, qui reste à confirmer. »). Relever les modèles servis/mesures
   exposés, ou « inconnu » ; ne pas inventer la qualité/finalité du service.
5. **Continuité/ambiguïté (G03/G04/G08)** : vérifier plusieurs passages,
   silence sans fin de session, puis ambiguïté (« supprime ce point » avec
   deux cibles possibles). Attendu : zéro effet de toute l'intervention,
   précision interne, pas de rejeu automatique. Minimiser seulement à votre
   initiative ; aucun tray/notification applicative ni focus volé.
   Demander naturellement la pause ; revenir consulter l'état.
6. **Pause/manuel (G04/G07)** : Pause arrête la capture et invalide.
   Reprendre explicitement seulement si voulu. Arrêter le vocal, éditer/annuler
   en manuel : aucun ancien résultat ne doit modifier ce texte.
   Une nouvelle session peut le modifier intentionnellement, jamais par rejeu.
7. **Cycle OS (G05), étape séparée avec votre accord** : après sauvegarde
   et avec uniquement données synthétiques, vous seul déclenchez
   verrouillage/déverrouillage puis veille/réveil. Attendu : arrêt/purge,
   manuel au retour, aucune auto-reprise. L'agent ne provoque pas ces événements.
   Initialisation WTS seule ne valide pas cette étape.
8. **Fermeture/pertes (G07/G09)** : tester sans destination sur une nouvelle
   note synthétique : fermeture suspendue, note accessible, micro arrêté.
   Enregistrer ensuite ; Fichier → Fermer après sauvegarde. X seul peut
   préparer/suspendre la fermeture. Les copies courantes/fermeture restent
   exclusives, **pas remplacement conditionnel final**.
   Coupure réseau/retrait micro seulement volontaire : erreur visible,
   pas de fournisseur caché, pas de récupération prétendue de paroles perdues.

Rapporter chaque étape PASS/FAIL/NON EXÉCUTÉ avec attendu/observé et mesures
sans audio, transcription, clé ni note réelle. Le protocole A–K complet et les
bornes de la tranche restent dans le [quickstart](../../specs/001-bloc-notes-vocal/quickstart.md).

| Gate | Recette Windows 11 réelle |
|---|---|
| G02 modèles/WSS/schema/compte | NON EXÉCUTÉ |
| G03 WASAPI/micro/rotation sans perte | NON EXÉCUTÉ |
| G04 arrière-plan/pause/absence indicateur ajouté | NON EXÉCUTÉ |
| G05 WTS/alimentation/arrêt/absence auto-reprise | Initialisation WTS PASS ; événements NON EXÉCUTÉS |
| G06 Credential Manager/champ masqué | NON EXÉCUTÉ |
| G07 opaque/Unicode/fichiers/fermeture/disque | NON EXÉCUTÉ |
| G08 fidélité/services synthétiques | NON EXÉCUTÉ |
| G09 NSIS/WebView2/ressources/délais | Binaire embarqué/fenêtre WebView2 lancés ; NSIS/recette visuelle/mesures NON EXÉCUTÉS |

## Arrêt et limites

Préférer **Fichier → Fermer après sauvegarde**. Attendre les acquittements et
récupérer toute saisie refusée avant de sortir ; ne pas forcer l'arrêt pour
contourner un refus de sauvegarde. Session détachée `windows-ui` conservée ;
après fermeture propre, l'agent peut arrêter ce helper via
`stop_bash(shellId="windows-ui")` s'il reste actif.
En dernier recours seulement, après sauvegarde, vérifier le chemin et le PID :

```powershell
$p = Get-Process -Id 25360 -ErrorAction SilentlyContinue
if ($p -and $p.Path -eq 'C:\Users\lmirguet\Development\vocalnotes-native-B02-W01\src-tauri\target\debug\vocalnotes.exe') {
    Stop-Process -Id 25360
}
```

L'ancienne UI Linux WSLg a été arrêtée avant cette reprise, aucune relance.
**GNOME/Wayland : NON EXÉCUTÉ, différé et non bloquant pour le développement.**
T010 reste partielle ; T011–T013 non commencées, autres dépendances conservées.
Couverture non mesurée, ni certification fonctionnelle ou multiplateforme.
Constitution approuvée inchangée ; 0 microphone démarré, 0 appel fournisseur,
0 accès trousseau par l'agent, pièce jointe/clé racine jamais lue.

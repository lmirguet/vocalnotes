# Prototype GNOME/Wayland — reprise native B04-GNOME

**Bilan courant de validation locale, 2026-10-31 :** [preuves et matrice G02–G09](prototype-gnome-final.md).
GNOME/Wayland uniquement, veille exclue par l’utilisateur, binaire direct sans
nouveau paquet. Dictée/correction/arrière-plan/pause/verrouillage/enchaînement
D6 PASS utilisateur. 74 Rust + 32 JS, build web/format, vrais fichiers (droits,
écriture partielle, opaque), trousseau isolé et perte de connexion du moniteur
GNOME PASS. Dernière recette visuelle opaque/copie/refus d’écrasement PASS utilisateur et
comparaison d’octets PASS. T006/T007 cochées pour la tranche GNOME/Wayland
convenue ; aucune certification Windows, veille, paquet ou fonction future.
Les états plus anciens ci-dessous sont historiques ; ce bilan courant prévaut.


## B04-GNOME — 2026-10-31

Périmètre demandé : reprise T006/T007 sur GNOME/Wayland uniquement.
Les résultats Windows antérieurs ne sont ni rejoués ni utilisés pour valider GNOME.
T006 et T007 restent ouvertes jusqu'aux recettes correspondantes.
**État : préparation automatisée et recette manuelle réussies ; écriture/présence trousseau confirmées, vocal à recetter.**

### Environnement réellement contrôlé

| Composant | Valeur |
|---|---|
| Système | Fedora Linux 44 Workstation, x86_64 |
| Desktop | GNOME Shell 50.6, session Wayland, `wayland-0` |
| CPU / RAM physique | AMD Ryzen 7 5800H, 16 processeurs logiques / environ 12 Gio |
| Rust / Cargo | RPM Fedora 1.99.1, rustfmt 1.10.0 |
| Node | 22.23.2, chemin explicite pour les commandes projet ; Node système 24.22.1 |
| SDK natifs | ALSA 1.2.16.1, D-Bus 1.16.2, GTK3 3.24.52, WebKitGTK 2.54.2 |
| Audio | PipeWire 1.6.10, WirePlumber 0.5.18 ; source par défaut `nx_remapped_out` (NoMachine) |
| Installation SDK | Effectuée par l'utilisateur avec DNF |

Contrôles D-Bus hors bac à sable, sans modification :
`org.gnome.ScreenSaver.GetActive` retourne `false` ;
`org.freedesktop.login1.Manager.PreparingForSleep` retourne `false` ;
`NameHasOwner(org.freedesktop.secrets)` retourne `true`.
Cela établit la présence des services, pas la réception des événements par
l'application, la capture ni une lecture/écriture de clé au trousseau.

### Vérifications de cette reprise

- Checklist `requirements.md` : 28/28 critères cochés, inchangée.
- Installation npm verrouillée : 105 paquets, Node 22.23.2.
- `npm test` : 31/31 tests PASS, quatre fichiers.
- `npm run build` : contrôle TypeScript et compilation Vite PASS.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` : PASS.
- `CARGO_HOME=/tmp/vocalnotes-cargo cargo test --locked --manifest-path
  src-tauri/Cargo.toml --lib --tests` : 67/67 tests PASS, aucun échec/ignoré.
  Neuf tests lib, puis 15 opérations, 3 audio, 11 contrats, 1 enveloppe,
  2 sauvegarde ordonnée, 9 pipeline, 4 services, 6 ports et 7 validité session.
  Les cibles main et types Windows exécutent zéro test runtime.
- `CARGO_HOME=/tmp/vocalnotes-cargo cargo build --offline --locked
  --manifest-path src-tauri/Cargo.toml --features tauri/custom-protocol
  --bin vocalnotes --example probe_lifecycle` : PASS.
- `src-tauri/target/debug/examples/probe_lifecycle`, hors bac à sable :
  PASS code 0, surveillance OS initialisée. Aucun micro, trousseau ou fournisseur
  touché ; les événements réels lock/suspend restent à recetter.

Inventaire PipeWire en lecture seule : entrée matérielle
`alsa_input.pci-0000_04_00.6.HiFi__Mic1__source` présente, entrée par défaut
NoMachine. Le choix de l'entrée effective doit être confirmé avant capture ;
aucun changement automatique du périphérique par défaut.

### Lancement natif et périmètre de packaging

Le RPM de diagnostic a été généré par `npm run tauri -- build --debug
--bundles rpm --ci` (63,55 Mio). L'utilisateur a demandé de poursuivre depuis
le binaire compilé, sans installation système supplémentaire. La demande
d'installation a été interrompue ; `rpm -q bloc-notes-vocal` confirme que
le paquet n'est pas installé. La génération ne valide donc pas G09.

Lancement direct, hors bac à sable, sans serveur Vite :

```bash
/home/laurent/projects/vocalnotes/src-tauri/target/debug/vocalnotes
```

Processus de cette reprise : PID 1152666, session outil 49945, avec processus
WebKit réseau et rendu présents. Aucun diagnostic d'erreur reçu au lancement.
SHA-256 du binaire :
`ef58a8ca7ad1a1e4bd42dd3863853633c9768c79e164fa8abe0bc5f9bdabd9be`.
Présence des processus uniquement : le rendu et les gestes utilisateur ne sont
pas présumés réussis. Ne pas relancer ou remplacer le binaire pendant la recette.
Les PID sont des observations datées, à revérifier avant toute action ultérieure.

Recette manuelle demandée : deux paragraphes fictifs, Gras, Annuler/Rétablir,
Fichier → Enregistrer une nouvelle copie vers un chemin neuf, puis Nouvelle note
après confirmation de sauvegarde. **PASS confirmé par l’utilisateur** : « ok, j’ai testé ces 3 actions... elles fonctionnent ». Ce retour couvre les trois groupes proposés : paragraphes/formatage/annulation, copie Markdown et nouvelle note. Il ne couvre pas le vocal, le trousseau, les blocs opaques ni les échecs disque.
Pour fermer : attendre les acquittements, sauvegarder puis Fichier → Fermer
après sauvegarde ; ne pas interrompre le processus avec une saisie non sauvée.

La première installation npm dans le bac à sable a rencontré des erreurs
DNS `EAI_AGAIN` ; interrompue puis relancée hors bac à sable avec succès.
Les dépendances Rust verrouillées sont téléchargées dans
`/tmp/vocalnotes-cargo`, sans changement des manifestes ni lockfiles.

### Trousseau et préparation du premier essai vocal

L’utilisateur confirme la saisie de la clé dans l’application et sa présence
au trousseau. Aucune valeur de clé lue par l’agent ni inscrite dans les preuves.
La confirmation ne certifie pas les essais de suppression/remplacement ou
d’indisponibilité du service.

Micro NoMachine choisi explicitement par l’utilisateur ; nouvelle lecture
`pactl get-default-source` : `nx_remapped_out`, conforme au choix. Le prototype
ouvre le périphérique d’entrée par défaut. Le diagnostic natif GNOME/login1
réussit à nouveau, PID applicatif 1152666 présent. Pas de capture lancée.

Premier essai proposé : autorisation explicite dans la fenêtre après information
sur les envois audio à Voxtral et texte/contexte à Mistral, coût potentiellement
non nul et conservation inconnue ; phrase fictive courte, arrêt dès erreur ou
au plus tard après 30 secondes. Ce délai est une consigne manuelle, pas un timer
produit. Aucun nouvel essai automatique ni transcription conservée dans le rapport.

### Premier essai vocal NoMachine — FAIL utilisateur, diagnostic B04-GNOME-D1

Aucun texte apparu. Après arrêt explicite, compteurs rapportés : 17 tentatives
STT, 6 LLM ordinaires, 12 prioritaires, 40 860 ms audio transmis ; coût inconnu.
Modèles affichés : `voxtral-mini-transcribe-realtime-2602` et
`mistral-large-latest`. Ces valeurs sont les compteurs de l’application,
pas une mesure de durée murale, une facture ni une preuve de fidélité.
Les modèles sont affichés après réception de résultats ; le nombre exact de
clarifications/refus/modifications de cet essai reste inconnu.

Erreur dans le protocole proposé par l’agent : « Crée une note… » peut demander
`new`, non intégré au vocal de cette tranche. Le prompt exige alors une
clarification sans aucune action. Hypothèse plausible, pas cause runtime prouvée.
L’arrêt remplace le message courant, ce qui empêche de conclure a posteriori.

Correctif D1, hors ligne : quatre compteurs (modification/clarification/refus/
sans modification) et dernier résultat statique natif, conservés en mémoire
après arrêt. Pas de transcription, réponse brute, historique vocal ou secret
persisté. Réponses dupliquées/invalidées non recomptées. Le message de succès
ne prétend plus avoir modifié une note lorsqu’un passage n’a rien changé.

Tests avant code : deux tests Rust et un test UI en échec, puis corrigés.
Validation source : 9 tests Rust lib + 57 dans le harnais des vrais tests
partagés = 66 PASS ; audio non rejoué dans ce harnais. 32 JS PASS,
typecheck/build frontend PASS. Binaire GUI ancien inchangé par les tests
(empreinte B04 conservée), correctif non chargé avant fermeture sûre et relink.
Aucun appel API, capture ou accès trousseau par l’agent pendant ce correctif.
T006/T007 ouvertes, aucun GO.

Chargement D1 : fermeture confirmée par l’utilisateur puis absence de processus
`vocalnotes` vérifiée. Build natif `--offline --locked --features
tauri/custom-protocol --bin vocalnotes` PASS ; assets compilés avec bilan vocal
vérifiés. Relance directe réussie, session outil 2460, aucun diagnostic reçu
au lancement. Ancien PID/session B04 décrits plus haut historiques.
SHA-256 actuel :
`7e55c5f174e99cc788fc945af0e9eb8d5e0bf97296d6a62ae0b26ca3b62a0047`.
Aucun nouveau RPM, capture ni appel fournisseur effectué par l’agent.

Prochain essai après chargement du correctif : utiliser la note déjà ouverte
et dicter seulement « Nous devons acheter trois cahiers pour lundi. », sans
commande de création de note. Attendre brièvement sans reparler puis relever
le bilan, arrêter après 30 secondes maximum ou immédiatement sur erreur.
Pas de nouvel essai automatique.

### Diagnostic D2 — console technique et réception audio

Après relance D1, l’utilisateur signale encore un fonctionnement non concluant
et demande explicitement des traces sur la console Linux, les messages UI étant
volatils. Aucun nouveau compteur ni contenu de passage n’est disponible pour
cet essai ; la réception correcte de la voix NoMachine reste non prouvée.

D2 ajoute des messages horodatés sur stderr, sans fichier de logs créé par
l’application : initialisation/arrêt, configuration numérique et catégorie de
route CPAL, VAD, WSS/format STT, final reçu (taille seulement), statut HTTP LLM,
validation du contrat et résultat natif (appliqué/clarification/refus/sans effet).
Une clarification issue du modèle est tracée comme catégorie, pas avec son texte.
Aucun secret, transcription, contenu de note, audio, réponse brute ou en-tête
fournisseur n’est envoyé à la console. La demande de traces techniques par
l’utilisateur autorise ce diagnostic console pour cette reprise ; elle n’ajoute
pas d’historique de contenu ni le parcours de diagnostic complet US7.

Mesures audio une fois par seconde, dans le worker natif et jamais dans le
callback temps réel : `input_samples`, `rms_ppm`, `peak_ppm`, `pcm_peak_ppm`,
`frames_30ms`, `voiced_frames`. Les niveaux sont en millionièmes de pleine échelle.
Une fenêtre sans trames est distincte de trames silencieuses ; le niveau natif
avant downmix est distinct du pic PCM normalisé, pour repérer une éventuelle
annulation entre canaux. Aucun échantillon n’est conservé par le vumètre.
La capture reste soumise au démarrage explicite, au trousseau et à la garde OS.

Test initial du vumètre rouge, puis vert : absence d’entrée, silence, signal
stéréo opposé annulé par downmix, pic i16 minimum sans overflow. Régressions
hors ligne : 10 Rust lib + 57 tests partagés PASS (67 ciblés), sans échec ;
format et diff propres. Aucune modification frontend depuis les 32 JS PASS D1,
pas de nouvelle exécution frontend nécessaire. Binaire D1 conservé, empreinte
7e55c5f…0047 inchangée ; D2 non chargé avant fermeture sûre.
Aucun nouveau test micro/API lancé par l’agent.

Après chargement, démarrer volontairement le vocal et parler quelques secondes
avec le micro NoMachine, puis arrêter. L’agent relit la console de la session
outil ; l’utilisateur n’a plus à recopier les messages changeants de l’interface.
Le signal et les niveaux ne prouvent pas à eux seuls l’intelligibilité de la voix.

### Fermeture demandée par l’utilisateur après D1

L’utilisateur indique ne pas parvenir à fermer l’application et demande à
l’agent de la fermer. PID 1169784 vérifié via `/proc/<pid>/exe` : chemin exact
du prototype de ce projet. SIGTERM envoyé uniquement à ce processus après
information sur la perte possible de saisie non sauvegardée. Absence du
processus confirmée ; session outil 2460 terminée code 143. Aucun SIGKILL.
D2 reconstruit avec interface embarquée (`cargo build --offline --locked
--features tauri/custom-protocol --bin vocalnotes`) : PASS. Relancé directement
en session outil 47020 ; premier message console reçu :
`Démarrage diagnostic console D2 ; microphone arrêté`.
SHA-256 du binaire D2 :
`9a2fdbf900b0c90e0d6f0c77702933c7182a9d2a55f7b8836f971e39368b8fce`.
Aucune capture ni requête fournisseur déclenchée par la relance.

La fermeture forcée ne certifie pas le parcours de sauvegarde/fermeture normal ;
cette difficulté reste à diagnostiquer dans T006/T007.

### Essai D2 observé dans la console — audio reçu, édition refusée

La console montre une entrée native 44 100 Hz, deux canaux, route CPAL `default`.
Des fenêtres de parole ont un RMS de l’ordre de 0,10–0,16 de pleine échelle,
avec 32–35 trames VAD positives par seconde ; le pic PCM monte près de la
pleine échelle. L’application reçoit donc un signal non silencieux, et le
résultat STT final non vide confirme que le traitement atteint la transcription.
Ces observations ne certifient ni fidélité ni calibration du niveau.

Un premier `transcription.done` vide est ignoré correctement. Le passage suivant
produit un final de 73 octets (contenu non journalisé), puis HTTP 200 LLM,
intervention conforme au contrat avec une action. Commit refusé :
`Texte attendu absent : préciser la demande ; aucun effet.`
Bilan : applied=0, clarifications=0, rejected=1, no_change=0.
Les appels prioritaires retournent aucune interruption. Arrêt demandé et flux
microphone fermé visibles dans la console. Aucun appel additionnel par l’agent.

Cause localisée : l’opération renvoyée recherche un texte existant dans une
note vide. Le validateur refuse à juste titre ; le type précis d’opération
replace/delete/move n’était pas encore journalisé dans D2. Le schéma générique
proposait ces opérations même quand seule une ancre d’insertion vide existait.

D3 : schéma d’édition restreint à insert pour ce contexte précis, block_id
imposé à l’identifiant natif fourni et anchor.expected vide imposé au schéma.
Le prompt distingue contenu à insérer et cible déjà présente. Aucun relâchement
des gardes natives ; note non vide/ensemble opaque conservent le contrat existant.
La console trace désormais le type d’action sans cible ni contenu.
Test initial rouge (quatre opérations offertes pour une note vide), puis
correctif validé hors ligne : 10 tests lib + 59 tests partagés, soit 69 Rust
ciblés PASS, sans échec. Format/diff PASS. Pas de changement frontend depuis
les 32 JS PASS D1. D2 conservé, empreinte 9a2fdbf…8fce inchangée ; D3 non chargé
avant relance sûre. Le succès fonctionnel réel reste à recetter.

### Fermeture D2 avant chargement D3

L’utilisateur confirme que l’application est déjà fermée. Absence de processus
`vocalnotes` vérifiée ; session outil 47020 terminée normalement code 0, dernier
message `Commande : arrêt/invalidation`. Aucun signal forcé pour cette fermeture.

D3 : build natif embarqué hors ligne/verrouillé PASS, relance directe en session
outil 45903. Les traces audio D2 restent actives (leur message de démarrage
porte encore ce libellé) ; le correctif de schéma D3 est embarqué. Micro arrêté
au lancement, aucun nouvel appel API par l’agent. SHA-256 courant :
`64db9f56edce584cd10f3b395ef8d43948ec8323d7aee8628f9148201328a056`.
Prochaine preuve attendue : insertion réelle de la dictée sur la note vide.

### Recette D3 — première dictée PASS utilisateur et console

L’utilisateur confirme que le texte est correctement apparu dans la note.
La console de session45903 confirme : final STT non vide (45 octets, contenu
masqué), HTTP200 LLM, intervention conforme avec une action `insert`, puis
`Modification de la note appliquée`, applied=1, clarifications=0, rejected=0,
no_change=0. Écoute poursuivie après commit, puis arrêt explicite et flux natif
fermé. Un premier final vide de bruit initial est ignoré sans appel LLM ordinaire.
Les réponses prioritaires observées n’interrompent pas la dictée.

Verdict borné : PASS première insertion vocale sur note vide sous Fedora
GNOME/Wayland avec entrée par défaut choisie NoMachine. Cela confirme le
correctif de ciblage initial et le parcours capture→STT→LLM→commit→affichage.
Ne certifie pas correction sur note existante, pause vocale, arrière-plan,
lock/suspend, rotation prolongée, intégrité fichiers ni installation RPM.
T006 reste partielle, T007 ouverte, aucun GO global.

Prochaine recette : redémarrage vocal explicite, demande de remplacement d’un
fragment de la note, attente de l’édition, puis demande de pause à la voix.
Vérifier texte modifié et état Pause ; relever les traces sans recopier le contenu
personnel. Arrêt de sécurité manuel au plus tard après 30 secondes ou sur erreur.

### Recette D3 — correction et pause confirmées

L’utilisateur confirme la réussite de la correction sur la note existante et de
la pause demandée dans la recette. Console session45903 : final STT de 42 octets
(contenu masqué), réponse LLM HTTP200, une action `replace`, puis modification
appliquée à 1793442147011 ms. Compteurs cumulés : applied=2, clarifications=0,
rejected=0, no_change=0. Commande pause à 1793442149392 ms, flux microphone
fermé à 1793442149394 ms. PASS correction et pause utilisateur, avec fermeture
native confirmée. La trace de commande seule ne distingue pas voix et bouton.
T006 reste partielle et T007 ouverte : arrière-plan, verrouillage et suspension,
rotation et autres gates restent à recetter. Prochain essai : écoute avec la
fenêtre en arrière-plan, puis retour dans la note et contrôle du texte.

### Recette arrière-plan D3 — FAIL, remplacement au lieu d’ajout

L’utilisateur constate que la nouvelle dictée remplace le texte précédent au
lieu de s’ajouter. Console session45903 : final STT non vide (39 octets,
contenu masqué), HTTP200, une action `replace` appliquée à 1793442221961 ms ;
compteurs cumulés applied=3, rejected=0. Pause ensuite, flux natif fermé.
L’opération est acceptée par les préconditions textuelles, mais son intention
est incorrecte selon le retour utilisateur. Aucun PASS de fidélité en arrière-plan.

D4 renforce la consigne du modèle : ajout et conservation par défaut, correction
limitée à une demande courante claire, aucune commande mémorisée rejouée ;
exemples ajout/correction/récit de procédure. Aucun détecteur lexical ni conversion
silencieuse de replace en insert. Cette consigne ne garantit pas à elle seule le
comportement du service ; un nouvel essai réel est nécessaire. Correctif source
non encore chargé dans la session45903 ; note ouverte préservée.
Vérification D4 : 10 tests Rust bibliothèque et 59 tests du harnais hors ligne
PASS ; formatage, diff et lecture YAML PASS. Ces contrôles ne valident pas
la décision sémantique du modèle distant.
T006 partielle, T007 ouverte.

### Recette native à renseigner

| Gate | État de cette reprise |
|---|---|
| G02 modèles/WSS/schema/compte | D6 : schéma accepté et enchaînement PASS sur cet essai ; autres critères non certifiés |
| G03 micro/normalisation/rotation sans perte | D6 : plusieurs passages appliqués PASS ; rotation prolongée/intégrité non certifiées |
| G04 arrière-plan/pause/affichages internes | Pause et scénario arrière-plan avec réserve PASS utilisateur D4 ; échec D3 conservé dans l’historique |
| G05 lock/sleep/perte abonnement/fermeture | Verrouillage/déverrouillage PASS utilisateur et micro fermé ; veille exclue localement par utilisateur ; perte abonnement non testée |
| G06 Secret Service/champ masqué/purge | Saisie/enregistrement et présence PASS utilisateur ; remplacement/suppression/indisponibilité non recettés |
| G07 Markdown/opaque/échecs disque | Manuel/formatage/undo/copie/nouvelle note PASS utilisateur ; opaque et échecs disque non recettés |
| G08 fidélité/services synthétiques | D6 : enchaînement PASS utilisateur/console ; insertion/correction et réserve déjà recettées, autres parcours non certifiés |
| G09 installation/RPM/ressources/délais | RPM généré ; installation non effectuée ; lancement direct constaté, mesures non qualifiées |

Première étape interactive : fenêtre native en mode Manuel, deux paragraphes
synthétiques, formatage, Annuler/Rétablir, copie Markdown vers un chemin neuf,
puis nouvelle note après sauvegarde. Ne pas démarrer le vocal pour cette étape.
Ensuite : saisie masquée d'une clé par l'utilisateur et recette micro/API courte
après information et accord contextualisé, puis arrière-plan/pause et cycle OS.
Ni secret, audio, transcription ni note personnelle ne doit figurer dans le rapport.

## Historique B01-R3 — NON EXÉCUTÉ sur GNOME

2026-10-31, B01-R3. **Aucun GO T007.**

Hôte de compilation : Ubuntu 24.04 WSL2, pas un desktop GNOME établi.
CPAL/ALSA compilé, 32 tests Rust/22 JS verts : ni micro, ni arrière-plan,
ni Secret Service ni GNOME certifiés. Le sondage natif de surveillance
sans micro/clé/API retourne code 2.

| Gate | Recette GNOME réelle |
|---|---|
| G02 modèles/WSS/schema/compte | D6 : schéma accepté et enchaînement PASS sur cet essai ; autres critères non certifiés |
| G03 micro/normalisation/rotation sans perte | D6 : plusieurs passages appliqués PASS ; rotation prolongée/intégrité non certifiées |
| G04 arrière-plan/pause/affichages internes | Pause et scénario arrière-plan avec réserve PASS utilisateur D4 ; échec D3 conservé dans l’historique |
| G05 lock/sleep/perte abonnement/fermeture | Verrouillage/déverrouillage PASS ; veille exclue localement par utilisateur ; perte abonnement non testée |
| G06 Secret Service/champ masqué/purge | NON EXÉCUTÉ |
| G07 Markdown/opaque/échecs disque | NON EXÉCUTÉ |
| G08 fidélité/services synthétiques | D6 : enchaînement PASS utilisateur/console ; insertion/correction et réserve déjà recettées, autres parcours non certifiés |
| G09 installation/RPM/ressources/délais | NON EXÉCUTÉ |

À fournir : desktop interactif GNOME/Wayland, matériel/versions, même arbre
et dépendances verrouillées. Suivre A–K du quickstart puis renseigner rapports
et mesures. Clé via champ masqué/trousseau seulement.
Avant micro/API : accord contextualisé, synthétique non sensible,
coût/conservation inconnus explicitement acceptés. Aucun lancement ici.
Un essai absent reste NON EXÉCUTÉ ; NO-GO → correction avant stories.

### Relance D4 — correctif chargé, recette en attente

Fermeture D3 confirmée par session45903 terminée avec code0 et absence de
processus vocalnotes. Build natif hors ligne avec assets embarqués PASS
(16,21 s). Lancement direct D4 en session5444, microphone arrêté au démarrage.
SHA-256 : `47ea81bc8cc50aed81a47f65b87f436830caa06460fc1562f6f12ccbec04f024`. Aucun paquet généré.
Prochaine recette : deux passages distincts, le second en arrière-plan ;
vérifier que le premier est conservé et le second ajouté, puis arrêter.
Le correctif de consigne reste non validé en conditions réelles.

### Retour utilisateur D4 — correction réussie, fusion des idées

L’utilisateur décrit une première dictée, une correction lexicale réussie, puis
une idée supplémentaire regroupée avec la première dans une même phrase.
Les deux objets sont conservés selon son retour ; ce cas ne prouve donc pas
une perte du contenu précédent. Le critère précédent imposant deux phrases
et une opération insert était trop restrictif par rapport à S1.1/S1.3 : la
note peut être structurée et reformulée, sous réserve de fidélité aux nuances.
La fusion rapportée transforme toutefois une formulation de prévision en
achat ; fidélité de cette nuance non certifiée. Aucun nouveau correctif appliqué.

Console session5444 : insert initial, puis plusieurs replace appliqués ;
compteur final applied=5, clarifications=0, rejected=0, no_change=0. Ces cinq
applications ne peuvent pas être attribuées exactement aux trois étapes
résumées par l’utilisateur sans journaliser le contenu ; aucune attribution
inventée. Arrêt explicite puis fermeture native du micro confirmés. Le retour
ne confirme pas explicitement le changement de fenêtre : arrière-plan à confirmer.
D4 ne garantit pas insert malgré sa consigne ; succès de correction utilisateur,
conservation des deux idées rapportée, fidélité fine et arrière-plan non certifiés.
T006 partielle, T007 ouverte.

### D4 — fidélité en arrière-plan PASS utilisateur, fermeture normale

Après la recette proposée « Prévoir des enveloppes, mais ne pas les acheter
avant confirmation », l’utilisateur confirme le test concluant et la fermeture
de l’application. Fidélité de la réserve et fonctionnement en arrière-plan PASS
sur ce scénario selon observation utilisateur ; aucun contenu de note relu.
La console montre plusieurs tentatives : certains finals STT vides ignorés,
un passage interrompu avant final et deux finales non vides de 67 et 59 octets.
Ces deux dernières donnent HTTP200, replace, applied=6 puis applied=7,
clarifications=0, rejected=0, no_change=0. La console ne permet pas d’attribuer
le contenu exact à ces passages ; aucune garantie de fiabilité STT générale.
Dernière pause à 1793442762168 ms, flux natif fermé à 1793442762171 ms ;
fermeture session5444 code0 et absence de processus confirmées.

Même binaire D4 relancé en session4710, microphone arrêté. Prochaine recette :
démarrage explicite, verrouillage GNOME par l’utilisateur, déverrouillage et
contrôle du mode manuel sans reprise automatique. Verrouillage, suspension et
perte de surveillance restent non exécutés à ce stade. T006 partielle/T007 ouverte.

### D4 — verrouillage GNOME PASS utilisateur et console

Session4710 : écoute active, puis interruption OS à 1793442895112 ms avec
passage en manuel et absence de reprise automatique annoncée ; flux microphone
natif fermé à 1793442895115 ms. Le message regroupe verrouillage/veille/perte de
surveillance ; l’utilisateur confirme avoir effectué la recette de verrouillage
et constater le mode manuel au retour. Aucun redémarrage du micro observé dans
les traces recueillies après cet événement. PASS pour ce scénario de verrouillage
et déverrouillage ; les 3 ms entre message et fermeture ne mesurent pas le délai
depuis le verrouillage lui-même. Veille et perte d’abonnement restent non testées.
Prochaine recette : suspension/reprise réelle, après sauvegarde et avec reprise
possible de la connexion NoMachine. Aucun changement de code ni reconstruction.
T006 partielle/T007 ouverte ; aucun GO global.

### Périmètre actualisé — veille exclue sur cet ordinateur

L’utilisateur ne peut pas tester la veille sur cet ordinateur et demande de
ne pas tenir compte de ce test. Recette suspension/reprise exclue du périmètre
local sur instruction utilisateur ; non exécutée, ni PASS ni FAIL, aucun
blocage local lié à son absence. Le verrouillage reste validé séparément.
Cette exclusion ne constitue pas une validation de la gestion de veille.
Prochaine recette interactive : trois passages vocaux distincts dans une même
session, avec une pause de silence entre chacun, pour vérifier conservation
et enchaînement. T006/T007 restent partielles/ouvertes pour les autres gates.

### D4 — enchaînement FAIL, réponse fournisseur refusée

Utilisateur : message « intervention incohérente ». Console session4710 :
audio non silencieux, finals STT 84 puis 43 octets ; première réponse HTTP200
refusée hors contrat strict, microphone fermé. Nouvelle session explicite :
final 86 octets, HTTP200, refus « Intervention incohérente : aucun effet. »,
fermeture micro puis arrêt utilisateur. Aucun commit sur ces réponses refusées.
Le second refus provient des invariants status/message/actions ou de leurs
limites ; les logs D4 ne permettent pas de choisir entre ces causes. Il ne
qualifie pas la cohérence des propos de l’utilisateur. Test enchaînement FAIL.

D5 : message corrigé pour identifier la réponse du service ; logs numériques
sur échec de décodage (catégorie JSON et position) et d’enveloppe (statut,
nombre d’actions, taille et caractère vide du message), sans contenu. Prompt
renforcé pour message exactement vide sur resolved et question sans actions
sur clarify. Aucun assouplissement de validation, aucune réponse corrigée ou
rejouée automatiquement. Cela améliore le diagnostic et la consigne ; la cause
historique exacte et la correction effective nécessitent une nouvelle recette.
Session4710 préservée, correctif non encore chargé. T006 partielle/T007 ouverte.

D5 vérification hors ligne : nouveau test de rejet d’enveloppes invalides
rouge puis vert ; 10 tests bibliothèque et 60 tests harnais PASS, formatage
et diff PASS. Aucun test réseau ; binaire GUI existant non modifié.

### Relance D5 — diagnostics chargés, essai en attente

Session4710 terminée code0 après fermeture utilisateur ; absence de processus
confirmée. Build natif hors ligne avec assets embarqués PASS (16,66 s).
Lancement direct D5 session56340, micro arrêté au démarrage. SHA-256 :
`15867549ce9db3645de95a23d9201e275248f43f6c15cb13f35c97984b75e15e`. Aucun paquet généré.
Prochaine recette : trois idées distinctes dans une même session, attendre
le traitement de chaque passage, puis arrêter. Cause historique exacte des
réponses refusées non déterminée ; comportement réel D5 non validé.
Veille toujours exclue du périmètre local sur instruction utilisateur.

### D5 — deuxième idée FAIL, cause d’enveloppe établie ; D6 préparé

Session56340 : première idée, final STT 109 octets, HTTP200, insert appliqué
(applied=1). Deuxième idée, final STT 113 octets, HTTP200 puis enveloppe refusée :
actions=2, message_bytes=0, status_clarify=1, message_blank=1. Micro fermé.
Cause précise : réponse fournisseur combine clarify avec actions et message
vide, incompatible avec les invariants natifs. Aucun contenu journalisé.
Le schéma D5 autorisait cette combinaison structurellement malgré le prompt.

D6 ajoute au schéma objet deux alternatives anyOf mutuellement exclusives :
resolved avec message exactement vide ; clarify avec message non blanc et
zéro action. Les variantes conservent les contraintes d’édition, y compris
l’ancre exacte et insert seul sur note vide. La validation native reste intacte,
aucune réparation ni conversion automatique des réponses. Documentation
consultée : https://docs.mistral.ai/studio/conversations/structured-output/custom
pour le mécanisme response_format ; l’acceptation effective des contraintes
par ce compte/modèle doit être vérifiée en recette réelle.
Correctif source non chargé ; session56340 préservée. T006 partielle/T007 ouverte,
veille exclue localement sur demande utilisateur.

D6 vérification : test de régression rouge puis vert ; 10 tests bibliothèque
et 61 harnais PASS. Validation indépendante jsonschema Draft202012 sur les
deux schémas réellement générés (note vide/non vide) : 18 cas PASS, dont
combinaison fautive observée, clarification vide/blanche, resolved avec
message et ancre vide native correcte/incorrecte. Formatage/diff PASS.
Compatibilité effective du service à vérifier après relance.

### Relance D6 — schéma corrigé chargé, recette en attente

Fermeture utilisateur confirmée par sortie code0 session56340 et absence de
processus vocalnotes. Build natif hors ligne avec assets embarqués PASS
(14,93 s), lancement direct session26421, microphone arrêté. SHA-256 :
`5b441b0be92924b6516b77574b066b116488c475b29e591e7401b9a39732f321`. Aucun paquet généré.
Reprendre trois idées distinctes dans une même session ; attendre le traitement
de chaque idée avant la suivante. L’acceptation effective du schéma par le
service et l’enchaînement ne sont pas encore validés. Veille exclue localement.

### D6 — enchaînement vocal PASS utilisateur et console

Utilisateur : tout a bien fonctionné cette fois-ci. Session26421 : réponses
HTTP200 avec schéma D6, plusieurs interventions conformes et appliquées,
compteur final applied=5, clarifications=0, rejected=0, no_change=0. Insert
et replace observés ; aucune erreur de contrat/enveloppe dans les traces
recueillies. Le retour porte sur la recette de trois idées ; cinq applications
sont observées sans attribution inventée aux trois idées ni lecture de contenu.
Finals STT vides ignorés sans appel LLM ordinaire. Pause à 1793443605599 ms,
flux natif fermé à 1793443605602 ms. Application conservée ouverte en pause.

PASS borné : acceptation effective du schéma D6 et enchaînement sur cet essai
Fedora GNOME/Wayland / microphone NoMachine. Ne constitue pas une garantie
de fidélité générale du modèle ou de stabilité prolongée. Les échecs D4/D5
restent historiques. Veille exclue localement à la demande utilisateur.
T006 partielle/T007 ouverte pour les autres gates (intégrité/opaque/échecs
fichiers, cas trousseau restants, mesures qualifiées et autres parcours).

# US1 — rédaction progressive, GNOME/Wayland

État courant : les essais ciblés réussis ci-dessous portent sur plusieurs versions
successives corrigées. Dernier binaire : `4be0527d…4c5f7` (repère temporaire).
L’intention est désormais une propriété persistante de la note ; les descriptions
antérieures du profil temporaire sont historiques. Deux éditions refusées pour
ancre/texte invalides restent une réserve de fiabilité ; aucun GO T031 global.

2026-10-31, lot B12-US1. T028–T030 implémentées ; **T031 ouverte,
recette native en cours**. Les succès T026 ne valident pas cette version.
Windows et veille réelle exclus selon le périmètre utilisateur.

## Changements et preuves hors ligne

- T028 : `domain/intention.rs` contient un profil but/forme/ton/contexte minimal,
  borné à 4 096 octets UTF-8 au total. Les champs sont libres pour les demandes
  naturelles. Le profil complet remplace le précédent ; chaîne vide = effacement.
  Il reste hors du contenu de note, des préférences et des diagnostics. Sur demande explicite pendant T031, le profil courant est désormais exposé au snapshot IPC destiné au champ d’intention de la fenêtre, sans persistance.
  Pause/reprise explicite le conserve ; arrêt/nouvelle session le purge.
  Le schéma fournisseur strict requiert tous les champs ; le décodeur accepte
  encore les anciennes réponses textuelles du corpus avec champs vides par défaut.
- T029 : `app/drafting.rs` prépare les seuls candidats et le profil courant,
  traite un passage final, puis remet sa réponse au contrôle de commit natif.
  Pas d’édition sur provisoire, d’historique de conversation, de réémission ni de
  fournisseur de secours. Le contrôleur utilise cette orchestration commune.
  Les échecs techniques sont distingués : reconnaissance STT,
  interprétation LLM (dont JSON invalide), application (cible, session, précondition).
  Une erreur sémantique avec réponse techniquement valide exige une comparaison
  humaine énoncé/transcription/note ; le code ne prétend pas en inférer la cause.
- T030 : plages UTF-8 issues des patches natifs acceptés, rattachées à la révision
  courante, avec type d’opération et nombre de caractères retirés. Les plages
  suivent les opérations ultérieures de la même intervention. Aucun texte effacé
  ni historique n’est ajouté au retour de changement. Refus = aucun retour partiel.
  La fenêtre distingue traitement et dernière modification appliquée ; elle
  affiche les extraits concernés et les retraits/déplacements en toutes lettres.
  « Repérer dans la note » fait défiler le bloc contenant le repère seulement au
  clic, sans changer la sélection. Pas de focus imposé ni notification système.
  Une intervention longue est tronquée dans l’aperçu, pas dans la note.

Tests : profil éphémère/purge/bornes/échec atomique ; corpus T027 rejoué via le
chemin applicatif ; séparation des échecs techniques et réponse tardive refusée ;
plages accentuées, déplacement/retrait, remappage après insertion et refus sans
effet partiel. Tests DOM : aucun repère de traitement inventé, révision périmée
ignorée, focus conservé, texte hostile inerte, suppression/déplacement perceptibles.

Cycle TDD : test du profil structuré en échec (méthode absente) avant intégration ;
test ChangeView en échec (module absent) avant création. Pas de nouvelle dépendance.
Les réponses de fidélité restent scriptées ; elles ne certifient pas les modèles.

## Matrice courante T031 — clôture non acquise

Les essais se répartissent sur les versions corrigées identifiées ci-dessous.
Les contrôles réussis sur une ancienne version ne certifient pas à eux seuls le
binaire final. La réserve de fiabilité empêche un GO global.

| Contrôle | État |
| --- | --- |
| Démarrage manuel sans capture, démarrage explicite | PASS console et parcours utilisateur |
| S1.1 : plusieurs passages et réserves | PASS ciblé utilisateur ; deux refus ultérieurs et précision STT en réserve |
| S1.2 : quatre formes, changements de ton | PASS ciblé sur versions successives après correction de l’intention |
| S1.3 : intention tardive, ajout, développement | PASS ciblé utilisateur ; aucune garantie générale de fidélité |
| Intention affichée, conservée après arrêt, sauvegardée/restaurée | PASS utilisateur et fichier synthétique |
| S1.4 : traitement distinct, repérage volontaire et surlignage | PASS ciblé ; pas de chronométrage du rendu |
| Déplacement/suppression, lecture d’une autre zone sans saut imposé | Hors ligne PASS ; recette native À TERMINER |
| Arrière-plan, pause vocale/bouton, reprise explicite | PASS ciblé utilisateur/console ; silence à qualifier séparément |
| Verrouillage GNOME sur écoute active sans reprise | PASS utilisateur et console sur dernier binaire |
| Manuel, undo, sauvegarde, intention conservée | PASS utilisateur ; structure du document vérifiée |
| Fermeture du dernier binaire | PASS utilisateur et processus absent |
| Conflit, trousseau, fautes sur version de fin de jalon | Preuves précédentes et tests hors ligne ; qualification native À TERMINER |
| Précision STT, erreurs de sens | RÉSERVE ; comparaison générale non réalisée |
| Mesures contextualisées | PARTIEL : quelques intervalles console, reste non mesuré |
| Windows, veille réelle, paquet | EXCLUS par l’utilisateur |

Aucun audio, transcription ou contenu utilisateur dans les journaux de recette.
Les noms/chiffres du corpus sont synthétiques. Les tests natifs qui sollicitent
un micro et les fournisseurs sont déclenchés explicitement dans la fenêtre.

## Lancement de recette

Build web, check Rust tous targets avec `tauri/custom-protocol`, build natif
embarqué PASS ; 13 tests bibliothèque + 131 tests partagés = 144 Rust PASS,
52 tests JavaScript PASS. Formatage et diff PASS. Aucun paquet généré.
Ancienne instance absente avant reconstruction. Binaire :
`c22795ec5dd905b18c790c4b0a12cbbd82df2220296a709e19e523aedbdbb432`.
Fenêtre ouverte, session outil 32914, journal `/tmp/vocalnotes-us1-native.log`.
Console au lancement : microphone arrêté. Contrôles utilisateur en attente.

## Premier essai T031 — PASS utilisateur sur le binaire B12

L’utilisateur confirme l’essai demandé : e-mail cordial, ajout d’une seconde idée,
conservation des informations et de la réserve, modifications signalées. Console :
une action d’intention, deux éditions appliquées, puis pause explicite à
1793479894045 et flux fermé à 1793479894048. Aucun contenu de note/transcription
collecté pour le rapport. Cette confirmation ne clôt pas le reste de la matrice.

## Complément demandé — champ d’intention au-dessus de la note

Demande utilisateur : afficher l’intention dans un champ distinct. Implémentation
`IntentionView.ts`, textarea en lecture seule avec but/forme/ton/contexte renseignés,
mis à jour depuis le profil canonique. Le profil conserve son cycle mémoire :
pause/reprise explicite le garde ; arrêt le purge. Rien n’est inséré dans la note
ou ses fichiers, aucune écriture de préférence ou de journal. Cette exposition
locale au snapshot remplace la restriction précédente « hors du snapshot ».

144 tests Rust et 54 tests JS PASS ; tests de composant et d’intégration fenêtre
pour emplacement au-dessus de la note, mise à jour sans édition ni focus imposé,
texte HTML inerte, lecture seule, conservation en pause et effacement à l’arrêt.
Build web/typecheck PASS. Nouvelle version native non chargée : la fenêtre B12
reste ouverte avec la note de recette ; attendre sauvegarde/fermeture utilisateur.

### Nouvelle version avec intention — lancée pour recette

Fermeture utilisateur confirmée, processus absent et ancienne session outil sortie
avec code 0. La console précédente contenait une erreur de sauvegarde avant la
fermeture : aucune sauvegarde réussie de cette note n’est déduite de cette sortie.
Build natif embarqué PASS (19,24 s), check tous targets embarqués PASS.
Empreinte : `0e5e5f6a6bfe0ed3b1f218595e6ee13c5a211281da8077375660d96ed85d2456`.
Nouvelle fenêtre en session outil 29343, micro arrêté selon le message de lancement.
Journal : `/tmp/vocalnotes-intention-ui-native.log`. Affichage et mise à jour du
champ d’intention restent à confirmer par l’utilisateur sur cette version.

## Changement de règle — persistance de l’intention

L’utilisateur confirme le fonctionnement du champ sur le binaire `0e5e5f6a…2456`,
puis demande de conserver le profil après arrêt et de l’enregistrer avec la note.
Cette nouvelle règle remplace les purges décrites précédemment, uniquement pour
l’intention. Détails et tests : [format du document](note-intention-format.md).
147 tests Rust et 54 JS PASS ; nouvelle recette native encore requise. L’ancienne
fenêtre n’a pas été remplacée ni arrêtée par l’agent.

### Intention persistante — recette ciblée validée

Après sauvegarde du fichier synthétique, fermeture, redémarrage puis ouverture
en manuel, l’utilisateur confirme la restauration du texte et de l’intention.
[Preuves et limites](note-intention-format.md). Ce résultat remplace le statut
en attente de cette recette ciblée ; T031 complète reste ouverte.

## Changement d’intention — ÉCHEC utilisateur, correction à recetter

L’utilisateur indique que la modification d’intention n’a pas eu lieu. Sur le
premier final non vide après démarrage, le journal montre cinq éditions sans
action drafting_intent, toutes appliquées. Sur un passage ultérieur, une action
d’intention existe, mais les valeurs ne sont pas journalisées : aucune conclusion
sur leur différence avec le profil précédent. Sans comparaison de transcription,
une erreur STT n’est pas exclue ; aucun contenu utilisateur n’est collecté ici.

La consigne de l’interpréteur exige désormais une action drafting_intent lorsque
le passage demande explicitement un changement de but, forme ou ton. L’ancien
profil ne prime pas sur cette demande. Un exemple synthétique e-mail cordial →
synthèse concise neutre conserve la réserve et montre la réponse attendue. Le
cœur signale une intention réellement modifiée dans son bilan et trace seulement
« profil modifié/inchangé », sans ses valeurs. Aucun classement lexical ni
réécriture automatique du profil en dehors de la réponse du modèle n’est ajouté.

148 tests Rust PASS, check embarqué tous targets PASS, format/diff PASS. Régression
scriptée via l’orchestrateur commun : ancien profil conservé après arrêt, nouveau
profil accepté, texte avec réserve intact et nouveau profil restauré après
sauvegarde/réouverture. Ce test ne prouve pas que le fournisseur choisira la bonne
action ; confirmation native encore requise. Fenêtre précédente laissée ouverte.

### Correction du changement d’intention — nouveau binaire lancé

Fermeture utilisateur confirmée et absence de processus vérifiée. Build natif
embarqué PASS (17,04 s). Empreinte :
`1a98ba35ca858efee1d9588bf48bf6639c27c82808694e4efec7bd93084bd81c`.
Fenêtre ouverte en manuel, micro arrêté au lancement. Session outil 96177, journal
`/tmp/vocalnotes-intention-change-native.log`. Le fichier synthétique de recette
existe toujours ; son profil courant n’est pas présumé identique à celui du début
de l’essai précédent. Nouveau contrôle : établir e-mail/cordial, puis demander
synthèse concise/neutre et vérifier le champ, le bilan et les informations
conservées. Confirmation utilisateur attendue ; aucun GO T031 à ce stade.

### Changement d’intention corrigé — PASS utilisateur

Sur le binaire `1a98ba35…bd81c`, l’utilisateur confirme l’essai demandé :
établissement du profil e-mail/cordial, passage à synthèse concise/neutre visible
dans le champ d’intention, réserve sur le devis conservée et pause finale.
Cette confirmation clôt la recette ciblée de la correction, sans effacer
l’échec initial ni certifier tous les changements de profil possibles.
La matrice T031 reste ouverte pour les autres usages et contrôles transversaux.

### Compte rendu factuel — PASS utilisateur

L’utilisateur confirme la reprise explicite, le passage au compte rendu factuel,
l’ajout de l’absence d’accord définitif et la conservation de la réserve sur le
devis, puis la pause. Validation bornée à cet essai ; la matrice T031 reste ouverte.

### Document professionnel et développement — PASS utilisateur

L’utilisateur confirme le passage au document professionnel formel, le
développement du point de vérification du devis, la conservation des réserves et
l’absence de date, responsable ou décision inventés, puis la pause. Les quatre
formes demandées ont été exercées dans cette recette progressive ; cela ne
constitue pas une évaluation exhaustive des quatre usages. Navigation/repérage
et contrôles transversaux du nouveau jalon restent à vérifier.

### Repérage — défilement confirmé, indicateur visuel demandé

L’utilisateur confirme que le clic fait défiler la page pour afficher le texte,
mais demande un indicateur visuel temporaire. Cette réponse ne confirme pas
séparément l’absence de saut de lecture avant le clic.

Ajout d’une décoration de nœud ProseMirror sur le bloc visuel correspondant au
repère natif : fond jaune et contour pendant deux secondes, uniquement au clic.
Aucun texte ni sélection modifiés, aucun format enregistré dans le Markdown.
Un nouveau clic réinitialise le repère ; expiration, changement de document,
édition, undo/redo et destruction retirent la décoration et annulent le timer.
Il s’agit d’un repère de bloc, pas d’un surlignage exact de chaque caractère modifié.

54 tests JS PASS ; vérification ciblée de la fenêtre PASS ; typecheck/build web
et check natif embarqué PASS. Rust métier inchangé ; les 148 tests Rust du lot
précédent ne sont pas rejoués. Fenêtre précédente laissée ouverte, reconstruction
et confirmation de l’indicateur natif en attente. T031 reste ouverte.

### Repère visuel — version native lancée

Fermeture confirmée par l’utilisateur et absence de processus vérifiée. Build
natif embarqué PASS (14,69 s). Empreinte :
`4be0527dfe2ba7b6d1140535dbcc7947bffc17c6e875822b0713857c3944c5f7`.
Session outil 60237, journal `/tmp/vocalnotes-locate-highlight-native.log`,
microphone arrêté au lancement. Le repère doit être vérifié après une nouvelle
édition acceptée : les retours temporaires de changement ne sont pas restaurés
à l’ouverture du fichier. Confirmation visuelle utilisateur encore attendue.

### Repère visuel — PASS utilisateur

Sur le binaire `4be0527d…4c5f7`, l’utilisateur confirme l’effet demandé après
une nouvelle édition et le clic de repérage : bloc visible avec fond jaune et
contour, puis retour à l’aspect normal après deux secondes. Validation ciblée ;
les autres contrôles transversaux de T031 restent ouverts.

Réserve sur les éditions de cette session : avant l’ajout accepté, la console
montre deux interventions refusées, respectivement pour ancre périmée et texte
attendu absent. Bilan : une édition appliquée, deux refus sans application.
La confirmation du surlignage ne valide pas ces réponses refusées ni la fiabilité
générale de l’interpréteur ; cette réserve reste ouverte dans T031.

### Arrière-plan et pause vocale — PASS utilisateur

Sur la version avec repère temporaire, l’utilisateur confirme une nouvelle
addition après réduction de la fenêtre, une pause demandée naturellement,
l’arrêt du micro et l’absence de retour forcé au premier plan. Les deux refus
antérieurs restent distincts de ce contrôle réussi. T031 reste ouverte.

### Verrouillage GNOME sur écoute active — PASS utilisateur et console

L’utilisateur confirme micro arrêté, aucune reprise automatique et conservation
de la note et de son intention. Console du binaire `4be0527d…4c5f7` :
initialisation 1793482546851, écoute effective 1793482546907, invalidation OS
1793482553160, fermeture native du flux 1793482553164. L’écoute était active
au moment du contrôle, qui ne se limite donc pas à constater un micro déjà arrêté.
Pas de veille réelle ni Windows exécutés. Manuel/sauvegarde/fermeture de fin de
jalon et réserves de fiabilité restent à qualifier avant toute clôture T031.

### Manuel, annulation et sauvegarde — PASS utilisateur

L’utilisateur confirme l’édition d’une phrase, son annulation classique, une
nouvelle addition conservée, l’enregistrement et la présence de l’intention.
Contrôle structurel du fichier synthétique : profil v1 valide et borné, corps
non vide ; aucun contenu de note ou de profil recopié dans les traces.
Ce contrôle ne compare pas le texte de l’addition manuelle, confirmé seulement
par l’utilisateur. Fermeture de cette version encore à vérifier.

## Bilan de ce passage — application fermée, T031 toujours ouverte

Fermeture confirmée par l’utilisateur et `pgrep -a -x vocalnotes` sans résultat.
Le dernier message technique signale l’arrêt/invalidation à 1793482663969 ; la
capture avait déjà été fermée au verrouillage. Aucun redémarrage effectué ici.
Sur le dernier binaire : deux éditions appliquées, deux refus. Le repère visuel,
l’arrière-plan, la pause naturelle, le verrouillage et le manuel sont confirmés ;
les refus initiaux ne deviennent pas des succès grâce aux essais suivants.

Prochaine priorité : diagnostiquer et réduire les réponses à ancre périmée/texte
attendu absent, puis terminer les contrôles natifs et mesures encore qualifiés
partiels dans la matrice. Aucun GO global T031 ni démarrage de T032 à ce stade.

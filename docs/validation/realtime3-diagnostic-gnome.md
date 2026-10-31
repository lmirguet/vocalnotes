# Recette Voxtral Realtime 3 et transcription temporaire

2026-10-31. Reprise ciblée autorisée par l’utilisateur après échec de la recette
T020–T022. Cette tranche anticipe une partie de T052/T053 pour diagnostiquer la
qualité ; elle ne valide pas US7, ses dépendances, ni les tâches intermédiaires.

## Modèle et protocole

Nouveau défaut et suggestion dans Préférences :
`voxtral-mini-transcribe-realtime-3-0`. Les identifiants déjà enregistrés sont
conservés : choisir explicitement ce modèle puis démarrer une nouvelle session
(pas une reprise de session suspendue). Aucun remplacement silencieux, fournisseur
de secours ou essai API lors du changement de réglage.

Le guide [migration](https://docs.mistral.ai/studio/audio/migrate-from-voxtral-2)
indique ce remplacement de l’identifiant 2602. La
[documentation temps réel](https://docs.mistral.ai/studio/audio/speech-to-text/transcribe-in-realtime)
décrit PCM s16le mono 16 kHz, `language: fr`, délai 480 ms, configuration avant
l’audio et finalisation par `input_audio.end`. L’adaptateur envoie ces paramètres
et retire le flush supplémentaire non décrit dans ce protocole. L’API réelle et
la disponibilité du modèle pour le compte restent à vérifier en recette.

## Vue temporaire

Case explicitement activable seulement pendant l’écoute. Avertissement de
sensibilité et d’inexactitude. Affichage séparé du provisoire concaténé et du
dernier passage non vide finalisé ; le final remplace les deltas pour ce passage.
Un passage vide dû au bruit ne masque pas le dernier final utile. Aucun
historique de passages, offset d’édition, outil ou commande issus de cette vue.

Le cœur borne chaque texte à 65 536 octets et refuse de le collecter si le
diagnostic est désactivé. Purge à la désactivation, pause, arrêt, changement de
note, nouveau départ/reprise et fermeture de fenêtre. Les mises à jour passent
par la garde de session du contrôleur ; les résultats périmés ne repeuplent pas
la vue. La note seule reste sauvegardée. Pas de fichier de transcription,
localStorage, console de contenu ou enregistrement audio. Les copies en mémoire
sont libérées sans promesse d’effacement physique des pages mémoire.

## Vérifications et recette

Tests : opt-in, limites, concaténation, final corrigé, final vide, séparation de
la note, purges pause/arrêt/reprise/désactivation ; rendu de texte non interprété
comme HTML et purge du DOM ; configuration française et fin de passage sans
flush via transport simulé. Les tests de préférences conservent les identifiants
personnalisés/anciens, afin de vérifier l’absence de migration silencieuse.

Recette native à effectuer :

1. Édition → Préférences : sélectionner le nouveau modèle et enregistrer.
2. Préparer une note de test vide. Démarrer le vocal explicitement, puis cocher
   « Afficher la transcription temporaire pour cette écoute ».
3. Dire « Acheter des cahiers ». Comparer le dernier passage finalisé à la parole,
   puis comparer la note au final. Ne pas confondre erreur STT et erreur LLM.
4. Si correct, ajouter les stylos puis corriger cahiers → classeurs.
5. Désactiver/réactiver la vue : vide. Pause/reprise : désactivée et vide.
6. Arrêt puis nouvelle session : désactivée et vide ; note conservée.

Windows et mise en veille hors périmètre utilisateur. Qualité et latence non
validées tant que la recette réelle n’est pas concluante.

Vérifications hors ligne de cette reprise : 12 tests Rust bibliothèque + 109
harnais partagé PASS ; 49 tests JavaScript PASS ; typecheck/build web PASS ;
check Rust toutes cibles avec ressources embarquées PASS ; format et diff PASS.
Journaux techniques : `/tmp/vocalnotes-realtime3-{rust,js,web,check}.log`.
Le test UI de récupération cible maintenant explicitement le champ de saisie
récupérable, pour le distinguer des nouveaux champs de transcription.

Aucune lecture de clé utilisateur, capture, requête fournisseur ou installation
par l’agent pendant ces vérifications ; le test existant de trousseau indisponible
utilise un sous-processus à adresse D-Bus invalide. La fenêtre précédente n’a
pas été redémarrée et le binaire principal reste
`f56ec81343d953871da779d209edd548633e831328167d22f45ee5ebfc6363a6`.
Fermeture volontaire demandée pour charger la nouvelle version.

## Lancement natif pour recette

Fermeture confirmée par l’utilisateur et absence de processus vérifiée. Source
par défaut `nx_remapped_out`. Build Rust embarqué réussi, sans paquet généré.
Nouvelle fenêtre lancée, microphone arrêté au démarrage, session outil 28385.
Binaire : `f30770b459f1c38eafcf409975b7da04e8d6fc508ef5a4e4019cf8de69b5e905`.
Journal technique : `/tmp/vocalnotes-realtime3-native.log` (sans transcription).
Sélection explicite du nouveau modèle dans les préférences et essai utilisateur
encore en attente ; ne pas confondre modèle proposé et modèle effectivement servi.

### Dictée et ajout — PASS utilisateur, réserve de reconnaissance

L’utilisateur compare le final STT à la note : ils correspondent, avec une
substitution d’article dans la première phrase. Cela situe cet écart au niveau
STT, sans démontrer une précision générale ni une supériorité comparative du
nouveau modèle. Il confirme ensuite la réussite de l’ajout de la deuxième idée
en conservant la première. Les traces techniques montrent des opérations insert
acceptées, sans clarification ni rejet à ce stade ; les compteurs sont cumulatifs
et ne permettent pas d’attribuer chaque appel à une seule consigne de recette.
Correction ciblée et purge de la vue restent à vérifier avec l’utilisateur.

### Correction ciblée — PASS utilisateur

L’utilisateur confirme la réussite du remplacement demandé du nom ciblé, avec
conservation de la deuxième idée. Dictée, ajout et correction de cette séquence
sont validés sur GNOME/Wayland avec le micro NoMachine. Cela ne valide pas une
qualité générale du modèle. Désactivation/réactivation du diagnostic, pause et
nouvelle session restent à recetter pour vérifier la purge visible du contenu.

Les traces de la correction finale montrent une opération replace acceptée
(compteur cumulé applied=6). Un essai intermédiaire avait été refusé pour
protection d’un bloc opaque (rejected=1) ; le journal sans contenu ne permet pas
de l’attribuer à une phrase précise. La confirmation utilisateur porte sur le
résultat final, pas sur une absence d’erreurs pendant toute la session.

### Désactivation, pause et reprise — PASS utilisateur

L’utilisateur confirme les trois contrôles : désactivation/réactivation vide
les champs ; après une courte dictée, Pause efface la transcription ; Reprendre
explicitement conserve la vue désactivée et vide. La note reste conservée.

Bilan de la recette ciblée : dictée, ajout, correction, désactivation et purge à
la pause/reprise validés par l’utilisateur sur GNOME/Wayland/NoMachine. Réserve
STT sur une substitution d’article et refus intermédiaire déjà consignés. Pas
de mesure comparative de qualité, ni de certification sans erreur. Purge après
arrêt/nouvelle session vérifiée hors ligne, pas encore confirmée séparément en
recette native. US7 complet et ses dépendances restent ouverts. Pas de recette
Windows ni veille. L’application est laissée dans l’état choisi par l’utilisateur.

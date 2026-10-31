# Bloc-notes vocal intelligent — cahier des charges

**Version :** 0.3 — document de cadrage et spécification du prototype  
**Date :** 30 octobre 2026  
**Statut :** synthèse des décisions exprimées et des propositions restant à valider

## 1. Vision

Créer une application desktop permettant de construire et de modifier des notes structurées en parlant naturellement. L'utilisateur exprime ses idées, ses corrections et ses instructions dans un même flux ; l'application transforme progressivement ces propos en une note synthétique, lisible et éditable.

Il ne s'agit **pas d'une simple dictée vocale** : l'application organise les idées, crée des sections et des listes, et permet de piloter la note à la voix. L'utilisateur reste maître du contenu : il juge la note, la corrige et peut annuler les modifications.

### Principes directeurs

1. **Fidélité** : ne pas inventer de faits, d'idées ni de conclusions non exprimés.
2. **Fluidité** : parler naturellement, sans mot-clé, syntaxe obligatoire ou bouton à maintenir pour chaque intervention.
3. **Autonomie contrôlée** : appliquer les modifications ordinaires immédiatement, sans validation systématique, et permettre leur annulation.
4. **Transparence** : rendre visibles l'état du microphone, les actions et les erreurs.
5. **Maîtrise des données** : conserver la note, mais pas l'audio, la transcription ni le lexique de session.
6. **Ouverture** : séparer les fournisseurs de transcription et de modèle de langage pour pouvoir les remplacer.
7. **Légèreté** : viser une application multiplateforme, rapide à lancer et sobre en ressources ; mesurer ces propriétés.

### 1.1 Cas d'utilisation visés

- **Rédaction d'un e-mail :** l'utilisateur énonce les éléments de son message et demande les ajustements souhaités ; l'application produit un brouillon qu'il peut copier et reprendre dans sa messagerie. L'envoi d'e-mails et l'intégration à une messagerie ne font pas partie du périmètre initial.
- **Compte rendu de réunion :** après une réunion, l'utilisateur rapporte oralement les points abordés et, s'il les précise, les décisions et actions. L'application les organise sans inventer de participants, d'attributions, de dates ou de décisions. Ce cas ne nécessite pas l'enregistrement de la réunion ni la transcription de plusieurs intervenants.
- **Note de synthèse :** l'utilisateur expose ses idées, éventuellement dans le désordre ; l'application les regroupe et en fait ressortir les points importants sans leur attribuer des conclusions non exprimées.
- **Document professionnel :** l'utilisateur prépare ou retravaille oralement un texte destiné à un document professionnel, avec une structure et un niveau de rédaction adaptés à sa demande.

Ces usages ne se limitent pas à des listes : selon l'intention de l'utilisateur, le résultat peut être rédigé sous forme de paragraphes, de sections, de listes ou d'un mélange. Le ton, le degré de détail et la structure peuvent être ajustés à la voix, sans ajout de faits. Aucun modèle de document particulier ni mode obligatoire n'est imposé à ce stade.

### 1.2 Intention de la note

Au début d'une nouvelle note, l'utilisateur peut exprimer oralement son intention, par exemple : « Je voudrais rédiger un e-mail », « Faisons un compte rendu de réunion » ou « Je prépare une note de synthèse ». Il peut ensuite donner des précisions sur le destinataire, le ton ou le niveau de détail, ou modifier cette intention en cours de rédaction.

L'application utilise cette intention comme contexte pour adapter la forme et la rédaction de la note. Elle distingue l'intention et les consignes du contenu à rédiger : « je voudrais écrire un e-mail » ne doit pas apparaître automatiquement dans le message. Elle ne déduit pas de faits, de destinataire, de décisions ou d'informations absentes des propos de l'utilisateur.

L'intention peut être donnée librement, sans formulaire, mot-clé ni choix de modèle obligatoire. Si elle n'est pas précisée, l'utilisateur peut commencer à parler normalement et la préciser ensuite. Une intention fournie en cours de rédaction ne doit pas provoquer la perte silencieuse du contenu existant.

## 2. Périmètre

### 2.1 Première version / prototype

- Application desktop en français, fonctionnelle dès le premier prototype sur Linux avec GNOME/Wayland (notamment Fedora) et sur Windows 11. macOS reste un objectif de la première version.
- Une note ouverte à la fois, nouvelle ou issue d'un fichier Markdown existant.
- Éditeur visuel, mode vocal principal et mode d'édition manuelle distinct.
- Session d'écoute continue activée une fois, y compris en arrière-plan, jusqu'à sa mise en pause ou son arrêt ; le passage en mode manuel, le verrouillage, la mise en veille et la fermeture de la fenêtre interrompent également la capture.
- Transcription progressive, mise en forme des idées et commandes naturelles d'édition.
- Corrections de mots par épellation, avec lexique temporaire de session.
- Application immédiate des modifications ordinaires, annulation et rétablissement.
- Fichier Markdown local, sauvegarde automatique, ouverture, copie et export du Markdown.
- Indication du coût estimé de la session, de l'état du microphone et des erreurs.
- Panneau de préférences pour configurer l'application et ses fournisseurs d'IA, notamment la clé API Mistral.
- Interfaces permettant plusieurs fournisseurs, avec Mistral comme première intégration visée.

Le fonctionnement sur **GNOME/Wayland et Windows 11 est une exigence dès le premier prototype**, et non une adaptation reportée à la fin du développement. Les choix techniques et chaque jalon fonctionnel doivent être validés sur ces deux environnements. La compatibilité macOS reste un objectif de la première version. Toute compatibilité annoncée doit être démontrée et non supposée ; les mesures et limites observées seront documentées.

### 2.2 Hors périmètre initial

- Application mobile, synchronisation entre appareils et collaboration multiutilisateur.
- Transcription de réunions à plusieurs et identification des intervenants.
- Recherche avancée dans une collection de notes.
- Dictionnaire personnel conservé d'une session à l'autre.
- Fonctionnement intégralement local et intégration effective d'OpenAI : évolutions envisageables, non garanties par le prototype.
- Enregistrement durable de l'audio, de la transcription ou du journal conversationnel.

## 3. Parcours utilisateur

1. L'utilisateur crée une note ou ouvre un fichier Markdown existant.
2. Il démarre explicitement une session vocale et autorise le microphone si nécessaire.
3. Il peut indiquer oralement l'intention de la note dès le départ, puis parle librement : idées, reformulations, instructions et corrections peuvent se succéder ou se mélanger. Il peut aussi préciser ou ajuster l'intention plus tard.
4. La transcription peut s'afficher progressivement dans une vue de débogage activée par l'utilisateur ; la note se met à jour au fil des propos dès qu'un passage est suffisamment complet pour être interprété. L'utilisateur voit où l'application modifie le document.
5. Les modifications ordinaires apparaissent sans demande préalable de confirmation. L'application indique brièvement l'action réalisée.
6. L'utilisateur peut corriger ou annuler oralement, mettre l'écoute en pause ou passer explicitement en mode manuel.
7. La note est enregistrée localement. À la fin de la session, les données temporaires de conversation et le lexique de session sont éliminés.

L'écoute reste active même si l'utilisateur passe à une autre application. Après un long silence, l'application peut **proposer** d'arrêter l'écoute, mais ne l'arrête pas automatiquement pour ce seul motif.

Le passage en mode manuel arrête la capture et invalide les traitements vocaux en attente. Le verrouillage de l'écran ou la mise en veille provoque ce même passage en mode manuel ; la reprise vocale exige une action explicite. À la fermeture de la fenêtre, la note est sauvegardée avant de terminer la session et de fermer la fenêtre.

## 4. Exigences fonctionnelles

### 4.1 Construction de la note

- **F01 — Parole continue :** accepter une session vocale en français, sans activation à chaque intervention.
- **F02 — Synthèse :** organiser et reformuler de façon concise, en préservant le sens, les chiffres, noms, décisions, réserves et questions importantes.
- **F03 — Fidélité :** ne pas ajouter de faits ou de conclusions ; conserver la distinction entre hypothèse, question et affirmation.
- **F04 — Structure :** créer des titres, sections, paragraphes et listes pour rendre la note claire, y compris lorsque les idées arrivent dans le désordre. Les catégories peuvent être déduites de l'organisation des propos, sans inventer leur contenu.
- **F05 — Placement :** rattacher une nouvelle idée à la section pertinente ou créer une section appropriée.
- **F06 — Ajustement :** accepter des demandes telles que « sois plus concis » ou « développe ce point », sans inventer de contenu pour le développer.

### 4.2 Instructions et modifications vocales

- **F07 — Langage naturel :** comprendre des formulations libres pour ajouter, corriger, reformuler, déplacer, supprimer ou restructurer des éléments, sans mot-clé obligatoire.
- **F08 — Intention :** distinguer une idée à noter d'une instruction adressée à l'application et d'une correction d'un propos précédent.
- **F09 — Intentions multiples :** traiter plusieurs idées et instructions dans une même intervention, dans un ordre cohérent.
- **F10 — Cible :** identifier l'élément visé à partir de la structure de la note, de son contenu et du contexte récent de la conversation ou des actions.
- **F11 — Ambiguïté :** demander une précision lorsque l'intention ou la cible ne peut pas être déterminée de manière suffisamment fiable ; ne pas modifier au hasard.
- **F12 — Portée :** limiter une modification à la zone concernée, sauf demande explicite de réorganisation ou de reformulation plus large.
- **F13 — Application :** appliquer directement les changements ordinaires, sans validation préalable. Un retour discret indique l'action effectuée.
- **F14 — Correction naturelle :** interpréter une formulation comme « non, je voulais plutôt dire… » à la lumière du contexte ; demander si la référence reste ambiguë.

Exemples de commandes : « ajoute une section sur les risques », « déplace le point sur les délais dans les priorités », « remplace maîtrise par réduction des coûts », « retire le deuxième point » et « réorganise le plan par priorité ».

### 4.3 Annulation et rétablissement

- **F15 — Annulation :** accepter « annule », « annule les deux dernières modifications » et des commandes équivalentes ; proposer aussi une commande au clavier ou dans l'interface.
- **F16 — Rétablissement :** permettre de rétablir une modification annulée, à la voix et dans l'interface.
- **F17 — Unité d'action :** une suppression ou réorganisation complète doit pouvoir être annulée comme une action cohérente. Le découpage des interventions mixtes en actions doit être explicite et testé.
- **F18 — Traitements en attente :** une demande d'annulation prononcée immédiatement après une action doit produire un résultat cohérent même si cette action n'est pas encore affichée ; un résultat tardif ne doit pas la réappliquer silencieusement.
- **F19 — Durée de l'historique :** l'annulation est requise pendant la session. L'historique est effacé à la fin de la session et n'est pas conservé après fermeture ; seule la note est conservée.

### 4.4 Épellation et correction des mots

- **F20 — Épellation :** permettre d'épeler un mot mal reconnu ou nouveau, sans syntaxe imposée : « Voxtral s'écrit V-O-X-T-R-A-L ».
- **F21 — Caractères :** comprendre autant que possible les indications de majuscule, accent, tiret et apostrophe.
- **F22 — Réutilisation :** conserver temporairement l'association utile entre un terme entendu et son orthographe corrigée pour améliorer son traitement pendant la session.
- **F23 — Portée :** corriger l'occurrence visée ; permettre une correction globale lorsqu'elle est explicitement demandée ou que sa portée est claire. Ne pas remplacer silencieusement des termes sans rapport.
- **F24 — Incertitude :** demander de répéter ou de préciser si le mot ou les lettres restent ambigus. L'épellation n'est pas elle-même ajoutée à la note, sauf demande explicite.
- **F25 — Fin de session :** supprimer le lexique temporaire. Les corrections déjà intégrées à la note sont conservées dans le document.

La transcription de lettres peut elle-même être erronée ; la compréhension de l'épellation doit être vérifiée expérimentalement avec le fournisseur choisi.

### 4.5 Modes d'édition

- **F26 — Mode vocal :** mode principal, utilisable sans clavier ; les idées et commandes modifient la note à partir de la parole.
- **F27 — Mode manuel :** l'utilisateur peut éditer directement la note dans un mode distinct.
- **F28 — Transition explicite :** le passage d'un mode à l'autre est visible et résulte d'une action de l'utilisateur, sauf le passage automatique en mode manuel au verrouillage ou à la mise en veille prévu par F71. En passant au mode manuel, la capture vocale s'arrête et les traitements vocaux en attente sont invalidés sans attendre leur résultat. Aucun résultat de ces traitements ne peut ensuite modifier la note ; les passages non intégrés sont signalés. Les modifications déjà appliquées sont conservées.
- **F29 — Conflits :** une réponse vocale tardive ne doit pas écraser une modification manuelle ; si la version de la note a changé, l'opération est réévaluée ou signalée plutôt qu'appliquée aveuglément.

### 4.6 Notes et Markdown

- **F30 — Édition visuelle :** présenter la note comme un document mis en forme ; prendre en charge au minimum titres, paragraphes, listes, gras et italique.
- **F31 — Fichiers :** créer, ouvrir, enregistrer et retrouver une note ; proposer « Enregistrer sous ».
- **F32 — Format :** utiliser un fichier Markdown local comme format de la note ; permettre la copie du Markdown brut et son export.
- **F33 — Note existante :** permettre de reprendre un fichier Markdown et de le retravailler oralement.
- **F34 — Préservation :** ne pas dégrader silencieusement les constructions non prises en charge par l'éditeur visuel, notamment tableaux et blocs de code : les préserver ou signaler la limite avant toute modification susceptible de les altérer.
- **F35 — Sauvegarde :** sauvegarder automatiquement et permettre de retrouver la dernière note enregistrée après redémarrage ou interruption.
- **F36 — Modification externe :** détecter ou gérer les modifications apportées au fichier par une autre application ; ne pas les écraser silencieusement.

### 4.7 Microphone, interruptions et arrière-plan

- **F37 — Démarrage :** l'utilisateur lance explicitement la session et accorde l'accès au microphone selon les règles du système.
- **F38 — Écoute :** l'écoute reste active lorsque la fenêtre perd le focus. Elle s'interrompt à la pause ou à l'arrêt demandé, au passage en mode manuel, au verrouillage, à la mise en veille ou à la fermeture de la fenêtre, selon F28, F71 et F72.
- **F39 — État :** afficher clairement les états de microphone et de traitement : écoute, pause, traitement en cours, demande de précision ou erreur.
- **F40 — Arrière-plan :** prévoir un indicateur visible hors de la fenêtre principale et un moyen rapide de mettre l'écoute en pause. Leur fonctionnement et leur ergonomie doivent être vérifiés dès le premier prototype sur GNOME/Wayland et Windows 11, puis sur toute autre plateforme annoncée comme prise en charge.
- **F41 — Silence :** une proposition d'arrêt après un long silence est permise ; elle ne provoque pas, à elle seule, un arrêt automatique.
- **F42 — Coupure :** signaler une perte de connexion, de microphone ou de service, et indiquer dans la mesure du possible ce qui a été traité et ce qui a pu être perdu. Ne pas prétendre avoir récupéré les paroles non traitées.
- **F43 — Confidentialité :** signaler que l'écoute en arrière-plan peut capter des paroles non destinées à l'application, y compris celles d'autres personnes.

### 4.8 Coûts et paramètres

- **F44 — Coût :** afficher une estimation du coût de la session pendant son utilisation, couvrant autant que possible transcription et LLM.
- **F45 — Transparence :** distinguer les mesures communiquées par les fournisseurs des estimations ; afficher les limites des données disponibles. L'estimation n'est pas une facture.
- **F46 — Configuration :** prévoir le choix séparé du fournisseur et du modèle de transcription et de ceux du LLM, ainsi que les paramètres et clés nécessaires.
- **F47 — Clés :** stocker les clés API dans le trousseau sécurisé du système, jamais dans les notes Markdown.
- **F48 — Pas de bascule implicite :** ne pas envoyer automatiquement les données à un autre fournisseur en cas de panne sans choix explicite de l'utilisateur.

Pour le prototype, l'interface et les adaptateurs peuvent ne proposer que Mistral, tout en conservant les points d'extension nécessaires à l'ajout d'autres fournisseurs.

### 4.9 Intention de rédaction

- **F49 — Expression vocale :** permettre à l'utilisateur d'indiquer librement l'intention de sa note, notamment dès le début d'une nouvelle note : e-mail, compte rendu, synthèse ou document professionnel, par exemple.
- **F50 — Adaptation :** utiliser l'intention et les indications éventuelles de ton ou de forme pour produire une rédaction adaptée, sans les insérer automatiquement comme contenu et sans inventer de faits.
- **F51 — Évolution :** permettre de préciser ou changer l'intention à la voix au fil de la session, sans perdre silencieusement le contenu existant. Si l'effet demandé sur la note est ambigu, demander une précision.
- **F52 — Souplesse :** ne pas imposer de déclaration d'intention, de modèle prédéfini ou de formulaire avant de commencer à dicter.

### 4.10 Visualisation des modifications en temps réel

- **F53 — Note vivante :** afficher progressivement les modifications dans la note pendant la session, sans attendre sa fin ni une validation systématique. L'objectif est une expérience aussi proche du temps réel que possible ; un délai de transcription et d'interprétation reste inévitable.
- **F54 — Zone modifiée :** montrer par un indicateur visuel où l'application est en train d'écrire ou vient d'appliquer une modification : par exemple un curseur distinct et une mise en évidence temporaire de la zone concernée.
- **F55 — Sens de l'indicateur :** distinguer le traitement en cours d'une modification effectivement appliquée. Ne pas afficher une position d'écriture précise tant que l'application ne connaît pas la cible ; signaler plutôt que l'interprétation est en cours.
- **F56 — Autres opérations :** rendre également repérables les déplacements, suppressions et réorganisations, qui ne correspondent pas nécessairement à un point d'écriture unique. Un retour d'action complète si nécessaire la mise en évidence dans le document.
- **F57 — Lisibilité :** l'indicateur ne doit pas masquer le texte et ne doit pas reposer uniquement sur la couleur. Les modalités exactes de mise en évidence restent à tester.
- **F58 — Respect de la navigation :** si l'application fait défiler la note pour montrer une modification, elle ne doit pas empêcher l'utilisateur de consulter une autre partie du document. En arrière-plan, les mises à jour ne doivent pas ramener la fenêtre au premier plan contre sa volonté.

L'affichage peut être progressif par passages ou opérations validés ; il ne promet pas une écriture mot à mot si le traitement ou la sécurité des modifications ne le permettent pas. Les modifications affichées restent soumises aux règles de fidélité, d'ordre et d'annulation.

### 4.11 Panneau de préférences

- **F59 — Accès :** proposer dans l'application un panneau de préférences permettant de consulter et modifier les réglages disponibles, sans modifier manuellement un fichier de configuration.
- **F60 — Fournisseurs :** permettre de sélectionner séparément, parmi les intégrations disponibles, le fournisseur et le modèle de transcription et ceux du modèle de langage. La première version vise Mistral ; OpenAI est prévu comme intégration ultérieure et ne doit pas être présenté comme fonctionnel tant qu'il n'est pas implémenté.
- **F61 — Clés API :** permettre de saisir, remplacer et supprimer la clé de chaque fournisseur intégré, notamment Mistral puis OpenAI lorsque son intégration existera. La saisie est masquée ; l'interface indique si une clé est enregistrée sans afficher sa valeur complète.
- **F62 — Stockage sécurisé :** enregistrer les clés dans le trousseau sécurisé du système, pas dans la note, les fichiers de configuration ordinaires ni les journaux. Si le trousseau est indisponible, signaler le problème sans enregistrer silencieusement la clé en clair.
- **F63 — Vérification :** permettre de tester explicitement la configuration d'un fournisseur et afficher un résultat compréhensible. Ce test peut contacter le fournisseur et engendrer un coût ; le signaler avant son exécution. Ne pas afficher la clé dans les messages d'erreur.
- **F64 — Réglages de comportement :** présenter, selon les fonctionnalités effectivement disponibles, les options relatives au microphone, à la proposition après silence prolongé, à l'affichage et aux indications de coût. Les choix précis et leurs valeurs par défaut restent à définir ; les exigences de confidentialité et d'écoute continue ne sont pas modifiées implicitement.
- **F65 — Application des changements :** indiquer si un réglage prend effet immédiatement ou à la prochaine session. Ne pas changer silencieusement de fournisseur pendant une session en cours ni envoyer des données à un fournisseur non choisi.

### 4.12 Transcription de débogage

- **F66 — Affichage optionnel :** permettre d'activer explicitement, pour la session en cours, une vue de débogage montrant le texte brut renvoyé par le service de transcription avant interprétation ou reformulation par le modèle de langage. Cette vue est désactivée par défaut.
- **F67 — Résultats intermédiaires :** distinguer les résultats provisoires des segments finalisés lorsque le fournisseur expose cette information, et refléter les corrections éventuelles apportées par le service.
- **F68 — Exactitude :** présenter ce texte comme la transcription brute reconnue, pas comme une reproduction garantie exacte des paroles réellement prononcées. Il peut contenir des erreurs ou omissions.
- **F69 — Confidentialité :** prévenir que la transcription peut contenir des données sensibles. Ne pas l'enregistrer durablement par défaut ; effacer la vue et les données temporaires associées à la fin de la session. L'audio n'est pas enregistré pour cette fonction.
- **F70 — Console :** une sortie vers la console peut être proposée comme option de débogage distincte et explicitement activée. Avertir qu'un terminal, une redirection ou un outil externe peut conserver cette sortie indépendamment de l'application ; ne pas l'émettre dans les journaux persistants de l'application par défaut.

### 4.13 Verrouillage, mise en veille et fermeture

- **F71 — Verrouillage et veille :** au verrouillage de l'écran ou à la mise en veille, arrêter la capture audio et passer en mode édition manuelle, en invalidant les traitements vocaux en attente selon F28. Au retour, rester en mode manuel ; ne pas reprendre automatiquement l'écoute. Vérifier ce comportement sur chaque plateforme.
- **F72 — Fermeture :** à la demande de fermeture de la fenêtre, arrêter la capture et invalider les traitements vocaux en attente selon F28, puis sauvegarder la note dans son état courant avant de terminer la session et de fermer la fenêtre. Ne pas continuer l'écoute en arrière-plan après fermeture. Si la sauvegarde échoue, signaler l'erreur et ne pas fermer silencieusement en perdant les modifications.

## 5. Compréhension des intentions : conception centrale

### 5.1 Cas à distinguer

| Parole | Interprétation attendue |
| --- | --- |
| « Il faut maîtriser les coûts. » | Ajouter une idée à la note. |
| « Déplace le point sur les coûts dans les priorités. » | Modifier la structure de la note. |
| « Non, je voulais dire maîtriser les coûts, pas les réduire. » | Corriger l'idée ou l'action visée par le contexte. |
| « Dans la procédure, il faut supprimer les fichiers temporaires. » | Noter une idée ; ne pas supprimer une partie de la note. |
| « La maîtrise des coûts est importante. Ajoute-la aux priorités et place la sécurité en premier. » | Comprendre une idée et plusieurs instructions liées. |
| « Supprime ce point. » | Résoudre la référence à partir du contexte ; demander si plusieurs cibles sont plausibles. |

Le système doit tenir compte du document courant, de la structure des sections, des derniers propos utiles et des dernières actions. La sélection ou le curseur, lorsqu'ils existent, peuvent également constituer des indices ; leur priorité exacte reste à tester. La transcription et le contexte conversationnel ne sont conservés que temporairement.

### 5.2 Limites et règles de sûreté

- Une transcription peut être incorrecte ; une interprétation peut aussi être erronée malgré une transcription correcte.
- Aucun score de confiance du modèle ne garantit à lui seul que l'action est juste. La stratégie combine contexte, validation technique, demande de précision si nécessaire et annulation.
- Une demande de clarification concerne une ambiguïté réelle ; ce n'est pas une validation systématique des modifications ordinaires.
- Les instructions exprimées dans la note existante sont du contenu à éditer, pas automatiquement des instructions adressées à l'application.
- L'utilisateur demeure responsable de vérifier que la synthèse reflète ses propos. La fidélité est une exigence à évaluer, non une garantie absolue.

## 6. Architecture envisagée

### 6.1 Chaîne de traitement

```text
Microphone
   ↓
Capture et transmission audio
   ↓
Adaptateur de transcription → transcription progressive
   ↓
Segmentation en passages cohérents
   ↓
Adaptateur LLM → intentions et opérations structurées
   ↓
Validation des opérations et de la version du document
   ↓
Application transactionnelle, historique et affichage
   ↓
Sauvegarde locale du Markdown
```

1. La transcription et la fin d'un passage sont détectées sans imposer une commande vocale ou un bouton de fin.
2. Le modèle de langage reçoit le passage, le contexte utile et l'état pertinent de la note ; il propose une ou plusieurs opérations structurées ou une demande de précision.
3. L'application vérifie la forme des opérations, l'existence des cibles et la version du document. Elle n'accorde pas au modèle le contrôle direct du fichier.
4. Les opérations sont appliquées de manière ordonnée et réversible. Une réponse périmée est réévaluée ou signalée, jamais appliquée aveuglément.
5. Le document est présenté dans l'éditeur et sauvegardé. Les effets d'une opération sont visibles et annulables.

La validation structurelle et le contrôle de version limitent les risques de corruption ou d'écrasement ; ils ne prouvent pas que l'interprétation sémantique est correcte.

### 6.2 Technologies candidates

- **Application desktop :** Tauri est le candidat recommandé, car il permet de cibler les trois systèmes en utilisant le moteur web disponible sur chacun. Ce choix, l'éditeur visuel et les dépendances natives restent à valider par le prototype.
- **Transcription initiale :** Voxtral en temps réel, sous réserve de disponibilité, de compatibilité API et de tests en français.
- **Interprétation initiale :** Mistral Large 4, candidat à évaluer pour les sorties structurées, la fidélité, la latence et le coût.
- **Éditeur :** composant visuel compatible avec la lecture et l'écriture du Markdown ; choix non arrêté.
- **Futur fournisseur :** OpenAI est un exemple de deuxième intégration, hors périmètre initial.

La légèreté et le temps de démarrage ne découlent pas automatiquement du choix de Tauri. Les dépendances, permissions micro et performances réelles doivent être testées sur chaque plateforme. Le choix du socle desktop est conditionné par son fonctionnement effectif sur GNOME/Wayland et Windows 11 dès le premier prototype, notamment pour la capture micro, l'arrière-plan, l'indicateur d'écoute, la pause rapide, le trousseau sécurisé et les transitions au verrouillage ou à la mise en veille.

### 6.3 Adaptateurs de fournisseurs

L'application distingue deux interfaces configurables indépendamment :

1. **Transcription :** audio en entrée, texte progressif et événements de session en sortie.
2. **Interprétation :** passage transcrit et contexte pertinent en entrée, opérations structurées ou demande de précision en sortie.

Chaque adaptateur traduit ses propres paramètres, capacités, erreurs et données de consommation vers des interfaces communes. Les API ne sont pas parfaitement interchangeables : transcription en continu, aide au vocabulaire, sorties structurées, latence et métriques doivent être testées pour chaque combinaison. Le lexique de session peut être transmis au moteur si celui-ci le permet, ou servir à corriger l'interprétation après transcription ; aucune adaptation ou réentraînement du modèle n'est présumé.

## 7. Données, confidentialité et sécurité

### 7.1 Données locales

| Donnée | Politique pour la première version |
| --- | --- |
| Note Markdown | Conservée localement et sauvegardée automatiquement. |
| Audio | Utilisé pour le traitement ; pas d'enregistrement durable par défaut. |
| Transcription et contexte conversationnel | Utilisés temporairement ; non conservés après la session. |
| Lexique de mots corrigés | En mémoire pendant la session ; oublié à la fin. |
| Historique d'annulation | Disponible pendant la session ; effacé à sa fin, sans persistance après fermeture. |
| Clés API | Dans le trousseau sécurisé du système, hors des fichiers Markdown. |

Les informations sensibles ne doivent pas être inscrites par inadvertance dans des journaux ou fichiers temporaires durables. Les limites des mécanismes de stockage temporaire et de récupération doivent être documentées.

### 7.2 Services externes

Une API hébergée reçoit l'audio nécessaire à la transcription ; le fournisseur du LLM reçoit le texte et le contexte de note nécessaires à l'interprétation. L'application indique clairement les services utilisés et la nature des données transmises.

Ne pas conserver localement la transcription ou l'audio ne garantit pas leur suppression chez les fournisseurs. Leurs conditions de traitement et de conservation doivent être vérifiées avant utilisation, en particulier pour des données professionnelles. Aucun basculement automatique vers un fournisseur non choisi.

## 8. Exigences non fonctionnelles

- **Réactivité :** afficher rapidement la transcription et mettre à jour la note après un passage suffisamment complet. Mesurer les délais réels de bout en bout ; ne pas confondre latence d'un modèle et latence totale de l'application.
- **Ressources :** mesurer dès le premier prototype le temps de lancement, la mémoire et la consommation de ressources sur GNOME/Wayland et Windows 11, avec le matériel et les versions précisés ; effectuer également ces mesures sur macOS avant d'en annoncer la prise en charge.
- **Résilience :** conserver la dernière note sauvegardée après plantage ou interruption ; rendre les pertes éventuelles de paroles explicites.
- **Fiabilité :** éviter les modifications involontaires, duplications et écrasements silencieux ; rendre les opérations annulables.
- **Portabilité :** valider dès le premier prototype, puis à chaque jalon fonctionnel, l'installation, les permissions micro, l'écoute en arrière-plan et l'indicateur de session sur GNOME/Wayland et Windows 11. Étendre ces vérifications à toute autre plateforme avant d'en annoncer la prise en charge.
- **Lisibilité :** fournir une interface en français, avec état du micro et des traitements compréhensible à tout moment.
- **Coûts :** regrouper les passages de manière cohérente plutôt que solliciter le LLM à chaque mot ; mesurer les compromis entre coût, fidélité et réactivité.

Les seuils chiffrés de latence, d'usage des ressources et de qualité restent à définir après les premières mesures ; ils ne sont pas présentés ici comme des résultats acquis.

## 9. Scénarios de recette du prototype

Chaque scénario est à exécuter et à documenter avec le fournisseur, le modèle, le système et les conditions d'essai. Évaluer séparément transcription, interprétation, modification de la note, délai et récupération d'erreur.

| ID | Scénario | Résultat attendu |
| --- | --- | --- |
| R01 | Exprimer des idées en désordre pendant plusieurs minutes. | Note structurée et fidèle, sans faits inventés ; nuances importantes conservées. |
| R02 | Mélanger idées et plusieurs commandes dans une même intervention. | Chaque intention est traitée dans un ordre cohérent, sans inscrire les commandes comme contenu par défaut. |
| R03 | Dire « il faut supprimer les fichiers temporaires » à propos d'une procédure. | L'idée est notée ; aucun élément de la note n'est supprimé par erreur. |
| R04 | Dire « supprime ce point » alors que plusieurs éléments sont plausibles. | Une précision est demandée ; aucune cible n'est supprimée au hasard. |
| R05 | Corriger une idée avec « non, je voulais dire… ». | L'élément visé est corrigé sans réécriture inutile du reste. |
| R06 | Demander une réorganisation puis « annule ». | La note revient à son état précédent ; l'action peut ensuite être rétablie. |
| R07 | Demander une suppression puis immédiatement son annulation, avant affichage. | La suppression n'est pas appliquée tardivement après l'annulation. |
| R08 | Épeler un terme mal transcrit, puis le répéter plus tard dans la session. | Le terme ciblé est corrigé et sa correction est réutilisée quand possible ; une lettre ambiguë peut être redemandée. |
| R09 | Fermer la session après correction d'un mot. | La correction reste dans la note ; le lexique temporaire n'est pas conservé. |
| R10 | Passer en mode manuel pendant un traitement vocal. | La capture s'arrête et les traitements en attente sont invalidés ; leurs résultats tardifs ne modifient pas la note. Les passages non intégrés sont signalés et l'édition manuelle est possible sans attendre ces résultats. |
| R11 | Ouvrir un Markdown contenant tableau ou bloc de code et le modifier. | Contenu non pris en charge préservé ou limite signalée avant une opération qui l'altérerait. |
| R12 | Modifier le fichier dans une autre application. | Le changement externe n'est pas écrasé silencieusement. |
| R13 | Passer à une autre fenêtre pendant l'écoute. | L'écoute et la mise à jour continuent ; état visible et pause rapide disponibles selon les capacités vérifiées du système. |
| R14 | Couper le réseau ou le microphone pendant la session. | L'utilisateur est averti ; le statut des paroles traitées ou potentiellement perdues est explicite. |
| R15 | Interrompre puis relancer l'application. | La dernière note sauvegardée est récupérable ; aucune transcription ou audio de session n'est conservé par défaut. |
| R16 | Installer et utiliser chaque jalon fonctionnel du prototype sur GNOME/Wayland et Windows 11 ; effectuer également la recette sur macOS avant d'en annoncer la prise en charge. | Dès le premier prototype, installation, démarrage, micro, arrière-plan et fonctions livrées sont vérifiés sur les deux environnements obligatoires ; limites et mesures sont documentées. La compatibilité macOS n'est annoncée qu'après vérification. |
| R17 | Observer le coût au fil d'une session. | Les composantes disponibles sont affichées comme estimation, avec leurs limites. |
| R18 | Dicter les éléments d'un e-mail puis demander une formulation plus concise. | Un brouillon rédigé est produit et ajusté sans ajout de faits ; il peut être repris dans une messagerie, sans envoi automatique. |
| R19 | Rapporter les points d'une réunion, une décision et une action sans responsable précisé. | Le compte rendu distingue les éléments exprimés sans inventer de responsable, de date ou de décision. |
| R20 | Exposer des idées dans le désordre et demander une note de synthèse. | Les idées sont regroupées clairement, sans conclusion ou contenu nouveau. |
| R21 | Dicter un passage pour un document professionnel et demander des paragraphes plutôt qu'une liste. | Le résultat adopte la forme demandée et reste fidèle aux informations exprimées. |
| R22 | Commencer une note par « je voudrais rédiger un e-mail », puis dicter le message. | L'application adapte la rédaction à un e-mail sans insérer la déclaration d'intention dans le brouillon ni inventer de destinataire. |
| R23 | Commencer à dicter sans préciser d'intention, puis demander une note de synthèse. | La dictée peut commencer immédiatement ; l'intention est prise en compte ensuite sans perte silencieuse du contenu. |
| R24 | Dicter plusieurs idées et demander une modification pendant une session. | La note évolue progressivement sans attendre la fin de la session ; la zone modifiée est repérable et l'état de traitement distinct d'une action appliquée. |
| R25 | Demander un déplacement ou une suppression, puis consulter une autre partie de la note ou passer dans une autre application. | L'action est rendue perceptible sans imposer une cible d'écriture fictive ni ramener la fenêtre au premier plan ; la navigation reste possible. |
| R26 | Saisir puis remplacer une clé Mistral dans les préférences. | La saisie est masquée, l'état de configuration est visible et la clé est stockée dans le trousseau sans apparaître dans la note ou les journaux. |
| R27 | Configurer un fournisseur alors que le trousseau sécurisé est indisponible. | L'utilisateur est averti ; aucune clé n'est enregistrée silencieusement en clair. |
| R28 | Tester explicitement une configuration ou changer de fournisseur. | Le test indique s'il peut contacter le service et entraîner un coût ; aucun basculement ou envoi vers un autre fournisseur n'a lieu silencieusement. |
| R29 | Activer la vue de débogage, dicter une phrase puis terminer la session. | Le texte brut du service est visible, ses segments provisoires et finalisés sont distingués lorsqu'ils sont disponibles, puis la vue est vidée en fin de session. |
| R30 | Utiliser l'application sans activer le débogage, puis activer explicitement la sortie console. | Aucune transcription n'est émise par défaut ; l'activation de la console avertit de sa conservation éventuelle par le terminal ou un outil externe. |
| R31 | Verrouiller l'écran ou mettre le système en veille pendant l'écoute, puis revenir. | La capture s'arrête, le mode manuel est activé et les traitements vocaux en attente sont invalidés. L'écoute ne reprend pas sans action explicite, et aucun résultat tardif ne modifie la note. |
| R32 | Fermer la fenêtre avec des modifications et un traitement vocal en attente ; répéter avec une sauvegarde en échec. | La capture s'arrête et le traitement en attente est invalidé. En cas de succès, la note courante est sauvegardée avant la fin de session et la fermeture, sans écoute résiduelle. En cas d'échec, l'erreur est visible et la fenêtre n'est pas fermée silencieusement en perdant les modifications. |
| R33 | Terminer une session ayant un historique d'annulation, puis ouvrir une nouvelle session. | La note conserve les modifications appliquées, mais l'historique de la session précédente n'est plus disponible et n'a pas été enregistré durablement. |

Le jeu de tests doit inclure hésitations, noms propres, chiffres, accents, interventions longues et ambiguïtés. Les résultats et les limites observées servent à fixer des objectifs mesurables avant une version plus aboutie.

## 10. Décisions validées, ouvertes et propositions

| Sujet | Situation actuelle |
| --- | --- |
| Plateformes dès le premier prototype | Décidé : GNOME/Wayland et Windows 11 sont obligatoires dès le départ et à chaque jalon fonctionnel ; macOS reste un objectif de la première version. |
| Demande de précision en arrière-plan | Les demandes de précision restent autorisées en cas d'ambiguïté réelle, sans confirmation systématique des modifications ordinaires. Moyen à choisir : notification visuelle, retour vocal ou combinaison. Le mécanisme doit rester utilisable hors de la fenêtre. |
| Historique après fermeture | Décidé : historique limité à la session, effacé à sa fin et non conservé après fermeture. |
| Priorité des indices pour désigner une cible | Note, contexte récent et éventuelle sélection sont des indices ; priorité et comportement en cas de conflit à tester. |
| Silence prolongé | Proposition d'arrêt autorisée ; seuil et présentation à définir. Pas d'arrêt automatique pour ce seul motif. |
| Verrouillage et mise en veille | Décidé : arrêt de la capture et passage en mode manuel, avec invalidation des traitements vocaux en attente ; reprise vocale uniquement sur action explicite. À tester sur chaque plateforme. |
| Fermeture de fenêtre | Décidé : arrêt de la capture, invalidation des traitements en attente, sauvegarde de la note courante, puis fin de session et fermeture. Un échec de sauvegarde est signalé sans fermeture silencieuse entraînant une perte. |
| Transition entre modes et traitements en attente | Décidé : le passage en manuel arrête le micro et invalide les traitements vocaux en attente, sans attendre leur résultat ; les passages non intégrés sont signalés et aucun résultat tardif n'est appliqué. |
| Éditeur visuel et prise en charge Markdown | Composant et stratégie de préservation des constructions non prises en charge à choisir et tester. |
| Copie vers une messagerie ou un document | Le Markdown peut être copié ; l'intérêt et les modalités d'une copie en texte brut ou avec mise en forme sont à préciser. |
| Indicateur de modification et défilement | Curseur, surlignage, durée d'affichage et comportement du défilement à concevoir et tester, y compris pour les modifications portant sur plusieurs zones. |
| Options et valeurs par défaut | Réglages précis du microphone, du silence, de l'affichage et du coût à déterminer selon les fonctions retenues et leurs contraintes sur chaque plateforme. |
| Choix définitifs des modèles et tarifs | Voxtral et Mistral Large 4 sont les candidats initiaux ; capacités, conditions, coûts et latences doivent être revérifiés et mesurés. |
| Objectifs chiffrés | Seuils de délai, de ressources et de qualité à fixer après mesures initiales. |

## 11. Étapes proposées

1. **Valider ce cahier des charges**, notamment les décisions ouvertes qui influencent le prototype.
2. **Construire et valider le socle sur GNOME/Wayland et Windows 11 dès le départ** : fenêtre Tauri candidate, éditeur visuel, fichier Markdown et capture micro ; vérifier les mécanismes d'arrière-plan et de stockage sécurisé sur les deux environnements avant de retenir définitivement le socle.
3. **Intégrer Mistral** : transcription continue, segmentation et interprétation en opérations structurées, avec coûts et erreurs visibles.
4. **Sécuriser les modifications** : validation, ordre des opérations, annulation, modes d'édition et gestion des coupures.
5. **Éprouver les scénarios vocaux** : fidélité, ambiguïtés, commandes mélangées, épellation, arrière-plan, performances et coût.
6. **Ajuster le périmètre** selon les résultats, puis étudier un deuxième fournisseur pour valider l'architecture d'adaptateurs.

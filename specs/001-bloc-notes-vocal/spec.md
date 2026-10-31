# Spécification de fonctionnalité : Bloc-notes vocal intelligent

**Fonctionnalité** : `001-bloc-notes-vocal`  
**Branche** : aucune ; projet sans Git, repérage par `.specify/feature.json`.  
**Création** : 2026-10-30  
**Statut** : spécification prête pour la planification ; aucune recette du produit réalisée.  
**Source** : pièce jointe en lecture seule, « Bloc-notes vocal intelligent — cahier des charges », version 0.3, 376 lignes, complétée et corrigée par les décisions explicites de la présente demande.  
**Référence de gouvernance** : [constitution actuelle, version 3.0.0](../../.specify/memory/constitution.md), lue sans modification. Le [cahier des charges historique](../../specifications-bloc-notes-vocal.md) reste inchangé.

### Périmètre et arbitrages conservés

L'utilisateur francophone construit et retravaille une note locale en parlant naturellement. Le résultat est une note structurée, pas une simple transcription. Les usages initiaux sont notamment le brouillon d'e-mail, le compte rendu rapporté après une réunion, la synthèse d'idées et le document professionnel (d'autres cas arriveront vite).

- Le **mode vocal est principal** : toutes les actions de ce mode sont exprimées à la voix, y compris les commandes de note et de session. Aucun clavier ni bouton à maintenir n'est requis pour chaque intervention. 
- Le **mode manuel** n'est disponible qu'après arrêt de l'interface vocale. C'est un éditeur texte classique avec annulation classique, sans historique persistant.
- Un mécanisme dédié d'annulation ou de rétablissement en mode vocal est **exclu**, et non simplement caché. Les anciennes exigences F15–F19 et recettes R06–R07 ne sont pas reprises. Une correction intentionnelle comme « non, je voulais dire… » reste une demande d'édition de contenu, pas un retour automatique à un état antérieur.
- Le texte saisi manuellement **peut être modifié intentionnellement à la reprise vocale**. Il n'existe pas de protection générale fondée sur son origine manuelle. En revanche, aucun résultat de l'ancienne session invalidée ne peut le modifier. Ce sont deux situations distinctes.
- GNOME/Wayland, notamment Fedora, et Windows 11 sont obligatoires dès le premier prototype et à chaque jalon fonctionnel. macOS reste un objectif de première version ; aucune compatibilité ni recette n'est présumée.
- La fermeture suit cet ordre : arrêt de capture et invalidation, sauvegarde de la note courante, puis fin de session et fermeture. Un échec de sauvegarde reste visible et empêche une fermeture silencieuse avec perte.
- Audio, transcription, conversation et lexique sont temporaires. La note seule est conservée comme contenu utilisateur ; les clés sont dans le trousseau sécurisé. Les préférences non sensibles ne constituent pas un historique de contenu.
- La transcription et l'interprétation par modèle de langage ont des responsabilités et choix de fournisseurs distincts ; elles peuvent néanmoins utiliser le même fournisseur. Mistral est la première intégration visée ; une intégration effective d'OpenAI est hors périmètre initial.
- Tauri, Voxtral et Mistral Large 4 restent des **candidats à évaluer**, non des choix imposés, des capacités vérifiées ou une architecture prescrite.

Le périmètre initial comprend une seule note ouverte, l'édition vocale et manuelle distinctes, les fichiers Markdown locaux, les préférences, la transparence des traitements et coûts et une vue temporaire de transcription. Il exclut mobile, synchronisation, collaboration, envoi d'e-mails, intégration de messagerie, enregistrement ou transcription de réunions multi-intervenants, recherche avancée dans une collection, dictionnaire personnel persistant, conservation de conversation et fonctionnement intégralement local garanti. La sortie de transcription vers une console externe n'est pas retenue initialement : elle introduirait une conservation externe incompatible avec le périmètre temporaire retenu.

### Orientation de l'interface

L'interface s'inspire de la simplicité d'un traitement de texte traditionnel comme
WordPad : une fenêtre centrée sur la note, sans reproduire une suite bureautique complète.

- Une barre de menus « Fichier », « Édition », « Aide » donne accès aux actions usuelles.
- Un bandeau simple rassemble la mise en forme compatible Markdown et les commandes
  visibles de démarrage et d'arrêt du mode vocal.
- La note mise en forme occupe la zone principale ; le mode courant, l'état d'écoute
  et les demandes de précision restent lisibles dans l'application.
- Les boutons de démarrage et d'arrêt complètent le pilotage vocal ; ils ne deviennent
  pas des gestes obligatoires à chaque intervention. L'édition manuelle reste accessible
  seulement après arrêt de l'interface vocale.

## Scénarios utilisateur et tests

Les scénarios ci-dessous définissent des résultats attendus, non des résultats constatés. Les priorités ordonnent la valeur utilisateur sans dispenser de vérifier les contraintes transversales dès le premier prototype. Les identifiants S1 à S4 et S6 à S8 sont conservés pour la traçabilité ; le parcours 5 a été retiré de cette présentation, sans suppression des exigences correspondantes.

### Parcours 1 — Construire une note fidèle à la voix (Priorité : P1)

L'utilisateur expose ses idées dans un même flux et obtient une note lisible qu'il peut reprendre hors de l'application.

**Pourquoi cette priorité** : c'est la valeur centrale du produit, au-delà de la dictée.

**Test indépendant** : partir d'une note vide, raconter un sujet avec hésitations, chiffres et réserves, puis relire et rouvrir le résultat enregistré.

**Scénarios d'acceptation** :

1. **S1.1 — Étant donné** une note vide et une session explicitement démarrée, **quand** l'utilisateur parle en français en plusieurs interventions sans réactivation, **alors** la note se structure progressivement en titres, paragraphes ou listes ; chiffres, noms, réserves, hypothèses et questions restent fidèles, sans fait ajouté.
2. **S1.2 — Étant donné** une note en cours, **quand** l'utilisateur annonce un e-mail, un compte rendu, une synthèse ou un document professionnel et demande une forme ou un ton, **alors** le texte adopte la forme demandée, sans insérer automatiquement l'annonce d'intention ni inventer de destinataire, responsable, date ou décision.
3. **S1.3 — Étant donné** une note commencée sans intention déclarée, **quand** l'utilisateur précise ensuite son intention, ajoute une idée ou demande de développer un point, **alors** le contenu existant est conservé sans perte silencieuse, l'idée rejoint une section pertinente et le développement n'invente aucune information.
4. **S1.4 — Étant donné** une note affichée, **quand** une modification est préparée puis appliquée, **alors** traitement et application sont distingués ; la zone réellement concernée est repérable sans masquer le texte, sans dépendre de la couleur seule et sans gêner la consultation d'une autre zone.

### Parcours 2 — Piloter et corriger sans commandes rigides (Priorité : P1)

L'utilisateur mêle idées, instructions et corrections, sans syntaxe obligatoire ni confirmation ordinaire.

**Pourquoi cette priorité** : la compréhension des intentions différencie le produit d'une transcription brute.

**Test indépendant** : ouvrir une note préparée et effectuer uniquement à la voix une série de modifications ciblées, dont une ambiguë et une correction épelée.

**Scénarios d'acceptation** :

1. **S2.1 — Étant donné** plusieurs sections, **quand** l'utilisateur ajoute, corrige, reformule, déplace, supprime puis réorganise des éléments en langage naturel, avec plusieurs intentions dans une intervention, **alors** les actions non ambiguës sont ordonnées, limitées aux cibles demandées et appliquées sans confirmation systématique ; les déplacements et suppressions sont perceptibles. Une intervention combinant ton, édition et sauvegarde conserve cet ordre ; si une de ses intentions est ambiguë, aucune action de cette intervention n'est exécutée.
2. **S2.2 — Étant donné** un récit de procédure, **quand** l'utilisateur dit « dans la procédure, il faut supprimer les fichiers temporaires », **alors** cette idée est notée sans suppression d'éléments de la note ; des instructions déjà écrites dans le fichier ne deviennent pas non plus des commandes.
3. **S2.3 — Étant donné** plusieurs cibles plausibles, **quand** l'utilisateur dit « supprime ce point », **alors** aucune action de cette intervention n'est exécutée et un message d'ambiguïté s'affiche uniquement dans l'application ; l'utilisateur peut préciser sa demande à la voix. Aucune cible n'est choisie au hasard et aucune partie de la demande ambiguë n'est appliquée automatiquement plus tard. Si la fenêtre est masquée, le message reste consultable au retour, sans notification système ni prise de premier plan imposée. Pour « non, je voulais dire… », seule la cible déterminée par le contexte est corrigée.
4. **S2.4 — Étant donné** un mot mal reconnu, **quand** l'utilisateur l'épelle avec majuscule, accent, tiret ou apostrophe et le réemploie, **alors** l'occurrence ciblée est corrigée, l'orthographe est réutilisée dans la session et aucune occurrence sans rapport n'est remplacée ; une lettre ambiguë appelle une précision. Une correction globale exige une portée explicite ou claire. L'épellation elle-même n'entre pas dans la note sauf demande explicite.
5. **S2.5 — Étant donné** le mode vocal, **quand** l'utilisateur demande une annulation ou un rétablissement d'historique, **alors** aucun mécanisme dédié de retour à un état antérieur n'est exécuté ; l'application explique cette limite et permet une correction de contenu explicitement formulée, sans prétendre avoir restauré une version précédente.

### Parcours 3 — Passer au texte manuel puis reprendre à la voix (Priorité : P1)

L'utilisateur peut arrêter le vocal, écrire normalement, annuler une saisie manuelle puis reprendre la rédaction orale sur l'ensemble de la note.

**Pourquoi cette priorité** : cette séparation garantit une maîtrise réelle sans faire de l'éditeur manuel un second mode vocal.

**Test indépendant** : retarder un traitement vocal, passer en manuel, modifier et annuler une saisie, reprendre le vocal et modifier intentionnellement un passage manuel.

**Scénarios d'acceptation** :

1. **S3.1 — Étant donné** une capture et un traitement en attente, **quand** le passage au manuel est demandé à la voix, **alors** l'interface vocale s'arrête, le traitement est invalidé sans attendre sa réponse, le texte déjà appliqué reste et les passages non intégrés sont signalés ; l'éditeur texte devient utilisable immédiatement.
2. **S3.2 — Étant donné** le mode manuel, **quand** l'utilisateur saisit du texte puis l'annule avec la commande classique, **alors** la modification manuelle est annulée sans traitement vocal simultané ; cet historique n'est pas enregistré durablement. Le contenu accepté par le cœur de l'application fait foi : si une modification est refusée ou si l'affichage diverge, l'éditeur revient à cette version et signale le refus, sans présenter la modification refusée comme sauvegardée ni la réintroduire par annulation. Un événement retardé ne ramène pas l'affichage à une ancienne version.
3. **S3.3 — Étant donné** un texte saisi manuellement et une reprise vocale explicite, **quand** l'utilisateur demande de reformuler ce texte, **alors** il est modifié normalement ; une réponse retardée de l'ancienne session, reçue avant ou après cette reprise, n'a aucun effet.
4. **S3.4 — Étant donné** une opération encore issue de la session active mais visant un état de note devenu périmé, **quand** son résultat arrive, **alors** il est réévalué ou le conflit est signalé, sans écrasement ni duplication aveugle ; ceci n'autorise jamais la réactivation d'un résultat d'une session invalidée.

### Parcours 4 — Garder des fichiers locaux intègres (Priorité : P1)

L'utilisateur crée, ouvre et retrouve une note autonome, même en cas d'interruption ou de modification externe.

**Pourquoi cette priorité** : la note est le seul contenu conservé et ne doit pas être perdue silencieusement.

**Test indépendant** : utiliser des fichiers préparés, y compris tableaux et code, provoquer une modification externe puis un échec d'écriture.

**Scénarios d'acceptation** :

1. **S4.1 — Étant donné** une note nouvelle ou existante, **quand** l'utilisateur crée, ouvre, enregistre, utilise « Enregistrer sous », copie le Markdown ou l'exporte, **alors** une seule note est ouverte à la fois et le fichier local reste utilisable sans l'application ; ces actions sont exprimables à la voix en mode vocal. Créer ou ouvrir une autre note invalide tous les traitements précédents en cours : leurs résultats tardifs ne modifient pas la nouvelle note, même si les contenus ou numéros de révision se ressemblent.
2. **S4.2 — Étant donné** une note avec titres, paragraphes, listes, gras, italique, tableaux et blocs de code, **quand** elle est affichée ou modifiée, **alors** les éléments pris en charge sont lisibles ; les autres sont préservés ou leur limite est signalée avant toute altération possible.
3. **S4.3 — Étant donné** un fichier changé dans une autre application, **quand** une sauvegarde ou un changement de note intervient, **alors** le conflit est signalé ou résolu sans écraser silencieusement le contenu externe ni les modifications locales non sauvegardées.
4. **S4.4 — Étant donné** une note modifiée et sauvegardée automatiquement, **quand** l'application est interrompue puis relancée, **alors** la dernière version effectivement enregistrée est retrouvable ; aucun contenu non sauvegardé n'est présenté comme récupéré avec certitude.
5. **S4.5 — Étant donné** la fenêtre principale, **quand** l'utilisateur consulte ses commandes, **alors** il trouve les menus « Fichier », « Édition », « Aide », un bandeau simple de mise en forme et des commandes visibles pour démarrer ou arrêter le mode vocal. La note reste la zone principale et les commandes respectent la séparation des modes.

### Parcours 6 — Configurer les services et comprendre les coûts (Priorité : P1)

L'utilisateur sait quels services traitent ses données et configure son accès sans exposer de secret.

**Pourquoi cette priorité** : les services externes conditionnent le fonctionnement initial et engagent confidentialité et dépenses.

**Test indépendant** : ouvrir les préférences, gérer une clé, tester une configuration valide ou invalide, puis observer une session et une panne.

**Scénarios d'acceptation** :

1. **S6.1 — Étant donné** les intégrations réellement disponibles, **quand** l'utilisateur ouvre les préférences, y compris par demande vocale dans ce mode, **alors** transcription et interprétation ont des choix séparés de fournisseur et modèle ; Mistral est l'intégration initiale visée et OpenAI n'est pas présenté comme disponible. Les réglages non sensibles effectivement proposés indiquent leur prise d'effet. Les services choisis et la nature des données nécessaires envoyées sont consultables ; les conditions de conservation externes sont signalées comme devant être vérifiées avant usage sensible, sans promesse de suppression par le fournisseur.
2. **S6.2 — Étant donné** une clé à saisir, remplacer ou supprimer, **quand** l'utilisateur agit dans les préférences, **alors** sa valeur est masquée, son état de présence est indiqué sans révélation complète et sa conservation passe par le trousseau sécurisé ; si celui-ci est indisponible, une erreur exclut tout enregistrement en clair.
3. **S6.3 — Étant donné** un test de configuration demandé explicitement, **quand** l'utilisateur le lance après information sur le contact externe et le coût possible, **alors** un résultat compréhensible est affiché sans révéler de clé, même dans une erreur.
4. **S6.4 — Étant donné** une session active, **quand** une panne ou un changement de réglage survient, **alors** aucun envoi vers un fournisseur non choisi n'a lieu ; un changement de fournisseur ou modèle s'applique à la prochaine session explicitement démarrée, sans bascule cachée.
5. **S6.5 — Étant donné** une session qui consomme des services, **quand** son coût évolue, **alors** l'utilisateur voit une estimation distinguant transcription et interprétation selon les informations disponibles, les données mesurées des données estimées et les composantes inconnues ; aucune facture exacte n'est promise.

### Parcours 7 — Inspecter temporairement une transcription (Priorité : P2)

L'utilisateur peut comprendre une erreur de reconnaissance sans constituer un nouvel historique de données sensibles.

**Pourquoi cette priorité** : ce diagnostic aide à évaluer le prototype sans détourner sa finalité.

**Test indépendant** : utiliser successivement une session ordinaire et une session avec vue de diagnostic, corriger un mot puis terminer la session.

**Scénarios d'acceptation** :

1. **S7.1 — Étant donné** une session ordinaire, **quand** l'utilisateur n'active pas le diagnostic, **alors** la transcription brute n'est ni affichée dans cette vue ni émise vers une console ou des journaux persistants.
2. **S7.2 — Étant donné** la vue explicitement activée pour la session, **quand** le service renvoie du texte, **alors** ce texte brut est présenté comme une reconnaissance possiblement erronée, avec avertissement de sensibilité ; provisoire, finalisé et corrections du service sont distingués lorsque ces informations existent.
3. **S7.3 — Étant donné** une session avec audio traité, transcription, conversation, lexique corrigé et vue ouverte, **quand** elle se termine puis qu'une nouvelle commence, **alors** ces données ne sont plus disponibles ; la vue est vidée, les corrections déjà appliquées restent dans la note et aucun historique manuel n'est récupéré après fermeture.

### Parcours 8 — Utiliser un prototype réellement multiplateforme (Priorité : P1)

L'utilisateur dispose des fonctions annoncées sur ses systèmes cibles et connaît les limites observées.

**Pourquoi cette priorité** : la compatibilité des deux environnements obligatoires ne peut pas être reportée en fin de développement.

**Test indépendant** : exécuter la même recette sur GNOME/Wayland et Windows 11 pour le premier prototype et chaque jalon ; préparer séparément la vérification macOS.

**Scénarios d'acceptation** :

1. **S8.1 — Étant donné** le premier prototype ou un jalon fonctionnel, **quand** il est évalué sur chacun des deux environnements obligatoires, **alors** installation, démarrage, permissions, capture, arrière-plan, indicateur dans l'application uniquement, pause rapide, trousseau, verrouillage, veille et fonctions livrées disposent de résultats documentés et de limites explicites.
2. **S8.2 — Étant donné** une annonce envisagée pour macOS, **quand** la prise en charge n'a pas encore été vérifiée, **alors** elle reste un objectif non confirmé ; aucune annonce de fonctionnement acquis ne remplace la recette.
3. **S8.3 — Étant donné** une séance d'évaluation, **quand** réactivité, démarrage, ressources, coût et qualité sont examinés, **alors** les délais de bout en bout, conditions, matériel, versions et limites sont consignés ; les erreurs de transcription sont distinguées des erreurs d'interprétation ou d'application, sans seuil de performance arbitraire.

### Cas limites

- Une correction arrive avant l'affichage du passage visé : respecter l'ordre logique et le contenu courant, sans ajouter de retour arrière vocal d'historique (S2.1, S3.4).
- Une cible n'est pas encore connue : afficher une interprétation en cours, pas un emplacement d'écriture fictif (S1.4).
- Une note contient des instructions hostiles ou impératives : elles restent du contenu et n'autorisent ni commande ni envoi supplémentaire (S2.2, S6.4).
- Un mot, une lettre ou une référence demeure ambiguë : demander une précision utile plutôt qu'inventer (S2.3–S2.4).
- Un résultat ancien arrive après une transition puis une reprise vocale : toujours l'ignorer, même si sa cible paraît encore valable (S3.3).
- Un chemin n'existe pas, n'est pas accessible ou le disque est plein : conserver les modifications et exposer l'échec, y compris lors de « Enregistrer sous » ou d'une fermeture (S4.3, REQ-037).
- Une note sans destination doit être sauvegardée avant fermeture : demander une destination sans poursuivre la capture ni fermer avec perte (S4.1, REQ-036–REQ-037).
- Une autre application modifie le fichier pendant la fermeture : ne pas résoudre le conflit par écrasement implicite (S4.3, REQ-037).
- Une capacité de transcription, de coût ou de trousseau manque : afficher la limite, sans prétendre disposer d'une mesure ou stocker un secret autrement (S6.2, S6.5, S7.2).
- Des paroles de tiers sont captées en arrière-plan : avertir de ce risque avant cette utilisation et fournir une pause vocale rapide (REQ-029, REQ-034).

## Exigences

### Exigences fonctionnelles

Les identifiants canoniques `REQ-XXX` suivent une séquence unique. Ils remplacent le préfixe d'exemple `FR-XXX` du modèle local sans en changer la structure. Les références Fxx/Rxx ci-dessous sont celles de la source historique, pas de nouvelles exigences. Chaque exigence définit un résultat observable et renvoie aux scénarios d'acceptation lorsqu'ils sont présents.

#### Rédaction et compréhension — principes I et II

- **REQ-001** : Accepter la parole française en écoute continue après un démarrage explicite, sans activation à chaque intervention ni syntaxe obligatoire. *(Source : F01, F07 ; S1.1, S2.1.)*
- **REQ-002** : Synthétiser et structurer les idées, même désordonnées, sans inventer de faits ni altérer noms, chiffres, décisions, réserves, questions ou hypothèses. *(F02–F05 ; S1.1, S1.3.)*
- **REQ-003** : Permettre d'ajuster oralement concision, développement, ton et forme, uniquement à partir des informations fournies. *(F06, F50 ; S1.2–S1.3.)*
- **REQ-004** : Distinguer idée, instruction et correction ; ne pas exécuter comme commandes les formulations rapportées ni les instructions écrites dans la note. *(F08, section 5 ; S2.2.)*
- **REQ-005** : Traiter les interventions à intentions multiples dans un ordre cohérent, y compris lorsqu'elles mêlent ton, édition et sauvegarde, sans duplication ni application d'une opération à un état périmé non réévalué. Vérifier l'absence d'ambiguïté de l'intervention entière avant d'en exécuter les actions ; sinon ne rien exécuter et signaler l'ambiguïté. *(F09, F29 et décision utilisateur ; S2.1, S2.3, S3.4.)*
- **REQ-006** : Permettre d'ajouter, corriger, reformuler, déplacer, supprimer et restructurer à la voix en limitant l'effet à la portée demandée. *(F07, F10, F12 ; S2.1.)*
- **REQ-007** : Résoudre les cibles à l'aide de la note et du contexte récent ; en cas d'ambiguïté réelle, n'exécuter aucune action de l'intervention et indiquer l'ambiguïté dans l'application. L'utilisateur peut préciser sa demande à la voix ; aucune action ambiguë n'est exécutée automatiquement plus tard. Ne pas émettre de notification dans le centre de notifications du bureau ; si la fenêtre est masquée, le message reste consultable au retour sans imposer le premier plan. *(F10–F11 et décision utilisateur ; S2.3.)*
- **REQ-008** : Appliquer les changements ordinaires sans confirmation systématique et indiquer discrètement ce qui a été fait. *(F13 ; S2.1.)*
- **REQ-009** : Comprendre les corrections contextuelles telles que « non, je voulais dire… », sans les transformer en commandes d'historique vocal. *(F14 et arbitrage ; S2.3, S2.5.)*
- **REQ-010** : Permettre d'exprimer ou de changer librement l'intention de rédaction, sans déclaration préalable obligatoire, insertion automatique des consignes ni perte silencieuse de contenu. *(F49–F52 ; S1.2–S1.3.)*
- **REQ-011** : Corriger par épellation avec indications de casse, accent, tiret et apostrophe ; demander une précision en cas d'incertitude et ne pas inscrire l'épellation dans la note sauf demande. *(F20–F21, F24 ; S2.4.)*
- **REQ-012** : Réutiliser l'orthographe corrigée dans un lexique temporaire de session, avec correction ciblée par défaut et portée globale uniquement explicite ou claire ; conserver les corrections dans la note après effacement du lexique. *(F22–F25 ; S2.4, S7.3.)*

#### Deux modes et validité des traitements — principes II et III

- **REQ-013** : Rendre toutes les actions du mode vocal exprimables à la voix, notamment gestion de note, correction, pause, arrêt, accès aux préférences et passage en manuel ; rendre visible le mode courant. Les autorisations système et la saisie de secrets restent des actes de configuration hors rédaction vocale. *(F26, F28 et arbitrage ; S2.1, S3.1, S4.1, S6.1.)*
- **REQ-014** : Fournir en manuel, seulement après arrêt de l'interface vocale, un éditeur texte classique avec annulation classique et sans traitement vocal concurrent. La représentation détenue par le cœur de l'application a priorité sur celle affichée : l'éditeur se resynchronise sur le contenu accepté par le cœur en cas de divergence ou de refus, avec signalement explicite et sans historique d'annulation capable de réintroduire une modification refusée. *(F27–F28 corrigés et décision utilisateur U2 ; S3.1–S3.2.)*
- **REQ-015** : Ne proposer ni exécuter de mécanisme dédié d'annulation ou de rétablissement vocal, ni conserver de pile d'historique vocal à cette fin ; expliquer la limite sans empêcher une demande de correction de contenu. *(Remplace F15–F19 et R06–R07 rejetés ; S2.5.)*
- **REQ-016** : Lors du passage au manuel, arrêter la capture, invalider immédiatement les traitements vocaux en attente sans attendre leur résultat, conserver les modifications appliquées et signaler les passages non intégrés. *(F28 ; S3.1.)*
- **REQ-017** : Empêcher tout résultat d'une session invalidée de modifier la note, y compris après reprise vocale. Créer ou ouvrir une autre note invalide tous les traitements précédents en cours ; aucun de leurs résultats ne peut modifier la nouvelle note. Pour un résultat encore actif mais périmé sur la même note, réévaluer ou signaler plutôt qu'écraser aveuglément. *(F29 et décisions utilisateur ; S3.3–S3.4, S4.1.)*
- **REQ-018** : Exiger une reprise vocale explicite et autoriser alors les modifications intentionnelles sur tout le contenu courant, y compris celui saisi manuellement, sans protection générale de son origine. *(Arbitrage explicite ; S3.3.)*
- **REQ-019** : Ne pas conserver durablement l'historique d'annulation manuelle et ne pas le restaurer après fermeture ou redémarrage. *(Politique de données corrigée ; S3.2, S7.3.)*

#### Note locale et perception des changements — principes II et III

- **REQ-020** : Créer, ouvrir, enregistrer, retrouver et enregistrer sous un autre nom une note Markdown locale, avec une seule note ouverte à la fois et sans dépendance à l'application pour lire le fichier. *(F31–F33 ; S4.1.)*
- **REQ-021** : Présenter titres, paragraphes, listes, gras et italique comme un document lisible et permettre copie et export du Markdown brut. Fournir une interface simple inspirée de WordPad, centrée sur la note, avec les menus « Fichier », « Édition », « Aide » et un bandeau de mise en forme compatible Markdown et de démarrage/arrêt du mode vocal, sans modifier la séparation des modes. *(F30, F32 et décision utilisateur sur l'interface ; S4.1–S4.2, S4.5.)*
- **REQ-022** : Préserver les constructions non prises en charge, notamment tableaux et code, ou avertir avant une modification susceptible de les altérer. *(F34 ; S4.2.)*
- **REQ-023** : Sauvegarder automatiquement la note et rendre retrouvable la dernière version effectivement sauvegardée après redémarrage ou interruption, en exposant tout échec. *(F35 ; S4.4.)*
- **REQ-024** : Détecter ou gérer les changements externes sans écrasement silencieux ; protéger également les modifications locales non sauvegardées lors d'un changement de note. *(F36, principe III ; S4.3.)*
- **REQ-025** : Afficher les changements progressivement pendant la session, par passages ou opérations applicables, sans attendre sa fin ni promettre une restitution mot à mot. *(F53 ; S1.1, S1.4.)*
- **REQ-026** : Distinguer traitement et modification appliquée et indiquer les zones réelles d'écriture, de déplacement, de suppression ou de réorganisation ; ne pas afficher de cible fictive. *(F54–F56 ; S1.4, S2.1.)*
- **REQ-027** : Garder le texte lisible, ne pas s'appuyer sur la couleur seule, respecter la navigation et ne jamais ramener la fenêtre au premier plan contre le choix de l'utilisateur. *(F57–F58 ; S1.4.)*

#### Écoute, interruptions et fermeture — principes II à IV

- **REQ-028** : Exiger un lancement explicite de la session et gérer l'autorisation microphone ; ne pas capturer avant cette entrée volontaire dans le mode vocal. *(F37.)*
- **REQ-029** : Maintenir l'écoute lorsque la fenêtre perd le premier plan, avec pause rapide exprimable à la voix. L'indicateur d'écoute et les demandes de précision s'affichent uniquement dans la fenêtre de l'application, même si celle-ci est masquée ; ils restent consultables au retour, sans indicateur hors fenêtre, notification système ni prise de premier plan imposée. *(F38, F40 révisé, section 10 et décision utilisateur ; S2.3.)*
- **REQ-030** : Afficher en français des états distincts et compréhensibles : écoute, pause, traitement, demande de précision, erreur et mode manuel. *(F39, section 8 ; S1.4, S3.1.)*
- **REQ-031** : Autoriser une proposition d'arrêt après silence prolongé sans arrêter automatiquement pour ce seul motif. *(F41.)*
- **REQ-032** : À une pause ou un arrêt demandé, cesser la capture et invalider les résultats non encore appliqués en signalant les passages non intégrés ; une reprise exige une action explicite. *(F38 et hypothèse de sûreté.)*
- **REQ-033** : Signaler refus ou perte de microphone, réseau ou service et distinguer ce qui est traité, non intégré ou potentiellement perdu, sans prétendre récupérer des paroles indisponibles. *(F42.)*
- **REQ-034** : Avertir du risque de captation de paroles non destinées à l'application, notamment celles de tiers en arrière-plan. *(F43 ; cas limites.)*
- **REQ-035** : Au verrouillage ou à la veille, arrêter la capture, passer en manuel et invalider les traitements en attente ; au retour, ne pas reprendre automatiquement. *(F71.)*
- **REQ-036** : À la fermeture, arrêter capture et traitements en attente, sauvegarder la note courante puis terminer la session et fermer ; aucune écoute ne subsiste après fermeture. *(F72.)*
- **REQ-037** : En cas d'échec de sauvegarde à la fermeture, maintenir accessibles les modifications, afficher l'erreur et permettre une nouvelle sauvegarde ou une autre destination sans fermeture silencieuse avec perte ni reprise de capture. *(F72, principe III.)*

#### Préférences, fournisseurs et confidentialité — principes IV et V

- **REQ-038** : Fournir un panneau de préférences utilisable sans édition d'un fichier de configuration, présentant seulement les intégrations et réglages réellement disponibles. *(F59, F64 ; S6.1.)*
- **REQ-039** : Séparer les choix de fournisseur et modèle pour transcription et interprétation afin que l'un puisse évoluer sans imposer l'autre ; viser Mistral initialement sans présenter OpenAI comme intégré. *(F46, F60, principe V ; S6.1.)*
- **REQ-040** : Permettre de saisir, remplacer et supprimer chaque clé intégrée, avec saisie masquée et indication de présence sans valeur complète affichée. *(F61 ; S6.2.)*
- **REQ-041** : Conserver les clés dans le trousseau sécurisé, jamais dans notes, configuration ordinaire ou journaux ; en cas d'indisponibilité, signaler le problème sans stockage en clair de substitution. *(F47, F62 ; S6.2.)*
- **REQ-042** : Permettre un test de configuration explicite, précédé d'une information sur le contact externe et le coût éventuel ; afficher un résultat compréhensible sans exposer de secret. *(F63 ; S6.3.)*
- **REQ-043** : Indiquer quand les réglages prennent effet ; appliquer les changements de fournisseur ou modèle à la prochaine session explicite et ne jamais basculer vers un fournisseur non choisi en cas de panne. *(F48, F65 ; S6.1, S6.4.)*
- **REQ-044** : Afficher pendant la session un coût estimé couvrant transcription et interprétation selon les données disponibles ; distinguer mesures, estimations, inconnues et limites, sans présenter une facture. *(F44–F45 ; S6.5.)*
- **REQ-045** : Informer des services choisis et des données envoyées, limiter les envois au nécessaire et exposer les conditions de conservation des fournisseurs à vérifier avant usage sensible ; ne pas confondre absence de conservation locale et suppression chez eux. *(Section 7.2, principe IV ; S6.1, S6.4 et S7.3.)*
- **REQ-046** : Ne pas conserver durablement audio, transcription, conversation ni lexique ; les supprimer en fin de session, conserver seulement la note comme contenu et éviter leur présence accidentelle dans journaux ou fichiers temporaires durables. *(Section 7.1, principe IV ; S7.1–S7.3.)*
- **REQ-047** : Proposer une vue de transcription brute désactivée par défaut, activable explicitement pour la seule session, accompagnée d'un avertissement de sensibilité et d'inexactitude possible. *(F66, F68–F69 ; S7.1–S7.2.)*
- **REQ-048** : Distinguer les segments provisoires et finalisés et refléter les corrections lorsque le service les expose ; vider la vue et ses données à la fin de session sans enregistrer d'audio pour cette fonction. *(F67, F69 ; S7.2–S7.3.)*
- **REQ-049** : Ne pas émettre de transcription dans une console ou un journal persistant ; la proposition optionnelle de console F70 reste hors périmètre initial. *(F70 restreint par la politique temporaire ; S7.1.)*

#### Qualité et plateformes — principes I, III et V, décision explicite de portabilité

- **REQ-050** : Livrer et vérifier chaque jalon, dès le premier prototype, sur GNOME/Wayland et Windows 11, couvrant installation, démarrage, micro, arrière-plan, indicateur dans l'application uniquement, pause rapide, trousseau, verrouillage, veille et fonctions livrées ; documenter les limites sur chacun. *(Section 2.1, F40 révisé, section 8, R16 ; S8.1.)*
- **REQ-051** : Maintenir macOS comme objectif de première version et ne pas annoncer sa prise en charge avant vérification effective des mêmes fonctions. *(Sections 2.1 et 8 ; S8.2.)*
- **REQ-052** : Mesurer dès le premier prototype puis à chaque jalon le démarrage, les ressources, les délais de bout en bout et les coûts avec matériel, versions, services et conditions ; distinguer transcription, interprétation et application et ne pas présenter de seuils non validés comme acquis. *(Section 8 ; S8.3.)*
- **REQ-053** : Évaluer la fidélité et les limites sur les quatre usages, avec hésitations, noms propres, chiffres, accents, interventions longues, intentions mêlées et ambiguïtés ; rapporter séparément les erreurs de reconnaissance, d'interprétation et de modification. *(Section 9 ; S1.1–S2.4, S8.3.)*

### Entités principales

- **Note** : contenu courant, structure, destination locale et état de sauvegarde. Son origine vocale ou manuelle ne constitue pas un droit de modification distinct.
- **Session vocale** : période d'usage explicitement démarrée, état d'écoute et ensemble temporaire d'intentions, de propos utiles et de traitements. Une session invalidée ne peut plus produire de modification.
- **Traitement en cours** : passage concerné, cible si connue, session d'origine et état de validité par rapport à la note courante ; ce n'est pas un historique d'annulation.
- **Intention de rédaction** : forme, ton et niveau de détail souhaités, modifiables pendant la rédaction sans devenir automatiquement du contenu.
- **Lexique temporaire** : associations de mots et d'orthographes corrigées, limitées à la session.
- **Historique manuel temporaire** : changements de l'éditeur texte servant à son annulation classique, sans persistance après fermeture.
- **Configuration de services** : choix séparés pour transcription et interprétation, paramètres non sensibles, présence des clés protégées et indication de prise d'effet.
- **Consommation de session** : mesures disponibles, estimations de coût et limites ; aucune conservation de conversation n'est nécessaire à sa présentation.

## Critères de réussite

### Résultats mesurables

Ces critères sont des obligations de recette à vérifier ultérieurement, pas des résultats acquis. Les dénombrements portent sur des cas définis, et non sur une promesse universelle de justesse d'un modèle. Aucun seuil de délai, de mémoire, de qualité statistique ou de coût n'est ratifié ici.

- **SC-001 — Valeur de rédaction** : pour chacun des quatre usages, une recette documentée permet de produire puis ajuster une note à la voix, sans clavier pour les actions du mode vocal ; la comparaison avec les propos relève les faits ajoutés, informations omises et nuances altérées, sans accepter d'ajout factuel dans les cas de recette. *(REQ-001–REQ-013, REQ-053 ; S1, S2.)*
- **SC-002 — Maîtrise des deux modes** : les quatre scénarios S3.1–S3.4 et le cas S2.5 démontrent un éditeur manuel avec annulation, zéro application de résultat invalidé et la modification vocale intentionnelle d'un passage manuel à la reprise, sans mécanisme d'annulation vocale. *(REQ-014–REQ-019.)*
- **SC-003 — Intégrité** : dans les cas de conflit, contenu non pris en charge et interruption de S4, ainsi que dans les cas d'échec de sauvegarde décrits dans les cas limites, aucune perte ni altération silencieuse n'est acceptée ; après une sauvegarde réussie, la note est réutilisable indépendamment de l'application. *(REQ-020–REQ-024, REQ-036–REQ-037.)*
- **SC-004 — Arrêt sûr** : chacune des transitions manuel, pause, arrêt, verrouillage, veille et fermeture est testée avec un résultat retardé ; zéro résultat invalidé modifie la note et zéro reprise automatique n'est observée après verrouillage ou veille. La fermeture réussie respecte l'ordre arrêt, sauvegarde, fin. *(REQ-016–REQ-018, REQ-028–REQ-037.)*
- **SC-005 — Confidentialité compréhensible** : la recette des préférences et de fin de session ne retrouve ni clé en clair dans les fichiers ordinaires ni audio, transcription, conversation, lexique ou historique manuel conservés par l'application ; services utilisés, données transmises et limites des coûts sont consultables. *(REQ-019, REQ-038–REQ-049 ; S6, S7.)*
- **SC-006 — Couverture des environnements** : deux comptes rendus de recette, un par environnement obligatoire, existent pour le premier prototype et chaque jalon ; toute fonction non vérifiée est désignée comme telle. macOS n'est annoncé compatible qu'avec son propre compte rendu. *(REQ-050–REQ-051 ; S8.1–S8.2.)*
- **SC-007 — Réactivité et sobriété observables** : à chaque jalon, les délais entre parole suffisamment complète et note mise à jour, temps de démarrage, ressources et coût sont mesurés dans les conditions documentées sur les deux environnements obligatoires ; l'utilisateur peut observer l'avancement avant la fin de session et distinguer attente et modification effective. Les mesures servent à proposer ensuite des objectifs chiffrés, pas à inventer des seuils validés. *(REQ-025–REQ-027, REQ-044, REQ-052 ; S1.4, S6.5, S8.3.)*

## Hypothèses et dépendances

- **Usage initial** : une personne rédige en français une note à la fois. Un compte rendu est rapporté après la réunion et ne suppose pas de capturer plusieurs intervenants.
- **Services disponibles** : l'usage vocal initial nécessite microphone autorisé, accès réseau, compte et crédits éventuels auprès des services choisis. Le mode manuel et l'accès au contenu local ne nécessitent pas leur disponibilité.
- **Entrée dans le mode vocal** : depuis un état sans capture, une action explicite de démarrage est nécessaire. Le mécanisme précis d'entrée est à concevoir ; il n'autorise pas une écoute cachée permanente. Les commandes une fois dans ce mode restent vocales.
- **Pause, arrêt et session** : par défaut de sûreté, pause et arrêt invalident les traitements non appliqués. La pause suspend la session sans reprendre automatiquement ; arrêt et passage en manuel terminent la session vocale et ses données temporaires. Le verrouillage et la veille suivent ce passage en manuel. L'historique de l'éditeur manuel appartient uniquement à sa durée d'utilisation courante ; sa persistance est exclue.
- **Préférences** : les changements de fournisseur ou modèle prennent effet à la prochaine session explicite ; les autres réglages indiquent leur prise d'effet. Les réglages optionnels non disponibles ne sont pas présentés comme opérationnels. Les variantes d'indicateur et de défilement sont des choix de conception à éprouver, sans exiger une nouvelle validation des arbitrages déjà rendus. Les demandes de précision sont visuelles dans l'application uniquement, sans notification du bureau.
- **Copie** : seule la copie du Markdown brut est requise initialement. Une copie enrichie vers la messagerie ou le traitement de texte n'est pas présumée.
- **Conservation** : la politique de l'application ne vaut pas garantie de suppression par les services externes, le système d'exploitation ou des outils tiers. Les limites observées du temporaire et de la récupération devront être documentées ; aucun journal de contenu durable n'est prévu.
- **Choix techniques ouverts** : les candidats cités dans le cadrage ne sont ni obligatoires ni validés. Disponibilité, aptitude au français, capacité d'épellation, conditions des fournisseurs, coût et compatibilité restent à éprouver. Aucun composant ni découpage interne n'est imposé par cette spécification.
- **Performance** : les objectifs chiffrés seront proposés à partir des premières mesures et soumis à décision ; l'absence de seuil aujourd'hui n'autorise pas à omettre les mesures.

### Traçabilité de la source et des arbitrages

| Éléments de la source | Couverture dans cette spécification | Traitement |
| --- | --- | --- |
| F01–F06 | REQ-001–REQ-003 | Construction et fidélité conservées. |
| F07–F14 | REQ-001, REQ-004–REQ-009 | Instructions et corrections conservées, sans retour arrière vocal. |
| F15–F19 ; R06–R07 | REQ-015, REQ-019 ; S2.5, S3.2 | Mécanisme vocal rejeté ; annulation réservée au manuel, non persistante. |
| F20–F25 | REQ-011–REQ-012, REQ-046 | Épellation et lexique temporaire conservés. |
| F26–F29 ; R10 | REQ-013–REQ-018 | Modes distincts ; résultats anciens invalidés, contenu manuel modifiable en nouvelle session. |
| F30–F36 ; R11–R12, R15 | REQ-020–REQ-024 | Markdown, sauvegarde et conflits conservés. |
| F37–F43 ; R13–R14 | REQ-028–REQ-034 | Écoute et transparence conservées, commandes du mode vocal à la voix. |
| F44–F48 ; R17 | REQ-039, REQ-041, REQ-043–REQ-045 | Coûts, choix séparés et absence de bascule implicite. |
| F49–F52 ; R18–R23 | REQ-003, REQ-010 ; S1.2–S1.3 | Quatre usages et intention libre conservés. |
| F53–F58 ; R24–R25 | REQ-025–REQ-027 | Affichage progressif conservé, mention d'annulabilité vocale écartée. |
| F59–F65 ; R26–R28 | REQ-038–REQ-043 | Préférences et trousseau conservés. |
| F66–F70 ; R29–R30 | REQ-047–REQ-049 | Vue temporaire retenue ; console externe optionnelle non retenue initialement. |
| F71–F72 ; R31–R32 | REQ-035–REQ-037 | Arrêt, invalidation, sauvegarde et fermeture sûre conservés. |
| R01–R05, R08–R09 | S1.1, S2.1–S2.4, S7.3 | Fidélité, ambiguïté et épellation conservées. |
| R16 ; sections 2.1 et 8 | REQ-050–REQ-053 ; S8 | Deux plateformes obligatoires dès le prototype ; macOS à vérifier. |
| R33 ; section 7.1 | REQ-019, REQ-046 ; S7.3 | Non-persistance conservée, sans historique vocal réintroduit. |
| Sections 5 et 6 | REQ-004–REQ-007, REQ-017, REQ-039 ; hypothèses | Besoins conservés ; chaîne technique, réversibilité vocale et choix candidats non imposés. |
| Sections 9 à 11 | Scénarios S1–S8, SC-001–SC-007, hypothèses | Recette future, décisions ouvertes de conception et mesures ; ni plan d'implémentation ni résultats supposés. |

### Alignement constitutionnel

| Principe actuel | Exigences concernées | Effet attendu |
| --- | --- | --- |
| I — Fidélité aux propos | REQ-001–REQ-012, REQ-053 | Sens conservé, intentions distinguées et ambiguïtés clarifiées. |
| II — Deux modes d'édition | REQ-013–REQ-019, REQ-025–REQ-037 | Vocal principal, changements ordinaires directs, manuel après arrêt avec annulation classique. |
| III — Propriété et intégrité des notes | REQ-017, REQ-020–REQ-024, REQ-036–REQ-037 | Fichier autonome, absence de conflit ou perte silencieuse. |
| IV — Confidentialité et sécurité | REQ-028–REQ-049 | Capture maîtrisée, données temporaires, clés protégées et services transparents. |
| V — Simplicité et indépendance technique | REQ-038–REQ-039, REQ-050–REQ-053 | Responsabilités séparées, décisions techniques justifiées par les besoins et les essais. |

Les exclusions d'annulation vocale et la possibilité de retravailler le contenu manuel sont des décisions explicites conservées ici, sans les réintroduire dans la constitution. Les contraintes de plateformes restent également des exigences de cette fonctionnalité, sans modifier la constitution.

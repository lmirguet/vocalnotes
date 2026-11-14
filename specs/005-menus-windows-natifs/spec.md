# Feature Specification: Menus et cycle de document Windows et Linux

**Feature Branch**: `lmirguet-capgemini-menus-windows-natifs`

**Created**: 2026-11-13

**Status**: Draft

**Input**: User description: "La barre de menu doit se comporter comme celle d'un éditeur de texte Windows standard : Fichier (Nouvelle note, Ouvrir, Enregistrer, Enregistrer sous, Exporter en markdown, Quitter), Edition (Préférence, Annuler, Rétablir), Aide (A propos). Le format par défaut est .von ; l'export .md exclut le contexte. Fermer la fenêtre équivaut à Quitter et propose l'enregistrement des modifications. Préférence ouvre une popup ressemblant à une fenêtre native Windows."

## Clarifications

### Session 2026-11-13

- Q: Le fichier .von doit-il rester directement lisible dans un éditeur de texte ordinaire ? → A: Oui : contenu Markdown et contexte clairement séparés dans un fichier texte lisible sans Vocalnotes.
- Q: Lorsqu'une dictée ou une reformulation est encore en cours, faut-il attendre son résultat avant de proposer de sauvegarder puis fermer ou remplacer la note ? → A: Arrêter l'écoute, attendre les résultats en cours, puis proposer la sauvegarde ; attente annulable et échec signalé sans fermeture automatique.
- Q: Après ouverture d'un ancien fichier .md, que doit faire Enregistrer si vous avez ajouté ou modifié le contexte de la note ? → A: Ouvrir Enregistrer sous en .von et adopter ce nouveau fichier après réussite ; le .md initial reste inchangé.
- Q: Un fichier .md peut-il conserver son format si aucun contexte n'a été défini ? → A: Oui, le fichier .md reste .md et Enregistrer conserve son chemin sans imposer de conversion en .von.
- Q: L'implémentation doit-elle aussi couvrir Linux ? → A: Oui, prévoir l'implémentation et la validation sous Linux en plus de Windows, avec les mêmes fonctions et protections de données.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Conserver son travail lors des changements de document (Priority: P1)

L'utilisateur crée, ouvre et enregistre une note comme dans un éditeur de bureau standard, sous Windows comme sous Linux. Il identifie le fichier courant et les modifications non enregistrées. Nouvelle note, Ouvrir, Quitter et la fermeture de fenêtre ne lui font jamais perdre silencieusement son travail.

**Why this priority**: La fiabilité de l'enregistrement et de la fermeture conditionne la confiance dans l'éditeur.

**Independent Test**: Saisir du contenu et du contexte, enregistrer en .von, rouvrir le fichier puis exercer chaque action qui remplace ou ferme la note avec chacune des décisions Enregistrer, Ne pas enregistrer et Annuler.

**Acceptance Scenarios**:

1. **Given** une note neuve sans chemin, **When** l'utilisateur choisit Enregistrer ou Ctrl+S, **Then** une boîte de choix de fichier native du système utilisé propose le format .von ; une fois l'enregistrement réussi, ce chemin devient celui de la note et l'indicateur de modification disparaît.
2. **Given** une note .von déjà enregistrée puis modifiée dans son contenu ou son contexte, **When** l'utilisateur choisit Enregistrer, **Then** les deux champs sont conservés au chemin courant sans redemander de nom et la note redevient non modifiée.
3. **Given** une note associée à un fichier, **When** l'utilisateur choisit Enregistrer sous et valide un autre chemin, **Then** une copie complète est enregistrée, ce nouveau chemin devient le chemin courant et le fichier initial reste inchangé.
4. **Given** une note modifiée, **When** l'utilisateur demande Nouvelle note, Ouvrir, Quitter ou ferme la fenêtre, **Then** une même confirmation propose Enregistrer, Ne pas enregistrer et Annuler avant tout remplacement ou fermeture.
5. **Given** cette confirmation, **When** l'utilisateur choisit Enregistrer, **Then** l'action demandée se poursuit uniquement après la réussite de l'enregistrement ; l'annulation du choix de fichier ou un échec conserve la note et la fenêtre ouvertes.
6. **Given** cette confirmation, **When** l'utilisateur choisit Ne pas enregistrer, **Then** l'action demandée se poursuit sans enregistrer ; **When** il choisit Annuler, **Then** le document, son chemin et ses modifications restent disponibles.
7. **Given** une note non modifiée, **When** l'utilisateur choisit Nouvelle note, **Then** le contenu et le contexte deviennent vides, sans chemin associé ni reprise de l'historique de la note précédente ; **When** il choisit Quitter ou ferme la fenêtre, **Then** aucune confirmation de sauvegarde n'est présentée.
8. **Given** un fichier .von valide sélectionné dans Ouvrir, **When** le chargement réussit, **Then** son contenu et son contexte sont restaurés, son chemin devient courant et la note apparaît non modifiée.
9. **Given** le choix de fichier Ouvrir ou Enregistrer sous affiché, **When** l'utilisateur annule, **Then** le document et son association au fichier restent inchangés.
10. **Given** un fichier existant comme destination, **When** l'utilisateur enregistre sous ce nom, **Then** il doit confirmer son remplacement ; refuser ne modifie aucun fichier.
11. **Given** une note active, **When** son contenu ou son contexte diffère de la dernière version enregistrée, **Then** la barre de titre affiche son nom, ou « Sans titre » sans chemin, et un indicateur de modification ; revenir à l'état enregistré retire cet indicateur.
12. **Given** une dictée ou une reformulation en cours, **When** l'utilisateur demande Nouvelle note, Ouvrir ou la fermeture, **Then** l'écoute s'arrête et l'application affiche une attente annulable jusqu'à réception des résultats déjà engagés ; ceux-ci sont appliqués à la note courante selon les protections existantes avant d'évaluer ses modifications et de proposer la sauvegarde.
13. **Given** cette attente, **When** l'utilisateur l'annule ou qu'un traitement échoue, **Then** le remplacement ou la fermeture est abandonné, la note reste ouverte avec les changements reçus, et l'échec éventuel est expliqué ; aucune écoute ne redémarre automatiquement.
14. **Given** une ancienne note .md dont le contexte a été ajouté ou modifié depuis l'ouverture, **When** l'utilisateur choisit Enregistrer, y compris depuis la confirmation de fermeture, **Then** Enregistrer sous propose un .von complet et laisse le .md initial intact ; le nouveau chemin et l'état non modifié ne sont adoptés qu'après réussite. Annuler ou échouer conserve la note ouverte avec ses modifications.
15. **Given** un fichier .md ouvert pour lequel aucun contexte n'a été défini, **When** l'utilisateur modifie uniquement le contenu puis choisit Enregistrer, y compris depuis la confirmation de fermeture, **Then** le contenu est enregistré au même chemin .md sans demander de conversion en .von et la note redevient non modifiée.
16. **Given** une note .von enregistrée sous Windows ou Linux, **When** l'utilisateur transfère ce fichier puis l'ouvre et le réenregistre sur l'autre système, **Then** le contenu Markdown et le contexte restent identiques, sans conversion manuelle ni dépendance à un chemin du système d'origine.

---

### User Story 2 - Utiliser une interaction de menus native du système (Priority: P1)

L'utilisateur retrouve les menus et commandes demandés et les parcourt à la souris ou au clavier sans devoir utiliser des panneaux contenant un chemin à saisir. Les raccourcis et les commandes de menu ont le même effet.

**Why this priority**: Le défaut signalé concerne autant l'interaction de la barre que ses actions.

**Independent Test**: Sans souris, parcourir les trois menus, activer une commande et revenir à l'éditeur ; refaire le parcours à la souris et vérifier les commandes indisponibles.

**Acceptance Scenarios**:

1. **Given** la fenêtre principale, **When** l'utilisateur ouvre les menus, **Then** il trouve uniquement les commandes ci-dessous dans chaque menu, dans l'ordre indiqué ; des séparateurs peuvent regrouper les actions.
2. **Given** aucun menu ouvert, **When** l'utilisateur utilise Alt ou F10, puis les flèches et Entrée, **Then** il peut atteindre et activer une commande sans souris ; Echap ferme le sous-menu puis permet de revenir à la zone précédemment active.
3. **Given** un menu ouvert à la souris, **When** l'utilisateur survole un autre titre de menu, **Then** celui-ci remplace le menu ouvert ; un clic extérieur ou Echap ferme le menu sans exécuter de commande.
4. **Given** un menu affiché, **When** l'utilisateur consulte ses actions, **Then** les commandes indisponibles sont grisées, non activables, et les raccourcis associés sont visibles.
5. **Given** une commande activée au menu ou au raccourci, **When** elle s'exécute, **Then** une seule action se produit, le menu se ferme et le focus revient à l'éditeur ou à la boîte de dialogue ouverte.
6. **Given** l'application sous Linux, **When** l'utilisateur parcourt les menus à la souris ou au clavier, **Then** les mêmes commandes et protections sont disponibles, avec le rendu du bureau Linux utilisé ; aucune commande ne devient indisponible au seul motif qu'elle avait été conçue initialement pour Windows.

| Menu | Commandes, dans l'ordre |
|------|-------------------------|
| Fichier | Nouvelle note ; Ouvrir ; Enregistrer ; Enregistrer sous ; Exporter en markdown ; Quitter |
| Edition | Préférence ; Annuler ; Rétablir |
| Aide | A propos |

| Commande | Raccourci Windows et Linux |
|----------|-------------------|
| Nouvelle note | Ctrl+N |
| Ouvrir | Ctrl+O |
| Enregistrer | Ctrl+S |
| Enregistrer sous | Ctrl+Maj+S |
| Quitter / fermer la fenêtre | Alt+F4 |
| Annuler | Ctrl+Z |
| Rétablir | Ctrl+Y |

---

### User Story 3 - Exporter uniquement le contenu Markdown (Priority: P2)

L'utilisateur partage une version Markdown de sa note sans divulguer le contexte enregistré avec elle. L'export est une copie destinée au partage, et non un changement de fichier de travail.

**Why this priority**: La distinction entre sauvegarde complète et export évite une perte de contexte ou son partage involontaire.

**Independent Test**: Préparer des textes distincts dans le contenu et le contexte, modifier le contenu sans enregistrer puis exporter ; comparer le résultat au contenu courant et vérifier que l'état du document n'a pas changé.

**Acceptance Scenarios**:

1. **Given** une note avec du contenu Markdown, un contexte distinct et éventuellement des instructions temporaires, **When** l'utilisateur choisit Exporter en markdown, **Then** une boîte de fichier native du système utilisé propose un fichier .md contenant uniquement le contenu Markdown courant, y compris les modifications non enregistrées, sans contexte, instructions ni métadonnées propres à l'application.
2. **Given** une note .von modifiée, **When** l'export réussit, **Then** son chemin .von, son contexte, son historique et son état modifié restent inchangés ; Enregistrer continue de cibler le .von.
3. **Given** une note sans fichier, **When** elle est exportée, **Then** elle reste sans chemin de travail associé et Enregistrer propose toujours un .von.
4. **Given** un export annulé ou en échec, **When** l'utilisateur revient à l'éditeur, **Then** son travail reste intact et tout échec est expliqué sans annonce trompeuse de réussite.

---

### User Story 4 - Annuler et rétablir les modifications (Priority: P2)

L'utilisateur annule et rétablit les changements éditoriaux dans la zone où il travaille, avec les mêmes commandes depuis Edition et depuis le clavier.

**Why this priority**: Annuler et Rétablir sont des garanties essentielles d'un éditeur de texte familier.

**Independent Test**: Modifier successivement le contenu et le contexte, annuler puis rétablir dans chaque zone ; vérifier l'indicateur de modification et l'état grisé des commandes.

**Acceptance Scenarios**:

1. **Given** une modification dans le contenu, le contexte ou les instructions visibles, **When** l'utilisateur active Annuler puis Rétablir, **Then** la modification de cette zone est retirée puis restaurée sans modifier les autres zones.
2. **Given** le focus quitté de l'éditeur pour ouvrir Edition, **When** l'utilisateur choisit Annuler, **Then** l'action concerne la dernière zone d'édition active et non un autre document ou un autre champ.
3. **Given** aucun changement annulable ou rétablissable dans la zone ciblée, **When** Edition est ouvert, **Then** la commande correspondante est grisée ; une nouvelle modification après Annuler invalide le rétablissement de la branche abandonnée.
4. **Given** une note enregistrée, **When** les annulations restaurent exactement son contenu et son contexte enregistrés, **Then** elle apparaît non modifiée ; une modification des seules instructions temporaires ne la rend pas modifiée.
5. **Given** une modification du contenu par dictée, mise en forme ou reformulation réussie, **When** l'utilisateur choisit Annuler, **Then** il peut retrouver le contenu précédent ; une reformulation est annulable en une seule action.

---

### User Story 5 - Régler les préférences dans une fenêtre familière (Priority: P2)

L'utilisateur ouvre Préférence dans une fenêtre de dialogue distincte de l'espace de rédaction, d'apparence Windows sous Windows et familière du bureau Linux sous Linux, puis conserve ou abandonne ses réglages sans affecter sa note.

**Why this priority**: La demande vise une véritable expérience de dialogue et non un panneau ajouté à l'éditeur.

**Independent Test**: Ouvrir Préférence, parcourir les réglages au clavier, annuler un changement puis en confirmer un autre et rouvrir le dialogue.

**Acceptance Scenarios**:

1. **Given** la fenêtre principale, **When** l'utilisateur choisit Edition > Préférence, **Then** une fenêtre de dialogue intitulée « Préférences » apparaît au-dessus de son propriétaire, avec barre de titre, bordure, bouton de fermeture, contrôles et boutons de validation d'apparence native du système utilisé.
2. **Given** ce dialogue ouvert, **When** l'utilisateur navigue avec Tab et Maj+Tab, **Then** le focus reste dans le dialogue, son emplacement est visible et la note ne reçoit pas les frappes destinées aux réglages ; une seconde demande d'ouverture n'en crée pas de doublon.
3. **Given** des réglages modifiés, **When** l'utilisateur confirme par OK, **Then** les réglages valides sont conservés pour une réouverture et l'application ultérieure respecte les règles existantes ; une erreur de validation ou d'enregistrement laisse le dialogue ouvert avec une explication.
4. **Given** des réglages non confirmés, **When** l'utilisateur choisit Annuler, Echap ou la croix, **Then** le dialogue ferme sans les enregistrer et le focus revient à l'éditeur.
5. **Given** une note ouverte, **When** l'utilisateur consulte ou enregistre ses préférences, **Then** le contenu, le contexte, le chemin et l'état modifié de la note restent inchangés.
6. **Given** la commande Aide > A propos, **When** l'utilisateur l'active, **Then** un dialogue présente le nom de l'application et sa version, se ferme par son bouton ou Echap et restitue le focus sans modifier la note.
7. **Given** le dialogue de préférences ouvert sous Linux, **When** l'utilisateur tente d'interagir avec la note ou ferme le dialogue par OK, Annuler, Echap ou la croix, **Then** la note reste bloquée pendant le dialogue puis redevient utilisable avec restitution du focus, selon les mêmes règles que sous Windows.

### Edge Cases

- Une écriture échoue faute de droits, de place ou d'accès au dossier : le texte courant, le chemin antérieur et l'état modifié sont conservés ; l'erreur est visible et une nouvelle tentative reste possible.
- Un fichier est illisible ou invalide : Ouvrir ne remplace pas la note courante et explique l'échec. Choisir Ne pas enregistrer avant cet échec ne doit pas détruire la note en mémoire.
- Un nom sans extension est saisi : l'enregistrement ajoute .von et l'export ajoute .md, sans dupliquer une extension déjà présente, y compris en majuscules.
- Une destination d'export correspond au fichier de travail courant : l'opération est refusée avec une explication pour ne pas remplacer une note complète par un export incomplet.
- Le fichier cible a changé depuis son ouverture : aucun écrasement silencieux ; l'utilisateur doit confirmer le remplacement ou choisir une autre destination.
- Plusieurs demandes de fermeture ou raccourcis surviennent pendant un dialogue ou une écriture : aucune confirmation en double ni action concurrente ne détruit la note.
- Une dictée ou une reformulation est en cours au moment de remplacer ou fermer la note : l'écoute s'arrête, une attente annulable précède la décision de sauvegarde et les résultats déjà engagés sont traités sur la note courante. Annuler l'attente ou rencontrer un échec abandonne l'action demandée et conserve le travail, sans redémarrage automatique de l'écoute. Aucun résultat tardif ne peut modifier le document suivant.
- Une note est vide mais possède un contexte modifié : la protection de sauvegarde s'applique ; une note entièrement vide et jamais modifiée peut être fermée sans confirmation.
- Une note a été sauvegardée puis modifiée à nouveau pendant une opération : seules les données effectivement enregistrées peuvent être considérées comme sauvegardées.
- Une ancienne note Markdown est ouverte : son contenu reste lisible et éditable, sans conversion destructive du fichier d'origine. Si son contexte a changé depuis l'ouverture, Enregistrer propose un .von complet et laisse le .md intact ; même si le contexte a été vidé, la modification ne doit pas être ignorée.
- Le bureau Linux diffère de Windows dans le placement ou le thème des fenêtres : les préférences restent modales et utilisables au clavier ; une fermeture ou un échec de création ne doit jamais laisser la fenêtre principale bloquée.
- Un fichier est déplacé d'un système à l'autre : les différences de séparateur de chemin ou de sensibilité à la casse ne modifient pas son contenu, et les garanties de conflit et de non-écrasement du document courant restent applicables sur chaque système.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: L'application MUST présenter exactement les trois menus et les commandes listés dans la User Story 2, dans cet ordre, avec les libellés demandés.
- **FR-002**: La barre MUST respecter les interactions décrites dans la User Story 2 sous Windows et Linux : activation par Alt/F10, navigation par flèches, validation par Entrée, fermeture par Echap ou clic extérieur, bascule entre menus à la souris et restitution du focus. Les menus MUST contenir des commandes, pas un champ de chemin ni un panneau de configuration.
- **FR-003**: Les commandes MUST proposer les raccourcis du tableau, afficher ceux-ci dans les menus et produire le même effet au clavier et à la souris, une seule fois par activation. Les menus et commandes MUST exposer des noms accessibles et leur état activé ou désactivé.
- **FR-004**: Nouvelle note MUST créer un document sans chemin, contenu ni contexte, après protection de la note courante. Ouvrir MUST proposer une boîte native de sélection de fichier adaptée au système utilisé et ne remplacer la note qu'après chargement réussi.
- **FR-005**: Enregistrer MUST écrire la note au chemin courant ou demander une destination en son absence, sauf pour une ancienne note .md dont le contexte a changé, qui suit FR-018. Enregistrer sous MUST toujours proposer une destination et n'adopter le nouveau chemin qu'après réussite.
- **FR-006**: Le format de travail proposé par défaut MUST être .von et MUST conserver le contenu Markdown et le contexte pour leur restauration à l'identique. Le fichier MUST rester directement lisible dans un éditeur de texte ordinaire sans Vocalnotes, avec le contenu Markdown et le contexte clairement séparés. Les instructions temporaires et l'historique d'annulation MUST NOT être enregistrés dans la note.
- **FR-007**: Les boîtes de fichier MUST proposer le filtre et l'extension correspondant à l'action, ajouter l'extension lorsqu'elle manque et demander confirmation avant de remplacer un fichier existant. Un échec ou une annulation MUST préserver le travail et ne pas annoncer un succès.
- **FR-008**: La note MUST être considérée modifiée lorsque son contenu ou son contexte diffère de la dernière sauvegarde réussie, ou de l'état vide initial sans sauvegarde. Cet état MUST être visible avec le nom du document dans la barre de titre ; les instructions temporaires et les préférences MUST NOT affecter cet état.
- **FR-009**: Nouvelle note, Ouvrir, Quitter, Alt+F4 et la croix de fenêtre MUST partager la même protection des modifications : Enregistrer poursuit uniquement après succès, Ne pas enregistrer poursuit sans écriture, Annuler interrompt l'action sans perte. Sans modification, aucune confirmation de sauvegarde n'est nécessaire.
- **FR-010**: Exporter en markdown MUST écrire le contenu Markdown courant seul dans un .md, sans contexte, instructions ou métadonnées propres à l'application, et MUST préserver l'identité, l'historique et l'état modifié du document de travail.
- **FR-011**: L'export MUST NOT écraser le fichier de travail ; les conflits externes détectés avant l'écriture MUST être signalés et faire l'objet d'une décision explicite avant remplacement. Si une modification externe concurrente n'est détectée qu'après installation, l'application MUST signaler l'échec, conserver les versions utiles et la note ouverte, sans annoncer une sauvegarde réussie ni restaurer aveuglément une version par-dessus un autre écrivain.
- **FR-012**: Annuler et Rétablir MUST agir sur la zone éditable active, ou la dernière zone active lors de l'ouverture du menu, conformément à la User Story 4. Une note nouvellement créée ou ouverte MUST NOT réutiliser l'historique du document précédent.
- **FR-013**: Les changements persistants issus de saisie, de dictée, de mise en forme et de reformulation MUST participer à l'état modifié et être annulables ; une reformulation réussie MUST être annulable en une action.
- **FR-014**: Préférence MUST ouvrir un unique dialogue modal d'apparence Windows sous Windows et native du bureau Linux sous Linux, avec les réglages existants, navigation clavier, validation OK, abandon par Annuler/Echap/croix et restitution du focus, conformément à la User Story 5. La modalité et la réactivation de la fenêtre principale après fermeture ou échec MUST être effectives sur les deux systèmes.
- **FR-015**: Les préférences MUST être conservées uniquement après validation réussie, sans modifier la note, sans contact fournisseur provoqué par leur simple enregistrement et sans changer les règles existantes de prise d'effet.
- **FR-016**: A propos MUST ouvrir un dialogue indiquant le nom et la version de l'application et pouvoir se fermer au clavier sans modifier la note.
- **FR-017**: Les demandes concurrentes de sauvegarde, remplacement ou fermeture MUST être coordonnées pour éviter les doublons et les pertes. Avant de remplacer ou fermer la note, l'application MUST arrêter l'écoute, attendre les résultats de dictée et de reformulation déjà engagés dans une attente visible et annulable, puis les traiter sur la note courante selon les protections existantes avant d'évaluer les modifications et de proposer la sauvegarde. Annuler l'attente ou rencontrer un échec MUST abandonner le remplacement ou la fermeture et conserver la note ouverte, avec une explication en cas d'échec et sans redémarrer automatiquement l'écoute. Aucun résultat de la note précédente ne MUST pouvoir atteindre une nouvelle note.
- **FR-018**: L'ouverture des notes Markdown déjà prises en charge MUST rester possible. Un fichier .md pour lequel aucun contexte n'a été défini MUST pouvoir rester .md : Enregistrer MUST écrire son contenu au chemin .md courant sans imposer de conversion en .von. Sans changement de contexte depuis l'ouverture, Enregistrer MUST conserver le chemin existant et préserver les informations déjà persistées. Dès que le contexte diffère de celui chargé à l'ouverture, Enregistrer MUST ouvrir Enregistrer sous avec le format .von pour conserver la note complète sans modifier le .md initial ; le nouveau chemin ne MUST être adopté qu'après réussite. L'annulation ou l'échec MUST conserver les modifications et empêcher la fermeture ou le remplacement demandé. Enregistrer sous MUST proposer .von.
- **FR-019**: L'application MUST implémenter l'ensemble des commandes et protections FR-001 à FR-018 sous Windows et Linux. Les différences de décoration, de thème et de dialogues MUST suivre le système utilisé sans réduire les capacités fonctionnelles ni les garanties d'intégrité.
- **FR-020**: Les fichiers .von et les exports .md MUST être interopérables entre Windows et Linux sans conversion manuelle ; un aller-retour sur l'autre système MUST préserver le contenu Markdown et le contexte de la note complète.

### Key Entities *(include if feature involves data)*

- **Document de travail** : contenu Markdown, contexte, chemin éventuel, dernière version sauvegardée et état modifié ; représenté par un .von pour les nouvelles sauvegardes par défaut.
- **Export Markdown** : copie partageable du contenu courant seul, en .md, sans association nouvelle au document de travail.
- **Historique d'édition** : changements annulables et rétablissables d'une zone de la note courante ; distinct du fichier sauvegardé.
- **Décision de sauvegarde** : choix explicite précédant une action destructive, avec poursuite conditionnée à la réussite de l'écriture si nécessaire.
- **Préférences** : réglages applicatifs existants, distincts des données de note, avec changements provisoires jusqu'à validation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

SC-001 à SC-010 s'appliquent séparément à Windows et à Linux ; un résultat obtenu sur un système ne valide pas l'autre.

- **SC-001**: Les 10 commandes demandées sont présentes dans les bons menus et dans le bon ordre ; 100 % sont accessibles sans souris et les 7 raccourcis listés exécutent la même action que le menu.
- **SC-002**: Dans la matrice des 4 actions Nouvelle note, Ouvrir, Quitter et croix de fenêtre, chaque choix Enregistrer, Ne pas enregistrer et Annuler produit le résultat attendu ; aucun essai avec annulation ou échec d'enregistrement ne perd le contenu ou le contexte.
- **SC-003**: Sur 20 cycles de sauvegarde et réouverture .von couvrant texte accentué, Markdown et contextes vides ou non vides, 100 % restituent exactement le contenu et le contexte ; chacun des 20 fichiers permet aussi de lire et distinguer ces deux champs dans un éditeur de texte ordinaire sans Vocalnotes.
- **SC-004**: Sur 20 exports de notes enregistrées ou non, 100 % des .md contiennent uniquement le contenu courant et aucun export ne modifie le chemin de travail ni l'état modifié.
- **SC-005**: Sur 20 séquences Annuler/Rétablir réparties entre les zones éditables et les sources de modification prévues, 100 % restaurent le texte attendu sans modifier une autre zone ; revenir à la version enregistrée supprime systématiquement l'indicateur de modification.
- **SC-006**: Sur 10 parcours de préférences, aucun dialogue dupliqué n'apparaît ; tous les changements confirmés sont retrouvés à la réouverture, aucun changement annulé n'est conservé et le focus revient à l'éditeur à chaque fermeture.
- **SC-007**: Pour les interactions locales de menu et l'ouverture du dialogue de préférences avec des réglages disponibles, une réponse visible intervient en moins de 300 ms dans au moins 95 % de 30 essais sur chacun des postes Windows et Linux de référence.
- **SC-008**: Sur chaque système de référence, au moins 4 utilisateurs sur 5 familiers d'un éditeur du système testé réalisent sans aide la création, la sauvegarde, la réouverture, l'export et la fermeture avec abandon ou annulation, et identifient le dialogue de préférences comme une fenêtre familière de ce système.
- **SC-009**: Dans les parcours Nouvelle note, Ouvrir et fermeture avec dictée ou reformulation en cours, 100 % des décisions de sauvegarde interviennent après traitement des résultats attendus ; une annulation de l'attente ou un échec conserve systématiquement la note ouverte et aucun résultat ne rejoint le document suivant.
- **SC-010**: Dans les parcours d'enregistrement d'une note .md sans contexte défini, 100 % conservent le chemin et le format .md sans imposer de conversion. Avec un contexte ajouté, modifié ou vidé par rapport à l'ouverture, 100 % proposent une sauvegarde complète en .von sans changer le .md initial ; une annulation ou un échec conserve systématiquement les modifications et la note ouverte.
- **SC-011**: Sur 20 échanges de fichiers par sens Windows vers Linux puis Linux vers Windows, 100 % des notes .von conservent exactement leur contenu et leur contexte après ouverture et réenregistrement ; 100 % des exports .md restent lisibles avec le seul contenu attendu.

## Assumptions

- Le périmètre couvre l'application desktop sous Windows et Linux, avec un document courant par fenêtre ; les onglets, fichiers récents, impression, associations système de fichiers et récupération automatique ne sont pas introduits. macOS n'entre pas dans cette extension de périmètre.
- La référence Linux reprend l'environnement déjà ciblé par le projet : Fedora avec GNOME sous Wayland ; Windows 11 reste la référence Windows. Le support garanti d'autres distributions, de KDE ou de sessions X11 n'est pas ajouté implicitement. Une simple compilation Linux, des tests simulés ou une fenêtre sous WSLg ne remplacent pas une recette dans le bureau Linux de référence.
- « Native » décrit ici le comportement observable, les dialogues de fichiers du système utilisé et l'apparence familière des préférences : Windows sous Windows, bureau Linux sous Linux. Il ne s'agit pas de reproduire visuellement Windows sous Linux ; aucun choix technique pour la barre ou le dialogue de préférences n'est imposé.
- Les noms de menu restent ceux demandés, notamment « Edition », « Préférence » et « A propos ». Les touches d'accès Alt doivent être distinctes dans un même menu ; leur lettre exacte sera choisie lors de la conception.
- Les nouveaux fichiers de travail utilisent .von sous forme de texte lisible sans Vocalnotes, avec contenu Markdown et contexte clairement séparés. Un fichier .md sans contexte défini peut rester .md et être enregistré au même chemin, sans conversion imposée. Les fichiers Markdown historiques restent ouvrables ; Enregistrer conserve leur chemin uniquement si leur contexte n'a pas changé depuis l'ouverture, sinon propose Enregistrer sous en .von en préservant le .md initial. Enregistrer sous propose toujours .von par défaut. La représentation exacte de la séparation contenu/contexte relève de la planification, sans perte des informations existantes.
- Le périmètre des préférences reste celui des réglages existants ; aucune nouvelle option, aucun fournisseur ni changement de politique des secrets n'est demandé. Le dialogue utilise OK et Annuler, sans bouton Appliquer.
- Le contenu exporté est le Markdown de la note, pas sa représentation visuelle ; un contenu vide peut être exporté comme fichier vide. Le contexte est exclu en tant que champ, sans supprimer du contenu un texte identique éventuellement saisi par l'utilisateur.
- Les instructions restent temporaires selon les règles existantes ; l'effacement volontaire de ces instructions ne déclenche pas la confirmation de sauvegarde de la note.
- La constitution actuelle, principe III, décrit les notes comme des fichiers Markdown locaux utilisables sans l'application. Le choix confirmé d'un .von textuel conserve l'accès au contenu Markdown et au contexte sans Vocalnotes ; la conformité de leur représentation exacte au principe III reste à vérifier lors de la planification. Cette spécification ne modifie pas la constitution. Les garanties de propriété locale, d'intégrité et de confidentialité restent applicables.
- L'implémentation devra préserver les modes de dictée, le contexte associé à la note et le déclenchement uniquement explicite de la reformulation déjà spécifiés ; aucune évolution de leur fonctionnement n'est demandée hors coordination avec les actions de document.

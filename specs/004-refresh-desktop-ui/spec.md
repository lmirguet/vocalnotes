# Feature Specification: Refonte de l'interface desktop Vocalnotes

**Feature Branch**: `lmirguet-capgemini-vocalnotes-version-2`

**Created**: 2026-11-13

**Status**: Draft

**Input**: User description: "utilise la maquette qui vient d'être produite pour mettre à jour le frontend de l'application"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Rédiger dans une fenêtre desktop claire (Priority: P1)

L'utilisateur travaille dans une fenêtre de bureau compacte qui présente une barre de titre, une barre de menus, des outils, une barre d'état et un éditeur Markdown occupant l'essentiel de la fenêtre. Le contexte est affiché dans un panneau plus petit au-dessus du contenu ; les deux restent modifiables quel que soit le mode sélectionné.

**Why this priority**: L'éditeur est l'espace de travail principal. Une hiérarchie compacte permet de rédiger sans détourner l'attention vers des cartes ou des éléments de tableau de bord.

**Independent Test**: Ouvrir une note dans la fenêtre desktop, identifier le contexte et l'éditeur de contenu, puis modifier les deux champs dans chacun des trois modes.

**Acceptance Scenarios**:

1. **Given** une note ouverte, **When** l'utilisateur observe la fenêtre, **Then** les modes et l'action de reformulation apparaissent dans la barre d'outils, les outils de mise en forme dans une barre distincte, et l'état de session dans la barre d'état.
2. **Given** le panneau de contexte et l'éditeur de contenu affichés, **When** l'utilisateur modifie l'un ou l'autre dans n'importe quel mode, **Then** seule la zone modifiée change.
3. **Given** la fenêtre à une taille de bureau usuelle, **When** l'utilisateur rédige, **Then** le contenu de la note occupe la majeure partie de l'espace de travail et le contexte reste dans un panneau plus compact au-dessus.
4. **Given** du contenu Markdown existant, **When** l'utilisateur ouvre la note dans cette interface, **Then** le contenu et ses styles restent affichés et éditables sans changement de sens.

### User Story 2 - Dicter dans le mode choisi sans commande vocale supplémentaire (Priority: P1)

L'utilisateur choisit Contenu, Contexte ou Instruction. Le mode sélectionné détermine la destination de la dictée et démarre l'écoute. Cliquer à nouveau sur le mode actif arrête l'écoute et désélectionne les modes ; cliquer sur un autre mode bascule vers sa cible. Il n'y a pas de commande graphique séparée pour autoriser, démarrer ou arrêter le vocal ; la barre d'état rend visible l'état vocal et sa cible.

**Why this priority**: Le choix du mode doit suffire à orienter la dictée, sans commandes vocales redondantes ni ambiguïté sur le champ destinataire.

**Independent Test**: Vérifier qu'un clic démarre chaque mode, qu'un second clic sur le mode actif arrête et désélectionne, et qu'un clic sur un autre mode bascule vers sa cible, y compris avec un segment Instruction en cours.

**Acceptance Scenarios**:

1. **Given** l'application qui s'ouvre sans mode sélectionné, **When** l'utilisateur clique sur un bouton de mode, **Then** l'écoute démarre pour la cible correspondante.
2. **Given** un mode actif, **When** l'utilisateur reclique sur le même bouton, **Then** l'écoute s'arrête et aucun mode n'est sélectionné.
3. **Given** un mode actif, **When** l'utilisateur clique sur un autre bouton de mode, **Then** l'écoute de l'ancien mode se termine, la nouvelle cible devient active et l'état vocal reflète le nouveau mode.
4. **Given** le mode Contexte ou Instruction actif, **When** l'utilisateur dicte, **Then** la transcription est dirigée vers la zone correspondant à ce mode et ne modifie pas les autres champs.
5. **Given** une session vocale active, **When** l'utilisateur ferme la note ou l'application, **Then** l'écoute prend fin.
6. **Given** l'interface affichée, **When** l'utilisateur cherche une commande vocale, **Then** aucune commande graphique distincte d'autorisation, de démarrage, de reprise, de pause ou d'arrêt n'est présentée.
7. **Given** un segment d'instruction encore en cours de transcription, **When** l'utilisateur clique sur un autre mode ou reclique sur Instruction, **Then** le segment est d'abord dirigé vers Instruction et son texte est ensuite effacé.

### User Story 3 - Consulter les instructions uniquement au besoin (Priority: P1)

L'utilisateur accède aux instructions temporaires en sélectionnant Instruction. Une petite fenêtre contextuelle sans bordure apparaît près de son bouton de mode ; ses instructions restent modifiables. À la sortie du mode, la fenêtre disparaît et son texte est effacé.

**Why this priority**: Les instructions ne doivent pas encombrer l'éditeur en usage normal ni être confondues avec le contexte ou le contenu.

**Independent Test**: Saisir une instruction, passer vers Contenu ou Contexte et vérifier que la fenêtre disparaît et que le texte temporaire est effacé ; revenir à Instruction et confirmer que la zone est vide.

**Acceptance Scenarios**:

1. **Given** Contenu ou Contexte sélectionné, **When** l'utilisateur observe l'interface, **Then** aucune zone d'instructions n'est visible.
2. **Given** un autre mode actif, **When** l'utilisateur sélectionne Instruction, **Then** la zone d'instructions apparaît dans une fenêtre contextuelle sans bordure et accepte la saisie manuelle.
3. **Given** du texte dans les instructions, **When** l'utilisateur sélectionne Contenu ou Contexte, **Then** la fenêtre se ferme et le texte est effacé.
4. **Given** un mode sélectionné, **When** l'utilisateur consulte les commandes de mise en forme, **Then** les icônes usuelles du titre, du gras, de l'italique, du paragraphe et de la liste à puces sont reconnaissables et chacune possède un nom accessible ainsi qu'une infobulle.
5. **Given** une note prête à être reformulée, **When** l'utilisateur n'a pas activé la commande explicite de reformulation, **Then** aucun appel au modèle de langage n'est déclenché par la saisie, la dictée ou le changement de mode.
6. **Given** l'utilisateur active la commande explicite de reformulation, **When** le traitement réussit, **Then** seul le contenu de la note est mis à jour selon les règles de fidélité déjà établies.
7. **Given** le mode Instruction est actif, **When** l'utilisateur clique sur « Valider les instructions », **Then** l'application enregistre d'abord le contexte et les instructions courants puis lance une seule reformulation ; si leur enregistrement échoue, aucun appel au modèle n'est lancé.

### Edge Cases

- L'utilisateur quitte Instruction alors qu'une transcription d'instruction est encore en cours : le segment est acheminé à sa cible d'origine avant l'effacement des instructions.
- L'utilisateur ferme la note ou l'application pendant une session vocale : l'écoute active prend fin.
- Une fenêtre d'instructions est ouverte et l'utilisateur sélectionne un autre mode : la fenêtre ne bloque pas la sélection du mode et se ferme.
- La fenêtre est réduite : les commandes restent accessibles et l'éditeur de contenu conserve l'espace de rédaction principal.
- Les instructions sont vides après un changement de mode : elles ne sont pas incluses dans une reformulation ultérieure.
- Le modèle échoue ou renvoie une réponse inutilisable : le contenu courant reste intact et l'échec est signalé, conformément au comportement existant.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: L'application MUST présenter une interface desktop compacte avec une barre de titre, une barre de menus, une barre d'outils des modes et de la reformulation, une barre de mise en forme distincte, un éditeur principal et une barre d'état.
- **FR-002**: L'application MUST placer le panneau de contexte au-dessus du contenu de la note, avec un séparateur visible entre les deux ; le panneau de contexte MUST être plus compact que l'éditeur de contenu et les deux zones MUST rester modifiables dans chacun des modes.
- **FR-003**: L'application MUST présenter les modes Contenu, Contexte et Instruction sous forme de boutons comprenant chacun une icône distincte et un libellé visible ; le mode actif MUST être identifiable.
- **FR-004**: L'application MUST faire apparaître la zone d'instructions uniquement en mode Instruction, dans une fenêtre contextuelle sans bordure. Cette zone MUST pouvoir être modifiée manuellement.
- **FR-005**: En quittant le mode Instruction, l'application MUST fermer la fenêtre contextuelle et effacer le texte des instructions ; tout segment déjà en cours de transcription MUST d'abord être dirigé vers son mode d'origine.
- **FR-006**: L'application MUST utiliser des icônes conventionnelles reconnaissables pour les commandes usuelles de mise en forme Markdown, avec un nom accessible et une infobulle pour chaque commande.
- **FR-007**: La sélection d'un mode MUST activer automatiquement la dictée vers la zone correspondant à ce mode et terminer la dictée du mode précédent ; la barre d'état MUST indiquer l'état vocal et sa cible.
- **FR-008**: L'interface MUST ne pas présenter de commande graphique distincte pour autoriser, démarrer, reprendre, mettre en pause ou arrêter le vocal.
- **FR-009**: La fermeture de la note ou de l'application MUST terminer toute écoute vocale active.
- **FR-010**: L'application MUST préserver le contenu, le contexte et les capacités Markdown existants pendant les changements d'interface ; les éléments de statut et métriques de session déjà présents MUST rester accessibles dans la barre d'état.
- **FR-011**: L'application MUST conserver la commande explicite de reformulation ; le système MUST NOT déclencher d'appel au modèle de langage automatiquement à la suite d'un changement de mode, d'une dictée ou d'une saisie.
- **FR-012**: Une reformulation réussie MUST mettre à jour uniquement le contenu de la note, en respectant les règles de fidélité et de préservation des idées existantes.
- **FR-013**: L'interface MUST garder l'éditeur de contenu comme élément visuel dominant et éviter la présentation en grandes cartes ou en tableau de bord.
- **FR-014**: En mode Instruction, la popup MUST proposer un bouton accessible « Valider les instructions » qui déclenche explicitement la reformulation existante avec le contexte, les instructions et le contenu courants ; l'action MUST attendre l'acquittement des champs et ne MUST NOT envoyer la requête si l'enregistrement échoue ou si une reformulation est déjà en cours.

### Key Entities *(include if feature involves data)*

- **Mode actif** : Contenu, Contexte ou Instruction ; détermine la cible de dictée et la visibilité de la fenêtre d'instructions.
- **Contexte de note** : texte éditable placé au-dessus du contenu et conservé avec celui-ci selon le comportement existant.
- **Contenu de note** : texte Markdown éditable qui occupe l'espace principal et reste seul modifiable par une reformulation réussie.
- **Instructions temporaires** : texte éditable visible dans une fenêtre contextuelle uniquement en mode Instruction et effacé à la sortie du mode.
- **État vocal** : indication visible de l'écoute active et de la cible associée au mode sélectionné.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Dans une fenêtre de bureau de 1280 × 800 pixels, l'éditeur de contenu occupe au moins la moitié de la hauteur utile de l'espace de travail, avec le contexte dans un panneau plus compact au-dessus.
- **SC-002**: Dans 30 essais, répartis à raison de 10 par mode, 100 % des changements de mode affichent le mode sélectionné, la cible vocale correspondante et la fenêtre d'instructions uniquement pour Instruction.
- **SC-003**: Dans 20 essais de sortie du mode Instruction, 100 % ferment la fenêtre contextuelle et effacent le texte après le routage de tout segment d'instruction déjà en cours.
- **SC-004**: Au moins 90 % des utilisateurs d'essai identifient le mode actif et la cible de dictée sans aide.
- **SC-005**: Dans 20 essais de mise en forme au clavier et à la souris, chaque commande possède un nom accessible et une infobulle, et l'utilisateur peut identifier son effet à partir de son icône.
- **SC-006**: Dans 100 essais couvrant la saisie, la dictée et les changements de mode sans activer Reformuler, aucun appel au modèle de langage n'est déclenché.
- **SC-007**: Dans 20 essais de fermeture d'une note ou de l'application pendant l'écoute, 100 % terminent la session vocale active sans laisser l'état affiché comme actif.

## Assumptions

- La maquette desktop Vocalnotes v2 publiée avec la demande sert de référence visuelle ; elle est illustrative et n'ajoute aucune fonction au-delà des exigences décrites ici.
- La spécification s'applique à l'interface desktop Windows existante ; les comportements de sauvegarde, de contexte, de Markdown, de fournisseurs vocaux et de reformulation restent inchangés sauf les adaptations explicitement décrites.
- À l'ouverture de l'application, aucun mode n'est sélectionné et le microphone reste arrêté. L'écoute démarre uniquement lorsque l'utilisateur sélectionne explicitement un mode, y compris Contenu.
- La fin d'un mode vocal correspond à la sélection d'un autre mode ou à la fermeture de la note ou de l'application. Changer de mode termine l'écoute précédente avant de diriger les nouvelles transcriptions vers la cible suivante.
- L'autorisation système du microphone est considérée comme déjà accordée pour ce parcours ; aucune commande de permission vocale propre à l'application n'est ajoutée à l'interface.
- Les instructions restent temporaires et sont effacées en quittant Instruction ; le contexte et le contenu restent conservés selon les règles existantes.
- L'action de reformulation reste explicite et ne s'exécute pas automatiquement. Les règles existantes de fidélité du modèle restent applicables.
- Les icônes mode et mise en forme servent à identifier des fonctions déjà prévues ; aucun nouveau format ou outil d'édition n'est ajouté.
- Les tableaux et les images dans le contenu ne sont pas introduits par cette mise à jour.

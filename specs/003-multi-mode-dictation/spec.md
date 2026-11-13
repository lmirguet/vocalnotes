# Spécification de fonctionnalité : Dictée multimode et reformulation de notes

**Feature Branch**: `lmirguet-capgemini-vocalnotes-version-2`

**Created**: 2026-11-13

**Status**: spécification prête pour la planification

**Input**: Refonte de Vocalnotes autour de trois modes distincts — dictée de contenu, dictée de contexte et instructions — avec une action explicite pour demander au modèle de langage de reformater la note en préservant les idées de l'utilisateur. L'éditeur de note prend en charge les styles Markdown usuels ; les tableaux et images sont envisagés pour une évolution ultérieure.

## Clarifications

### Session 2026-11-13

- Q: Should the context text be saved with its note and restored when the note is reopened, or kept only for the current editing session? → A: Save context with the note and restore it when the note is reopened.
- Q: When a user changes dictation mode while a spoken segment is still being transcribed, which field should receive that segment? → A: Route the segment to the mode active when speech began, even if the mode changes before transcription finishes.
- Q: Should tables and images be part of this v2 editor, or remain a future enhancement while v2 supports Markdown styling such as bold, italics, and headings? → A: V2 supports Markdown styling (bold, italics, headings, etc.); tables and images are future enhancements.
- Q: When the user leaves Instructions mode while a spoken instruction is still being transcribed, should the mode switch wait for the transcription to finish before clearing instructions? → A: Yes; wait for pending instruction segments to be transcribed and routed to Instructions, then switch modes and clear the instruction text.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Dicter dans le champ correspondant au mode actif (Priority: P1)

L'utilisateur choisit la destination de sa dictée en sélectionnant un mode. Le contexte et le contenu de la note restent séparés ; les instructions disposent d'une zone dédiée qui n'est visible qu'en mode Instructions.

**Why this priority**: séparer les propos destinés à la note des informations de contexte et des consignes évite que l'application confonde le contenu à rédiger avec les instructions de transformation.

**Independent Test**: sélectionner successivement chacun des trois modes, dicter une phrase différente et vérifier que chaque transcription est ajoutée uniquement au champ correspondant.

**Acceptance Scenarios**:

1. **Given** le mode Dictée de contenu, **When** l'utilisateur dicte du texte, **Then** la transcription est ajoutée au contenu de la note sans modifier le contexte.
2. **Given** le mode Dictée de contexte, **When** l'utilisateur dicte du texte, **Then** la transcription est ajoutée au contexte sans modifier le contenu.
3. **Given** le mode Instructions, **When** l'utilisateur dicte une consigne, **Then** elle est ajoutée à la zone d'instructions, visible dans ce mode seulement, sans modifier le contexte ni le contenu.
4. **Given** du texte saisi dans le contexte et le contenu, **When** l'utilisateur change de mode, **Then** ces deux textes restent inchangés et les zones restent modifiables manuellement.
5. **Given** du texte dans la zone d'instructions, **When** l'utilisateur quitte le mode Instructions, **Then** la zone disparaît et son texte est effacé.
6. **Given** un segment de parole en cours de transcription, **When** l'utilisateur change de mode avant que la transcription soit terminée, **Then** le segment est ajouté à la zone correspondant au mode actif au début de la parole. Si le segment a été dicté en mode Instructions, le changement de mode attend la fin de sa transcription ; le texte est ensuite ajouté à Instructions avant que la sortie du mode n'efface cette zone.

### User Story 2 - Saisir et organiser manuellement les informations (Priority: P1)

L'utilisateur peut compléter ou corriger les champs visibles au clavier, indépendamment du mode de dictée sélectionné.

**Why this priority**: les modes orientent la dictée sans empêcher l'utilisateur de corriger les champs directement.

**Independent Test**: saisir manuellement du texte dans le contexte, le contenu et, en mode Instructions, la zone d'instructions ; vérifier que chaque champ conserve son propre texte.

**Acceptance Scenarios**:

1. **Given** n'importe quel mode, **When** l'utilisateur saisit ou modifie du texte dans le contexte ou le contenu, **Then** seule la zone éditée change.
2. **Given** le mode Instructions, **When** l'utilisateur saisit ou modifie une consigne, **Then** seule la zone d'instructions change.
3. **Given** du texte dans le contexte et le contenu, **When** l'utilisateur sélectionne chaque mode, **Then** le mode actif est identifiable et aucun des deux champs n'est effacé.
4. **Given** une note avec un contexte enregistré, **When** l'utilisateur ferme puis rouvre cette note, **Then** le contexte est restauré avec la note.
5. **Given** du contenu avec des titres, du gras ou de l'italique Markdown, **When** l'utilisateur consulte ou modifie la note, **Then** ces styles sont affichés correctement et restent éditables.

### User Story 3 - Reformater le contenu à la demande (Priority: P1)

L'utilisateur déclenche explicitement une reformulation depuis la barre d'outils. Le modèle reçoit le contenu actuel et, lorsqu'ils ne sont pas vides, le contexte et les instructions. Seul le contenu de la note est mis à jour.

**Why this priority**: le contrôle explicite du traitement sépare la rédaction de la demande d'amélioration et évite tout appel automatique pendant la dictée ou la saisie.

**Independent Test**: remplir les champs, vérifier qu'aucun traitement ne se produit pendant la saisie ou les changements de mode, puis déclencher la reformulation et contrôler l'entrée envoyée et le champ modifié.

**Acceptance Scenarios**:

1. **Given** du texte dans les champs, **When** l'utilisateur saisit, dicte ou change de mode sans déclencher la commande de reformulation, **Then** aucun appel au modèle de langage n'est effectué.
2. **Given** un contenu non vide et un contexte ou des instructions renseignés, **When** l'utilisateur déclenche la reformulation, **Then** le modèle reçoit l'état courant du contenu ainsi que chaque champ non vide.
3. **Given** une réponse valide du modèle, **When** elle est appliquée, **Then** elle remplace le contenu courant de la note sans modifier le contexte.
4. **Given** un contenu sans instruction explicite demandant d'ajouter des idées, **When** le modèle le reformate, **Then** la sortie conserve les concepts et informations fournis et se limite à la mise en forme, au style, à la ponctuation, à l'orthographe et à de légères adaptations de phrasé.
5. **Given** une instruction explicite demandant un ajout ou une adaptation de fond, **When** le modèle reformate le contenu, **Then** il peut effectuer uniquement les ajouts ou adaptations autorisés par cette instruction.
6. **Given** un contenu vide, **When** l'utilisateur demande une reformulation, **Then** aucun appel n'est effectué et l'application explique qu'il n'y a pas de contenu à reformater.
7. **Given** une erreur de traitement ou une réponse inutilisable, **When** elle survient, **Then** le contenu existant reste intact et l'application signale l'échec.
8. **Given** que l'utilisateur modifie le contenu pendant un traitement, **When** une réponse fondée sur l'ancien contenu arrive, **Then** elle ne remplace pas les modifications plus récentes et l'application signale que le résultat doit être relancé.
9. **Given** un contenu avec des titres, du gras ou de l'italique Markdown, **When** l'utilisateur demande une reformulation, **Then** les styles restent correctement représentés dans le contenu mis à jour.

### Edge Cases

- Le contexte est vide ou ne contient que des espaces : il est omis de la demande.
- Les instructions sont vides ou ne contiennent que des espaces : elles sont omises de la demande.
- L'utilisateur quitte le mode Instructions avant d'avoir déclenché une reformulation : les instructions sont effacées et ne sont pas envoyées.
- Une note sans contexte enregistré s'ouvre avec un champ de contexte vide.
- La transcription d'un segment peut arriver après un changement de mode : elle est toujours dirigée vers la zone associée au mode actif au début de la parole.
- Si l'utilisateur demande à quitter le mode Instructions pendant qu'un segment dicté dans ce mode est encore en cours de transcription, le changement de mode attend sa finalisation ; le segment est d'abord dirigé vers Instructions, puis le texte des instructions est effacé lors du changement de mode.
- Le contenu est vide : la commande de reformulation ne lance pas de traitement.
- Le traitement échoue ou renvoie un résultat vide ou inutilisable : le contenu courant est préservé et l'échec est visible.
- Le contenu est modifié avant le retour d'une reformulation : la réponse obsolète ne peut pas écraser la modification.
- Le contenu contient des titres, du gras ou de l'italique Markdown : ces styles sont préservés lors de l'affichage, de l'édition et de la reformulation.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: L'application MUST proposer les modes Dictée de contenu, Dictée de contexte et Instructions, sélectionnables depuis la barre d'outils.
- **FR-002**: L'application MUST afficher en permanence une zone de contexte au-dessus d'une zone de contenu plus grande ; la zone d'instructions MUST être visible uniquement en mode Instructions.
- **FR-003**: L'application MUST diriger chaque segment transcrit vers la zone correspondant au mode actif au début de la parole, même si le mode change avant la fin de la transcription, sans modifier les autres zones.
- **FR-004**: L'utilisateur MUST pouvoir saisir et modifier manuellement le contexte et le contenu dans chacun des modes, ainsi que les instructions lorsque leur zone est visible.
- **FR-005**: L'application MUST conserver le contexte et le contenu lors des changements de mode, enregistrer le contexte avec la note et le restaurer à sa réouverture. En quittant le mode Instructions, elle MUST effacer le texte des instructions.
- **FR-006**: L'application MUST fournir dans la barre d'outils une commande explicite de reformulation et MUST appeler le modèle uniquement lorsque l'utilisateur déclenche cette commande.
- **FR-007**: Lors d'une reformulation, l'application MUST envoyer l'état courant du contenu et inclure le contexte et les instructions uniquement s'ils contiennent du texte non blanc.
- **FR-008**: Une réponse valide MUST mettre à jour uniquement le contenu de la note ; le contexte et les autres champs MUST rester inchangés.
- **FR-009**: Sans instruction explicite autorisant un ajout de fond, le modèle MUST préserver les idées et informations fournies et se limiter à la présentation, au style, à la ponctuation, à l'orthographe et à de légères adaptations de phrasé.
- **FR-010**: Si le contenu est vide, l'application MUST empêcher l'appel et expliquer à l'utilisateur qu'il n'y a rien à reformater.
- **FR-011**: En cas d'échec ou de réponse inutilisable, l'application MUST préserver le contenu et signaler l'échec sans présenter le traitement comme réussi.
- **FR-012**: Une réponse calculée à partir d'un contenu devenu obsolète MUST NOT écraser des modifications plus récentes de la note ; l'application MUST informer l'utilisateur que le résultat doit être recalculé.
- **FR-013**: L'éditeur de note MUST afficher et permettre l'édition de styles Markdown usuels, au minimum les titres, le gras et l'italique.
- **FR-014**: L'application MUST préserver ces styles Markdown lors de l'enregistrement, de la réouverture et des reformulations, sauf instruction explicite demandant de les modifier.
- **FR-015**: Si l'utilisateur demande à quitter le mode Instructions alors qu'un segment dicté dans ce mode est encore en cours de transcription, l'application MUST différer le changement de mode jusqu'à la fin de la transcription, diriger le segment vers Instructions, puis effacer le texte des instructions lors du changement de mode.

### Key Entities *(include if feature involves data)*

- **Contenu de la note** : texte principal structuré par l'utilisateur avec des styles Markdown, et seul champ modifié par une reformulation valide.
- **Contexte** : texte facultatif enregistré avec la note, restauré à sa réouverture, conservé entre les modes et transmis avec chaque reformulation lorsqu'il n'est pas vide.
- **Instructions** : texte facultatif, visible et modifiable en mode Instructions ; il est effacé quand l'utilisateur quitte ce mode.
- **Mode actif** : l'un des trois modes qui détermine la destination de la dictée et la visibilité de la zone d'instructions.
- **Reformulation** : demande explicite composée des champs non vides et du contenu courant, avec une réponse destinée à remplacer le contenu de la note.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Dans 30 essais de dictée répartis également entre les trois modes, dont certains où le mode change avant la fin de la transcription, 100 % des transcriptions sont ajoutées uniquement au champ ciblé au début de la parole.
- **SC-002**: Dans 20 essais d'édition et de changement de mode, 100 % des modifications de contexte et de contenu sont conservées ; les instructions sont effacées à chaque sortie du mode Instructions.
- **SC-003**: Dans 100 essais couvrant la saisie, la dictée, les changements de mode et la reformulation explicite, aucun appel au modèle n'est effectué avant l'action explicite de reformulation.
- **SC-004**: Dans 20 essais de reformulation sans instruction d'ajout, aucune sortie n'introduit de concept ou d'information absents des champs fournis.
- **SC-005**: Dans tous les essais d'échec et de réponse obsolète, le contenu le plus récent reste intact et l'utilisateur reçoit un retour compréhensible.
- **SC-006**: Au moins 90 % des utilisateurs d'essai identifient le mode actif et saisissent une dictée dans la zone attendue sans aide.
- **SC-007**: Dans 20 essais d'enregistrement et de réouverture, 100 % des notes retrouvent leur contexte enregistré.
- **SC-008**: Dans 20 essais contenant des titres, du gras ou de l'italique Markdown, 100 % des styles restent affichés et éditables après enregistrement, réouverture et reformulation.
- **SC-009**: Dans tous les essais où l'utilisateur quitte le mode Instructions pendant une transcription en attente, le mode ne change qu'après la finalisation et le routage du segment ; les instructions sont ensuite effacées.

## Assumptions

- Le démarrage et l'arrêt de la capture vocale, la transcription, les fournisseurs du modèle de langage et le cycle de vie des fichiers de notes restent régis par le comportement existant ; cette fonctionnalité définit le routage, les champs d'édition et la reformulation.
- Les zones de contexte et de contenu restent modifiables manuellement dans tous les modes. Les instructions ne sont saisissables ou modifiables que lorsque leur zone est affichée en mode Instructions.
- Le contexte et le contenu sont enregistrés avec la note et restaurés à sa réouverture ; la zone d'instructions est temporaire et son effacement en quittant le mode est une décision confirmée par l'utilisateur. Si une dictée d'instruction est encore en cours, la sortie du mode attend sa transcription et son routage avant d'effacer la zone.
- L'éditeur de note prend en charge au minimum les titres, le gras et l'italique Markdown. Les tableaux et les images dans le contenu ne font pas partie de cette fonctionnalité et sont réservés à une évolution ultérieure.
- Les règles d'interaction de cette spécification v2 remplacent les règles antérieures incompatibles concernant les modes et les transformations automatiques ; les principes de fidélité aux propos restent applicables.

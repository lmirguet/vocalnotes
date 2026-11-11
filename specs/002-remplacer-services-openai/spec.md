# Feature Specification: Remplacer les services Mistral par OpenAI

**Feature Branch**: aucune nouvelle branche ; fonctionnalité repérée par `.specify/feature.json`.

**Created**: 2026-11-11

**Status**: spécification clarifiée ; modèles retenus après décisions utilisateur et essais de faisabilité bornés, recette produit à réaliser.

**Input**: User description: "modifie l'application vocalnotes pour remplacer les services Mistral par les services OpenAI. Notamment utilise le modèle \"gpt-4o-transcribe\" pour le speech-to-text, et le modèle \"gpt-6.1-luna\" pour les appels LLM."

Cette fonctionnalité remplace le fournisseur de transcription et d'interprétation,
sans refondre le bloc-notes. Elle complète la
[spécification initiale](../001-bloc-notes-vocal/spec.md) et respecte la
[constitution 3.0.0](../../.specify/memory/constitution.md).
Elle remplace uniquement l'orientation initiale réservant OpenAI à une évolution
ultérieure et faisant de Mistral la première intégration.

Les noms OpenAI, `gpt-realtime-whisper` et `gpt-6-luna` constituent des contraintes
de produit explicitement demandées, non des choix d'architecture.
La demande initiale citait `gpt-6.1-luna` ; après vérification des modèles accessibles,
l'utilisateur a explicitement retenu `gpt-6-luna`.
L'utilisateur a ensuite autorisé le remplacement de `gpt-4o-transcribe` par
`gpt-realtime-whisper` à condition qu'un essai confirme le texte pendant la parole.
Cet essai synthétique a satisfait cette condition, sans modifier les seuils de recette.

## Clarifications

### Session 2026-11-11

- Q: Comment l’application doit-elle choisir la langue de transcription entre le français et l’anglais ? → A: Détection automatique sans réglage obligatoire ; le passage du français à l’anglais entre interventions est accepté sans arrêter la session.
- Q: Quel identifiant de modèle retenir après vérification des familles GPT 6 et GPT 6.1 accessibles ? → A: `gpt-6-luna`, choisi explicitement par l'utilisateur en remplacement de `gpt-6.1-luna`.
- Q: Faut-il accepter le texte progressif seulement après la fin de chaque passage avec le modèle demandé ? → A: Non ; conserver l'objectif de texte pendant la parole et étudier une autre stratégie avant de finaliser le plan.
- Q: Peut-on remplacer `gpt-4o-transcribe` par `gpt-realtime-whisper` si un essai confirme le texte pendant la parole ? → A: Oui ; remplacement explicitement autorisé et condition satisfaite par un essai anglais synthétique de dix secondes. `gpt-6-luna` et les exigences de qualité et de délai restent inchangés.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Dicter en français ou en anglais (Priority: P1)

L'utilisateur démarre le mode vocal et parle en plusieurs interventions. La
transcription progresse pendant la parole, puis les passages terminés deviennent
disponibles pour la rédaction, sans bouton à maintenir ni envoi manuel de fichier.

**Why this priority**: la dictée continue est la fonction centrale et doit rester
utilisable sur l'ordinateur où le précédent fournisseur rencontre un obstacle réseau.

**Independent Test**: transcrire une série de passages de référence en français
et en anglais, sans interprétation de la note, et comparer les résultats aux propos.

**Acceptance Scenarios**:

1. **Given** un accès OpenAI autorisé et le mode vocal démarré, **When**
   l'utilisateur dicte plusieurs passages en français, **Then** la transcription
   progressive et les passages finaux proviennent de `gpt-realtime-whisper`, sans
   contact avec Mistral ni réactivation entre les interventions.
2. **Given** les mêmes conditions avec une dictée anglaise, **When** l'utilisateur
   parle en anglais, **Then** les propos sont transcrits en anglais, sans traduction
   imposée vers le français.
3. **Given** une intervention en cours, **When** des résultats provisoires puis
   le résultat final arrivent, **Then** les résultats provisoires ne modifient pas
   la note et le passage final ne peut être intégré qu'une seule fois.
4. **Given** une session arrêtée, mise en pause ou invalidée par un événement
   système, **When** un résultat retardé arrive, **Then** il ne modifie ni la note
   ni l'intention courante ; la reprise exige l'action explicite prévue.
5. **Given** une session active avec une intervention française terminée,
   **When** l'utilisateur poursuit par une intervention anglaise, **Then** la
   langue est détectée automatiquement, sans réglage ni arrêt de session, et
   chaque intervention est transcrite dans sa langue.

---

### User Story 2 - Rédiger et corriger la note avec OpenAI (Priority: P1)

L'utilisateur exprime des idées, une intention ou une correction. Le service
d'interprétation transforme les passages terminés en modifications fidèles,
ordonnées et visibles, sans changer les règles d'édition existantes.

**Why this priority**: remplacer uniquement la transcription laisserait les appels
de rédaction dépendants de Mistral et ne satisferait pas la demande.

**Independent Test**: fournir des passages transcrits de référence et une note
préparée, puis contrôler la rédaction, les corrections et les demandes de précision.

**Acceptance Scenarios**:

1. **Given** une note vide et un passage final, **When** l'utilisateur demande
   une note structurée, **Then** l'interprétation utilise `gpt-6-luna` chez OpenAI,
   respecte les faits et la langue demandée et n'ajoute pas d'information inventée.
2. **Given** une note avec une intention persistante, **When** l'utilisateur
   précise le ton, ajoute un point ou corrige un passage en français ou en anglais,
   **Then** seules les modifications pertinentes sont appliquées et l'intention
   reste cohérente avec la demande.
3. **Given** une instruction réellement ambiguë, **When** l'interprétation ne
   permet pas de choisir une modification fiable, **Then** l'utilisateur reçoit
   une demande de précision plutôt qu'une modification arbitraire.
4. **Given** une réponse invalide, incompatible avec la note courante ou provenant
   d'une session périmée, **When** elle est reçue, **Then** elle est refusée,
   l'incident est visible sans contenu sensible et la note reste intacte.
5. **Given** le modèle demandé absent ou inaccessible, **When** un traitement
   en a besoin, **Then** l'indisponibilité est explicite et aucun autre modèle
   ni fournisseur n'est choisi silencieusement.

---

### User Story 3 - Passer à OpenAI sans perdre sa configuration utile (Priority: P1)

L'utilisateur retrouve ses notes et ses préférences utiles, renseigne une clé
OpenAI dans le parcours sécurisé de l'application et sait quels services traitent
ses données.

**Why this priority**: une migration doit fonctionner pour une installation
existante sans confondre les clés des deux fournisseurs ni perdre une note.

**Independent Test**: démarrer depuis une configuration Mistral existante, puis
configurer OpenAI, redémarrer et vérifier les notes, les modèles et les secrets.

**Acceptance Scenarios**:

1. **Given** une nouvelle installation, **When** l'utilisateur consulte les
   préférences, **Then** OpenAI et les deux modèles demandés sont les valeurs
   par défaut et la clé requise est clairement identifiée comme une clé OpenAI.
2. **Given** des préférences Mistral et une dernière note existantes, **When**
   l'application migre la configuration, **Then** les deux sélections de service
   deviennent celles demandées, le chemin de note et les autres préférences
   compatibles sont conservés et la transition est signalée.
3. **Given** une ancienne clé Mistral, **When** l'utilisateur démarre le vocal,
   **Then** cette clé n'est ni réutilisée comme clé OpenAI ni envoyée à OpenAI ;
   si la clé OpenAI manque, le vocal est refusé avec un message explicite.
4. **Given** une clé OpenAI ajoutée, remplacée ou supprimée depuis l'application,
   **When** l'utilisateur redémarre, **Then** son état reflète le trousseau
   sécurisé sans exposer la valeur dans la note, les préférences ou les diagnostics.
5. **Given** une note existante avec une intention, **When** l'utilisateur l'ouvre,
   l'édite manuellement, la sauvegarde et la rouvre après migration, **Then**
   le corps et l'intention sont conservés conformément au format existant.

---

### User Story 4 - Comprendre une interruption sans perdre la note (Priority: P2)

L'utilisateur distingue un accès refusé, un quota épuisé, un problème réseau ou un
traitement invalide. Il peut conserver et éditer sa note même si le vocal échoue.

**Why this priority**: le changement de fournisseur ne doit ni masquer les
restrictions de l'entreprise ni transformer une erreur distante en perte de données.

**Independent Test**: provoquer des erreurs contrôlées pour chaque service et
vérifier le message, l'arrêt du vocal et l'intégrité de la dernière note.

**Acceptance Scenarios**:

1. **Given** un refus d'authentification, de droit d'accès ou de quota, **When**
   un service échoue, **Then** l'utilisateur voit une catégorie compréhensible,
   aucun succès fictif n'est affiché et aucune relance automatique ne répète l'envoi.
2. **Given** une interruption réseau, un certificat refusé ou un blocage du flux,
   **When** la session ne peut continuer, **Then** le mode vocal s'arrête selon
   le comportement existant, la note reste disponible et la sécurité de connexion
   n'est pas affaiblie.
3. **Given** une fermeture ou une mise en veille pendant un traitement, **When**
   le traitement se termine tardivement, **Then** il ne modifie pas la note ;
   un échec de sauvegarde ne permet pas une fermeture silencieuse avec perte.

### Edge Cases

- Préférences anciennes avec modèles Mistral personnalisés : remplacement annoncé
  par les modèles demandés, sans transfert des anciens identifiants vers OpenAI.
- Configuration déjà OpenAI : conservation des choix compatibles, sans migration
  répétée à chaque démarrage.
- Trousseau verrouillé, clé absente ou supprimée : édition manuelle disponible,
  vocal indisponible avec explication ; aucune lecture automatique d'un fichier de clé.
- Modèle de langage introuvable ou non autorisé : blocage explicite du traitement
  concerné, sans substitution par un modèle au nom approchant.
- Silence ou passage final vide : aucune rédaction et aucune duplication de note.
- Résultats provisoires répétés, final dupliqué ou réception hors ordre :
  aucun passage appliqué deux fois, aucun écrasement par un résultat périmé.
- Réponse de rédaction tronquée ou non conforme : aucune modification partielle.
- Alternance français/anglais au sein d'une même intervention : qualité à mesurer
  séparément ; aucune garantie de mélange bilingue parfait n'est déduite des essais
  monolingues.
- Accès fonctionnel sous Linux mais filtré sous Windows : résultats distincts ;
  aucune prise en charge d'une plateforme n'est déclarée sur la preuve d'une autre.
- Audio ou contexte déjà transmis lors d'une interruption : l'arrêt local ne
  prétend pas annuler un traitement distant ni sa facturation éventuelle.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Le système DOIT utiliser OpenAI pour la transcription et pour
  l'interprétation, avec respectivement `gpt-realtime-whisper` et `gpt-6-luna`
  comme sélections par défaut après migration et sur une installation neuve.
- **FR-002**: Le système DOIT maintenir une dictée audio en direct, avec
  transcription progressive et fin de passage sans intervention manuelle
  systématique ; la transcription d'un fichier après enregistrement ne suffit pas.
- **FR-003**: La transcription et l'interprétation DOIVENT prendre en charge le
  français et l'anglais ; le service ne DOIT pas imposer une traduction. La langue
  du contenu est conservée sauf demande explicite de l'utilisateur. La langue de
  transcription DOIT être détectée automatiquement sans réglage obligatoire ;
  l'utilisateur DOIT pouvoir changer de langue entre interventions sans arrêter
  la session.
- **FR-004**: Seuls les passages finaux valides de la session active DOIVENT
  déclencher l'interprétation ; chaque passage ne DOIT produire qu'une application
  ordonnée de ses modifications, sans duplication ni résultat périmé accepté.
- **FR-005**: L'interprétation DOIT conserver les comportements existants de
  rédaction progressive, correction, intention persistante et clarification,
  sans invention de faits ni confirmation systématique des modifications ordinaires.
- **FR-006**: Toute proposition de modification DOIT être contrôlée avant
  application ; une réponse invalide ou incompatible DOIT laisser la note et
  l'intention inchangées et produire un incident visible.
- **FR-007**: Les contrôles de pause, arrêt, changement de session, verrouillage,
  veille et fermeture DOIVENT continuer à invalider les traitements en cours ;
  aucun résultat tardif ne DOIT modifier la note.
- **FR-008**: Les préférences DOIVENT distinguer transcription et interprétation,
  afficher OpenAI et les modèles sélectionnés, conserver les réglages non sensibles
  compatibles et annoncer la migration d'une configuration Mistral.
- **FR-009**: La clé OpenAI DOIT être gérée par le trousseau sécurisé via les
  actions d'ajout, remplacement et suppression existantes adaptées à OpenAI.
  Les anciennes clés Mistral ne DOIVENT être ni réutilisées ni supprimées implicitement.
  Aucun fichier local de clé ne DOIT devenir une source automatique de l'application.
- **FR-010**: Le système NE DOIT effectuer aucun envoi à Mistral après migration,
  ni basculer automatiquement vers un autre fournisseur ou modèle en cas d'échec.
  Les anciennes sélections Mistral ne DOIVENT plus être proposées comme choix actifs.
- **FR-011**: L'utilisateur DOIT pouvoir identifier le service recevant l'audio,
  celui recevant le texte et le contexte utile, ainsi que les coûts et conditions
  de conservation connus ou explicitement inconnus, avant un essai réel.
- **FR-012**: Les erreurs de connexion, de certificat, d'authentification, de droit,
  de modèle, de quota, de délai et de réponse DOIVENT être visibles et distinguées
  lorsqu'elles sont identifiables, sans secret, audio, transcription ou contenu
  de note dans les diagnostics et sans réémission automatique.
- **FR-013**: Audio, transcription et contexte de session DOIVENT rester
  temporaires localement. Le format des notes, l'intention enregistrée, la
  sauvegarde, l'édition manuelle et son annulation DOIVENT rester inchangés.
- **FR-014**: Les essais réseau DOIVENT conserver la validation des certificats
  et les règles réseau de l'entreprise ; un refus ne DOIT déclencher aucun
  contournement. Les recettes Linux GNOME/Wayland et Windows 11 DOIVENT être
  documentées séparément, y compris les blocages.
- **FR-015**: La disponibilité de l'identifiant exact `gpt-6-luna`, son accès
  depuis le projet OpenAI et son aptitude aux modifications attendues DOIVENT
  être confirmés avant d'annoncer l'intégration comme utilisable. Si cette
  dépendance échoue, un changement de modèle exige une décision explicite
  de l'utilisateur.
- **FR-016**: Les indications courantes de l'interface et de la documentation
  d'utilisation DOIVENT décrire OpenAI sans laisser croire que Mistral est
  encore actif ; les anciennes preuves de validation DOIVENT rester identifiées
  comme historiques, sans être présentées comme des résultats OpenAI.

### Key Entities

- **Sélection de service** : fournisseur et modèle choisis séparément pour la
  transcription et pour l'interprétation, conservés sans secret.
- **Identifiant d'accès OpenAI** : secret détenu par le trousseau ; l'application
  expose seulement sa présence et les actions autorisées.
- **Session vocale** : période d'écoute explicitement démarrée, avec état actif
  ou invalidé, passages provisoires et passages finaux temporaires.
- **Proposition d'édition** : modifications ou demande de précision portant sur
  une note et une intention à un état donné, acceptées seulement après contrôle.
- **Note et intention** : contenu local durable conservant son format et ses
  règles de sauvegarde indépendamment du fournisseur de traitement.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Sur un corpus fixé avant recette de 10 passages français et
  10 passages anglais, le taux d'erreur de mots est inférieur ou égal à 10 %
  dans chaque langue ; aucun nom, nombre ou négation critique du corpus ne
  change de sens. Les conditions audio et les références sont conservées
  comme données de recette non confidentielles. Les 20 passages sont dictés
  dans une même session, en alternant les langues entre passages, sans sélection
  manuelle de langue ni arrêt du vocal.
- **SC-002**: Pour au moins 18 de ces 20 passages dictés en direct, un premier
  texte provisoire est visible au plus tard 2 secondes après le début de la parole.
  Pour au moins 18 passages, la mise à jour de la note est visible au plus tard
  5 secondes après la fin de la parole, dans les conditions réseau documentées.
- **SC-003**: Un corpus fixé de 12 demandes, 6 par langue, couvrant rédaction,
  correction, intention et ambiguïté produit 12 résultats conformes à leurs
  attentes explicites, sans fait ajouté ni perte de contenu non ciblé.
- **SC-004**: Sur 10 séquences d'interruption couvrant pause, arrêt, verrouillage,
  veille et fermeture, aucune réponse retardée ne modifie la note ou l'intention ;
  aucun passage n'est appliqué deux fois.
- **SC-005**: Dans les recettes d'installation neuve et de migration, 100 % des
  notes, intentions et chemins de note de référence restent récupérables après
  sauvegarde et réouverture ; aucune ancienne clé n'est réutilisée implicitement.
- **SC-006**: Pour chaque catégorie d'échec prévue, l'utilisateur voit une
  explication et conserve l'accès à sa note ; aucune modification partielle,
  réémission automatique ou bascule de fournisseur n'est observée.
- **SC-007**: Sur l'ensemble des essais, aucun secret ni contenu temporaire
  n'apparaît dans les notes sauvegardées, préférences ou diagnostics en dehors du
  contenu et de l'intention volontairement rédigés ; aucun envoi à l'ancien
  fournisseur n'est constaté.
- **SC-008**: Une recette complète est publiée pour chacun des deux environnements
  obligatoires ; un environnement bloqué est signalé comme non validé, et non
  comme compatible par extrapolation.

## Assumptions

- La demande porte sur le remplacement effectif de Mistral, non sur l'ajout d'un
  sélecteur multifournisseur, une refonte de l'interface ou de nouvelles commandes.
  Aucun nettoyage de tout le code historique n'est exigé s'il n'est plus actif.
- Les choix de modèles restent séparés et configurables comme auparavant ;
  les identifiants demandés sont les valeurs par défaut, sans repli automatique.
- Français et anglais signifient au minimum des interventions monolingues dans
  chacune des deux langues, détectées automatiquement avec alternance possible
  entre interventions dans une même session. Le mélange au sein d'une phrase
  est évalué séparément, sans élargir cette migration à un nouveau mode de traduction.
- Le compte dispose d'un projet de service autorisé et d'un quota adapté ;
  un abonnement à l'interface conversationnelle ne constitue pas cette preuve.
  Aucun essai facturable avec audio n'est autorisé par la seule rédaction de cette
  spécification.
- La session de transcription authentifiée précédemment ouverte constitue une
  preuve limitée de connectivité et de configuration, pas une recette audio,
  une garantie de quota, ni une validation de Windows.
- Les essais de planification autorisés ont utilisé trente secondes d'audio
  anglais synthétique au total et deux petites générations de contenu fictif.
  `gpt-realtime-whisper` a émis un premier texte avant la fin de la parole et
  `gpt-6-luna` a répondu aux deux contrats structurés réduits. Ces résultats
  n'attestent ni les seuils du corpus complet, ni le français, ni la prise
  en charge native des plateformes ; la recette reste obligatoire.
- La consultation authentifiée du catalogue a retourné `gpt-6-luna`, explicitement
  choisi par l'utilisateur. L'identifiant initial `gpt-6.1-luna` était absent et
  sa consultation retournait 404. La présence au catalogue ne prouve ni les droits
  de génération, ni le quota, ni l'aptitude aux sorties structurées.
- Les seuils de qualité et de délai ci-dessus sont des objectifs de recette,
  non des performances déjà mesurées. Les conditions réseau et matérielles
  doivent accompagner les résultats.
- Les principes de la constitution priment sur les mentions historiques
  contradictoires : pas d'annulation dédiée en vocal, annulation classique
  en manuel et protection des secrets par le trousseau.

# Contract: Fichiers de note et export

Statut : contrat cible de la fonctionnalité, non encore implémenté. Le même contrat s'applique sous Windows et Linux, sans variante de format par plateforme.

## Format de travail `.von`

Le fichier est du Markdown UTF-8 lisible sans Vocalnotes. Réutiliser exactement le namespace et le codec de `src-tauri/src/storage/note_document.rs`.

Quand l'intention n'est pas vide, le document comporte :

1. Le préfixe `<!-- vocalnotes:intention:v1` suivi de LF.
2. L'objet JSON de l'intention historique, avec les quatre chaînes `purpose`, `form`, `tone`, `context`.
3. Le délimiteur LF + `-->` + deux LF.
4. Le corps Markdown exact, sans normalisation de ses fins de ligne.

Exemple de fichier, uniquement illustratif :

````markdown
<!-- vocalnotes:intention:v1
{"purpose":"","form":"","tone":"","context":"Réunion de préparation"}
-->

# Compte rendu

Texte de la note.
````

Le contexte est clairement séparé du corps et consultable dans un éditeur texte ; un moteur Markdown ordinaire masque le commentaire. Les champs historiques non édités sont préservés.

### Règles d'encodage/décodage

- L'encodeur garde les échappements existants de `&`, `<` et `>` dans les chaînes JSON, notamment pour empêcher un contexte contenant `-->` de fermer l'enveloppe.
- Une intention vide peut être omise. Si le corps commence lui-même par le namespace réservé, encoder une enveloppe vide externe, comme le codec existant, pour conserver ce corps sans ambiguïté.
- À la lecture, un fichier sans namespace est un corps Markdown avec intention vide. Aucun format binaire `.von` n'est introduit.
- Namespace connu mais version inconnue, JSON invalide, champs inattendus, délimiteur absent, UTF-8 invalide ou limite dépassée : échec explicite, pas de réinterprétation silencieuse ni remplacement de la note en mémoire.
- Ne pas normaliser le Markdown ni retirer les blocs opaques, références, espaces, CRLF ou Unicode.
- Limite de fichier complet : 1 048 576 octets ; limite d'intention cumulée : 4096 octets avant encodage. Le codec conserve aussi son garde de 32 768 octets de métadonnées JSON encodées. Rejeter explicitement ce qui ne peut pas être représenté dans ces limites.
- Le fichier ne contient ni instructions temporaires, ni piles undo/redo, ni audio, ni historique de transcription, ni clé API.

## Lecture et sauvegarde des `.md` historiques

- Ouvrir prend en charge les `.md` simples et ceux munis de l'enveloppe historique, avec le même décodage strict.
- Un `.md` sans contexte défini reste `.md` quand Enregistrer est choisi : aucune conversion imposée, aucun sélecteur de destination redemandé pour cette seule raison.
- Si le contexte reste égal à celui chargé, conserver le chemin et préserver les éventuelles métadonnées déjà présentes.
- Si le contexte diffère de celui chargé, Enregistrer demande une destination `.von` complète ; le `.md` initial n'est pas modifié. Vider un contexte initialement non vide constitue une modification. Annuler un ajout de contexte jusqu'à l'état d'ouverture annule la nécessité de conversion.
- Enregistrer sous et la première sauvegarde d'une nouvelle note proposent `.von`. Il ne s'agit pas d'une migration automatique de tous les anciens fichiers.

## Export `.md`

Les octets du résultat sont exactement les octets UTF-8 du champ `source` capturé pour l'export.

- Aucun appel à l'encodeur de note ; aucun contexte, intention d'application, instruction ou autre enveloppe ajoutée.
- Aucun nettoyage sémantique du corps : si l'utilisateur a écrit un texte identique à son contexte ou un commentaire Markdown, il reste dans le contenu exporté.
- Le corps peut être vide ; le résultat est alors un fichier vide.
- L'export inclut les modifications du contenu acceptées mais non enregistrées. Il ne change ni destination, ni format, ni `savedState`, ni `dirty`, ni historique du document.
- Une destination représentant le document courant est refusée, y compris via casse différente lorsque le système la considère comme le même fichier, lien physique ou alias détecté par identité. Deux noms Linux différant seulement par la casse ne sont pas présumés identiques. Le contrôle par simple suffixe/chaîne ne suffit pas.
- Un export sur un autre fichier existant exige confirmation et contrôle de sa version. Une annulation ou un conflit conserve la note de travail inchangée.

## Choix de chemin et résultat d'écriture

- Extensions reconnues sans tenir compte de la casse : `.von`, `.md`. Ne jamais produire `note.von.von` ou `note.MD.md`.
- Sans extension, ajouter celle de l'action ; avec extension incompatible, expliquer et demander un choix valide sans renommer silencieusement.
- Les chemins proviennent des dialogues Rust propriétaires de `main`, pas d'une entrée textuelle libre exposée au renderer.
- Enregistrer sous adopte le nouveau chemin après résultat vérifié seulement. Un fichier préparé, une copie de secours ou une sauvegarde d'ancienne version n'est pas une nouvelle destination implicitement choisie.
- La confirmation de remplacement vise une version observée précise. Une modification externe ultérieure provoque un conflit ; elle n'est pas couverte par un accord général antérieur.
- Aucun écrasement direct par troncature. Une incertitude après installation conserve les versions utiles et un diagnostic ; ne pas tenter un rollback qui pourrait effacer un écrivain externe.
- Les adaptations natives préservent ces mêmes garanties : remplacement contrôlé Windows, échange Linux et installation sans écrasement pour une destination absente. Un refus de volume, de primitive ou de métadonnées protégées est expliqué ; aucune copie n'est adoptée automatiquement comme destination de travail.

## Échanges Windows et Linux

- Utiliser un unique codec UTF-8 : aucune conversion de Unicode, des espaces ou des fins de ligne du corps. L'enveloppe écrite conserve ses délimiteurs LF ; le corps peut conserver LF, CRLF ou un mélange identique à sa source.
- Le document transporte seulement l'intention et le corps ; il ne contient ni chemin Windows/Linux, ni identité disque, ni préférence, ni historique de sauvegarde. L'hôte récepteur choisit un chemin local et établit sa propre `DiskVersion`.
- La copie ou le transport de recette doit préserver les octets, sans filtre de fins de ligne Git ni transformation d'un éditeur externe. Une simple différence d'identité disque après transfert n'est pas une corruption du format.
- Pour SC-011, exercer 20 échanges Windows vers Linux et 20 Linux vers Windows. À chaque réception, ouvrir puis réenregistrer le `.von` et comparer les octets de `source` et les quatre chaînes d'intention avec l'état émetteur ; l'export doit être exactement `source` sans enveloppe ajoutée. Ne pas exiger une sérialisation JSON identique pour des espaces JSON externes non canoniques.
- Couvrir corps vide, contexte vide/non vide, accents/emoji, Unicode non normalisé, espaces finaux, LF/CRLF, corps commençant par le namespace, séquence `-->` dans le contexte, anciennes métadonnées et extensions `.VON`/`.MD`. Les noms de recette doivent pouvoir être créés sur les deux OS ; tester séparément la sensibilité à la casse sur chaque hôte.

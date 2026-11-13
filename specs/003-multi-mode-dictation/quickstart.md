# Guide de validation — Dictée multimode et reformulation

## Prérequis

- Node.js `>=22.23.2 <23`, npm, Rust `1.99.1` et les prérequis Tauri de la
  plateforme.
- Dépendances du dépôt restaurées à partir des manifests verrouillés.
- Pour un essai manuel vocal/LLM : clé OpenAI configurée via le parcours
  sécurisé existant et consentement explicite. Les tests automatisés utilisent
  des transports factices et ne doivent pas joindre un fournisseur.

## Vérifications automatisées

Depuis la racine du dépôt :

```powershell
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml
```

Résultats attendus :

- Tests UI : destinations, visibilité, changements différés, saisie manuelle,
  absence d'appels LLM implicites, état de chargement, erreur et résultat
  périmé.
- Tests Rust : capture de la cible au début de la parole, routage de la
  transcription finale, transition hors d'Instructions différée, persistance du
  contexte, validation du contrat strict, limites et rejet des résultats
  périmés.
- Aucun test automatisé n'effectue de requête réseau réelle.

## Scénarios de recette

1. **Routage** — Démarrer l'écoute explicitement. Dicter séparément en mode
   contenu et contexte ; vérifier que seul le champ ciblé change. Démarrer une
   phrase en mode contenu, changer de mode avant son résultat STT et vérifier
   qu'elle revient au contenu.
2. **Sortie d'Instructions** — Démarrer une dictée en mode Instructions puis
   demander un autre mode avant son final STT. Vérifier que le mode reste
   Instructions jusqu'à la finalisation et au routage, puis change et efface
   les instructions.
3. **Édition et persistance** — Modifier manuellement contexte et contenu dans
   les trois modes. Fermer puis rouvrir le fichier Markdown et vérifier la
   restauration du contexte et des styles titres/gras/italique ; vérifier que
   les instructions ne sont pas conservées.
4. **Déclenchement explicite** — Modifier le contenu, dicter, saisir du contexte
   et changer de mode sans utiliser Reformuler. Vérifier qu'aucune requête LLM
   n'est émise. Utiliser ensuite Reformuler et vérifier une seule requête avec
   contenu actuel et champs facultatifs non blancs.
5. **Application et protection** — Vérifier que seule la note est modifiée par
   une réponse valide. Modifier la note pendant une requête factice, puis
   résoudre la requête ; vérifier que sa réponse obsolète est rejetée.
6. **Échecs** — Simuler une erreur réseau, un refus, une sortie JSON invalide,
   une réponse vide, un dépassement de limites et un bloc opaque altéré.
   Vérifier que le contenu récent reste intact et que l'erreur est visible.
7. **Fidélité** — Utiliser des exemples Markdown synthétiques avec et sans
   instructions explicites d'ajout. Vérifier les changements de présentation
   et l'absence d'idées nouvelles dans les cas sans autorisation.

Les nombres d'essais et seuils d'acceptation sont définis dans
[spec.md — Success Criteria](./spec.md#success-criteria-mandatory). Ne pas
considérer un essai manuel de modèle comme preuve d'un seuil de qualité sans
exécuter la série complète.

# Migration OpenAI — bilan d'implémentation

**2026-11-11 — automatisation réalisée ; migration non déclarée livrable.**

Modèles approuvés : `gpt-realtime-whisper` (STT) et `gpt-6-luna` (Responses
ordinaire/priorité). Pas de repli, d'import de clé ou de contournement TLS.
`store:false` ne garantit pas la non-conservation fournisseur.

## Preuves

- [Contrôles automatisés](./openai-automated.md).
- [GNOME/Wayland : non exécutée](./openai-gnome.md).
- [Windows 11 : non exécutée](./openai-windows.md).
- [Tâches et état réel](../../specs/002-remplacer-services-openai/tasks.md).

## Couverture fonctionnelle

| Exigences | Implémentation / preuve disponible | Limite restante |
|---|---|---|
| FR-001, FR-002 | Nouveaux adaptateurs, stream unique, conversion, tests STT | Service et microphone intégrés non recettés |
| FR-003 | Détection automatique sans langue/prompt forcés, consignes bilingues | Qualité réelle FR/EN non mesurée |
| FR-004, FR-006 | Final corrélé, tickets/révision, application atomique, tests | Acceptation du schéma complet par le service non confirmée |
| FR-005 | Gardes et corpus de rédaction simulé, intention persistante | 12 demandes réelles non évaluées |
| FR-007 | Barrières et dix interruptions simulées | Comportement OS réel absent |
| FR-008 | Migration atomique/idempotente, notification et préférences | Recette de profil natif absente |
| FR-009 | Compte OpenAI isolé, IPC sans valeur secrète | Trousseaux réels non recettés |
| FR-010 | Transports Mistral natifs désactivés et testés | Observation réseau native non faite |
| FR-011 | Consentement/documentation, coûts/conservation inconnus explicites | Conditions du projet et coût de recette à vérifier |
| FR-012 | Catégories fermées, erreurs/annulations simulées | TLS/réseau natifs non exercés |
| FR-013 | Pas de spool ; régressions stockage/édition/annulation | Vérification native à compléter |
| FR-014 | TLS normal ; rapports distincts avec blocages | Deux recettes obligatoires manquantes |
| FR-015 | Sonde de planification : deux Responses réduites acceptées | Schéma complet et aptitude métier réelle encore à recetter |
| FR-016 | UI/guide OpenAI, preuves antérieures signalées historiques | Nouvelle recette visuelle à réaliser |

## Critères de réussite

| Critère | Résultat réel disponible | Décision |
|---|---|---|
| SC-001 | Corpus fixé 20 passages, 10/langue ; aucun WER mesuré | NON VALIDÉ |
| SC-002 | Événements de peinture corrélés testés ; pas de vrais repères parole/rendu | NON VALIDÉ |
| SC-003 | 12 attentes fixées, 6/langue ; sorties réelles non observées | NON VALIDÉ |
| SC-004 | 10 interruptions simulées sans effet tardif ; OS non recetté | NON VALIDÉ en natif |
| SC-005 | Migration/stockage simulés passants ; récupération native non mesurée | NON VALIDÉ en natif |
| SC-006 | Matrice automatisée passante ; réseau/trousseau/OS natifs non recettés | NON VALIDÉ en natif |
| SC-007 | Contrats/diagnostics vérifiés hors fournisseur ; pas de capture réseau native | NON VALIDÉ en natif |
| SC-008 | Deux rapports publiés avec statut non exécuté | NON VALIDÉ |

63 tests UI et build de production passent. Voir le rapport automatisé pour
les nombres Rust finaux et les contrôles d'exemples. Aucun nouvel appel facturé
pendant l'implémentation, coût de future recette inconnu. Les mesures de
planification (dont 0,799 s sans microphone/UI) ne satisfont pas SC-002.

## Reste à faire

T042 : compléter/calibrer les repères réels de parole et visibilité.
T043 : fermer le gate après instrumentation complète, pas seulement après tests.
T044/T045 : nouvel accord explicite, puis recettes sur les deux plateformes.
T046 : finaliser ce bilan avec leurs observations, coûts et décision.
Ces tâches restent ouvertes ; aucun modèle ni seuil n'est assoupli.

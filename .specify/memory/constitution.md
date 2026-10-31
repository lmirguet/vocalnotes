<!--
Rapport d'impact : 2.0.0 → 3.0.0.
Révision majeure : II. Maîtrise par l'utilisateur → II. Deux modes d'édition.
Retrait de l'annulation en mode vocal et de l'interdiction de modifier un contenu manuel.
Suppression du principe VI et des sections Place des spécifications et Gouvernance.
Aucune section ajoutée ; modèles de spécification, plan et tâches inchangés.
Suivi : aligner le cahier des charges sur les nouvelles règles des deux modes,
notamment les exigences d'annulation et de protection de l'édition manuelle.
Version : 3.0.0 ; adoption : 2026-10-30 ; dernière modification : 2026-10-30.
-->

# Bloc-notes vocal intelligent — Constitution

## Principes essentiels

### I. Fidélité aux propos

L'application transforme la parole naturelle en notes structurées, sans inventer
de faits ni altérer le sens. Elle distingue le contenu à rédiger des instructions
d'édition et demande une précision lorsqu'une ambiguïté empêche d'agir correctement.

### II. Deux modes d'édition

Le mode vocal est le mode principal et la raison d'être de l'application :
toutes les actions y sont exprimées à la voix. Les modifications ordinaires s'appliquent sans confirmation
systématique.

Après l'arrêt de l'interface vocale, le mode manuel offre un éditeur de texte
classique avec une commande d'annulation.

### III. Propriété et intégrité des notes

Les notes sont des fichiers Markdown locaux, utilisables sans l'application.
La sauvegarde et l'édition préservent leur contenu ; aucun conflit ni échec
ne doit entraîner une perte ou une dégradation silencieuse.

### IV. Confidentialité et sécurité

L'audio, la transcription et le contexte de session restent temporaires ;
seule la note est conservée comme contenu utilisateur. Les clés API sont protégées
par le trousseau sécurisé du système. Les envois aux services externes sont limités
au nécessaire et transparents, sans bascule vers un fournisseur non choisi.

### V. Simplicité et indépendance technique

L'architecture reste simple et sobre. La transcription et l'interprétation
disposent d'interfaces distinctes, indépendantes des fournisseurs.
Les choix techniques sont justifiés par les besoins et les essais,
sans abstraction ni dépendance superflue.

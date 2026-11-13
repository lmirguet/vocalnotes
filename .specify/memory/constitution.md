<!--
Rapport d'impact : 4.0.0 → 5.0.0.
Révision majeure : II. Dictée multimode et contrôle explicite.
Le mode sélectionné gouverne le cycle de capture ; l'édition manuelle reste
possible dans chaque mode et les appels LLM sont déclenchés explicitement.
Les principes de fidélité, d'intégrité des notes et de confidentialité sont conservés.
Version : 4.0.0 ; adoption : 2026-11-13 ; dernière modification : 2026-11-13.
-->

# Bloc-notes vocal intelligent — Constitution

## Principes essentiels

### I. Fidélité aux propos

L'application transforme la parole naturelle en notes structurées, sans inventer
de faits ni altérer le sens. Elle distingue le contenu à rédiger des instructions
d'édition et demande une précision lorsqu'une ambiguïté empêche d'agir correctement.

### II. Dictée multimode et contrôle explicite

La dictée possède trois destinations distinctes : contenu de la note, contexte
et instructions temporaires. Le mode sélectionné détermine la destination et
le cycle de capture : sa sélection démarre l'écoute pour cette destination, et
sa fin arrête l'écoute. L'interface n'ajoute pas de commande vocale distincte
de permission, démarrage, reprise, pause ou arrêt ; elle rend l'état réel et la
cible de l'écoute visibles, et conserve la divulgation des envois audio.

Chaque segment de transcription reste dirigé selon la destination active au
début de la parole. L'utilisateur peut modifier manuellement le contenu et tout
champ visible dans chacun des modes, y compris pendant l'écoute, sans affaiblir
les protections de révision et d'intégrité. Le modèle de langage n'interprète ni
la dictée ni les changements de mode automatiquement ; il n'est appelé que par
une action explicite de l'utilisateur. Le contexte est conservé avec sa note ;
les instructions temporaires sont effacées à la sortie de leur mode, après le
routage des segments déjà en cours.

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

# Prototype GNOME/Wayland — NON EXÉCUTÉ

2026-10-31, B01-R3. **Aucun GO T007.**

Hôte de compilation : Ubuntu 24.04 WSL2, pas un desktop GNOME établi.
CPAL/ALSA compilé, 32 tests Rust/22 JS verts : ni micro, ni arrière-plan,
ni Secret Service ni GNOME certifiés. Le sondage natif de surveillance
sans micro/clé/API retourne code 2.

| Gate | Recette GNOME réelle |
|---|---|
| G02 modèles/WSS/schema/compte | NON EXÉCUTÉ |
| G03 micro/normalisation/rotation sans perte | NON EXÉCUTÉ |
| G04 arrière-plan/pause/affichages internes | NON EXÉCUTÉ |
| G05 lock/sleep/perte abonnement/fermeture | NON EXÉCUTÉ |
| G06 Secret Service/champ masqué/purge | NON EXÉCUTÉ |
| G07 Markdown/opaque/échecs disque | NON EXÉCUTÉ |
| G08 fidélité/services synthétiques | NON EXÉCUTÉ |
| G09 installation/RPM/ressources/délais | NON EXÉCUTÉ |

À fournir : desktop interactif GNOME/Wayland, matériel/versions, même arbre
et dépendances verrouillées. Suivre A–K du quickstart puis renseigner rapports
et mesures. Clé via champ masqué/trousseau seulement.
Avant micro/API : accord contextualisé, synthétique non sensible,
coût/conservation inconnus explicitement acceptés. Aucun lancement ici.
Un essai absent reste NON EXÉCUTÉ ; NO-GO → correction avant stories.

# US1 — mesures

2026-10-31, B12-US1. Recette native T031 en attente. Aucune mesure de la version
T026 n’est attribuée à T028–T030.

À relever avec empreinte du binaire, plateforme/micro et échantillonnage :
démarrage, ressources au repos/pendant dictée, fin VAD→final, final→commit,
commande pause→fermeture capture. Le rendu visible exige une observation
séparée. Coût inconnu sans facture/tarif du compte. Aucun seuil de performance
ni précision générale ne découle des tests hors ligne.

## Premier relevé du binaire avec repère temporaire

Binaire `4be0527dfe2ba7b6d1140535dbcc7947bffc17c6e875822b0713857c3944c5f7`,
GNOME/Wayland, micro NoMachine ; journal `/tmp/vocalnotes-locate-highlight-native.log`.
Sur l’essai de verrouillage confirmé par l’utilisateur : initialisation→écoute
56 ms (1793482546851→1793482546907) ; callback OS→fermeture native du micro
4 ms (1793482553160→1793482553164). Pause vocale antérieure : décision de priorité
→fermeture native 4 ms (1793482495046→1793482495050).
Ces intervalles console ne mesurent ni événement physique→arrêt, ni parole→pause,
ni lancement à froid→rendu. Un échantillon par intervalle, aucun seuil déduit.

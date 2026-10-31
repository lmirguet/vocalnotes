# T026 — mesures des fondations

2026-10-31. GNOME/Wayland/Fedora 44, Ryzen 7 5800H, NoMachine,
WebKitGTK 2.54.2. Recette locale terminée ; aucune cible de performance ratifiée.

| Mesure | Résultat actuel | Limite |
| --- | --- | --- |
| Démarrage à froid jusqu’au rendu | NON MESURÉ | Premier message console différent du rendu visible |
| Ressources au repos | Relevé principal ci-dessous | RSS partagée, pas mémoire exclusive |
| Ressources pendant dictée | NON MESURÉ | À distinguer du repos |
| Fin VAD → final STT | 429–883 ms, trois passages | Ne mesure pas la fin réelle de la parole |
| Final STT → commit | 1 318–1 671 ms, trois passages | Inclut orchestration et LLM ; pas rendu UI |
| Commande pause → fermeture micro | 4 ms, un échantillon | Ne mesure pas parole → pause |
| Coût STT/LLM effectif | INCONNU | Pas de facture ni tarif de compte vérifié |
| Windows, veille réelle, paquet | EXCLUS | Aucun résultat déduit de GNOME |

Les éventuels échantillons seront attribués au binaire et aux timestamps de cette
recette. Pas de transcription, audio ou contenu de note dans les mesures.

## Première dictée et pause — un échantillon

Binaire `403f9fc7…4ee`, journal `/tmp/vocalnotes-foundations-native.log`.
Timestamps console en millisecondes :

| Intervalle | Valeur | Début → fin |
| --- | --- | --- |
| Début initialisation → écoute | 86 ms | 1793478136643 → 1793478136729 |
| Fin VAD → final STT non vide | 781 ms | 1793478143769 → 1793478144550 |
| Final STT → commit | 1 671 ms | 1793478144550 → 1793478146221 |
| Fin VAD → commit | 2 452 ms | 1793478143769 → 1793478146221 |
| Commande bouton Pause → fermeture native micro | 4 ms | 1793478151898 → 1793478151902 |

Un échantillon seulement. Dictée avec erreur de reconnaissance signalée par
l’utilisateur ; ces délais ne constituent pas une validation de fidélité. Pas
de mesure parole réelle→affichage, délai vocal→pause ou temps fournisseur isolé.

## Ressources — un relevé au repos après conflit

Processus principal PID 1407193, micro arrêté selon dernier état confirmé,
temps écoulé depuis lancement 11 min 46 s : RSS 187 556 Kio (environ 183 Mio).
CPU `%CPU` de `ps` : 5,1 %, moyenne cumulée depuis le lancement, **pas** mesure
instantanée au repos ni charge isolée de dictée. Pas de somme WebKit ni de mesure
PSS/mémoire exclusive. Un seul échantillon ; aucun plafond ou objectif déduit.

## Trois passages non vides corrélés

| Échantillon | Fin VAD → final | Final → commit | Fin VAD → commit |
| --- | --- | --- | --- |
| 1 | 781 ms | 1671 ms | 2452 ms |
| 2 | 883 ms | 1318 ms | 2201 ms |
| 3 | 429 ms | 1321 ms | 1750 ms |

Corrélation vérifiée dans chaque session d’écoute (le final de pause vocale
interrompu est exclu, ainsi que les passages vides). Troisième passage :
1793478405385 → 1793478405814 → 1793478407135. n=3, sans percentile de production ni seuil
ratifié. Plusieurs dictées ont une réserve de fidélité ; délais et qualité
restent deux observations distinctes.

Fermeture finale : processus absent après confirmation utilisateur, session
outil de lancement terminée avec code 0. Modèle configuré rétabli à Voxtral
Realtime 3 ; LLM `mistral-large-latest`. Modèle sélectionné et modèle servi ne
sont pas assimilés : les journaux de cette recette ne consignent pas les noms
servis, qui restent consultables dans la fenêtre avant sa fermeture.

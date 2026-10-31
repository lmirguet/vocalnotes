# Mesures du prototype — GNOME/Wayland

## Relevé courant D6 — 2026-10-31

Binaire `5b441b0b…f321`, Fedora44 GNOME50.6 Wayland, microphone NoMachine,
entrée 44 100 Hz stéréo, modèle STT/LLM intégrés. Session26421, cinq modifications
appliquées et recette utilisateur concluant. Quatre couples de timestamps
complets sont transcrits ci-dessous ; ce sous-échantillon ne décrit pas les
cinq passages ni un percentile de production.

| Passage observé | Fin VAD→final STT | Final STT→commit | Fin VAD→commit |
|---|---:|---:|---:|
| 120 octets, contenu masqué | 267 ms | 1 483 ms | 1 750 ms |
| 67 octets, contenu masqué | 274 ms | 1 677 ms | 1 951 ms |
| 49 octets, contenu masqué | 283 ms | 1 991 ms | 2 274 ms |
| 44 octets, contenu masqué | 220 ms | 2 637 ms | 2 857 ms |

Les fins VAD sont les messages console avant flush/end, pas l'instant réel de
fin de parole. Final→commit inclut appel LLM et application native ; ni temps
LLM isolé ni affichage webview. Dernière commande pause→fermeture native du
micro : 3 ms (1793443605599→1793443605602), pas délai depuis les mots prononcés.

Ressources observées à 10:51:11 UTC sur 5,037 s, dernière situation connue en
pause, aucune capture lancée par la sonde : RSS processus principal 188 572 Kio
(environ 184 Mio) ; somme RSS des trois processus applicatif/WebKit 457 980 Kio
(environ 447 Mio) ; CPU sur l’intervalle 1,39 % d’un cœur. La somme RSS compte
plusieurs fois les pages partagées ; ce n’est ni la mémoire physique exclusive
ni la charge pendant une dictée. Un seul échantillon, pas de maximum garanti.

Démarrage à froid jusqu’au rendu, délai parole→affichage et charge CPU vocale :
NON MESURÉS. Aucun seuil ratifié, SLA ou facture déduit. Coût effectif inconnu,
usage de tokens facturés non relevé. Le prix public générique ne suffit pas à
facturer un alias mutable ou à connaître les conditions particulières du compte.


## D3, 2026-10-31 — observation unique de dictée réussie

Fedora44 GNOME50.6/Wayland, binaire64db9f56…a056, source NoMachine choisie,
entrée native44100Hz/stéréo. Timestamps console en ms ; un seul passage
non vide, aucune distribution statistique ni objectif de performance revendiqué.

| Intervalle observé | Valeur |
|---|---:|
| Fin VAD/flush → transcription.done | 260 ms |
| Final STT → commit natif | 1 247 ms |
| Fin VAD/flush → commit natif | 1 507 ms |

Le second intervalle inclut interprétation et application ; ce n’est pas une
mesure isolée du temps fournisseur. Latence parole réelle → affichage non mesurée.
Prix, conservation et coût effectif inconnus. Audio non enregistré, transcription
non consignée. Arrêt explicite et flux fermé observés. Pas de nouvelle mesure
Windows ni certification multiplateforme.

## Historique B01-R3 — runtime NON MESURÉ

2026-10-31, B01-R3. Aucun seuil atteint, performance ou facture annoncés.

| Mesure | GNOME/Wayland | Windows 11 |
|---|---|---|
| Démarrage/installation | NON MESURÉ | NON MESURÉ |
| CPU/RAM/matériel/micro | NON RELEVÉ | NON RELEVÉ |
| Parole→STT→LLM→commit/bout en bout | NON MESURÉ | NON MESURÉ |
| Flush→done/rotation/file suivante | NON MESURÉ | NON MESURÉ |
| Pause naturelle/faux arrêts | NON MESURÉ | NON MESURÉ |
| Modèles réellement servis | INCONNUS | INCONNUS |
| Tarif/conservation fournisseur | INCONNUS | INCONNUS |

Compteurs en mémoire raccordés dans la fenêtre : tentatives STT/LLM/priorité,
ms audio envoyées, modèles servis, derniers délais flush→done/LLM/priorité/commit.
Délai initial : **non mesuré**, pas zéro. Une tentative ne prouve pas un service
facturé ; pas de tarifs inventés. Relever refus/erreurs aussi.

Ce lot : **0 appel Mistral, 0 micro démarré**, aucun coût d'essai engagé.
22 tests JS/32 Rust et builds ne prouvent pas les performances desktop.
750 ms coalescence, 810 ms VAD et limites mémoire/passage = paramètres
d'ingénierie à calibrer, **pas résultats ni SLA**.

Prochain relevé : mêmes scénarios synthétiques et binaire/version sur les deux
cibles, accords micro/API/coût contextualisés, aucun sensible, étapes et conditions
matérielles datées. Aucun GO T007 présent.

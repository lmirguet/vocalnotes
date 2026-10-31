# Mesures du prototype — runtime NON MESURÉ

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

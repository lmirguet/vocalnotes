# Recette OpenAI — Windows 11

**Statut : NON EXÉCUTÉE / NON VALIDÉE — 2026-11-11.**

L'implémentation est réalisée sur Linux. Aucun poste Windows 11, Credential
Manager, microphone, WebView2 ou cycle verrouillage/veille Windows n'a été testé.
Les contrôles Linux et le type-check ne qualifient pas cette plateforme.

## Blocages et mesures absentes

- Machine Windows 11 et configuration matérielle/réseau non disponibles ici.
- Nouvelle autorisation microphone, envois OpenAI et coûts non obtenue.
- SC-001 à SC-008 : **non validés sur Windows** ; aucun WER, comptage
  18/20, résultat 12/12, récupération 100 % ou mesure de coût.
- Référence synchronisée de parole/rendu non calibrée.
- Effet actuel de Zscaler sur le binaire et les certificats Windows : inconnu.

Modèles prévus : `gpt-realtime-whisper` et `gpt-6-luna`, sans substitution.
Coût futur inconnu ; budget de planification déjà consommé.
Tout refus réseau/certificat doit rester un blocage explicite, sans contournement.
Suivre le [guide](../../specs/002-remplacer-services-openai/quickstart.md)
après autorisation et publier les observations réelles. T045 reste ouverte,
aucun GO.

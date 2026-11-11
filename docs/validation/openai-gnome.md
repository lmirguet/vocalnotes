# Recette OpenAI — GNOME/Wayland

**Statut : NON EXÉCUTÉE / NON VALIDÉE — 2026-11-11.**

Sources migrées, contrôles automatisés décrits dans
[le rapport](./openai-automated.md). Aucun binaire natif lancé ni microphone
activé pour cette migration. La présence de variables d'affichage/D-Bus ne prouve
pas une session GNOME/Wayland utilisable.

## Blocages et mesures absentes

- Nouvelle autorisation microphone, envois OpenAI et coûts non obtenue.
- Budget de planification épuisé ; aucun essai facturable supplémentaire réalisé.
- Référence synchronisée des vrais début/fin de parole et du rendu non calibrée.
- OS/matériel/micro/réseau/binaire de recette non qualifiés.
- WER FR/EN, éléments critiques, latences 18/20, rédaction 12/12 et dix
  interruptions natives : **non mesurés**.
- Migration et trousseau réels, sauvegarde/réouverture, confidentialité et absence
  de trafic Mistral : **non recettés sur cette plateforme**.

Modèles prévus : `gpt-realtime-whisper` et `gpt-6-luna`, sans substitution.
Tarifs/coût de la recette : inconnus, pas présumés nuls.
Suivre le [guide](../../specs/002-remplacer-services-openai/quickstart.md)
après autorisation. Ne pas reprendre un résultat historique Mistral ou un test
simulé comme preuve. T044 reste ouverte, aucun GO.

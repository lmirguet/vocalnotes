# Runtime validation — B02-W01, 2026-10-31

**Cible :** Windows natif, copie
`C:\Users\lmirguet\Development\vocalnotes-native-B02-W01`.
**Verdict : NEEDS_SIGNOFF**, pas GO T006/T007.

| Étape | Résultat | Code | Preuve / limite |
|---|---|---:|---|
| Tests frontend Windows | PASS | 0 | 22/22 Vitest, 3 fichiers réutilisés |
| Typecheck/build web Windows | PASS | 0 | npm run build |
| fmt/test/check/build MSVC | PASS | 0 | 44/44 Rust, assets embarqués |
| Initialisation WTS | PASS | 0 | probe_lifecycle, pas événements lock/suspend |
| Démarrage Windows | PASS | — | PID 25360, HWND 396550, titre Bloc-notes vocal, répond, WebView2 enfant 9376 |
| Recette native visuelle / A–K | NON EXÉCUTÉ | — | Gestes utilisateur, WASAPI, keyring, Mistral et cycle OS attendus |
| Source Linux finale | PASS | 0 | 44 Rust/22 JS, fmt/check/build natif et frontend |
| GNOME/Wayland | NON EXÉCUTÉ | — | Différé, non bloquant pour développement |

## Stratégie appliquée

Desktop Tauri : tests déterministes Rust/Vitest existants réutilisés,
compilation/link MSVC réelle, démarrage WebView2 et recette interactive native
selon le plan/quickstart. Pas serveur/DB/Docker ; pas de remplacement de la
recette native par Playwright/navigateur ou jsdom. Tests composant/contrat :
**pas E2E**, aucun appel fournisseur ni microphone/trousseau par l'agent.
Code de sortie du processus UI encore actif non disponible ; readiness
vérifiée par HWND/réponse/processus, contenu visuel non certifié.

## Réutilisation et correctifs

`tests/scaffold.test.ts`, `src/ui/Startup.test.ts`,
`src/ui/Prototype.test.ts` et suites Rust réutilisés.
Modification minimale du seul scaffold : comparaison envDir par chemin
absolu natif (`fileURLToPath`/`resolve`) plutôt qu'un `/` Linux.
Premier Windows : 21/22, code 1 ; correction source et copie, puis 22/22
code 0. Aucun abaissement du contrôle de confinement.
Premier WaitForInputIdle pendant initialisation : erreur, puis vérification
durable HWND/titre/réponse/WebView2 PASS ; pas contournement UI.

## Écarts et preuves détaillées

Node Windows 25.8.2/npm 11.8.0 hors plage Node 22, EBADENGINE non résolu.
Rustup a auto-installé 1.99.1 pendant le premier sondage : non intentionnel,
scripts suivants interdisent l'auto-installation. Aucun SDK/Node/WebView2
installé par l'agent, dépendances projet seules ensuite restaurées.
Couverture non mesurée ; NSIS non construit, installation non recettée.
T006/T007 toujours ouvertes et T010 partielle ; aucun nouveau lot fondations.

Commandes, versions, empreinte de l'exe, recette graduée et arrêt après
sauvegarde : [prototype-windows.md](prototype-windows.md).
Preuves structurées : [batch-report.yaml](../../specs/001-bloc-notes-vocal/batch-report.yaml),
section `windows_preparation`, et checkpoint actuel.
Session détachée `windows-ui` conservée pour l'utilisateur, sans serveur Vite.
Prochain constat : fenêtre/menus/note/Manuel, **sans démarrer le micro**.

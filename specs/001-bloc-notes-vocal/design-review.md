# Revue d'implémentation — T001 et T004

2026-10-30. Revue documentaire terminée, **aucune certification du prototype**.
Sources prioritaires : constitution 3.0.0 inchangée, spec, plan, research,
quickstart, tasks et checkpoints actuels. Aucun fichier secret ni pièce jointe
de clé consulté. Le plan joint n'est pas modifié.

## T001 — décisions reprises

- Tauri 2, Rust édition 2024, Node 22, TypeScript 5/DOM et Vite ; aucun framework
  UI ajouté. ProseMirror visuel, pas de source + aperçu ni de conversation.
- Une fenêtre WordPad : Fichier/Édition/Aide, bandeau Markdown et démarrer/arrêter.
- Q01 est résolue : états et précisions dans la fenêtre seulement, même masquée.
  Aucun tray, notification bureau, TTS, extension GNOME ou focus imposé.
  L'écoute arrière-plan et la pause naturelle restent obligatoires.
- Le vocal est principal après entrée explicite. Le manuel suit l'arrêt ;
  undo en mémoire limité à cette période, jamais undo vocal.
- CPAL natif, downmix/rubato hors callback, PCM mono s16le 16 kHz, VAD local,
  buffers bornés sans spool. Un silence délimite un passage, pas la session.
- STT et interprétation sont deux ports indépendants. Modèles et incertitudes
  fournisseurs : [provider-check.md](../../docs/validation/provider-check.md).
- GNOME/Wayland et Windows 11 obligatoires dès T007. WSLg n'est pas GNOME et ne
  certifie pas Windows. Inventaire : [environments.md](../../docs/validation/environments.md).

## T004 — contrats à traduire sans les affaiblir

### Autorité et frontières (U1)

Rust conserve le Markdown canonique. Un résultat est identifié par note, session,
génération, passage, opération, séquence et révision. La vérification est répétée
au commit sous l'autorité sérialisée, avec déduplication et préconditions UTF-8.
Une génération invalidée ne redevient jamais valide.

Créer/ouvrir/réouvrir, après protection du dirty et acceptation du changement,
invalide et purge les anciens traitements **avant** le chargement. Même fichier,
contenu ou révision identique ne réhabilite rien. Échec d'ouverture : ancienne
note accessible, anciennes requêtes toujours invalides. Une requête active mais
périmée est recalculée une fois ou signalée, jamais appliquée aveuglément.

### Projection et acquittement (U2)

Proposition : transactionId, noteId, période manuelle, baseRevision et patches
attendus. Un seul envoi en vol, suivants ordonnés. Réponse accepted/rejected
identifiée ; accepted décrit les patches réellement appliqués. La projection
en attente n'est ni confirmée ni sauvegardée. Ignorer doublons/événements anciens.

Refus, trou de révisions ou divergence : arrêter les envois, abandonner la file,
demander le canonique et reconstruire **sans rejeu**. Réinitialiser l'historique
avec avertissement ; conserver séparément en mémoire la saisie refusée, copiable
explicitement jusqu'à la fin de la période. Signaler sa suppression. Undo/redo
manuel suit les mêmes acquittements, sans doubler une entrée à l'acceptation.
Changement de note ou de mode invalide les propositions antérieures.

### Enveloppe d'intentions (U3)

Valider la réponse entière avant tout effet. Résolu : actions ordonnées
edit/session_action/drafting_intent, liste vide pour no_action. Ambigu :
clarify avec message et **aucune** action exécutable. Aucun undo/redo ni outil
arbitraire. Schéma strict, champ filaire `schema`, propriétés obligatoires,
`additionalProperties=false` sur chaque objet ; validation locale obligatoire.

Préparer les patches sur une copie de travail, vérifier toutes les cibles et
préconditions avant publication. Ton → édition → sauvegarde conserve cet ordre.
Changement de note = barrière terminale ; portée incertaine après = ambiguïté.
Une clarification ultérieure est une nouvelle demande, aucune action suspendue
ne reprend automatiquement. L'interruption prioritaire reste indépendante :
arrêt/invalidation seulement, aucune mutation sur delta.

### Markdown, disque et fermeture

Plages source pulldown-cmark ; patch de la plus petite unité représentable,
octets hors cible inchangés (Unicode/CRLF/espaces). Code, tableaux, HTML et autres
unsupported : îlots opaques, refus avant altération, sans sérialisation globale.
Rendu sans HTML actif, protocole dangereux ni image distante.

Une note, dirty protégé avant changement. Vérifier identité et empreinte disque,
watcher indicatif. Fichier voisin exclusif de note, flush/sync, échange contrôlé
Linux / ReplaceFileW avec sauvegarde Windows ; conserver la version déplacée
jusqu'au contrôle. Conflit : préserver les deux versions sans rollback destructeur.
Primitive absente/volume non local/lien : proposer une copie locale exclusive,
pas d'écrasement. Fermeture : arrêt/invalidation → sauvegarde → fin/sortie ;
échec visible, note accessible, capture arrêtée.

### Secrets, OS et erreurs

Saisie masquée brève → IPC dédié → keyring réel, champ vidé après envoi.
Backends explicites : GNOME sync-secret-service + crypto-rust ; Windows
windows-native ; macOS apple-native à vérifier ultérieurement. Aucun import de
fichier/env, fallback mémoire prétendant réussir, retour de valeur ou log secret.
Webview sans réseau fournisseur, accès fs/shell générique ou ressources distantes.

ScreenSaver/login1 sous GNOME, WTS/WM_POWERBROADCAST Windows : initialiser l'état,
surveiller disparition du service. Perte de surveillance = arrêt sûr ou vocal
refusé, manuel disponible. Pause invalide ; arrêt/manuel/veille/verrouillage
invalident et purgent ; aucune reprise automatique.

Politique U4/A1 : erreur française catégorisée, sans détails de requête/contenu.
Une action indisponible est désactivée ou refusée explicitement, jamais annoncée
réussie. Le scaffold n'annonce pas d'écoute, de sauvegarde ou de clé configurée.

## Assertions de prototype à écrire avant T006

| Groupe | Assertions déterministes, synthétiques hors réseau |
|---|---|
| C1 pipeline | Delta corrigé puis done : aucun commit sur delta, un seul commit sur final ; rotation avec passage suivant borné ; done tardif/dupliqué sans effet indu ; interruption pendant traitement et saturation invalident sans mutation. |
| U1 | Résultat tardif après new/open/reopen, même contenu/révision ; échec d'ouverture n'annule pas l'invalidation ; contrôle noteId et génération au commit. |
| U2 | Un seul envoi, accepted/rejected corrélés ; doublons/anciens événements sans régression ; refus resync sans rejeu ; récupération mémoire ; undo confirmé/reset explicite. |
| U3 | Ambiguïté au début/à la fin : zéro effet de toute l'intervention ; nouvelle précision sans reprise ; ordre ton/édition/save ; barrière note ; interruption sans contenu. |
| Intégrité | Patch local préserve CRLF/Unicode et bloc opaque ; conflit/écriture refusée/fermeture échouée sans perte. |
| Confidentialité | Pas de capture spontanée, secret masqué/vidé et trousseau indisponible sans fallback ; pas de logs de contenu ni cache de session. |

Ces tests ne prouvent ni microphone, ni trousseau, ni desktop, ni qualité du modèle.
T007 exige les recettes réelles G02–G09 sur les deux OS et le consentement API.
La revue est terminée ; les assertions ci-dessus ne sont pas déclarées exécutées.

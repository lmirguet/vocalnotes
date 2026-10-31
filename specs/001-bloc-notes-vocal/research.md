# Recherche résolue — Bloc-notes vocal

**Consultation : 2026-10-30.** Choix d'implémentation arrêtés pour la génération des tâches ; aucune recette du produit exécutée.

## Méthode et niveau de preuve

La spécification actuelle et la constitution 3.0.0 priment sur l'ancienne recherche jointe, conservée en lecture seule. Pas de nouveau cadrage ni de question fonctionnelle ouverte. Les choix ci-dessous sont des décisions d'ingénierie, pas des succès de prototype.

Recherche sur des URL et questions techniques publiques uniquement, avec une recherche dédiée aux sources Mistral. Aucun code privé, note, audio, clé ou contenu utilisateur transmis ; aucun appel authentifié, installation, build ou test runtime. Les domaines `docs.mistral.ai`, y compris `/dev` et `/llms.txt`, et `mistral.ai` échouent à la vérification TLS ; celle-ci n'a jamais été désactivée. Le moteur de recherche renvoie une erreur 504. Les dépôts **officiels** accessibles apportent les preuves alternatives, figées ci-dessous :

- SDK Python Mistral : commit `90e300904b64c36e0630f72ef475dbab7912068e`.
- Cookbook Mistral : commit `0ce219a9ac212d5521c22df47bbc889527416bc6`.

Ces preuves ne démontrent ni droits d'un compte, ni facturation contractuelle, ni qualité française du futur binaire. Les tarifs et la conservation fournisseur non vérifiés sont explicitement inconnus ; ils ne sont pas remplacés par des chiffres de blogs.

## R01 — Socle desktop et UI minimale

**Décision :** Tauri 2, cœur Rust édition 2024, interface TypeScript 5 et DOM natif, ressources compilées par Vite/Node 22 LTS ; aucune couche React/Vue ajoutée. Un seul document visuel ProseMirror 1, menus Fichier/Édition/Aide dans la fenêtre et bandeau Markdown/démarrer/arrêter. Patchs de dépendances et toolchain exacts verrouillés en T005, pas une technologie laissée « à choisir ».

**Justification :** capture, secrets, réseau et autorité de mutation restent natifs ; l'UI n'impose ni serveur ni moteur Chromium embarqué. Tauri documente WebView2 sous Windows 11 et WebKitGTK sous Linux : ce sont précisément les deux environnements de recette obligatoires. L'absence de tray retire la dépendance d'affichage à une extension GNOME.

**Alternatives écartées :** Electron (moteur homogène mais distribution plus lourde, sans résoudre les invariants natifs), Qt (autre pile et pont d'édition Markdown à réaliser également), navigateur seul (capture et cycle OS insuffisants pour le besoin), framework UI généraliste (inutile pour une fenêtre et quelques panneaux). Un NO-GO concret au prototype peut justifier une révision, pas une attente indéfinie de choix.

Sources lues : [Tauri, moteurs](https://v2.tauri.app/reference/webview-versions/), [frontières et capacités](https://v2.tauri.app/security/).

## R02 — Voxtral hébergé, protocole et finalité

**Décision :** transcription dédiée avec **`voxtral-mini-transcribe-realtime-2602`**, pas Voice Agents et pas auto-hébergement. Le [cookbook officiel, serveur](https://github.com/mistralai/cookbook/blob/0ce219a9ac212d5521c22df47bbc889527416bc6/mistral/audio/realtime-fact-checker/server.py) et son [explication](https://github.com/mistralai/cookbook/blob/0ce219a9ac212d5521c22df47bbc889527416bc6/mistral/audio/realtime-fact-checker/01-realtime-fact-checker.md) utilisent exactement ce slug avec `client.audio.realtime.transcribe_stream`. Cette source résout le manque du plan précédent.

Le vieux défaut `voxtral-mini-2601` de l'exemple SDK n'est pas retenu. Le slug `voxtral-mini-transcribe-realtime-3` observé dans le README Voice Agents concerne un autre parcours ; il n'est pas transposé à cet endpoint sans preuve. La [carte open-weight](https://huggingface.co/mistralai/Voxtral-Mini-4B-Realtime-2602/blob/main/README.md) confirme famille/français, mais **ne prouve pas** un slug hébergé.

**Format retenu :** PCM mono signé 16 bits little-endian, **16 000 Hz**, sans en-tête WAV dans les messages. L'[exemple officiel SDK](https://github.com/mistralai/client-python/blob/90e300904b64c36e0630f72ef475dbab7912068e/examples/mistral/audio/async_realtime_transcription_stream.py) fournit `AudioFormat(encoding="pcm_s16le", sample_rate=16000)` et conversion `-f s16le -ar 16000 -ac 1`. Sa conversion en fichier temporaire n'est **pas reprise** : le produit normalise seulement en mémoire.

**Transport :** `wss://api.mistral.ai/v1/audio/transcriptions/realtime?model=voxtral-mini-transcribe-realtime-2602`, en-tête de handshake `Authorization: Bearer …` côté Rust. Le [client officiel](https://github.com/mistralai/client-python/blob/90e300904b64c36e0630f72ef475dbab7912068e/src/mistralai/extra/realtime/transcription.py) construit cet endpoint, force WSS et passe les en-têtes ; [connexion](https://github.com/mistralai/client-python/blob/90e300904b64c36e0630f72ef475dbab7912068e/src/mistralai/extra/realtime/connection.py) : audio base64, `input_audio.append`, `input_audio.flush`, `input_audio.end`, mise à jour de session. Attendre la création de session et configurer le format avant audio.

**Sémantique vérifiée et limite :**

| Événement | Ce qu'il prouve | Usage choisi |
|---|---|---|
| `transcription.text.delta` | Texte incrémental ; aucune finalité de passage garantie par ce seul nom | Diagnostic provisoire ; détection prioritaire d'interruption seulement, pas édition ordinaire |
| `transcription.segment` | Le [type officiel](https://github.com/mistralai/client-python/blob/90e300904b64c36e0630f72ef475dbab7912068e/docs/models/transcriptionstreamsegmentdelta.md) porte text/start/end, sans drapeau final | Segment qualifié comme tel, jamais assimilé à une intention complète |
| `transcription.done` | Le [type officiel](https://github.com/mistralai/client-python/blob/90e300904b64c36e0630f72ef475dbab7912068e/docs/models/transcriptionstreamdone.md) contient texte complet, modèle, usage et segments éventuels | Texte final du flux audio terminé, réconciliation des provisoires |
| `input_audio.flush/end` | Messages client de vidage/fin, pas preuve d'intention utilisateur | Fin automatique d'un passage détecté localement, pas fin de la session d'usage |

Le cookbook traite `done` comme transcription de référence quand l'audio se termine ; il découpe aussi les deltas par ponctuation et reconnaît les limites des abréviations. Cette heuristique de ponctuation seule est **écartée pour les mutations**.

**Fin de passage arrêtée :** capture continue native, VAD local WebRTC et rotation de flux STT à frontière parole/silence ; traitement ordinaire sur texte `done` uniquement. Tampon du passage suivant borné pendant finalisation, pas de trou de capture ni disque ; saturation = erreur/non-intégré explicites. Le silence ne termine jamais la session. Une intervention longue est découpée si les limites du service l'exigent, sans inventer une durée maximale aujourd'hui. Les fragments restent « suite attendue » si insuffisants.

La pause/arrêt à voix reste prioritaire : interprétation sémantique des deltas pour **interruption seulement**, avec génération invalidée et zéro mutation sur ce résultat. Toutes requêtes sont comptées ; rapidité, faux arrêts, finalisation et rotation sans pertes sont des critères GO/NO-GO de T007. Il ne s'agit ni d'un mot-clé rigide ni d'un bouton requis par intervention.

**Alternatives écartées :** édition sur chaque delta (finalité non prouvée), attendre `done` de toute la session (pas progressif), API Voice Agents et TTS (hors besoin et autre protocole), HTTP upload sur disque, modèle local (autre matériel et maintenance). Un événement futur inconnu ne reçoit pas une sémantique inventée.

## R03 — Modèle d'interprétation et JSON Schema

**Décision :** identifiant API **`mistral-large-latest`** avec `response_format={"type":"json_schema", ...}` strict, validé localement avant mutation. C'est l'identifiant exact employé dans l'[exemple officiel de schéma personnalisé](https://github.com/mistralai/client-python/blob/90e300904b64c36e0630f72ef475dbab7912068e/examples/mistral/chat/structured_outputs_with_json_schema.py), avec `strict=True` et `additionalProperties=False`.

**Justification :** c'est la combinaison modèle/sortie structurée directement démontrée par la source officielle accessible, contrairement à un slug versionné deviné. Le [type JSONSchema officiel](https://github.com/mistralai/client-python/blob/90e300904b64c36e0630f72ef475dbab7912068e/src/mistralai/client/models/jsonschema.py) mappe `schema_definition` Python vers le champ JSON **`schema`**. Rust envoie le champ filaire, sans adopter le nom interne Python.

**Changement du candidat :** « Mistral Large 4 » n'est pas retenu comme modèle confirmé. La [carte Large 3](https://huggingface.co/mistralai/Mistral-Large-3-675B-Instruct-2512/blob/main/README.md) prouve des poids Large 3, pas le modèle courant derrière l'alias ni son prix hébergé. On ne prétend donc ni Large 3 ni Large 4 testé ou servi.

**Limite explicite :** `latest` est mutable. Pour rendre une recette reproductible, T002/T007 relèvent le modèle réellement retourné et le catalogue du compte au premier essai autorisé ; un identifiant versionné équivalent n'est figé qu'après preuve de disponibilité et d'acceptation du schéma. Ce contrôle n'est pas une nouvelle question utilisateur ni une technologie indécise. Si le couple retenu est refusé, ne pas basculer silencieusement : afficher l'échec et revoir l'intégration. L'alias documenté suffit pour écrire l'adaptateur ; une version de service reproductible n'est pas revendiquée aujourd'hui.

Le plan définit opérations, clarification, actions de session, préconditions et invalidation. JSON strict ne garantit ni fidélité, ni cible, ni sécurité sémantique. Les contrôles locaux restent obligatoires ; pas d'outils arbitraires ni conversation hébergée.

**Alternatives écartées :** slug `mistral-large-2512` inféré d'un nom de poids (preuve insuffisante), sortie JSON libre seule (contrainte de forme moindre), réécriture complète de note (portée et unsupported), commande rigide (contraire au besoin), agents hébergés avec historique (inutile).

## R04 — Capture et cycle OS

**Décision :** CPAL natif, WASAPI Windows, ALSA via l'environnement PipeWire Fedora non sandboxé, CoreAudio pour macOS ; downmix et rubato vers 16 kHz hors callback. Tampon mémoire borné, aucune capture avant démarrage explicite ni spool. VAD local WebRTC, paramètres d'ingénierie calibrés au prototype sans les confondre avec des seuils produit ratifiés.

**Justification :** [CPAL](https://docs.rs/cpal/latest/cpal/) documente périphériques/configurations, callbacks dédiés et erreurs, pas une garantie de fonctionnement sur chaque desktop. [webrtc-vad](https://docs.rs/webrtc-vad/latest/webrtc_vad/) est retenu uniquement pour parole/silence ; il ne comprend ni intention ni fin sémantique.

GNOME : `org.gnome.ScreenSaver.GetActive/ActiveChanged` selon l'[interface GNOME officielle](https://github.com/GNOME/gnome-shell/blob/main/data/dbus-interfaces/org.gnome.ScreenSaver.xml), complété par `login1.PrepareForSleep` pour veille. Windows : [notifications WTS](https://learn.microsoft.com/en-us/windows/win32/termserv/wm-wtssession-change) et [alimentation](https://learn.microsoft.com/en-us/windows/win32/power/wm-powerbroadcast). Interruption, invalidation immédiate, manuel au retour ; perte de surveillance = arrêt sûr. Aucun raccourci global, extension GNOME ou prise de focus nécessaires.

**Alternatives écartées :** capture `getUserMedia` liée au scheduling webview, focus assimilé au verrouillage, réseau annulé sans barrière de commit. Les interfaces sont documentées ; leur ponctualité et intégration restent des essais, pas une incompatibilité constatée.

## R05 — Q01 et deux modes : décisions conservées

**Décision utilisateur inchangée :** écoute et ambiguïtés **dans la fenêtre uniquement**, même masquée ; consultation au retour volontaire. Pas de tray, indicateur hors fenêtre, notification système, synthèse vocale ou focus forcé. Écoute continue arrière-plan et pause naturelle restent obligatoires.

**Décision d'état :** autorité native unique, session/génération/revision et idempotence au commit. Pause suspend avec micro coupé et invalidation des résultats ; arrêt/manuel/verrouillage/veille terminent et purgent ; reprise explicite, jamais automatique. Une nouvelle session peut modifier intentionnellement le contenu manuel, jamais appliquer une réponse d'une session invalidée. Undo mémoire uniquement pendant l'édition manuelle courante ; aucune pile vocale.

**Précisions utilisateur U1 et U3 :** créer ou ouvrir une note invalide tous les traitements précédents en cours ; le commit vérifie aussi l'identité de note, sans réutiliser une ancienne génération même à la réouverture du même fichier. En cas d'ambiguïté, aucune action de l'intervention n'est exécutée : l'application signale l'ambiguïté dans sa fenêtre. Une précision vocale forme une nouvelle demande, sans reprise automatique d'une liste suspendue. Les intentions mixtes non ambiguës sont validées ensemble puis exécutées dans l'ordre explicite.

**Alternatives écartées :** extension GNOME, outil de notification, booléen d'annulation seul, « protection » générale des textes manuels, historique vocal caché. Q01 est résolue, pas à reposer.

## R06 — Éditeur WordPad compatible Markdown

**Décision :** ProseMirror 1, un seul éditeur visuel ; sous-ensemble paragraphes/titres/listes/gras/italique et bandeau. Vocal = consultation read-only sans plugin history ; manuel = saisie et history en mémoire. Menus DOM accessibles dans la fenêtre, pas de dépendance aux menus globaux du shell.

**Justification :** [guide ProseMirror](https://prosemirror.net/docs/guide/) : schémas, états, transactions et vue d'édition riche ; [exemple Markdown](https://prosemirror.net/examples/markdown/) : parseur/sérialiseur Markdown et document visuel. L'exemple ne garantit **pas** préservation octet pour octet.

Source canonique Rust, parseur [pulldown-cmark avec offsets](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/struct.OffsetIter.html), projection ProseMirror et pont de patches par plages source. Le plan précise transactions, plus petite unité affectée, fins de ligne et préconditions. Constructions non supportées = îlots opaques non éditables ; toute opération risquant leur altération est refusée et expliquée avant commit. Hors cible, les octets source sont conservés, pas resérialisés. Rendu sans HTML actif, scripts, URL dangereuses ou chargement distant automatique.

**Décision utilisateur U2 :** la représentation du cœur a priorité sur l'affichage.
Le plan définit les propositions manuelles identifiées et acquittées, la resynchronisation
sur l'instantané canonique en cas de refus ou divergence, et l'historique d'annulation
fondé sur les transactions acceptées. Pas de rejeu automatique d'une modification refusée ;
sa saisie reste récupérable séparément en mémoire pendant la période manuelle.

**Alternatives écartées :** textarea avec aperçu obligatoire (pas l'édition visuelle demandée), CodeMirror source seulement, Tiptap plus couche de framework (non nécessaire ici), sérialisation globale standard (perte de forme et unsupported), éditeur riche bureautique (scope disproportionné). Le pont est du travail réel à tester dès prototype et T039, pas une capacité magique de la bibliothèque.

## R07 — Trousseau et saisie des clés

**Décision :** keyring **3.6.3**, backends explicites `sync-secret-service`+`crypto-rust` GNOME, `windows-native` Windows, `apple-native` macOS. Échec si backend indisponible ; jamais backend mock ou fichier clair en production.

**Justification :** [documentation versionnée](https://docs.rs/keyring/3.6.3/keyring/) : pas de fonctionnalités par défaut ; absence de backend = mock. Le choix explicite évite cette fausse sécurité. La documentation du `latest` de keyring a changé de modèle ; on ne lui emprunte pas des fonctionnalités en supposant la même API.

Préférences locales, champ masqué à durée courte, commande IPC dédiée, pas de copie dans le store d'UI, champ vidé après envoi, aucun retour de valeur du cœur. Le secret doit transiter en mémoire pour être saisi et utilisé : « pas de secret dans le frontend » ne signifie pas absence magique du buffer de saisie. Contenu distant/script actif interdits, capacités limitées.

**Alternatives écartées :** dialogue natif supplémentaire (complexité non exigée par la constitution), coffre fichier/Stronghold présenté comme trousseau, `.env`, variable d'environnement comme UX produit, Linux keyutils seul, fallback mémoire prétendant conserver une clé.

## R08 — Fichiers et conflits

**Décision :** fichiers Markdown UTF-8 locaux, aucune base ; destination et dernière note en préférences non sensibles, autosauvegarde contrôlée, identité et empreinte disque, watcher comme signal seulement. La stratégie détaillée est inline dans le plan.

**Justification :** le simple couple « vérifier puis renommer » laisse une course. Retenir la version disque déplacée jusqu'au contrôle après remplacement permet d'exposer et préserver un conflit au lieu d'écraser silencieusement. Sous Windows, [ReplaceFileW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-replacefilew) documente une copie de sauvegarde optionnelle ; sous Linux, échange atomique sur système de fichiers compatible. Si la primitive ou la stabilité manque, nouvelle copie exclusive et signalement, pas remplacement aveugle.

Seuls des fichiers de **note** peuvent être utilisés pour préparer l'écriture ; aucun cache audio/transcription/conversation. Copies de conflit explicites, pas historique d'annulation. Fermeture : arrêt/invalidation, sauvegarde, fin/sortie ; erreur laisse la note accessible sans micro. Tester interruption, disque plein, changements externes pendant sauvegarde et limites de primitives ; pas de garantie universelle contre des écritures externes sans coopération.

**Alternatives écartées :** fichier écrasé directement, watcher seul, base propriétaire, resérialisation globale, rollback qui pourrait détruire une nouvelle écriture externe.

## R09 — Tarifs, conservation et mesures

**Décision :** coût de session STT et LLM séparés ; durée audio effectivement envoyée, tokens retournés, modèle réellement servi, source/date du tarif si disponible. **Aucun tarif officiel vérifié dans cette reprise.** Les prix STT/minute et LLM/tokens restent « inconnus » dans l'UI tant qu'une source officielle applicable n'est pas disponible ; ne jamais afficher zéro par défaut.

Pour un tarif confirmé ultérieurement : coût STT = durée facturable × prix de son unité ; LLM = composantes d'usage × prix entrée/sortie, avec gestion explicite des usages absents et éventuels détails de cache documentés. Aucune règle d'arrondi/minimum ou taxation inventée. L'alias mutable exige de vérifier que le tarif correspond bien au modèle servi. Une estimation n'est pas une facture. Aucun seuil de performance, mémoire ou coût n'est ratifié.

**Conservation fournisseur : non vérifiée.** Pages officielles à consulter avant usage sensible : [documentation Mistral](https://docs.mistral.ai/), [tarifs](https://mistral.ai/pricing), [conditions](https://mistral.ai/terms). Ces liens sont des destinations de contrôle, pas des pages prétendument lues avec succès. Test de compte uniquement synthétique, explicitement informé du contact/coût ; aucun contact métier nécessaire.

Localement : mémoire bornée et purge des références, transcription opt-in pour la session, aucun stockage browser/log/console de contenu. Pas de promesse d'effacement physique du swap/crash dump ou suppression chez le fournisseur. Les scans de traces et leur périmètre font partie des recettes.

**Alternatives écartées :** chiffres de blogs repris comme officiels, prix sans date, garantie de zero-retention déduite d'un SDK, audio d'essai personnel, conservation de conversation pour calculer des coûts.

## Synthèse des gates

| Gate conservée | État de planification | Critère futur et échéance |
|---|---|---|
| G01 — UX GNOME | RÉSOLUE | Q01 et WordPad conservés, pas d'extension |
| G02 — modèles/protocole | DÉCISION DOCUMENTÉE | T007 : accès compte, modèle réellement servi et acceptation schéma ; indisponibilité = erreur, pas fallback |
| G03 — capture | CHOIX ARRÊTÉ | T007 : mono 16 kHz, flux en arrière-plan, rotation sans perte ; saturation explicite |
| G04 — affichages/pause | DÉFINI | T007 puis chaque jalon : pause naturelle rapide, état/précision internes et pas de focus volé |
| G05 — cycle/session | DÉFINI | T007 puis chaque jalon : lock/sleep/fermeture, résultats tardifs rejetés, zéro auto-reprise |
| G06 — secrets | CHOIX ARRÊTÉ | T007/T017 : vrais trousseaux, saisie/purge, absence de clair et de mock |
| G07 — intégrité | STRATÉGIE DÉFINIE | Prototype minimal puis T011/T039/T043 : octets hors cible, opaque, conflits et échecs |
| G08 — services/qualité | PROTOCOLE DE RECETTE DÉFINI | T007 synthétique et informé ; T057 quatre usages ; tarifs absents affichés inconnus, pas de sensible sans conditions vérifiées |
| G09 — plateformes/mesures | PRÉREQUIS DE RECETTE | Deux desktops dès T007 et chaque jalon ; pas de certification sur un hôte distant |
| G10 — revue interfaces | SUFFISANT POUR TÂCHES | T004 revoit les définitions inline avant codage ; aucun design autonome attendu |

**Conclusion :** recherche de choix terminée, aucune clarification fonctionnelle ouverte. Limites externes identifiées : accès réel du compte, version derrière l'alias, tarifs/conditions inaccessibles et mesures desktop sont des contrôles futurs explicites. L'absence de ces essais ne constitue pas un ERROR de planification et n'autorise aucune annonce de compatibilité ou de qualité acquise.

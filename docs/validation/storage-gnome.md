# T011–T013 — stockage et fermeture, GNOME/Wayland

Lot B05-STORAGE du 2026-10-31, après validation du prototype T006/T007.
Implémentation et vérification hors ligne sur Fedora ; recette de la nouvelle
version dans la fenêtre encore à effectuer. Windows et veille hors périmètre.

## Comportement implémenté

- Une seule note active. Ouverture UTF-8 bornée, empreinte SHA-256 et identité
  disque conservées ; contenu Markdown opaque et octets hors modification préservés.
- Une modification acceptée autosauvegarde vers la destination connue. Sans
  destination, la note reste modifiée en mémoire. « Enregistrer sous » reste une
  création exclusive ; « Enregistrer » utilise la destination courante.
- Inotify signale les changements du répertoire ; chaque sauvegarde relit le
  fichier, même sans signal. Aucun contenu externe n'est importé automatiquement.
  L'avertissement de stockage reste visible après pause ou renouvellement du statut.
- Préparation exclusive dans le même répertoire, écriture et synchronisation,
  puis échange Linux `renameat2(RENAME_EXCHANGE)`. La version déplacée reste
  disponible jusqu'au contrôle de contenu et d'identité après échange.
- Un conflit conserve des copies Markdown locales/externes et leurs chemins,
  garde la note modifiée en mémoire et suspend les sauvegardes suivantes vers
  cette destination. Pas de restauration qui écrase un écrivain externe.
- Liens symboliques et fichiers non réguliers refusés, traversée des répertoires
  sans suivi de liens. Permissions ordinaires conservées ; liens physiques,
  métadonnées particulières ou primitive indisponible conduisent à une nouvelle
  copie explicite. Systèmes de fichiers non reconnus comme locaux refusés.
- Fermeture : arrêt de capture/invalidation, sauvegarde, libération du verrou
  du cœur, destruction de fenêtre. Un échec ou une destination absente conserve
  la fenêtre et la note ; aucune reprise automatique de capture.

## Vérifications

Tests écrits avant le code : module stockage absent, autosave absent, module
fermeture inaccessible puis méthodes de surveillance absentes ont produit les
échecs attendus avant correction.

| Vérification finale | Résultat |
| --- | --- |
| Rust bibliothèque + harnais `offline_contract_tests` | 10 + 80 PASS |
| JavaScript, avertissement persistant et actualisation sans révision inclus | 34 PASS |
| `cargo check --offline --locked --all-targets` | PASS |
| Typecheck et build web | PASS |
| Format Rust et `git diff --check` | PASS |

Les tests de stockage utilisent de vrais fichiers : Markdown CRLF/opaque,
remplacement par un nouvel inode de même contenu, modification externe avant
sauvegarde et exactement à l'échange, nouvelle modification après échange,
permissions, liens physiques et symboliques, création exclusive et surveillance.
Un sous-processus sous `RLIMIT_FSIZE` provoque une véritable écriture partielle :
destination précédente intacte et copie partielle signalée. Un autre sous-processus
sort immédiatement avant/après échange : les deux contenus restent récupérables.
Les tests de fermeture vérifient invalidation, absence de destination et échec de
destruction de la fenêtre après sauvegarde, sans ouvrir de fenêtre réelle.

Logs de cette exécution : `/tmp/vocalnotes-storage-all.log`,
`/tmp/vocalnotes-storage-js.log`, `/tmp/vocalnotes-storage-web.log`,
`/tmp/vocalnotes-storage-check.log`. Ces chemins temporaires ne sont pas des
artefacts pérennes du dépôt.

## Limites et recette native restante

Le test d'écriture partielle exerce le chemin d'erreur disque ; aucun volume
physiquement plein ni coupure électrique réelle n'a été provoqué. L'échange
atomique ne fournit pas une garantie universelle contre un processus du même
utilisateur qui réécrit arbitrairement les fichiers entre les contrôles.
Les fichiers de récupération ne contiennent que des notes Markdown ; après
une interruption, leur examen reste explicite, sans import automatique.

Le binaire principal n'a été ni reconstruit, ni remplacé, ni lancé par ce lot.
SHA-256 inchangé :
`5b441b0be92924b6516b77574b066b116488c475b29e591e7401b9a39732f321`.
Les tests utilisent le harnais exemple pour préserver l'instance du prototype.
Aucun appel fournisseur, micro, trousseau ou installation système pendant ce lot.
`sha2` et `libc` deviennent des dépendances directes aux versions déjà verrouillées.

Après fermeture volontaire de l'ancienne instance puis reconstruction native :

1. Ouvrir une note synthétique locale ; modifier un paragraphe et vérifier le
   fichier sans création systématique de copies, en conservant un bloc opaque.
2. Modifier le fichier depuis un autre éditeur ; vérifier l'avertissement sans
   remplacement du texte dans VocalNotes. Modifier ensuite la note et vérifier
   les copies de conflit et leurs chemins, puis résoudre via une nouvelle destination.
3. Fermer une note sauvegardable, puis une note sans destination ou avec erreur
   de stockage : sortie seulement dans le premier cas, note accessible dans les autres.

La suite du plan est T014–T016 : frontière IPC et extraction de l'interface.

## Recette manuelle complémentaire — PASS sur scénarios exercés

Ouverture, modification/formatage/annulation, autosave contrôlé, conservation des
blocs protégés, changement externe sans import, copies de conflit, résolution par
nouvelle destination et fermeture normale validés avec l'utilisateur. Fichiers
contrôlés après fermeture et absence de processus vérifiée. Voir la chronologie
dans [le rapport IPC/interface](ipc-ui-gnome.md), y compris le correctif de lisibilité.
Les tests non exercés dans la fenêtre ne sont pas certifiés par cette recette.

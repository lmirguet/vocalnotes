# T023–T025 — cycle de vie et état d’écoute

Lot B09-LIFECYCLE, 2026-10-31. Implémentation GNOME/Wayland et vérifications hors
ligne. Windows et mise en veille réelle exclus selon les consignes utilisateur.
La veille reste couverte comme événement simulé de sûreté, sans suspendre l’hôte.

## Changements

`platform/session_devices.rs` fournit la frontière périphériques du contrôleur :
surveillance OS, présence de clé, ouverture et arrêt du flux audio. En production,
`start_explicit` utilise toujours `NativeDevices` (GNOME/login1, trousseau, CPAL).
L’injection est une API Rust pour les tests, sans commande IPC ni préférence
permettant de contourner la surveillance native. Les deux chemins utilisent le
même contrôleur, les mêmes gardes de session et le même superviseur d’erreurs.

Ordre conservé : démarrage explicite → abonnement OS vérifié → présence de clé →
ouverture du micro → écoute. Silence seul sans arrêt. Toute erreur d’un worker
(capture, STT, LLM ou interruption prioritaire) invalide la session et revient en
manuel, sans reconnexion ni répétition automatique. Une fermeture de canal audio
est désormais une erreur explicite, même sans callback d’erreur CPAL.

Le moniteur natif conserve les abonnements GNOME `ActiveChanged`, login1
`PrepareForSleep`, la vérification initiale après abonnement et la surveillance
des propriétaires/services. Une annulation déjà demandée interdit son
initialisation ; une invalidation reçue pendant l’initialisation est mémorisée,
même si un déverrouillage suit immédiatement. Le callback d’invalidation est
consommé une fois. Le même drapeau d’arrêt est partagé avec l’ouverture du micro.
Une ouverture tardive est abandonnée sans réactiver une ancienne session.

L’invalidation et le signal d’arrêt sont synchrones sous les verrous du cœur ;
la destruction physique du flux et la jonction des workers restent asynchrones.
Les tests ne mesurent pas un délai matériel. Le polling de santé et les délais
D-Bus existants restent bornés, sans garantie de détection instantanée d’une
perte de bus. Les événements d’une ancienne génération n’arrêtent pas la nouvelle.

`SessionStatus.ts` affiche l’état canonique indépendamment d’une projection
manuelle ou d’un message ancien : initialisation (micro pas encore prêt), écoute,
pause/mode manuel (micro arrêté), traitement, précision ou erreur. État capture
et activité sont distincts. Le passage à « écoute » est publié seulement après
installation du contrôle de capture. Avertissement tiers et bouton de pause
immédiatement disponible pendant l’initialisation/l’écoute, vers la commande
partagée. Le parcours prioritaire vocal reste branché sur le même arrêt. Aucun
tray, notification, activation de fenêtre ni déplacement de focus ajouté.

## Tests et portée

Tests écrits avant l’intégration : échec initial Rust sur la frontière absente,
échec UI sur le composant absent. Journaux rouges :
`/tmp/vocalnotes-lifecycle-red.log`, `/tmp/vocalnotes-sessionstatus-red.log`.

Sept tests d’intégration dans `tests/lifecycle.rs` utilisent le vrai contrôleur
avec capture silencieuse et moniteur en mémoire :

- aucun périphérique ouvert au lancement ; silence maintient l’écoute ; pause
  coupe le contrôle et invalide un résultat retardé ; reprise explicite ;
- surveillance indisponible refusée avant clé/capture ;
- invalidation lock/sleep/perte moniteur simulée : manuel sans reprise automatique,
  ancien callback sans effet sur une nouvelle session ;
- perte de capture : arrêt sans reprise ; fermeture invalide et arrête la capture ;
- événement reçu avant l’acquittement du moniteur : pas d’ouverture de micro ;
- arrêt pendant ouverture : capture tardive détruite, nouvelle session préservée ;
- moniteur natif déjà annulé : aucun abonnement ni callback.

Un test unitaire du superviseur commun injecte une erreur de service et vérifie
le retour en manuel/erreur et l’invalidation ; une annulation ne relance pas le
travail en attente. Les contrats fournisseur de T020 continuent de tester les
erreurs HTTP/réseau sur transports simulés, sans appel Mistral. Les tests UI
couvrent les états, l’avertissement, la pause et la conservation du focus. Les
suites existantes couvrent aussi la sauvegarde/fermeture et la priorité vocale.

Aucun vrai micro, événement OS, trousseau utilisateur ou fournisseur utilisé pour
ces nouveaux tests. Le test négatif existant de trousseau utilise un sous-processus
avec adresse D-Bus invalide. Aucune installation, dépendance ou paquet ajouté.

## Prochaine étape : T026

Recette réelle du nouveau binaire et mesures des fondations restent à effectuer :
démarrage sans capture, initialisation/écoute visibles, silence et arrière-plan,
pause vocale/bouton, reprise explicite, verrouillage GNOME sans reprise, erreurs
et fermeture avec note préservée. Ne pas provoquer de veille sur cet ordinateur.
Les recettes précédentes sont des preuves de leur version, pas de ce nouveau lot.
T026 reste ouverte ; aucun passage aux stories ni certification multiplateforme.

## Résultats hors ligne

| Vérification | Résultat |
| --- | --- |
| Rust bibliothèque | 13 PASS |
| Harnais partagé, dont sept tests de cycle de vie | 116 PASS |
| JavaScript | 50 PASS |
| Typecheck et build web | PASS |
| Check Rust toutes cibles, ressources embarquées | PASS |
| Format Rust et diff | PASS |

Journaux : `/tmp/vocalnotes-lifecycle-{rust,js,web,check}.log`.
Aucun taux de couverture annoncé. Le binaire principal reste inchangé :
`f30770b459f1c38eafcf409975b7da04e8d6fc508ef5a4e4019cf8de69b5e905`.
Aucune fenêtre arrêtée ou relancée par ce lot. Les checklists restent inchangées
(28/28). Aucun hook before/after : `.specify/extensions.yml` absent.

# Intention attachée au fichier Markdown

Demande utilisateur du 2026-10-31 : conserver l’intention après arrêt du vocal et
la sauvegarder avec la note. Ce choix remplace son ancien cycle de session.

Le fichier contient un commentaire HTML initial réservé, puis le corps Markdown :

```markdown
<!-- vocalnotes:intention:v1
{"purpose":"Présenter le devis","form":"e-mail","tone":"cordial","context":""}
-->

Texte de la note.
```

Le commentaire n’est pas affiché dans le rendu Markdown ordinaire. VocalNotes
le sépare du corps à l’ouverture : il apparaît dans le champ d’intention, pas dans
l’éditeur. Le corps conserve ses octets, notamment ses fins de ligne et blocs
opaques. Les caractères pouvant fermer un commentaire sont échappés en JSON.
Sans intention, le fichier reste du Markdown ordinaire, sauf si son corps commence
lui-même par le préfixe réservé (un en-tête vide protège alors ce corps littéral).
Les documents aux métadonnées réservées invalides ou de version inconnue sont
refusés à l’ouverture, avec un message explicite et sans modification du fichier.

Le profil contient uniquement but/forme/ton/contexte minimal explicitement utiles,
limités ensemble à 4 096 octets UTF-8. Il constitue une propriété de la note,
conservée après pause, arrêt, reprise et redémarrage. Ouvrir ou créer une autre note
charge son propre profil, vide pour un ancien fichier sans métadonnées. Audio,
transcription, lexique et historique de conversation ne sont pas ajoutés au fichier.
L’intention enregistrée reste une donnée ; elle ne rejoue aucune commande.

Une modification d’intention seule marque le document modifié et déclenche la
sauvegarde contrôlée si une destination existe. Corps et profil sont écrits en
une seule transaction du stockage existant, sans fichier annexe. Les conflits,
copies de récupération, copies vers une nouvelle destination et sauvegardes de
fermeture portent tous sur le document complet. Le plafond du stockage de 1 Mio
s’applique au fichier entier, métadonnées comprises ; un dépassement refuse
l’écriture et conserve le document en mémoire.

Validation hors ligne : réouverture réelle de fichier après modification du
profil seul ; conservation après arrêt/nouveau démarrage ; conflit provoqué par
modification externe du seul profil avec conservation des deux documents ; copie
résolue avec bon profil ; CRLF/opaque inchangés ; caractères de fin de commentaire,
préfixe réservé littéral et métadonnées invalides/version inconnue. Le test de
persistance était en échec avec l’ancien cycle avant implémentation.

Recette native à effectuer sur une nouvelle version après fermeture de l’ancienne
fenêtre. Le succès de l’ancien champ temporaire ne vaut pas validation de cette
persistance.

## Binaire lancé pour recette

Fermeture confirmée par l’utilisateur et absence du processus vérifiée. Build
natif embarqué PASS (18,41 s). Empreinte :
`d1d2330953e2660afffeaa3ea179aee16e12e19cbc0b178f2a2ae82d1b761a16`.
Session outil 61051, journal technique `/tmp/vocalnotes-intention-persistence-native.log`.
Microphone arrêté au lancement selon la console. Destination synthétique neuve
préparée : `/tmp/vocalnotes-intention-recette-kx_by0qn/note.md` (non créée par l’agent).
Sauvegarde et réouverture via la fenêtre restent à confirmer ; T031 non close.

## Sauvegarde réelle et relance

L’utilisateur signale la fermeture ; absence de processus vérifiée. Le fichier
de recette existe avec un corps non vide et des métadonnées v1 valides, les quatre
champs renseignés et un profil dans la borne de 4 096 octets. Aucun contenu du
profil ou de la note n’est copié au rapport. Cette vérification démontre la
persistance sur disque, pas encore l’affichage restauré dans la fenêtre.
Même binaire relancé, session outil 18320, journal
`/tmp/vocalnotes-intention-reopen-native.log` ; microphone arrêté au lancement.
Ouverture du fichier et affichage de l’intention en manuel à confirmer par
l’utilisateur, sans démarrage vocal nécessaire. T031 reste ouverte.

## Réouverture — PASS utilisateur

L’utilisateur confirme que le fichier rouvert restaure le texte et l’intention,
notamment la forme e-mail et le ton cordial, sans démarrer le vocal. La chaîne
sauvegarde du document complet → fermeture → redémarrage → ouverture et affichage
est validée pour ce fichier sur GNOME/Wayland. Les autres contrôles T031 restent
ouverts ; aucune certification générale de fidélité STT ou Windows n’est déduite.

# Checklist de qualité de spécification : Bloc-notes vocal intelligent

**Objet** : valider la complétude et la qualité de la spécification avant planification.  
**Création** : 2026-10-30  
**Fonctionnalité** : [spec.md](../spec.md)  
**Portée du contrôle** : revue documentaire ; aucune exécution ni recette du produit.

## Qualité du contenu

- [x] CHK001 Aucun détail d'implémentation imposé : les technologies citées restent des candidats de cadrage.
- [x] CHK002 Contenu centré sur la valeur utilisateur et les besoins.
- [x] CHK003 Prose française compréhensible par les parties prenantes, sans mots normatifs anglais.
- [x] CHK004 Sections obligatoires du modèle remplies, dans l'ordre : scénarios, exigences, critères de réussite, hypothèses.

## Complétude des exigences

- [x] CHK005 Aucun marqueur de clarification à résoudre.
- [x] CHK006 Exigences testables, non ambiguës et formulées en résultats observables, avec renvois aux scénarios conservés lorsqu'ils existent.
- [x] CHK007 Critères de réussite mesurables par les cas de recette et les mesures demandées.
- [x] CHK008 Critères de réussite indépendants d'une architecture ou d'un composant.
- [x] CHK009 Scénarios d'acceptation définis pour les parcours retenus.
- [x] CHK010 Cas limites identifiés, notamment réponses tardives, conflits, sauvegarde et secrets.
- [x] CHK011 Périmètre initial et exclusions clairement délimités.
- [x] CHK012 Hypothèses, dépendances aux services et décisions de conception restantes explicites.

## Préparation de la fonctionnalité

- [x] CHK013 Chaque exigence possède au moins un critère d'acceptation identifiable.
- [x] CHK014 Les scénarios couvrent les parcours principaux et sont priorisés.
- [x] CHK015 Les résultats attendus sont reliés aux exigences et aux critères de réussite, sans prétendre qu'ils sont déjà obtenus.
- [x] CHK016 Aucune architecture, technologie candidate ou solution interne n'est transformée en obligation.

## Traçabilité et décisions validées

- [x] CHK017 Les identifiants REQ-001 à REQ-053 sont uniques et séquentiels ; leur correspondance avec le format d'exemple local est expliquée.
- [x] CHK018 Les cinq principes de la constitution actuelle sont couverts sans modifier cette dernière.
- [x] CHK019 Le vocal est principal, toutes ses actions sont vocales ; l'éditeur manuel classique n'est accessible qu'après arrêt de l'interface vocale.
- [x] CHK020 Aucun mécanisme ou historique dédié d'annulation vocale n'est réintroduit ; F15–F19 et R06–R07 sont explicitement écartés.
- [x] CHK021 La reprise vocale peut modifier le contenu manuel ; les résultats obsolètes de l'ancienne session restent toujours invalides.
- [x] CHK022 GNOME/Wayland et Windows 11 sont obligatoires dès le premier prototype et à chaque jalon ; macOS reste un objectif à vérifier.
- [x] CHK023 La fermeture respecte arrêt et invalidation, sauvegarde, puis fin de session et fermeture ; l'échec reste explicite sans perte silencieuse.
- [x] CHK024 Audio, transcription, conversation et lexique restent temporaires ; clés au trousseau ; historique manuel non persistant.
- [x] CHK025 Fournisseurs de transcription et d'interprétation configurés séparément ; Mistral visé initialement, OpenAI hors initial ; candidats non imposés.
- [x] CHK026 Aucun résultat vérifié ni seuil de performance arbitraire n'est présenté comme acquis.
- [x] CHK027 Les liens locaux, les références de scénarios et les renvois d'exigences sont valides.
- [x] CHK028 La pièce jointe, le cahier historique et la constitution sont restés inchangés.

## Notes de validation

- **Précisions U1/U3 après analyse** : créer ou ouvrir une note invalide tous les traitements précédents en cours. Une intervention ambiguë n'exécute aucune de ses actions et produit un message dans l'application ; la précision vocale est une nouvelle demande, sans reprise automatique. S2.1, S2.3, S4.1 et REQ-005/007/017 précisés ; plan, tâches, recherche, recette et checkpoints alignés. Aucune validation d'implémentation n'est revendiquée.
- **Décision U2** : la représentation du cœur est prioritaire sur l'éditeur. REQ-014 et S3.2 précisent le résultat attendu ; le plan définit acquittement/refus, resynchronisation canonique et historique manuel cohérent. T038/T039, recherche, recette et checkpoints alignés ; validation documentaire seulement, pas encore de nouvelle analyse globale ni de test produit.
- **Orientation d'interface** : fenêtre simple inspirée de WordPad, menus Fichier/Édition/Aide et bandeau de mise en forme et démarrage/arrêt vocal. REQ-021 et S4.5 définissent l'attendu ; plan, recherche et checkpoints alignés. La séparation vocal/manuel et le format Markdown sont conservés.
- **Affichages précisés par l'utilisateur** : S2.3, S8.1, REQ-007, REQ-029 et REQ-050 prévoient l'indicateur d'écoute et les ambiguïtés dans l'application uniquement, même si elle est masquée, sans indicateur hors fenêtre, notification du bureau ni premier plan imposé. L'écoute en arrière-plan et la pause à la voix restent requises ; plan, recherche, protocole de validation et checkpoints alignés.
- **Retrait du parcours 5** : section et six scénarios supprimés à la demande de l'utilisateur, sans retrait des exigences fonctionnelles correspondantes. Renvois ajustés ; identifiants des autres parcours et scénarios conservés pour éviter de les réattribuer.
- **Itération 1 — revue documentaire et contrôle des références** : le contrôle a détecté le renvoi introductif « S1.1 à S8.5 », alors que le dernier scénario était S8.3. Ce renvoi a été corrigé. Les critères d'épellation et d'information sur les services ont également été précisés pour rendre leur acceptation entièrement explicite.
- **Itération 2 — validation finale** : critères ci-dessus satisfaits après correction et nouveau contrôle des identifiants, renvois, liens, marqueurs et empreintes des trois documents protégés. Les contrôles portent sur ce que la spécification exige et permet de vérifier, pas sur la conformité d'un produit encore inexistant.
- **Preuve de séparation des modes** : « C'est un éditeur texte classique avec annulation classique » ; REQ-015 exclut explicitement toute pile d'historique vocal.
- **Preuve de distinction intention/obsolescence** : « Le texte saisi manuellement peut être modifié intentionnellement à la reprise vocale » ; REQ-017 exclut même après reprise les résultats d'une session invalidée.
- **Preuve de fermeture sûre** : REQ-036 fixe la séquence ; REQ-037 traite l'échec de sauvegarde sans reprise de capture ni disparition des modifications.
- **Preuve de prudence sur les mesures** : « Aucun seuil de délai, de mémoire, de qualité statistique ou de coût n'est ratifié ici. » Les critères mesurent une couverture de recette, une absence de pertes silencieuses et des observations de délais, sans inventer de promesse chiffrée.
- **Conception restant à planifier, non bloquante** : choix définitif des composants et modèles, modalités d'entrée volontaire dans le mode vocal, présentation interne de l'état d'écoute et des précisions, préservation des constructions Markdown, réglages optionnels et objectifs chiffrés après mesures.
- **Hypothèse explicitée** : pause et arrêt invalident les traitements non intégrés ; un changement de fournisseur ou modèle prend effet à la prochaine session. Ces choix de sûreté ne remettent pas en question les décisions déjà validées.
- **Restriction explicite** : la console externe optionnelle F70 n'est pas retenue initialement ; la vue de transcription temporaire couvre le besoin de diagnostic sans créer une conservation externe.
- **Prochaine étape** : `/speckit.plan`. Aucune clarification matérielle bloquante n'a été identifiée ; ni plan, ni tâches, ni code applicatif ne sont produits ici.

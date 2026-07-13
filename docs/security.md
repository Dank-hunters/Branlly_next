# Sécurité

## Modèle de menace initial

Branlly traite comme non fiables :

- les réponses et trames réseau d’Ollama ;
- les fichiers de mémoire chargés depuis le disque ;
- les identifiants et titres de fenêtres fournis par l’OS ;
- les arguments de lancement demandés par l’interface ;
- toutes les données franchissant la frontière Tauri.

## Mesures actives

- Ollama est limité à `localhost`, IPv4 loopback ou IPv6 loopback ;
- les URL Ollama contenant des identifiants sont refusées ;
- aucun service d’IA distant ni télémétrie n’est utilisé ;
- aucune commande n’est interpolée dans un shell ;
- les arguments d’applications restent des tableaux structurés ;
- le streaming NDJSON est validé image par image et borné par le timeout HTTP ;
- les requêtes longues sont annulables ;
- la mémoire est versionnée et revalidée avant utilisation ;
- `unsafe_code` est interdit dans les crates du workspace ;
- Tauri n’expose actuellement qu’une commande de lecture `bootstrap_status` ;
- aucun plugin shell, filesystem ou HTTP Tauri n’est activé ;
- la CSP interdit les scripts, images, polices et connexions externes ;
- les secrets, `.env`, journaux, données et artefacts sont ignorés par Git ;
- `cargo audit` et `pnpm audit --prod` font partie des contrôles locaux et CI.

## Dépendances Linux héritées

Tauri 2 utilise encore les bindings GTK3/WebKitGTK sur Linux. RustSec signale plusieurs bindings GTK3 comme non maintenus et une ancienne alerte `glib` concernant un itérateur que Branlly n’utilise pas directement. Ces alertes sont transitives et ne peuvent pas être supprimées sans remplacement du backend Linux de Tauri.

Elles doivent rester surveillées à chaque mise à jour. Une vulnérabilité RustSec exploitable fait échouer `cargo audit`; les simples avertissements de maintenance sont documentés mais ne sont pas masqués.

## Fonctions système futures

Avant d’activer lancement, fermeture de processus, Bluetooth ou réseau :

1. valider toutes les entrées ;
2. ne jamais passer par `sh -c` ;
3. appliquer le moindre privilège ;
4. demander confirmation pour les actions destructrices ;
5. annoncer une capacité uniquement quand son implémentation est disponible ;
6. ajouter des tests d’abus et de non-régression.

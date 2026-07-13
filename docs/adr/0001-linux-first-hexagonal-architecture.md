# ADR 0001 — Architecture hexagonale Linux-first

- Statut : accepté
- Date : 2026-07-13

## Contexte

La version historique est liée à PowerShell, WPF et Win32. Un port direct propagerait ces dépendances et rendrait Linux, notamment Wayland, fragile.

## Décision

Créer un second produit avec un domaine Rust indépendant de l’OS. Définir les ports système dans `branlly-platform`, implémenter Linux en premier, puis Windows. L’interface Tauri dépend des ports et du domaine, jamais l’inverse.

## Conséquences

- le cœur est testable sans bureau graphique ;
- les limitations Wayland sont représentées comme capacités, pas comme erreurs surprises ;
- davantage de code d’adaptation est nécessaire au départ ;
- la version WPF reste disponible pendant la migration.

# Développement

## Prérequis Linux

```bash
sudo ./scripts/install-linux-deps.sh
```

Installer Rust stable avec `rustfmt` et `clippy`, Node 22+ et Corepack. Le projet fixe pnpm à la version déclarée dans `apps/desktop/web/package.json`.

## Contrôle complet

```bash
./scripts/check.sh
```

Le contrôle échoue au premier problème : formatage, Clippy, tests Rust, vérification Svelte/TypeScript, tests frontend ou build Vite.

## Lancement en développement

```bash
source "$HOME/.cargo/env"
cd apps/desktop
./web/node_modules/.bin/tauri dev
```

Sur un disque monté WSL (`/mnt/c`), le premier démarrage de Vite et la première compilation GTK peuvent être lents. Les compilations suivantes utilisent le cache Cargo.

## Session de bureau

L'adaptateur choisit Wayland avant X11 lorsque les deux variables sont présentes. C'est volontaire : annoncer des capacités X11 dans une session réellement Wayland créerait des actions trompeuses.

Les capacités non implémentées restent désactivées dans le contrat envoyé au frontend.

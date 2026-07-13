# Branlly Next

Assistant de bureau multiplateforme développé Linux-first avec Rust, Tauri 2, Svelte, TypeScript et Ollama local.

## Lancer Branlly sous Linux ou WSL

Depuis l’installation locale actuelle :

```bash
ollama pull qwen2.5:3b
source "$HOME/.cargo/env"
cd /mnt/c/Users/dnkhunters/Desktop/branlly-next/apps/desktop
./web/node_modules/.bin/tauri dev
```

La première compilation Tauri/GTK peut prendre plusieurs minutes. Les lancements suivants utilisent le cache Cargo et sont plus rapides.

### Première installation après clonage

```bash
git clone git@github.com:Dank-hunters/Branlly_next.git
cd Branlly_next

sudo ./scripts/install-linux-deps.sh
source "$HOME/.cargo/env"

cd apps/desktop/web
corepack pnpm install --frozen-lockfile
cd ..
./web/node_modules/.bin/tauri dev
```

Rust stable avec `rustfmt` et `clippy`, Node.js 22+, Corepack et WSLg ou un bureau Linux graphique sont requis.

## État actuel

- fenêtre Tauri transparente sans décoration ;
- animation de Branlly avec 32 frames PNG ;
- cœur métier indépendant du système d’exploitation ;
- humeur, énergie et historique de conversation ;
- mémoire versionnée et validée ;
- client Ollama asynchrone avec streaming et annulation ;
- modèle local par défaut `qwen2.5:3b` ;
- détection X11, Wayland et environnement sans affichage ;
- interface Svelte et communication sécurisée avec Tauri ;
- chat local fonctionnel avec réponses streamées, historique et annulation ;
- menu radial avec désactivation explicite des modules indisponibles ;
- déplacement de la fenêtre depuis Branlly ;
- détection visible de la disponibilité d’Ollama et du modèle ;
- contrôles Rust, TypeScript, Svelte et sécurité automatisés.

Les fonctions système non implémentées ne sont jamais annoncées comme disponibles. Le chat nécessite une instance Ollama Linux joignable sur `127.0.0.1:11434` avec `qwen2.5:3b` installé.

## Architecture

```text
branlly-next/
├── apps/desktop/
│   ├── src-tauri/                 # application native et commandes Tauri
│   └── web/                       # interface Svelte/TypeScript
├── crates/
│   ├── branlly-core/              # domaine indépendant de l’OS
│   ├── branlly-ollama/            # client Ollama local
│   ├── branlly-platform/          # contrats système communs
│   ├── branlly-platform-linux/    # adaptateur Linux
│   └── branlly-platform-windows/  # frontière Windows isolée
├── docs/
├── scripts/
└── tests/
```

## Sécurité

- Ollama limité aux adresses loopback locales ;
- aucune télémétrie ni IA distante ;
- aucun plugin shell Tauri ;
- politique CSP restrictive ;
- aucune interpolation de commande shell ;
- `unsafe_code` interdit dans le workspace ;
- audits `cargo audit` et `pnpm audit --prod` ;
- secrets, données locales, journaux et fichiers `.env` exclus de Git.

## Vérification complète

Depuis la racine du dépôt :

```bash
./scripts/check.sh
```

Ce script exécute :

```text
cargo fmt --check
cargo clippy avec les avertissements traités comme erreurs
cargo test
cargo audit
pnpm audit
svelte-check
vitest
vite build
```

## Documentation

- [`docs/architecture.md`](docs/architecture.md)
- [`docs/development.md`](docs/development.md)
- [`docs/security.md`](docs/security.md)
- [`docs/adr`](docs/adr)

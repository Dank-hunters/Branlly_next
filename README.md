# Branlly Next

Réécriture multiplateforme de Branlly, développée Linux-first avec un cœur Rust indépendant du système et une future interface Tauri 2/Svelte.

## État

- Phase 0 : workspace, Tauri 2/Svelte, qualité logicielle et CI terminés
- Phase 1 : cœur métier indépendant de l’OS et mémoire versionnée terminés
- Phase 2 : client Ollama asynchrone, streaming NDJSON et annulation terminés
- Phase 3 : contrats système et détection X11/Wayland créés ; intégrations Linux à poursuivre
- Version Windows WPF de référence conservée séparément dans `../branlly`

## Principes

- aucune dépendance système dans `branlly-core` ;
- adaptateurs Linux et Windows séparés ;
- Ollama local, modèle par défaut `qwen2.5:3b` ;
- aucune opération réseau bloquante sur le thread UI ;
- chaque bug corrigé par un test de non-régression.

## Vérification

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## Développement Linux

```bash
sudo ./scripts/install-linux-deps.sh
./scripts/check.sh
```

L’interface Tauri transparente utilise les 32 frames de Branlly et interroge le cœur natif pour son état initial.

Voir [`docs/architecture.md`](docs/architecture.md), [`docs/development.md`](docs/development.md), [`docs/security.md`](docs/security.md) et [`docs/adr`](docs/adr).

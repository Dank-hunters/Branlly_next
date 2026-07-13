# Architecture

Branlly Next suit une architecture hexagonale : le domaine ne connaît ni l’interface, ni le réseau, ni le système d’exploitation.

## Dépendances autorisées

```text
apps/desktop -> branlly-core + branlly-ollama + branlly-platform
branlly-platform-linux   -> branlly-platform
branlly-platform-windows -> branlly-platform
branlly-ollama           -> branlly-core
branlly-core             -> aucune crate Branlly
```

Une dépendance dans le sens inverse est interdite.

## Frontières

- `branlly-core` : état, personnalité, conversation, mémoire abstraite et commandes métier.
- `branlly-ollama` : transport HTTP Ollama asynchrone et streaming.
- `branlly-platform` : contrats système et types communs.
- `branlly-platform-linux` : implémentation Linux avec capacités X11/Wayland.
- `branlly-platform-windows` : future implémentation Win32.
- `apps/desktop` : composition Tauri et interface Svelte.

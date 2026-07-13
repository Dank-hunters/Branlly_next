<script lang="ts">
  import { onMount } from 'svelte'
  import { nextFrame } from './lib/animation'
  import { fetchBootstrapStatus, isTauriRuntime, PREVIEW_STATUS } from './lib/backend'

  let frame = 0
  let paused = false
  let status = PREVIEW_STATUS
  let backendReady = false

  onMount(() => {
    let active = true
    const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches
    const timer = reducedMotion
      ? undefined
      : window.setInterval(() => {
          if (!paused) frame = nextFrame(frame)
        }, 110)

    if (isTauriRuntime()) {
      fetchBootstrapStatus()
        .then((nativeStatus) => {
          if (active) {
            status = nativeStatus
            backendReady = true
          }
        })
        .catch((error: unknown) => console.error('Native bootstrap failed', error))
    }

    return () => {
      active = false
      if (timer !== undefined) window.clearInterval(timer)
    }
  })

  function toggleAnimation() {
    paused = !paused
  }
</script>

<main class="desktop-companion" aria-label="Branlly">
  <section class="status" aria-label="État de Branlly">
    <span class:status__pulse--ready={backendReady} class="status__pulse" aria-hidden="true"></span>
    <span>BRANLLY // {backendReady ? 'LOCAL' : 'PREVIEW'}</span>
    <span class="status__model">{status.model.toUpperCase()} · E{status.energy}</span>
  </section>

  <button
    class="companion"
    type="button"
    aria-label={paused ? "Reprendre l'animation de Branlly" : "Mettre l'animation de Branlly en pause"}
    on:click={toggleAnimation}
  >
    <span class="companion__aura" aria-hidden="true"></span>
    <img src={`/assets/branlly/frame-${frame}.png`} alt="Branlly, compagnon trombone" width="182" height="180" />
  </button>

  <nav class="quick-actions" aria-label="Actions rapides">
    <button type="button" aria-label="Ouvrir le menu radial">
      <span aria-hidden="true">◉</span>
      MENU
    </button>
    <button type="button" aria-label="Ouvrir le chat local">
      <span aria-hidden="true">⌁</span>
      CHAT
    </button>
  </nav>

  <p class="hint">CLIC : PAUSE · GLISSER : DÉPLACER</p>
</main>

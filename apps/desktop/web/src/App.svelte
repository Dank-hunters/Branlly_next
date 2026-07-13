<script lang="ts">
  import { LogicalSize } from '@tauri-apps/api/dpi'
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import { onMount } from 'svelte'
  import { nextFrame } from './lib/animation'
  import {
    cancelChat,
    fetchBootstrapStatus,
    isTauriRuntime,
    PREVIEW_STATUS,
    sendChat,
    type ChatEvent,
  } from './lib/backend'

  type View = 'pet' | 'menu' | 'chat' | 'games' | 'game'
  type UiMessage = { role: 'user' | 'assistant' | 'error'; content: string }

  let frame = 0
  let paused = false
  let status = PREVIEW_STATUS
  let backendReady = false
  let view: View = 'pet'
  let input = ''
  let busy = false
  let messages: UiMessage[] = []
  let activeGame = ''

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

  function startDragging(event: PointerEvent) {
    if (backendReady && event.button === 0) {
      getCurrentWindow().startDragging().catch((error: unknown) =>
        console.error('Window dragging failed', error),
      )
    }
  }

  function receiveChatEvent(event: ChatEvent) {
    if (event.type === 'delta') {
      const copy = [...messages]
      const last = copy.at(-1)
      if (last?.role === 'assistant') {
        copy[copy.length - 1] = { ...last, content: last.content + event.payload }
        messages = copy
      }
    } else if (event.type === 'error') {
      messages = [...messages, { role: 'error', content: event.payload }]
    }
  }

  async function submitChat() {
    const outgoing = input.trim()
    if (!outgoing || busy || !status.ollamaAvailable) return
    input = ''
    busy = true
    messages = [
      ...messages,
      { role: 'user', content: outgoing },
      { role: 'assistant', content: '' },
    ]
    try {
      await sendChat(outgoing, receiveChatEvent)
      status = await fetchBootstrapStatus()
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error)
      if (messages.at(-1)?.role !== 'error') {
        messages = [...messages, { role: 'error', content: detail }]
      }
    } finally {
      busy = false
    }
  }

  async function stopChat() {
    await cancelChat().catch((error: unknown) => console.error('Chat cancellation failed', error))
  }

  async function changeView(next: View) {
    view = next
    if (backendReady) {
      const expanded = next === 'game'
      await getCurrentWindow()
        .setSize(new LogicalSize(expanded ? 960 : 390, expanded ? 700 : 390))
        .catch((error: unknown) => console.error('Window resize failed', error))
    }
  }

  async function launchGame(path: string) {
    activeGame = path
    await changeView('game')
  }
</script>

<main class:desktop-companion--expanded={view === 'game'} class="desktop-companion" aria-label="Branlly">
  <section class="status" aria-label="État de Branlly">
    <span class:status__pulse--ready={status.ollamaAvailable} class="status__pulse" aria-hidden="true"></span>
    <span>BRANLLY // {status.ollamaAvailable ? 'LOCAL' : backendReady ? 'OLLAMA OFF' : 'PREVIEW'}</span>
    <span class="status__model">{status.model.toUpperCase()} · E{status.energy}</span>
  </section>

  {#if view === 'pet'}
    <button
      class="companion"
      type="button"
      aria-label={paused ? "Reprendre l'animation de Branlly" : "Mettre l'animation de Branlly en pause"}
      on:pointerdown={startDragging}
      on:click={toggleAnimation}
    >
      <span class="companion__aura" aria-hidden="true"></span>
      <img src={`/assets/branlly/frame-${frame}.png`} alt="Branlly, compagnon trombone" width="182" height="180" />
    </button>

    <nav class="quick-actions" aria-label="Actions rapides">
      <button type="button" aria-label="Ouvrir le menu radial" on:click={() => (view = 'menu')}>
        <span aria-hidden="true">◉</span> MENU
      </button>
      <button type="button" aria-label="Ouvrir le chat local" on:click={() => (view = 'chat')}>
        <span aria-hidden="true">⌁</span> CHAT
      </button>
    </nav>
    <p class="hint">CLIC : PAUSE · GLISSER : DÉPLACER</p>
  {:else if view === 'menu'}
    <section class="radial" aria-label="Menu radial">
      <div class="radial__rings" aria-hidden="true"></div>
      <button class="radial__item radial__item--top" type="button" on:click={() => changeView('chat')}>CHAT</button>
      <button class="radial__item radial__item--right" type="button" disabled>SYSTÈME</button>
      <button class="radial__item radial__item--bottom" type="button" on:click={() => changeView('games')}>JEUX</button>
      <button class="radial__item radial__item--left" type="button" disabled>APPS</button>
      <button class="radial__core" type="button" aria-label="Fermer le menu" on:click={() => changeView('pet')}>B</button>
      <p>MODULES INDISPONIBLES MASQUÉS</p>
    </section>
  {:else if view === 'chat'}
    <section class="chat" aria-label="Chat local">
      <header>
        <button type="button" aria-label="Fermer le chat" on:click={() => changeView('pet')}>‹</button>
        <div><strong>BRANLLY LINK</strong><small>{status.ollamaAvailable ? 'OLLAMA LOCAL' : 'MOTEUR INDISPONIBLE'}</small></div>
        {#if busy}<button type="button" class="stop" on:click={stopChat}>STOP</button>{/if}
      </header>
      <div class="chat__history" aria-live="polite">
        {#if messages.length === 0}
          <p class="empty">Hmm. Tu avais besoin de quelque chose ?</p>
        {/if}
        {#each messages as message}
          {#if message.content}<p class:assistant={message.role === 'assistant'} class:user={message.role === 'user'} class:error={message.role === 'error'}>{message.content}</p>{/if}
        {/each}
        {#if busy && messages.at(-1)?.content === ''}<p class="thinking">ANALYSE…</p>{/if}
      </div>
      <form on:submit|preventDefault={submitChat}>
        <input bind:value={input} maxlength="4000" disabled={!status.ollamaAvailable || busy} placeholder={status.ollamaAvailable ? 'Écrire un message…' : 'Ollama indisponible'} aria-label="Message" />
        <button type="submit" disabled={!status.ollamaAvailable || busy || !input.trim()}>ENVOYER</button>
      </form>
    </section>
  {:else if view === 'games'}
    <section class="game-picker" aria-label="Mini-jeux">
      <header><button type="button" on:click={() => changeView('menu')}>‹</button><strong>DIVERTISSEMENT</strong></header>
      <button type="button" on:click={() => launchGame('/games/subwaylike/index.html')}><strong>METRO RUSH</strong><span>Course urbaine</span></button>
      <button type="button" on:click={() => launchGame('/games/blockcraft-lite/index.html')}><strong>BLOCKCRAFT LITE</strong><span>Construction voxel</span></button>
    </section>
  {:else}
    <section class="game-host" aria-label="Mini-jeu actif">
      <header><button type="button" on:click={() => changeView('games')}>FERMER LE JEU</button></header>
      <iframe src={activeGame} title="Mini-jeu Branlly"></iframe>
    </section>
  {/if}
</main>

<script lang="ts">
  import { LogicalPosition, LogicalSize } from '@tauri-apps/api/dpi'
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import { onMount } from 'svelte'
  import { nextFrame } from './lib/animation'
  import {
    cancelChat,
    cleanupTemporaryFiles,
    closeOpenWindow,
    fetchBootstrapStatus,
    focusOpenWindow,
    getPointerPosition,
    getSystemSnapshot,
    isTauriRuntime,
    launchShortcut,
    listOpenWindows,
    PREVIEW_STATUS,
    searchWikipedia,
    sendChat,
    type ChatEvent,
    type SystemSnapshot,
    type WikiResult,
    type WindowInfo,
  } from './lib/backend'

  type View = 'pet' | 'menu' | 'chat' | 'games' | 'game' | 'apps' | 'windows' | 'system' | 'search'
  type UiMessage = { role: 'user' | 'assistant' | 'error'; content: string }
  type MenuPage = 'main' | 'ia' | 'applications' | 'games' | 'pc' | 'network' | 'cleanup' | 'branlly' | 'appearance'
  type RadialItem = { label: string; action: string; color?: string; disabled?: boolean }

  const radialMenus: Record<MenuPage, { title: string; description: string; items: RadialItem[] }> = {
    main: { title: 'SYSTEM MENU', description: '< SELECT >', items: [
      { label: 'IA', action: 'menu:ia', color: '#7dd3fc' },
      { label: 'APPS', action: 'menu:applications', color: '#60a5fa' },
      { label: 'JEUX', action: 'menu:games', color: '#fdba74' },
      { label: 'PC', action: 'menu:pc', color: '#2dd4bf' },
      { label: 'BRANLLY', action: 'menu:branlly', color: '#f472b6' },
      { label: 'REDÉMARRER', action: 'restart', color: '#f87171' },
      { label: 'QUITTER', action: 'quit', color: '#ef4444' },
    ]},
    ia: { title: 'IA', description: 'Discussion, recherche et messages', items: [
      { label: 'LOCAL', action: 'chat' }, { label: 'OPENAI', action: 'unavailable', disabled: true },
      { label: 'WIKI', action: 'search' }, { label: 'TERMINAL', action: 'unavailable', disabled: true },
      { label: 'RETOUR', action: 'menu:main' },
    ]},
    applications: { title: 'APPLICATIONS', description: 'Raccourcis vers tes applis', items: [
      { label: 'DISCORD', action: 'shortcut:discord', color: '#5865f2' },
      { label: 'YT MUSIC', action: 'shortcut:youtube-music', color: '#ff0033' },
      { label: 'TWITCH', action: 'shortcut:twitch', color: '#9146ff' },
      { label: 'STREMIO', action: 'shortcut:stremio', color: '#6c5ce7' },
      { label: 'DISNEY+', action: 'shortcut:disney', color: '#113ccf' },
      { label: 'RETOUR', action: 'menu:main' },
    ]},
    games: { title: 'JEUX', description: 'Mini-jeux et bordel interactif', items: [
      { label: 'TOUS', action: 'games' }, { label: 'METRO', action: 'game:metro' },
      { label: 'BLOCKCRAFT', action: 'game:blockcraft' }, { label: 'RETOUR', action: 'menu:main' },
    ]},
    pc: { title: 'PC', description: 'Contrôle système rangé par blocs', items: [
      { label: 'RÉSEAU', action: 'menu:network' }, { label: 'NETTOYAGE', action: 'menu:cleanup' },
      { label: 'TÂCHES', action: 'shortcut:task-manager' }, { label: 'PÉRIPHÉRIQUES', action: 'system' },
      { label: 'DIAGNOSTIC', action: 'system' }, { label: 'RETOUR', action: 'menu:main' },
    ]},
    network: { title: 'RÉSEAU', description: 'Tout ce qui connecte ce PC', items: [
      { label: 'WI-FI', action: 'shortcut:wifi-settings' }, { label: 'BLUETOOTH', action: 'shortcut:bluetooth-settings' },
      { label: 'CARTES', action: 'shortcut:network-adapters' }, { label: 'APPAREILS', action: 'system' },
      { label: 'RETOUR', action: 'menu:pc' },
    ]},
    cleanup: { title: 'NETTOYAGE', description: 'Faire le ménage, malheureusement', items: [
      { label: 'TEMP', action: 'cleanup' }, { label: 'CORBEILLE', action: 'shortcut:recycle-bin' },
      { label: 'DISQUE', action: 'shortcut:disk-cleanup' }, { label: 'RETOUR', action: 'menu:pc' },
    ]},
    branlly: { title: 'BRANLLY', description: 'Réglages du compagnon', items: [
      { label: 'APPARENCE', action: 'menu:appearance' }, { label: 'HUMEUR', action: 'mood' },
      { label: 'SUIVI', action: 'follow' }, { label: 'SON', action: 'sound' },
      { label: 'REDÉMARRER', action: 'restart' }, { label: 'RETOUR', action: 'menu:main' },
    ]},
    appearance: { title: 'APPARENCE', description: 'Choisis l’apparence de Branlly', items: [
      { label: 'TROMBONE FLEMMARD', action: 'appearance' }, { label: 'RETOUR', action: 'menu:branlly' },
    ]},
  }

  let frame = 0
  let status = PREVIEW_STATUS
  let backendReady = false
  let view: View = 'pet'
  let input = ''
  let busy = false
  let messages: UiMessage[] = []
  let activeGame = ''
  let windows: WindowInfo[] = []
  let system: SystemSnapshot | null = null
  let wikiQuery = ''
  let wikiResults: WikiResult[] = []
  let moduleError = ''
  let dragTimer: number | undefined
  let dragging = false
  let followCursor = false
  let soundEnabled = true
  let menuPage: MenuPage = 'main'

  onMount(() => {
    let active = true
    const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches
    const timer = reducedMotion
      ? undefined
      : window.setInterval(() => {
          frame = nextFrame(frame)
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

    const followTimer = window.setInterval(async () => {
      if (!followCursor || view !== 'pet' || !status.capabilities.canFollowPointer) return
      try {
        const pointer = await getPointerPosition()
        await getCurrentWindow().setPosition(new LogicalPosition(pointer.x + 18, pointer.y + 18))
      } catch (error) {
        followCursor = false
        console.error('Pointer following failed', error)
      }
    }, 650)
    window.addEventListener('click', playUiSound, true)
    return () => {
      active = false
      if (timer !== undefined) window.clearInterval(timer)
      window.clearInterval(followTimer)
      window.removeEventListener('click', playUiSound, true)
    }
  })

  function beginDragging(event: PointerEvent) {
    if (!backendReady || event.button !== 0) return
    dragging = false
    dragTimer = window.setTimeout(() => {
      dragging = true
      getCurrentWindow().startDragging().catch((error: unknown) =>
        console.error('Window dragging failed', error),
      )
    }, 180)
  }

  function endDragging() {
    if (dragTimer !== undefined) window.clearTimeout(dragTimer)
    dragTimer = undefined
  }

  function openMainMenu() {
    if (dragging) {
      dragging = false
      return
    }
    menuPage = 'main'
    void changeView('menu')
  }

  async function showMenu(page: MenuPage) {
    menuPage = page
    await changeView('menu')
  }

  async function handleRadialAction(action: string) {
    if (action.startsWith('menu:')) {
      menuPage = action.slice(5) as MenuPage
    } else if (action.startsWith('shortcut:')) {
      const shortcut = action.slice(9)
      if (shortcut === 'recycle-bin' && !window.confirm('Vider complètement la corbeille ?')) return
      await runShortcut(shortcut)
    } else if (action === 'chat') {
      await changeView('chat')
    } else if (action === 'search') {
      await changeView('search')
    } else if (action === 'games') {
      await changeView('games')
    } else if (action === 'game:metro') {
      await launchGame('/games/subwaylike/index.html')
    } else if (action === 'game:blockcraft') {
      await launchGame('/games/blockcraft-lite/index.html')
    } else if (action === 'windows') {
      await showWindows()
    } else if (action === 'system' || action === 'mood') {
      await showSystem()
    } else if (action === 'cleanup') {
      await cleanTemporaryFiles()
    } else if (action === 'follow') {
      followCursor = !followCursor
    } else if (action === 'sound') {
      soundEnabled = !soundEnabled
    } else if (action === 'appearance') {
      moduleError = 'Trombone flemmard sélectionné.'
    } else if (action === 'restart') {
      window.location.reload()
    } else if (action === 'quit') {
      await closeBranlly()
    }
  }

  async function closeBranlly() {
    await getCurrentWindow().close()
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
      const size = next === 'game' ? [960, 700] : next === 'menu' ? [560, 590] : [390, 390]
      await getCurrentWindow()
        .setSize(new LogicalSize(size[0], size[1]))
        .catch((error: unknown) => console.error('Window resize failed', error))
    }
  }

  async function launchGame(path: string) {
    activeGame = path
    await changeView('game')
  }

  async function showWindows() {
    moduleError = ''
    try {
      windows = await listOpenWindows()
      await changeView('windows')
    } catch (error) {
      moduleError = String(error)
    }
  }

  async function refreshSystem() {
    moduleError = ''
    try {
      system = await getSystemSnapshot()
    } catch (error) {
      moduleError = String(error)
    }
  }

  async function showSystem() {
    await changeView('system')
    await refreshSystem()
  }

  async function runShortcut(id: string) {
    moduleError = ''
    try { await launchShortcut(id) } catch (error) { moduleError = String(error) }
  }

  async function activateWindow(id: string) {
    try { await focusOpenWindow(id) } catch (error) { moduleError = String(error) }
  }

  async function terminateWindow(id: string) {
    try {
      await closeOpenWindow(id)
      windows = await listOpenWindows()
    } catch (error) { moduleError = String(error) }
  }

  async function runWikiSearch() {
    moduleError = ''
    try { wikiResults = await searchWikipedia(wikiQuery) } catch (error) { moduleError = String(error) }
  }

  async function cleanTemporaryFiles() {
    if (!window.confirm('Supprimer les éléments temporaires inutilisés depuis plus de 24 heures ?')) return
    try {
      const report = await cleanupTemporaryFiles()
      moduleError = `${report.removedEntries} élément(s) temporaire(s) supprimé(s).`
    } catch (error) { moduleError = String(error) }
  }

  function playUiSound() {
    if (!soundEnabled) return
    const context = new AudioContext()
    const oscillator = context.createOscillator()
    const gain = context.createGain()
    oscillator.frequency.value = 520
    gain.gain.setValueAtTime(0.025, context.currentTime)
    gain.gain.exponentialRampToValueAtTime(0.0001, context.currentTime + 0.055)
    oscillator.connect(gain).connect(context.destination)
    oscillator.start()
    oscillator.stop(context.currentTime + 0.055)
    oscillator.onended = () => context.close()
  }
</script>

<main class:desktop-companion--expanded={view === 'game'} class:desktop-companion--menu={view === 'menu'} class="desktop-companion" aria-label="Branlly">
  {#if view === 'pet'}
    <button
      class="companion"
      type="button"
      aria-label="Ouvrir le menu de Branlly"
      on:pointerdown={beginDragging}
      on:pointerup={endDragging}
      on:pointercancel={endDragging}
      on:contextmenu|preventDefault={showWindows}
      on:click={openMainMenu}
    >
      <span class="companion__aura" aria-hidden="true"></span>
      <img src={`/assets/branlly/frame-${frame}.png`} alt="Branlly, compagnon trombone" width="182" height="180" />
    </button>

  {:else if view === 'menu'}
    <section class="radial" aria-label={`Menu ${radialMenus[menuPage].title}`}>
      <div class="radial__rings" aria-hidden="true"></div>
      <span class="radial__title">. {radialMenus[menuPage].title} .</span>
      <span class="radial__telemetry">{status.mood.toUpperCase()} // ENERGY {status.energy}</span>
      {#each radialMenus[menuPage].items as item, index}
        <button
          class="radial__item"
          type="button"
          disabled={item.disabled}
          style={`--angle:${-90 + index * (360 / radialMenus[menuPage].items.length)}deg;--accent:${item.color ?? '#38bdf8'}`}
          on:click={() => handleRadialAction(item.action)}
        >{item.label}</button>
      {/each}
      <button class="radial__core" type="button" aria-label="Retour au menu principal" on:click={() => (menuPage = 'main')}>{menuPage === 'main' ? 'B' : 'CORE'}</button>
      <button class="radial__exit" type="button" aria-label="Fermer le menu" on:click={() => changeView('pet')}>×</button>
      <p>{radialMenus[menuPage].description}</p>
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
      <header><button type="button" on:click={() => showMenu('games')}>‹</button><strong>DIVERTISSEMENT</strong></header>
      <button type="button" on:click={() => launchGame('/games/subwaylike/index.html')}><strong>METRO RUSH</strong><span>Course urbaine</span></button>
      <button type="button" on:click={() => launchGame('/games/blockcraft-lite/index.html')}><strong>BLOCKCRAFT LITE</strong><span>Construction voxel</span></button>
    </section>
  {:else if view === 'apps'}
    <section class="module-panel" aria-label="Applications">
      <header><button type="button" on:click={() => changeView('menu')}>‹</button><strong>APPLICATIONS</strong></header>
      <div class="module-grid">
        {#each [['discord','DISCORD'],['steam','STEAM'],['twitch','TWITCH'],['youtube-music','YOUTUBE MUSIC'],['stremio','STREMIO'],['disney','DISNEY+']] as shortcut}
          <button type="button" on:click={() => runShortcut(shortcut[0])}>{shortcut[1]}</button>
        {/each}
      </div>
      <button class="wide-action" type="button" on:click={showWindows}>FENÊTRES OUVERTES</button>
      <button class="wide-action" type="button" on:click={() => changeView('search')}>RECHERCHE WIKIPÉDIA</button>
      {#if moduleError}<p class="module-error">{moduleError}</p>{/if}
    </section>
  {:else if view === 'windows'}
    <section class="module-panel" aria-label="Fenêtres ouvertes">
      <header><button type="button" on:click={() => showMenu('applications')}>‹</button><strong>FENÊTRES OUVERTES</strong></header>
      <div class="window-list">
        {#each windows as window}
          <div><button type="button" on:click={() => activateWindow(window.id)}><strong>{window.applicationId ?? 'APP'}</strong><span>{window.title}</span></button><button class="close-window" aria-label={`Fermer ${window.title}`} type="button" on:click={() => terminateWindow(window.id)}>×</button></div>
        {:else}<p>Aucune fenêtre détectée ou fonction indisponible sous Wayland.</p>{/each}
      </div>
      {#if moduleError}<p class="module-error">{moduleError}</p>{/if}
    </section>
  {:else if view === 'system'}
    <section class="module-panel" aria-label="État système">
      <header><button type="button" on:click={() => showMenu('pc')}>‹</button><strong>SYSTÈME</strong><button type="button" on:click={refreshSystem}>↻</button></header>
      <div class="system-card"><span>RÉSEAU</span><strong>{system?.network?.toUpperCase() ?? 'ANALYSE'}</strong></div>
      <button class="wide-action" type="button" disabled={!status.capabilities.canFollowPointer} on:click={() => (followCursor = !followCursor)}>{followCursor ? 'ARRÊTER LE SUIVI DU CURSEUR' : 'SUIVRE LE CURSEUR'}</button>
      <button class="wide-action" type="button" on:click={cleanTemporaryFiles}>NETTOYER LES FICHIERS TEMPORAIRES</button>
      <h3>BLUETOOTH</h3>
      <div class="device-list">{#each system?.bluetoothDevices ?? [] as device}<p><span>{device.name}</span><strong>{device.connected ? 'CONNECTÉ' : 'CONNU'}</strong></p>{:else}<p>Aucun appareil ou service indisponible.</p>{/each}</div>
      <h3>PÉRIPHÉRIQUES CONNECTÉS</h3>
      <div class="device-list">{#each system?.connectedDevices ?? [] as device}<p><span>{device.name}</span><strong>ACTIF</strong></p>{:else}<p>Aucun périphérique détecté.</p>{/each}</div>
      {#if moduleError}<p class="module-error">{moduleError}</p>{/if}
    </section>
  {:else if view === 'search'}
    <section class="module-panel" aria-label="Recherche Wikipédia">
      <header><button type="button" on:click={() => showMenu('ia')}>‹</button><strong>WIKIPÉDIA</strong></header>
      <form class="wiki-form" on:submit|preventDefault={runWikiSearch}><input bind:value={wikiQuery} maxlength="120" placeholder="Rechercher…" aria-label="Recherche Wikipédia"/><button type="submit">CHERCHER</button></form>
      <div class="wiki-results">{#each wikiResults as result}<a href={result.url} target="_blank" rel="noreferrer"><strong>{result.title}</strong><span>{result.description}</span></a>{/each}</div>
      {#if moduleError}<p class="module-error">{moduleError}</p>{/if}
    </section>
  {:else}
    <section class="game-host" aria-label="Mini-jeu actif">
      <header><button type="button" on:click={() => changeView('games')}>FERMER LE JEU</button></header>
      <iframe src={activeGame} title="Mini-jeu Branlly"></iframe>
    </section>
  {/if}
</main>

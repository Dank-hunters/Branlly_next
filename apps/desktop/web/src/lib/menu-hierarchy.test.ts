import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const appSource = readFileSync(fileURLToPath(new URL('../App.svelte', import.meta.url)), 'utf8')

describe('historical Branlly menu hierarchy', () => {
  it.each([
    ['main', ['IA', 'APPS', 'JEUX', 'PC', 'BRANLLY', 'REDÉMARRER', 'QUITTER']],
    ['ia', ['LOCAL', 'OPENAI', 'WIKI', 'TERMINAL', 'RETOUR']],
    ['applications', ['DISCORD', 'YT MUSIC', 'TWITCH', 'STREMIO', 'DISNEY+', 'RETOUR']],
    ['pc', ['RÉSEAU', 'NETTOYAGE', 'TÂCHES', 'PÉRIPHÉRIQUES', 'DIAGNOSTIC', 'RETOUR']],
    ['network', ['WI-FI', 'BLUETOOTH', 'CARTES', 'APPAREILS', 'RETOUR']],
    ['cleanup', ['TEMP', 'CORBEILLE', 'DISQUE', 'RETOUR']],
    ['branlly', ['APPARENCE', 'HUMEUR', 'SUIVI', 'SON', 'REDÉMARRER', 'RETOUR']],
  ])('keeps the %s menu labels', (_menu, labels) => {
    for (const label of labels) expect(appSource).toContain(`label: '${label}'`)
  })

  it('keeps intentionally removed games out of the hierarchy', () => {
    expect(appSource).not.toContain("label: 'WHIP'")
    expect(appSource).not.toContain("label: 'PONG'")
  })
})

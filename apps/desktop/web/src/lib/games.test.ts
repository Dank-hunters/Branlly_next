import { existsSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const publicRoot = new URL('../../public/', import.meta.url)

describe('bundled Branlly games', () => {
  it.each(['games/subwaylike/index.html', 'games/blockcraft-lite/index.html'])(
    'ships %s in every desktop package',
    (relativePath) => {
      expect(existsSync(fileURLToPath(new URL(relativePath, publicRoot)))).toBe(true)
    },
  )
})

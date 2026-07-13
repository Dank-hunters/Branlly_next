import { describe, expect, it, vi } from 'vitest'

import { fetchBootstrapStatus, isTauriRuntime, PREVIEW_STATUS, validateChatMessage } from './backend'

describe('native bootstrap contract', () => {
  it('uses the exact Tauri command and validates its response', async () => {
    const invoke = vi.fn().mockResolvedValue(PREVIEW_STATUS)

    await expect(fetchBootstrapStatus(invoke)).resolves.toEqual(PREVIEW_STATUS)
    expect(invoke).toHaveBeenCalledWith('bootstrap_status')
  })

  it('rejects corrupted energy from the native boundary', async () => {
    const invoke = vi.fn().mockResolvedValue({ ...PREVIEW_STATUS, energy: 101 })

    await expect(fetchBootstrapStatus(invoke)).rejects.toThrow(/invalid Branlly status/i)
  })

  it('normalizes chat input and rejects invalid sizes before IPC', () => {
    expect(validateChatMessage('  Bonjour  ')).toBe('Bonjour')
    expect(() => validateChatMessage('   ')).toThrow(RangeError)
    expect(() => validateChatMessage('x'.repeat(4_001))).toThrow(RangeError)
  })

  it('detects native runtime without user-agent heuristics', () => {
    expect(isTauriRuntime({ __TAURI_INTERNALS__: {} })).toBe(true)
    expect(isTauriRuntime({})).toBe(false)
  })
})

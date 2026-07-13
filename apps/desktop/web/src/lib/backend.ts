import { invoke } from '@tauri-apps/api/core'

export type Mood = 'sleepy' | 'neutral' | 'curious' | 'happy' | 'irritated'

export interface PlatformCapabilities {
  canListWindows: boolean
  canFocusWindows: boolean
  canPositionOverlay: boolean
  canFollowPointer: boolean
  canQueryNetwork: boolean
  canQueryBluetooth: boolean
}

export interface BootstrapStatus {
  model: string
  mood: Mood
  energy: number
  capabilities: PlatformCapabilities
}

export type InvokeBackend = <T>(command: string) => Promise<T>

export const PREVIEW_STATUS: BootstrapStatus = {
  model: 'qwen2.5:3b',
  mood: 'neutral',
  energy: 65,
  capabilities: {
    canListWindows: false,
    canFocusWindows: false,
    canPositionOverlay: false,
    canFollowPointer: false,
    canQueryNetwork: false,
    canQueryBluetooth: false,
  },
}

export async function fetchBootstrapStatus(
  invokeBackend: InvokeBackend = invoke,
): Promise<BootstrapStatus> {
  const status = await invokeBackend<BootstrapStatus>('bootstrap_status')
  if (!status.model.trim() || !Number.isInteger(status.energy) || status.energy < 0 || status.energy > 100) {
    throw new Error('Backend returned an invalid Branlly status')
  }
  return status
}

export function isTauriRuntime(target: object = globalThis): boolean {
  return '__TAURI_INTERNALS__' in target
}

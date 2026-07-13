import { Channel, invoke } from '@tauri-apps/api/core'

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
  ollamaAvailable: boolean
}

export type InvokeBackend = <T>(command: string, args?: Record<string, unknown>) => Promise<T>

export type ChatEvent =
  | { type: 'delta'; payload: string }
  | { type: 'complete' }
  | { type: 'error'; payload: string }

export const PREVIEW_STATUS: BootstrapStatus = {
  model: 'qwen2.5:3b',
  mood: 'neutral',
  energy: 65,
  ollamaAvailable: false,
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

export function validateChatMessage(message: string): string {
  const normalized = message.trim()
  if (!normalized || [...normalized].length > 4_000) {
    throw new RangeError('Le message doit contenir entre 1 et 4000 caractères.')
  }
  return normalized
}

export async function sendChat(
  message: string,
  receive: (event: ChatEvent) => void,
): Promise<void> {
  const channel = new Channel<ChatEvent>()
  channel.onmessage = receive
  await invoke('chat', { message: validateChatMessage(message), onEvent: channel })
}

export async function cancelChat(): Promise<void> {
  await invoke('cancel_chat')
}

export function isTauriRuntime(target: object = globalThis): boolean {
  return '__TAURI_INTERNALS__' in target
}

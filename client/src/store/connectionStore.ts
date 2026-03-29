import { create } from 'zustand'
import { ConnectionStatusType } from '../types/connection'

interface ConnectionState {
  status: ConnectionStatusType
  progress: number
  stage: string | null
  relayUrl: string
  error: string | null

  // Used by useSecureRoute hook
  setConnecting: () => void
  updateProgress: (progress: number, stage: string) => void
  setEstablished: () => void
  setError: (msg: string) => void
  reset: () => void

  // Used by relay connection
  connect: () => Promise<void>
  disconnect: () => Promise<void>
  pollStatus: () => Promise<void>
}

// Helper: try to invoke Tauri command, return fallback if not in Tauri context
async function tryInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    return await invoke<T>(cmd, args)
  } catch {
    return null
  }
}

const DEFAULT_RELAY = 'wss://relay.hades.im/v1/ws'

export const useConnectionStore = create<ConnectionState>((set, get) => ({
  status: 'idle',
  progress: 0,
  stage: null,
  relayUrl: DEFAULT_RELAY,
  error: null,

  setConnecting: () => {
    set({ status: 'connecting', progress: 0, stage: null, error: null })
  },

  updateProgress: (progress: number, stage: string) => {
    set({ progress, stage, status: 'establishing' })
  },

  setEstablished: () => {
    set({ status: 'established', progress: 100, stage: 'Secure route established' })
  },

  setError: (msg: string) => {
    set({ status: 'error', error: msg })
  },

  reset: () => {
    set({ status: 'idle', progress: 0, stage: null, error: null })
  },

  connect: async () => {
    set({ status: 'connecting', error: null })
    try {
      await tryInvoke('connect_relay', { relayUrl: get().relayUrl })
      set({ status: 'established' })
    } catch (err) {
      set({ status: 'error', error: String(err) })
    }
  },

  disconnect: async () => {
    await tryInvoke('disconnect_relay')
    set({ status: 'idle', progress: 0, stage: null })
  },

  pollStatus: async () => {
    try {
      const status = await tryInvoke<string>('get_connection_status')
      if (status) {
        set({ status: status as ConnectionStatusType })
      }
    } catch (_) {
      set({ status: 'idle' })
    }
  },
}))

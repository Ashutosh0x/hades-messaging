import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error'

interface ConnectionState {
  status: ConnectionStatus
  relayUrl: string
  error: string | null

  connect: () => Promise<void>
  disconnect: () => Promise<void>
  pollStatus: () => Promise<void>
}

const DEFAULT_RELAY = 'wss://relay.hades.im/v1/ws'

export const useConnectionStore = create<ConnectionState>((set, get) => ({
  status: 'disconnected',
  relayUrl: DEFAULT_RELAY,
  error: null,

  connect: async () => {
    set({ status: 'connecting', error: null })
    try {
      await invoke('connect_relay', { relayUrl: get().relayUrl })
      set({ status: 'connected' })
    } catch (err) {
      set({ status: 'error', error: String(err) })
    }
  },

  disconnect: async () => {
    try {
      await invoke('disconnect_relay')
    } catch (_) { /* ignore */ }
    set({ status: 'disconnected' })
  },

  pollStatus: async () => {
    try {
      const status = await invoke<string>('get_connection_status')
      set({ status: status as ConnectionStatus })
    } catch (_) {
      set({ status: 'disconnected' })
    }
  },
}))

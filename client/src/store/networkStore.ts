import { create } from 'zustand'

// M3 FIX: Safe invoke + listen wrappers — no crash in browser dev mode
async function tryInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    return await invoke<T>(cmd, args)
  } catch {
    return null
  }
}

async function tryListen(event: string, handler: (payload: any) => void): Promise<() => void> {
  try {
    const { listen } = await import('@tauri-apps/api/event')
    const unlisten = await listen(event, (e) => handler(e.payload))
    return unlisten
  } catch {
    return () => {}
  }
}

export type TorStatus = 'disconnected' | 'connecting' | 'building' | 'connected' | 'error'

interface NetworkState {
  status: TorStatus
  progress: number
  latency: number
  activeNode: string | null
  relayConnected: boolean

  setStatus: (status: TorStatus, progress?: number) => void
  setLatency: (ms: number) => void
  initListeners: () => Promise<void>
}

export const useNetworkStore = create<NetworkState>((set) => ({
  status: 'disconnected',   // M3 FIX: was 'building'
  progress: 0,              // M3 FIX: was 34
  latency: 0,
  activeNode: null,
  relayConnected: false,

  setStatus: (status, progress = 0) => set({ status, progress }),
  setLatency: (ms) => set({ latency: ms }),

  // M3 FIX: Real event listeners instead of no-op stubs
  initListeners: async () => {
    await tryListen('tor-status-change', (payload: {
      state: TorStatus
      progress: number
    }) => {
      set({ status: payload.state, progress: payload.progress })
    })

    await tryListen('tor-latency-update', (payload: number) => {
      set({ latency: payload })
    })

    await tryListen('tor-node-change', (payload: string) => {
      set({ activeNode: payload })
    })

    await tryListen('relay-status', (payload: { connected: boolean }) => {
      set({ relayConnected: payload.connected })
    })

    // Get initial relay status from backend
    const connStatus = await tryInvoke<string>('get_connection_status')
    if (connStatus) {
      set({
        relayConnected: connStatus === 'connected',
        status: connStatus === 'connected' ? 'connected' : 'disconnected',
        progress: connStatus === 'connected' ? 100 : 0,
      })
    }
  },
}))

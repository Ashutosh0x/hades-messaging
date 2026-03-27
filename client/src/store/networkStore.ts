import { create } from 'zustand'

// No-op stub for browser dev mode — real Tauri event listeners only work inside the Tauri runtime.
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const listen = async <T>(_event: string, _cb: (e: { payload: T }) => void) => {
  return { unlisten: () => {} }
}

export type TorStatus = 'disconnected' | 'building' | 'connected'

interface NetworkState {
  status: TorStatus
  progress: number // 0 to 100
  latency: number
  activeNode: string | null
  
  setStatus: (status: TorStatus, progress?: number) => void
  setLatency: (ms: number) => void
  initListeners: () => void
}

export const useNetworkStore = create<NetworkState>((set) => ({
  status: 'building',
  progress: 34, // Mock initial state
  latency: 0,
  activeNode: null,

  setStatus: (status, progress = 0) => set({ status, progress }),
  setLatency: (ms) => set({ latency: ms }),

  initListeners: () => {
    listen<{ state: TorStatus; progress: number }>('tor-status-change', (event: any) => {
      set({ status: event.payload.state, progress: event.payload.progress })
    })

    listen<number>('tor-latency-update', (event: any) => {
      set({ latency: event.payload })
    })

    listen<string>('tor-node-change', (event: any) => {
      set({ activeNode: event.payload })
    })
  }
}))

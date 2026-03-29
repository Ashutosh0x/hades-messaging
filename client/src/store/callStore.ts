import { create } from 'zustand'

// ── Types ──
export interface CallEntry {
  id: string
  contactId: string
  contactName: string
  type: 'voice' | 'video'
  direction: 'incoming' | 'outgoing' | 'missed'
  duration: number | null   // seconds, null if missed
  timestamp: string
}

interface CallStore {
  history: CallEntry[]
  isLoading: boolean
  error: string | null
  activeCall: {
    contactId: string
    contactName: string
    type: 'voice' | 'video'
    startedAt: number
  } | null

  loadHistory: () => Promise<void>
  startCall: (contactId: string, contactName: string, type: 'voice' | 'video') => void
  endCall: () => void
  clearHistory: () => void
}

// M2 FIX: Safe invoke wrapper — no crash in browser dev mode
async function tryInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    return await invoke<T>(cmd, args)
  } catch {
    return null
  }
}

// ── Store ──
export const useCallStore = create<CallStore>()((set, get) => ({
  history: [],          // M2/M11 FIX: empty, not MOCK_HISTORY
  isLoading: false,
  error: null,
  activeCall: null,

  loadHistory: async () => {
    set({ isLoading: true, error: null })

    try {
      // M2 FIX: Load from Tauri backend, not mock data
      const result = await tryInvoke<CallEntry[]>('get_call_history')

      if (result) {
        set({ history: result, isLoading: false })
      } else {
        // No backend — show empty state, NOT mock data
        set({ history: [], isLoading: false })
      }
    } catch (err: any) {
      set({ error: err?.message ?? 'Failed to load call history', isLoading: false })
    }
  },

  startCall: (contactId, contactName, type) => {
    set({
      activeCall: {
        contactId,
        contactName,
        type,
        startedAt: Date.now(),
      }
    })
  },

  endCall: () => {
    const state = get()
    if (!state.activeCall) return

    const duration = Math.floor((Date.now() - state.activeCall.startedAt) / 1000)
    const newEntry: CallEntry = {
      id: crypto.randomUUID(),
      contactId: state.activeCall.contactId,
      contactName: state.activeCall.contactName,
      type: state.activeCall.type,
      direction: 'outgoing',
      duration,
      timestamp: new Date().toISOString(),
    }

    set({
      activeCall: null,
      history: [newEntry, ...state.history],
    })
  },

  clearHistory: () => {
    set({ history: [] })
  },
}))

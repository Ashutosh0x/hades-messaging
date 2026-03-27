import { create } from 'zustand'
import FeatureFlags from '../utils/featureFlags'

// ── Types ──
export interface CallEntry {
  id: string
  contactId: string
  name: string
  type: 'incoming' | 'outgoing' | 'missed'
  media: 'voice' | 'video'
  duration: number | null   // seconds, null if missed
  timestamp: number          // unix ms
}

interface CallStore {
  history: CallEntry[]
  isLoading: boolean
  error: string | null

  loadHistory: () => Promise<void>
  addEntry: (entry: Omit<CallEntry, 'id'>) => void
  clearHistory: () => void
}

// ── Mock data (dev mode only) ──
const now = Date.now()
const HOUR = 3600_000
const DAY = 86400_000

const MOCK_HISTORY: CallEntry[] = [
  { id: '1', contactId: 'c1', name: 'Alex Morgan',  type: 'incoming', media: 'voice', duration: 263,  timestamp: now - 2 * HOUR },
  { id: '2', contactId: 'c2', name: 'Jordan Lee',   type: 'missed',   media: 'voice', duration: null, timestamp: now - 5 * HOUR },
  { id: '3', contactId: 'c3', name: 'Echo Vault',   type: 'outgoing', media: 'video', duration: 725,  timestamp: now - 7 * HOUR },
  { id: '4', contactId: 'c1', name: 'Alex Morgan',  type: 'outgoing', media: 'voice', duration: 524,  timestamp: now - DAY - 3 * HOUR },
  { id: '5', contactId: 'c4', name: 'Cipher Node',  type: 'missed',   media: 'video', duration: null, timestamp: now - DAY - 8 * HOUR },
  { id: '6', contactId: 'c2', name: 'Jordan Lee',   type: 'incoming', media: 'voice', duration: 72,   timestamp: now - DAY - 12 * HOUR },
]

// ── Store ──
export const useCallStore = create<CallStore>()((set) => ({
  history: [],
  isLoading: false,
  error: null,

  loadHistory: async () => {
    set({ isLoading: true, error: null })

    try {
      if (FeatureFlags.useMockData) {
        // Dev mode: return mock data
        await new Promise(r => setTimeout(r, 300)) // simulate latency
        set({ history: MOCK_HISTORY, isLoading: false })
        return
      }

      // Production: call Tauri backend
      // const data = await invoke<CallEntry[]>('get_call_history', { limit: 100, offset: 0 })
      // set({ history: data, isLoading: false })

      // Fallback until Tauri is wired
      set({ history: MOCK_HISTORY, isLoading: false })
    } catch (err: any) {
      set({ error: err?.message ?? 'Failed to load call history', isLoading: false })
    }
  },

  addEntry: (entry) => {
    const newEntry: CallEntry = { ...entry, id: crypto.randomUUID() }
    set((state) => ({ history: [newEntry, ...state.history] }))
  },

  clearHistory: () => {
    set({ history: [] })
  },
}))

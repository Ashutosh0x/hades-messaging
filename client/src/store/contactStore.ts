import { create } from 'zustand'
import FeatureFlags from '../utils/featureFlags'

// ── Types ──
export interface Contact {
  id: string
  name: string
  publicKey: string
  initial: string
  verified: boolean
  status: 'online' | 'recently' | 'offline'
  lastSeen: number | null  // unix ms, null = online
  color: string
}

interface ContactStore {
  contacts: Map<string, Contact>
  isLoading: boolean

  loadContacts: () => Promise<void>
  getContact: (id: string) => Contact | undefined
  allContacts: () => Contact[]
  verifiedContacts: () => Contact[]
  unverifiedContacts: () => Contact[]
}

// ── Mock data (dev mode) ──
const MOCK_CONTACTS: Contact[] = [
  { id: 'c1', name: 'Alex Morgan',   publicKey: 'ed25519:AE2889Z1C0FFEE01', initial: 'A', verified: true,  status: 'recently', lastSeen: Date.now() - 1800_000, color: '#2ECC71' },
  { id: 'c2', name: 'Jordan Lee',    publicKey: 'ed25519:1A5B7D3CA652B1E8', initial: 'J', verified: false, status: 'offline',  lastSeen: Date.now() - 86400_000, color: '#3498DB' },
  { id: 'c3', name: 'Echo Vault',    publicKey: 'ed25519:DA094A42F701B3C9', initial: 'E', verified: true,  status: 'online',   lastSeen: null, color: '#9B59B6' },
  { id: 'c4', name: 'Cipher Node',   publicKey: 'ed25519:8821EF5ET0C2D3A1', initial: 'C', verified: true,  status: 'recently', lastSeen: Date.now() - 7200_000, color: '#E67E22' },
  { id: 'c5', name: 'Shadow Relay',  publicKey: 'ed25519:F4E2A1B70C9D5832', initial: 'S', verified: false, status: 'offline',  lastSeen: Date.now() - 259200_000, color: '#E74C3C' },
]

// ── Store ──
export const useContactStore = create<ContactStore>()((set, get) => ({
  contacts: new Map(),
  isLoading: false,

  loadContacts: async () => {
    set({ isLoading: true })
    try {
      let contactsList: Contact[]

      if (FeatureFlags.useMockData) {
        await new Promise(r => setTimeout(r, 200))
        contactsList = MOCK_CONTACTS
      } else {
        // Production: const contactsList = await invoke<Contact[]>('get_contacts')
        contactsList = MOCK_CONTACTS // fallback until Tauri wired
      }

      const map = new Map<string, Contact>()
      contactsList.forEach(c => map.set(c.id, c))
      set({ contacts: map, isLoading: false })
    } catch {
      set({ isLoading: false })
    }
  },

  getContact: (id: string) => get().contacts.get(id),

  allContacts: () => Array.from(get().contacts.values()),

  verifiedContacts: () =>
    Array.from(get().contacts.values()).filter(c => c.verified),

  unverifiedContacts: () =>
    Array.from(get().contacts.values()).filter(c => !c.verified),
}))

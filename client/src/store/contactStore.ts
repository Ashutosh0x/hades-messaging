import { create } from 'zustand'

// ── Types ──
export interface Contact {
  id: string
  name: string
  display_name: string
  identity_key: string // hex
  safety_number?: string
  verified: boolean
  created_at: string
  initial: string
}

interface ContactState {
  contacts: Contact[]
  loading: boolean

  loadContacts: () => Promise<void>
  fetchContacts: () => Promise<void>
  addContact: (id: string, name: string, identityKeyHex: string) => Promise<void>
  deleteContact: (id: string) => Promise<void>
  verifiedContacts: () => Contact[]
  unverifiedContacts: () => Contact[]
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

// ── Store ──
export const useContactStore = create<ContactState>((set, get) => ({
  contacts: [],
  loading: false,

  loadContacts: async () => {
    set({ loading: true })
    try {
      const raw = await tryInvoke<Contact[]>('get_contacts')
      const contacts = (raw ?? []).map(c => ({
        ...c,
        name: c.name || c.display_name || 'Unknown',
        initial: (c.name || c.display_name || '?')[0].toUpperCase(),
      }))
      set({ contacts, loading: false })
    } catch (err) {
      console.error('Fetch contacts failed:', err)
      set({ loading: false })
    }
  },

  fetchContacts: async () => {
    return get().loadContacts()
  },

  addContact: async (id, name, identityKeyHex) => {
    await tryInvoke('add_contact', {
      id,
      displayName: name,
      identityKeyHex,
    })
    await get().loadContacts()
  },

  deleteContact: async (id) => {
    await tryInvoke('delete_contact', { id })
    set((state) => ({
      contacts: state.contacts.filter((c) => c.id !== id),
    }))
  },

  verifiedContacts: () => {
    return get().contacts.filter(c => c.verified)
  },

  unverifiedContacts: () => {
    return get().contacts.filter(c => !c.verified)
  },
}))

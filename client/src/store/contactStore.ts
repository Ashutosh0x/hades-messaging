import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

// ── Types ──
export interface Contact {
  id: string
  display_name: string
  identity_key: string // hex
  safety_number?: string
  verified: boolean
  created_at: string
}

interface ContactState {
  contacts: Contact[]
  loading: boolean

  fetchContacts: () => Promise<void>
  addContact: (id: string, name: string, identityKeyHex: string) => Promise<void>
  deleteContact: (id: string) => Promise<void>
}

// ── Store ──
export const useContactStore = create<ContactState>((set) => ({
  contacts: [],
  loading: false,

  fetchContacts: async () => {
    set({ loading: true })
    try {
      const contacts = await invoke<Contact[]>('get_contacts')
      set({ contacts, loading: false })
    } catch (err) {
      console.error('Fetch contacts failed:', err)
      set({ loading: false })
    }
  },

  addContact: async (id, name, identityKeyHex) => {
    await invoke('add_contact', {
      id,
      displayName: name,
      identityKeyHex,
    })
    // Re-fetch
    const contacts = await invoke<Contact[]>('get_contacts')
    set({ contacts })
  },

  deleteContact: async (id) => {
    await invoke('delete_contact', { id })
    set((state) => ({
      contacts: state.contacts.filter((c) => c.id !== id),
    }))
  },
}))

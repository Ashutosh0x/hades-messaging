import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

interface SecurityState {
  vaultUnlocked: boolean
  identityPubkey: string | null
  loading: boolean
  error: string | null

  initializeVault: (passphrase: string) => Promise<void>
  lockVault: () => Promise<void>
  unlockVault: (passphrase: string) => Promise<void>
  generateIdentity: () => Promise<string>
  emergencyWipe: () => Promise<void>
}

export const useSecurityStore = create<SecurityState>((set) => ({
  vaultUnlocked: false,
  identityPubkey: null,
  loading: false,
  error: null,

  initializeVault: async (passphrase: string) => {
    set({ loading: true, error: null })
    try {
      await invoke('initialize_database', { passphrase })
      const pubkey = await invoke<string | null>('get_identity_pubkey')
      set({ vaultUnlocked: true, identityPubkey: pubkey, loading: false })
    } catch (err) {
      set({ error: String(err), loading: false })
    }
  },

  lockVault: async () => {
    try {
      await invoke('lock_database')
    } catch (_) { /* ignore */ }
    set({ vaultUnlocked: false, identityPubkey: null })
  },

  unlockVault: async (passphrase: string) => {
    set({ loading: true, error: null })
    try {
      await invoke('unlock_database', { passphrase })
      const pubkey = await invoke<string | null>('get_identity_pubkey')
      set({ vaultUnlocked: true, identityPubkey: pubkey, loading: false })
    } catch (err) {
      set({ error: String(err), loading: false })
    }
  },

  generateIdentity: async () => {
    const pubkey = await invoke<string>('generate_identity')
    set({ identityPubkey: pubkey })
    return pubkey
  },

  emergencyWipe: async () => {
    try {
      await invoke('emergency_wipe')
    } catch (_) { /* ignore */ }
    set({ vaultUnlocked: false, identityPubkey: null })
  },
}))

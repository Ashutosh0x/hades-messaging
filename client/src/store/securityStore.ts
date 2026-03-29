import { create } from 'zustand'

export type NotificationConfig = 'sealed' | 'sender_only' | 'full'

interface VaultState {
  isLocked: boolean
  isDuressMode: boolean
}

interface SecurityState {
  vault: VaultState
  vaultUnlocked: boolean
  identityPubkey: string | null
  loading: boolean
  error: string | null
  notificationConfig: NotificationConfig

  initializeVault: (passphrase: string) => Promise<void>
  lockVault: () => Promise<void>
  unlockVault: (passphrase: string) => Promise<void>
  generateIdentity: () => Promise<string>
  emergencyWipe: () => Promise<void>
  setNotificationConfig: (config: NotificationConfig) => void
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

export const useSecurityStore = create<SecurityState>((set) => ({
  vault: {
    isLocked: false,
    isDuressMode: false,
  },
  vaultUnlocked: false,
  identityPubkey: null,
  loading: false,
  error: null,
  notificationConfig: 'sender_only',

  initializeVault: async (passphrase: string) => {
    set({ loading: true, error: null })
    try {
      await tryInvoke('initialize_database', { passphrase })
      const pubkey = await tryInvoke<string>('get_identity_pubkey')
      set({
        vaultUnlocked: true,
        identityPubkey: pubkey,
        vault: { isLocked: false, isDuressMode: false },
        loading: false,
      })
    } catch (err) {
      set({ error: String(err), loading: false })
    }
  },

  lockVault: async () => {
    await tryInvoke('lock_database')
    set({
      vaultUnlocked: false,
      identityPubkey: null,
      vault: { isLocked: true, isDuressMode: false },
    })
  },

  unlockVault: async (passphrase: string) => {
    set({ loading: true, error: null })
    try {
      await tryInvoke('unlock_database', { passphrase })
      const pubkey = await tryInvoke<string>('get_identity_pubkey')
      set({
        vaultUnlocked: true,
        identityPubkey: pubkey,
        vault: { isLocked: false, isDuressMode: false },
        loading: false,
      })
    } catch (err) {
      set({ error: String(err), loading: false })
    }
  },

  generateIdentity: async () => {
    const pubkey = await tryInvoke<string>('generate_identity')
    if (pubkey) set({ identityPubkey: pubkey })
    return pubkey ?? ''
  },

  emergencyWipe: async () => {
    await tryInvoke('emergency_wipe')
    set({
      vaultUnlocked: false,
      identityPubkey: null,
      vault: { isLocked: true, isDuressMode: false },
    })
  },

  setNotificationConfig: (config) => set({ notificationConfig: config }),
}))


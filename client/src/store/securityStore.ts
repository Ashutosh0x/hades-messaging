import { create } from 'zustand'
import FeatureFlags from '../utils/featureFlags'

// -------------------------------------------------------------------
// Types
// -------------------------------------------------------------------

export interface FingerprintData {
  chunks: string[]    // 10 uppercase hex chunks (5 rows x 2 cols)
  isVerified: boolean
}

export interface VaultState {
  isLocked: boolean
  isDuressMode: boolean
  failedAttempts: number
  lastAttemptAt: number | null
}

export type NotificationConfig = 'sealed' | 'sender_only' | 'full'

interface SecurityStore {
  // -- Fingerprint ---------------------------------------------------
  fingerprints: Map<string, FingerprintData>
  loadFingerprint: (contactId: string) => Promise<void>
  getFingerprint: (contactId: string) => FingerprintData | undefined
  markVerified: (contactId: string) => void

  // -- Vault ---------------------------------------------------------
  vault: VaultState
  unlockVault: (pin: string) => Promise<boolean>
  lockVault: () => void

  // -- Device Config -------------------------------------------------
  notificationConfig: NotificationConfig
  setNotificationConfig: (config: NotificationConfig) => void
}

// -------------------------------------------------------------------
// Mock fingerprints — seeded per contactId via simple deterministic hash
// -------------------------------------------------------------------

function mockFingerprint(contactId: string): string[] {
  // Produce a deterministic hex string from the contactId
  let hash = 0
  for (let i = 0; i < contactId.length; i++) {
    hash = ((hash << 5) - hash + contactId.charCodeAt(i)) | 0
  }
  const hex = Math.abs(hash).toString(16).toUpperCase().padStart(40, 'ABCDEF0123456789')
  const chunks: string[] = []
  for (let i = 0; i < 40; i += 4) {
    chunks.push(hex.slice(i, i + 4))
  }
  return chunks
}

// -------------------------------------------------------------------
// Store
// -------------------------------------------------------------------

export const useSecurityStore = create<SecurityStore>((set, get) => ({
  // -- Fingerprint ---------------------------------------------------
  fingerprints: new Map(),

  loadFingerprint: async (contactId: string) => {
    if (get().fingerprints.has(contactId)) return

    let chunks: string[]

    if (FeatureFlags.useMockData) {
      // Local mock — deterministic per contact
      chunks = mockFingerprint(contactId)
    } else {
      // Production: invoke Rust BLAKE3 fingerprint derivation
      // const { invoke } = await import('@tauri-apps/api/core')
      // chunks = await invoke<string[]>('get_contact_fingerprint', { contactId })
      chunks = mockFingerprint(contactId) // fallback until Tauri is wired
    }

    set((state) => {
      const next = new Map(state.fingerprints)
      next.set(contactId, { chunks, isVerified: false })
      return { fingerprints: next }
    })
  },

  getFingerprint: (contactId: string) => get().fingerprints.get(contactId),

  markVerified: (contactId: string) => {
    set((state) => {
      const next = new Map(state.fingerprints)
      const existing = next.get(contactId)
      if (existing) {
        next.set(contactId, { ...existing, isVerified: true })
      }
      return { fingerprints: next }
    })
  },

  // -- Vault ---------------------------------------------------------
  vault: {
    isLocked: true,
    isDuressMode: false,
    failedAttempts: 0,
    lastAttemptAt: null,
  },

  unlockVault: async (pin: string): Promise<boolean> => {
    // Demo implementation for Duress logic
    // If the user enters exactly "9999", we unlock in Duress Mode
    // Otherwise any 4+ PIN unlocks in normal mode
    const isDuress = pin === '9999'
    const ok = pin.length >= 4

    set((state) => ({
      vault: {
        isLocked: !ok,
        isDuressMode: isDuress,
        failedAttempts: ok ? 0 : state.vault.failedAttempts + 1,
        lastAttemptAt: Date.now(),
      },
    }))
    return ok
  },

  lockVault: () => {
    set({ vault: { isLocked: true, isDuressMode: false, failedAttempts: 0, lastAttemptAt: null } })
  },

  // -- Device Config -------------------------------------------------
  notificationConfig: 'sealed', // Default to maximum privacy
  setNotificationConfig: (config: NotificationConfig) => {
    set({ notificationConfig: config })
  },
}))

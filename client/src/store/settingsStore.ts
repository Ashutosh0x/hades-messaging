import { create } from 'zustand'

// S6 FIX: Safe invoke wrapper
async function tryInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    return await invoke<T>(cmd, args)
  } catch {
    return null
  }
}

// Exported types for Settings.tsx
export type ClipboardTimeout = '1 min' | '3 min' | '5 min' | '10 min' | 'Never'
export type SelfDestructTimer = 'Off' | '5 min' | '1 hour' | '24 hours' | '7 days'

interface SettingsState {
  readReceipts: boolean
  typingIndicators: boolean
  screenshotGuard: boolean
  incognitoKeyboard: boolean
  burnDefault: number | null
  notificationPrivacy: 'full' | 'sender_only' | 'hidden' | 'silent'
  anonymityRouting: boolean
  clipboardTimeout: ClipboardTimeout
  selfDestructTimer: SelfDestructTimer
  loaded: boolean

  loadSettings: () => Promise<void>
  setSetting: <K extends keyof SettingsState>(key: K, value: SettingsState[K]) => void

  // Toggle helpers for Settings.tsx
  toggleReadReceipts: () => void
  toggleTypingIndicators: () => void
  toggleScreenshotGuard: () => void
  toggleIncognitoKeyboard: () => void
  toggleAnonymityRouting: () => void
  setClipboardTimeout: (t: ClipboardTimeout) => void
  setSelfDestructTimer: (t: SelfDestructTimer) => void
}

const SETTINGS_KEY = 'user_settings'

export const useSettingsStore = create<SettingsState>((set, get) => ({
  readReceipts: true,
  typingIndicators: true,
  screenshotGuard: false,
  incognitoKeyboard: false,
  burnDefault: null,
  notificationPrivacy: 'sender_only',
  anonymityRouting: false,
  clipboardTimeout: '3 min',
  selfDestructTimer: 'Off',
  loaded: false,

  loadSettings: async () => {
    // S6 FIX: Load from persistent backend kv_store
    const saved = await tryInvoke<string>('kv_get', { key: SETTINGS_KEY })
    if (saved) {
      try {
        const parsed = JSON.parse(saved)
        set({ ...parsed, loaded: true })
      } catch {
        set({ loaded: true })
      }
    } else {
      set({ loaded: true })
    }
  },

  setSetting: (key, value) => {
    set({ [key]: value } as any)
    persistSettings(get)
  },

  toggleReadReceipts: () => {
    set((s) => ({ readReceipts: !s.readReceipts }))
    persistSettings(get)
  },
  toggleTypingIndicators: () => {
    set((s) => ({ typingIndicators: !s.typingIndicators }))
    persistSettings(get)
  },
  toggleScreenshotGuard: () => {
    set((s) => ({ screenshotGuard: !s.screenshotGuard }))
    persistSettings(get)
  },
  toggleIncognitoKeyboard: () => {
    set((s) => ({ incognitoKeyboard: !s.incognitoKeyboard }))
    persistSettings(get)
  },
  toggleAnonymityRouting: () => {
    set((s) => ({ anonymityRouting: !s.anonymityRouting }))
    persistSettings(get)
  },
  setClipboardTimeout: (t) => {
    set({ clipboardTimeout: t })
    persistSettings(get)
  },
  setSelfDestructTimer: (t) => {
    set({ selfDestructTimer: t })
    persistSettings(get)
  },
}))

// Persist all settings to the backend kv_store
function persistSettings(get: () => SettingsState) {
  const state = get()
  const toSave = {
    readReceipts: state.readReceipts,
    typingIndicators: state.typingIndicators,
    screenshotGuard: state.screenshotGuard,
    incognitoKeyboard: state.incognitoKeyboard,
    burnDefault: state.burnDefault,
    notificationPrivacy: state.notificationPrivacy,
    anonymityRouting: state.anonymityRouting,
    clipboardTimeout: state.clipboardTimeout,
    selfDestructTimer: state.selfDestructTimer,
  }

  tryInvoke('kv_set', {
    key: SETTINGS_KEY,
    value: JSON.stringify(toSave),
  })
}

import { create } from 'zustand'

export type ClipboardTimeout = '1 min' | '3 min' | '5 min' | '10 min' | 'Never'
export type SelfDestructTimer = 'Off' | '5 min' | '1 hour' | '24 hours' | '7 days'

interface SettingsState {
  // Privacy
  readReceipts: boolean
  typingIndicators: boolean
  metadataMinimization: boolean

  // Network
  anonymityRouting: boolean

  // Security
  screenshotGuard: boolean
  incognitoKeyboard: boolean
  clipboardTimeout: ClipboardTimeout
  selfDestructTimer: SelfDestructTimer

  // Actions
  toggleReadReceipts: () => void
  toggleTypingIndicators: () => void
  toggleAnonymityRouting: () => void
  toggleScreenshotGuard: () => void
  toggleIncognitoKeyboard: () => void
  setClipboardTimeout: (t: ClipboardTimeout) => void
  setSelfDestructTimer: (t: SelfDestructTimer) => void
}

export const useSettingsStore = create<SettingsState>((set) => ({
  readReceipts: false,
  typingIndicators: false,
  metadataMinimization: true,     // Always active (non-toggleable)
  anonymityRouting: false,
  screenshotGuard: true,
  incognitoKeyboard: true,
  clipboardTimeout: '5 min',
  selfDestructTimer: '24 hours',

  toggleReadReceipts: () => set((s) => ({ readReceipts: !s.readReceipts })),
  toggleTypingIndicators: () => set((s) => ({ typingIndicators: !s.typingIndicators })),
  toggleAnonymityRouting: () => set((s) => ({ anonymityRouting: !s.anonymityRouting })),
  toggleScreenshotGuard: () => set((s) => ({ screenshotGuard: !s.screenshotGuard })),
  toggleIncognitoKeyboard: () => set((s) => ({ incognitoKeyboard: !s.incognitoKeyboard })),
  setClipboardTimeout: (t) => set({ clipboardTimeout: t }),
  setSelfDestructTimer: (t) => set({ selfDestructTimer: t }),
}))

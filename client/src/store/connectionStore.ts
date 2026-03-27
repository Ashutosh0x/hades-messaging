import { create } from 'zustand'
import { ConnectionStatusType } from '../types/connection'

interface ConnectionState {
  status: ConnectionStatusType
  progress: number
  stage: string
  isTorConnected: boolean
}

interface ConnectionActions {
  setConnecting: () => void
  updateProgress: (progress: number, stage: string) => void
  setEstablished: () => void
  setError: (stage: string) => void
  reset: () => void
}

export const useConnectionStore = create<ConnectionState & ConnectionActions>((set) => ({
  status: 'idle',
  progress: 0,
  stage: '',
  isTorConnected: false,

  setConnecting: () => set({
    status: 'connecting',
    progress: 0,
    stage: 'Initializing',
    isTorConnected: false,
  }),

  updateProgress: (progress, stage) => set({
    status: progress >= 100 ? 'established' : 'establishing',
    progress: Math.min(100, Math.max(0, progress)),
    stage,
  }),

  setEstablished: () => set({
    status: 'established',
    progress: 100,
    stage: 'Secure route established',
    isTorConnected: true,
  }),

  setError: (stage) => set({
    status: 'error',
    stage: `Failed: ${stage}`,
    isTorConnected: false,
  }),

  reset: () => set({
    status: 'idle',
    progress: 0,
    stage: '',
    isTorConnected: false,
  }),
}))

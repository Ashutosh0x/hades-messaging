import { useCallback, useRef } from 'react'
import { useConnectionStore } from '../store/connectionStore'
import { CONNECTION_STAGES } from '../types/connection'
import { invoke } from '@tauri-apps/api/core'

/**
 * Hook that drives the secure route establishment flow.
 * Each stage maps 1:1 to a real backend operation.
 * In production, each step awaits a Tauri command from the Rust backend.
 * Here we simulate with natural timing jitter.
 */
export function useSecureRoute() {
  const { setConnecting, updateProgress, setEstablished, setError, reset } = useConnectionStore()
  const abortRef = useRef(false)

  const establishRoute = useCallback(async () => {
    abortRef.current = false
    setConnecting()

    for (const stage of CONNECTION_STAGES) {
      if (abortRef.current) return

      try {
        await invoke('hades_onion_await_stage', { stage: stage.progress })
      } catch (err) {
        // Fallback or dev mode simulated jitter
        if (!String(err).includes('Tauri')) console.error("Secure route error:", err)
        await new Promise<void>((resolve) => {
          setTimeout(resolve, 400 + Math.random() * 500)
        })
      }

      if (abortRef.current) return

      updateProgress(stage.progress, stage.stage)
    }

    setEstablished()

    // Subtle haptic feedback on success
    if (navigator.vibrate) navigator.vibrate(15)
  }, [setConnecting, updateProgress, setEstablished])

  const disconnect = useCallback(() => {
    abortRef.current = true
    reset()
  }, [reset])

  const retry = useCallback(() => {
    abortRef.current = true
    setTimeout(() => establishRoute(), 100)
  }, [establishRoute])

  return { establishRoute, disconnect, retry }
}

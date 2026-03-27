import { useEffect, useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { Shield, ShieldCheck, WifiOff, RotateCcw, ICON_SIZE } from '../ui/icons'
import { useConnectionStore } from '../store/connectionStore'
import { useSecureRoute } from '../hooks/useSecureRoute'
import './SecureRouteIndicator.css'

/**
 * Full-width connection indicator that replaces the old TorStatusBar.
 * Shows real-time progress through the 8-stage secure route establishment.
 * Auto-dismisses 5 seconds after successful connection.
 */
export default function SecureRouteIndicator() {
  const { status, progress, stage } = useConnectionStore()
  const { establishRoute, retry } = useSecureRoute()
  const [dismissed, setDismissed] = useState(false)

  // Auto-start connection on mount
  useEffect(() => {
    establishRoute()
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  // Auto-dismiss 5s after established
  useEffect(() => {
    if (status === 'established') {
      const timer = setTimeout(() => setDismissed(true), 5000)
      return () => clearTimeout(timer)
    } else {
      setDismissed(false)
    }
  }, [status])

  const isConnecting = status === 'connecting' || status === 'establishing'
  const isConnected = status === 'established'
  const isFailed = status === 'error'

  // If it's idle, we don't render anything, but we keep AnimatePresence mounted
  return (
    <AnimatePresence>
      {status !== 'idle' && !dismissed && (
        <motion.div
          className={`secure-route-indicator ${isConnected ? 'state-connected' : isFailed ? 'state-error' : 'state-connecting'}`}
        initial={{ height: 0, opacity: 0 }}
        animate={{ height: 'auto', opacity: 1 }}
        exit={{ height: 0, opacity: 0 }}
        transition={{ type: 'spring', damping: 25, stiffness: 300 }}
      >
        <div className="sri-content">
          {/* Status Row */}
          <div className="sri-status-row">
            <div className="sri-left">
              {/* Pulsing dot */}
              <motion.div
                className={`sri-dot ${isConnected ? 'dot-green' : isFailed ? 'dot-red' : 'dot-amber'}`}
                animate={isConnecting ? {
                  opacity: [0.5, 1, 0.5],
                  scale: [0.9, 1.1, 0.9],
                } : { opacity: 1, scale: 1 }}
                transition={isConnecting ? {
                  repeat: Infinity,
                  duration: 2,
                  ease: 'easeInOut',
                } : {}}
              />

              {/* Icon */}
              {isConnected 
                ? <ShieldCheck size={14} color="var(--accent-secure)" />
                : isFailed 
                  ? <WifiOff size={14} color="var(--danger)" />
                  : <Shield size={14} color="var(--accent-warning)" />
              }

              {/* Label */}
              <span className="sri-label">
                {isConnected 
                  ? 'Secure Route Established' 
                  : isFailed 
                    ? 'Connection Failed' 
                    : 'Establishing Secure Route'
                }
              </span>
            </div>

            <div className="sri-right">
              <AnimatePresence mode="wait">
                {isConnecting && (
                  <motion.span
                    key="progress"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    className="sri-percent"
                  >
                    {progress}%
                  </motion.span>
                )}
                {isConnected && (
                  <motion.span
                    key="check"
                    initial={{ opacity: 0, scale: 0.6 }}
                    animate={{ opacity: 1, scale: 1 }}
                    className="sri-check"
                  >
                    ✓
                  </motion.span>
                )}
                {isFailed && (
                  <motion.button
                    key="retry"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    className="sri-retry-btn"
                    onClick={retry}
                  >
                    <RotateCcw size={14} color="var(--danger)" />
                    <span>Retry</span>
                  </motion.button>
                )}
              </AnimatePresence>
            </div>
          </div>

          {/* Progress Bar */}
          <div className="sri-progress-track">
            <motion.div
              className={`sri-progress-fill ${isConnected ? 'fill-green' : isFailed ? 'fill-red' : 'fill-amber'}`}
              initial={{ width: 0 }}
              animate={{ width: `${isConnected || isFailed ? 100 : progress}%` }}
              transition={{ duration: 0.4, ease: 'easeOut' }}
            />
          </div>

          {/* Current Stage */}
          <AnimatePresence mode="wait">
            {isConnecting && stage && (
              <motion.div
                key={stage}
                initial={{ opacity: 0, y: 4 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -4 }}
                className="sri-stage"
              >
                {stage}…
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </motion.div>
      )}
    </AnimatePresence>
  )
}

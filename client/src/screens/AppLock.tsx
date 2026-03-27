import { useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { useTranslation } from 'react-i18next'
import { useSecurityStore } from '../store/securityStore'
import { HapticManager } from '../utils/haptics'
import {
  ShieldCheck, Fingerprint, Delete, Check,
  ICON_SIZE, ICON_STROKE,
} from '../ui/icons'
import './AppLock.css'

const PIN_LENGTH = 6

export default function AppLock({ onUnlock }: { onUnlock: () => void }) {
  const { t } = useTranslation()
  const [view, setView] = useState<'main' | 'pin'>('main')
  const [pin, setPin] = useState('')
  const [isError, setIsError] = useState(false)
  const [isUnlocked, setIsUnlocked] = useState(false)
  const { unlockVault, vault } = useSecurityStore()

  const handleKeyPress = (digit: string) => {
    if (pin.length >= PIN_LENGTH || isUnlocked) return
    HapticManager.impact('light')

    const next = pin + digit
    setPin(next)

    if (next.length === PIN_LENGTH) {
      verifyPin(next)
    }
  }

  const handleDelete = () => {
    if (pin.length > 0) {
      setPin(pin.slice(0, -1))
      HapticManager.selection()
    }
  }

  const verifyPin = async (code: string) => {
    const ok = await unlockVault(code)
    if (ok) {
      setIsUnlocked(true)
      HapticManager.notification('success')
      setTimeout(onUnlock, 600)
    } else {
      setIsError(true)
      HapticManager.notification('error')
      setTimeout(() => { setIsError(false); setPin('') }, 500)
    }
  }

  const handleBiometric = async () => {
    HapticManager.selection()
    const ok = await unlockVault('biometric')
    if (ok) {
      setIsUnlocked(true)
      HapticManager.notification('success')
      setTimeout(onUnlock, 600)
    }
  }

  return (
    <motion.div
      className="lock-screen"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0, scale: 0.95 }}
      transition={{ duration: 0.3 }}
    >
      {/* Ambient background */}
      <div className="lock-bg">
        <div className="gradient-orb orb-1" />
        <div className="gradient-orb orb-2" />
      </div>

      <AnimatePresence mode="wait">
        {view === 'main' ? (
          /* ─── Sentinel State ─── */
          <motion.div
            key="main"
            className="lock-main"
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, y: -30 }}
            transition={{ duration: 0.3, ease: [0.19, 1, 0.22, 1] }}
          >
            {/* Shield ring */}
            <motion.div
              className="shield-container"
              animate={
                isUnlocked
                  ? { rotate: 360, scale: [1, 1.2, 0] }
                  : isError
                  ? { x: [-10, 10, -10, 10, 0] }
                  : {}
              }
              transition={{ duration: isUnlocked ? 0.6 : 0.4 }}
            >
              <div className={`shield-ring ${isError ? 'error' : ''} ${isUnlocked ? 'unlocked' : ''}`}>
                {isUnlocked ? (
                  <motion.div initial={{ scale: 0 }} animate={{ scale: 1 }} transition={{ type: 'spring', damping: 15 }}>
                    <Check size={48} color="var(--accent-secure)" strokeWidth={2} />
                  </motion.div>
                ) : (
                  <ShieldCheck size={48} color={isError ? 'var(--danger)' : 'var(--accent-secure)'} strokeWidth={1.5} />
                )}
              </div>

              {/* Pulse rings */}
              <motion.div
                className="pulse-ring"
                animate={{ scale: [1, 1.5], opacity: [0.4, 0] }}
                transition={{ duration: 2, repeat: Infinity, repeatDelay: 0.5 }}
              />
              <motion.div
                className="pulse-ring delayed"
                animate={{ scale: [1, 1.5], opacity: [0.25, 0] }}
                transition={{ duration: 2, delay: 0.6, repeat: Infinity, repeatDelay: 0.5 }}
              />
            </motion.div>

            <motion.h1
              className="lock-title"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.15 }}
            >
              {t('lock.title')}
            </motion.h1>

            <motion.p
              className="lock-subtitle"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.25 }}
            >
              {t('lock.subtitle')}
            </motion.p>

            <motion.div
              className="lock-actions-group"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.35 }}
            >
              <motion.button
                className="biometric-btn"
                onClick={handleBiometric}
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.97 }}
                disabled={isUnlocked}
              >
                <Fingerprint size={ICON_SIZE.md} strokeWidth={ICON_STROKE.default} />
                {t('lock.unlockBiometrics')}
              </motion.button>

              <button
                className="enter-pin-link"
                onClick={() => { setView('pin'); HapticManager.impact('medium') }}
                disabled={isUnlocked}
              >
                {t('lock.enterPin')}
              </button>
            </motion.div>
          </motion.div>
        ) : (
          /* ─── PIN Pad State ─── */
          <motion.div
            key="pin"
            className="pin-view"
            initial={{ y: '100%' }}
            animate={{ y: 0 }}
            exit={{ y: '100%' }}
            transition={{ type: 'spring', damping: 25, stiffness: 200 }}
          >
            <div className="pin-header">
              <h2 className="pin-instruction">{t('lock.pinInstruction')}</h2>

              {/* PIN dots */}
              <motion.div
                className="pin-dots"
                animate={isError ? { x: [-10, 10, -10, 10, 0] } : {}}
                transition={{ duration: 0.4 }}
              >
                {Array.from({ length: PIN_LENGTH }).map((_, i) => (
                  <motion.div
                    key={i}
                    className={`pin-dot ${i < pin.length ? 'filled' : ''} ${isError ? 'error' : ''}`}
                    animate={
                      i < pin.length
                        ? { scale: [1, 1.25, 1], backgroundColor: isError ? 'var(--danger)' : 'var(--accent-secure)' }
                        : { scale: 1 }
                    }
                    transition={{ type: 'spring', stiffness: 500 }}
                  />
                ))}
              </motion.div>

              {/* Error / attempts */}
              <AnimatePresence>
                {isError && (
                  <motion.p
                    className="pin-error-text"
                    initial={{ opacity: 0, y: -5 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0 }}
                  >
                    {t('lock.incorrectPin')}
                  </motion.p>
                )}
              </AnimatePresence>
              {vault.failedAttempts > 0 && (
                <p className="pin-attempts-text">
                  {t('lock.attemptsRemaining', { count: Math.max(0, 10 - vault.failedAttempts) })}
                </p>
              )}
            </div>

            {/* Number pad */}
            <div className="numpad-grid">
              {['1', '2', '3', '4', '5', '6', '7', '8', '9'].map((num, i) => (
                <motion.button
                  key={num}
                  className="numpad-key"
                  onClick={() => handleKeyPress(num)}
                  whileTap={{ scale: 0.92, backgroundColor: 'var(--bg-elevated)' }}
                  initial={{ opacity: 0, scale: 0.8 }}
                  animate={{ opacity: 1, scale: 1 }}
                  transition={{ delay: i * 0.03, type: 'spring', stiffness: 400, damping: 20 }}
                  disabled={isUnlocked}
                >
                  <span className="numpad-digit">{num}</span>
                </motion.button>
              ))}

              {/* Bottom row: biometric / 0 / delete */}
              <motion.button
                className="numpad-key action-key"
                onClick={() => { setView('main'); setPin(''); setIsError(false) }}
                whileTap={{ scale: 0.92 }}
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ delay: 0.3 }}
              >
                <Fingerprint size={20} color="var(--accent-secure)" />
              </motion.button>

              <motion.button
                className="numpad-key"
                onClick={() => handleKeyPress('0')}
                whileTap={{ scale: 0.92, backgroundColor: 'var(--bg-elevated)' }}
                initial={{ opacity: 0, scale: 0.8 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ delay: 0.3 }}
                disabled={isUnlocked}
              >
                <span className="numpad-digit">0</span>
              </motion.button>

              <motion.button
                className="numpad-key action-key"
                onClick={handleDelete}
                whileTap={{ scale: 0.92 }}
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ delay: 0.33 }}
              >
                <Delete size={20} color="var(--text-secondary)" />
              </motion.button>
            </div>

            {/* Back to biometric link */}
            <motion.button
              className="back-to-bio"
              onClick={() => { setView('main'); setPin(''); setIsError(false) }}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.4 }}
            >
              <Fingerprint size={14} color="var(--accent-secure)" />
              {t('lock.useBiometrics')}
            </motion.button>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  )
}

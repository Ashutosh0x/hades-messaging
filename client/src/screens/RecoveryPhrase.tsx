import { useState, useMemo, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'
import { HapticManager } from '../utils/haptics'
import {
  ArrowLeft, ShieldCheck, Copy, Eye, EyeOff, Check, AlertTriangle, Loader,
  ICON_SIZE,
} from '../ui/icons'
import './RecoveryPhrase.css'

// M10 FIX: Safe invoke wrapper
async function tryInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core')
  return invoke<T>(cmd, args)
}

type Step = 'display' | 'quiz' | 'confirmed'

export default function RecoveryPhrase() {
  const navigate = useNavigate()
  const { t } = useTranslation()
  const [step, setStep] = useState<Step>('display')
  const [revealed, setRevealed] = useState(false)
  const [copied, setCopied] = useState(false)
  
  const [words, setWords] = useState<string[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    // M10 FIX: First check sessionStorage (set during onboarding)
    const stored = sessionStorage.getItem('hades_mnemonic')
    if (stored) {
      setWords(stored.split(' '))
      setLoading(false)
      return
    }

    // Otherwise, try to generate from backend
    tryInvoke<string[]>('generate_recovery_phrase')
      .then(res => {
        setWords(res)
        setLoading(false)
      })
      .catch(err => {
        console.error('Failed to generate recovery phrase:', err)
        // M10 FIX: Show error instead of MOCK_WORDS
        setError('Could not retrieve recovery phrase. Ensure your vault is unlocked.')
        setLoading(false)
      })
  }, [])

  // Quiz state
  const quizIndices = useMemo(() => {
    const indices: number[] = []
    while (indices.length < 3) {
      const r = Math.floor(Math.random() * 24)
      if (!indices.includes(r)) indices.push(r)
    }
    return indices.sort((a, b) => a - b)
  }, [])
  const [quizAnswers, setQuizAnswers] = useState<Record<number, string>>({})
  const [quizError, setQuizError] = useState(false)

  const handleCopy = async () => {
    await navigator.clipboard.writeText(words.join(' '))
    setCopied(true)
    HapticManager.selection()
    setTimeout(() => setCopied(false), 2000)
  }

  const handleQuizSubmit = () => {
    const correct = quizIndices.every(i => quizAnswers[i]?.toLowerCase().trim() === words[i])
    if (correct) {
      setStep('confirmed')
      HapticManager.notification('success')
    } else {
      setQuizError(true)
      HapticManager.notification('error')
      setTimeout(() => setQuizError(false), 2000)
    }
  }

  return (
    <div className="recovery-screen">
      {/* Header */}
      <header className="rp-header">
        <button className="rp-back" onClick={() => navigate(-1)}>
          <ArrowLeft size={ICON_SIZE.md} color="var(--text-primary)" />
        </button>
        <h1 className="rp-title">{t('recovery.title')}</h1>
        <div style={{ width: 24 }} />
      </header>

      <main className="rp-content">
        <AnimatePresence mode="wait">
          {step === 'display' && (
            <motion.div
              key="display"
              className="rp-display"
              initial={{ opacity: 0, y: 16 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -16 }}
            >
              {/* Warning */}
              <div className="rp-warning">
                <AlertTriangle size={18} color="var(--accent-warning)" />
                <p>{t('recovery.warningText')}</p>
              </div>

              {loading ? (
                <div className="rp-loading">
                  <Loader className="anim-spin" size={32} color="var(--text-secondary)" />
                </div>
              ) : error ? (
                <div className="rp-loading">
                  <AlertTriangle size={32} color="var(--accent-warning)" />
                  <p style={{ color: 'var(--text-secondary)', marginTop: '0.5rem' }}>{error}</p>
                </div>
              ) : (
                <div className={`rp-word-grid ${!revealed ? 'blurred' : ''}`}>
                  {words.map((word, i) => (
                    <motion.div
                      key={i}
                      className="rp-word-card"
                      initial={{ opacity: 0, scale: 0.8 }}
                      animate={{ opacity: 1, scale: 1 }}
                      transition={{ delay: revealed ? i * 0.03 : 0 }}
                    >
                      <span className="rp-word-num">{i + 1}</span>
                      <span className="rp-word-text">{word}</span>
                    </motion.div>
                  ))}

                  {/* Blur overlay */}
                {!revealed && (
                  <div className="rp-blur-overlay">
                    <EyeOff size={32} color="var(--text-secondary)" />
                    <p>{t('recovery.tapToReveal')}</p>
                    <button className="rp-reveal-btn" onClick={() => { setRevealed(true); HapticManager.selection() }}>
                      <Eye size={16} /> {t('recovery.reveal')}
                    </button>
                  </div>
                )}
              </div>
              )}

              {/* Actions */}
              {revealed && (
                <motion.div
                  className="rp-actions"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  transition={{ delay: 0.5 }}
                >
                  <button className="rp-copy-btn" onClick={handleCopy}>
                    {copied ? <Check size={16} /> : <Copy size={16} />}
                    {copied ? t('recovery.copied') : t('recovery.copyAll')}
                  </button>
                  <button className="rp-continue-btn" onClick={() => setStep('quiz')}>
                    {t('recovery.iWroteItDown')}
                  </button>
                </motion.div>
              )}
            </motion.div>
          )}

          {step === 'quiz' && (
            <motion.div
              key="quiz"
              className="rp-quiz"
              initial={{ opacity: 0, y: 16 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -16 }}
            >
              <h2 className="rp-quiz-title">{t('recovery.quizTitle')}</h2>
              <p className="rp-quiz-desc">{t('recovery.quizDesc')}</p>

              <div className="rp-quiz-fields">
                {quizIndices.map(i => (
                  <div key={i} className={`rp-quiz-field ${quizError ? 'error' : ''}`}>
                    <label>Word #{i + 1}</label>
                    <input
                      type="text"
                      autoComplete="off"
                      spellCheck={false}
                      value={quizAnswers[i] || ''}
                      onChange={e => setQuizAnswers({ ...quizAnswers, [i]: e.target.value })}
                      placeholder={`Enter word #${i + 1}`}
                    />
                  </div>
                ))}
              </div>

              {quizError && (
                <motion.p
                  className="rp-quiz-error"
                  initial={{ opacity: 0, y: 4 }}
                  animate={{ opacity: 1, y: 0 }}
                >
                  <AlertTriangle size={14} /> {t('recovery.quizFailed')}
                </motion.p>
              )}

              <button
                className="rp-verify-btn"
                onClick={handleQuizSubmit}
                disabled={quizIndices.some(i => !quizAnswers[i])}
              >
                <ShieldCheck size={18} /> {t('recovery.verify')}
              </button>
            </motion.div>
          )}

          {step === 'confirmed' && (
            <motion.div
              key="confirmed"
              className="rp-confirmed"
              initial={{ scale: 0.8, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
            >
              <motion.div
                className="rp-check-circle"
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                transition={{ type: 'spring', damping: 12, stiffness: 200 }}
              >
                <ShieldCheck size={48} color="var(--accent-secure)" />
              </motion.div>
              <h2>{t('recovery.confirmed')}</h2>
              <p>{t('recovery.confirmedDesc')}</p>
              <button className="rp-done-btn" onClick={() => navigate(-1)}>
                {t('common.done')}
              </button>
            </motion.div>
          )}
        </AnimatePresence>
      </main>
    </div>
  )
}

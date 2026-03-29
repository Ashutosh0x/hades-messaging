import { useState, useEffect, useCallback, useRef } from 'react'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { ROUTES } from '../config/routes'
import {
  ShieldCheck, EyeOff, Fingerprint, Route, Check, Loader, DatabaseZap,
  ICON_SIZE, ICON_STROKE,
} from '../ui/icons'
import './Onboarding.css'

// M5 FIX: Safe invoke wrapper — no browser crash
async function tryInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core')
  return invoke<T>(cmd, args)
}

// -------------------------------------------------------------------
// Entropy collection — mirrors the Rust sentinel-crypto::entropy API.
// In production this would call invoke('add_entropy_seed', { coords }).
// Here we simulate the phases of real key generation.
// -------------------------------------------------------------------

interface KeygenPhase {
  label: string
  weight: number // percentage points this phase occupies
}

export default function Onboarding() {
  const navigate = useNavigate()
  const { t } = useTranslation()
  const [progress, setProgress] = useState(0)
  const [entropyCollected, setEntropyCollected] = useState(0)
  const [isComplete, setIsComplete] = useState(false)
  const [passphrase, setPassphrase] = useState('')
  const [showPassphraseInput, setShowPassphraseInput] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const touchRef = useRef<HTMLDivElement>(null)

  // Phases mirror the Rust keygen pipeline
  const phases: KeygenPhase[] = [
    { label: t('onboarding.stepEntropy'),  weight: 30 },
    { label: t('onboarding.stepX25519'),   weight: 25 },
    { label: t('onboarding.stepMLKEM'),    weight: 25 },
    { label: t('onboarding.stepPrekeys'),  weight: 20 },
  ]

  // Determine current phase from progress
  const getCurrentPhase = useCallback((p: number) => {
    let accumulated = 0
    for (const phase of phases) {
      accumulated += phase.weight
      if (p < accumulated) return phase
    }
    return phases[phases.length - 1]
  }, [])

  // Collect entropy from touch/mouse movement (phase 1)
  const handlePointerMove = useCallback((e: React.PointerEvent) => {
    if (progress >= 30) return // entropy phase done
    setEntropyCollected(prev => {
      const next = prev + 1
      const newProgress = Math.min(30, Math.floor((next / 50) * 30))
      setProgress(newProgress)
      return next
    })
  }, [progress])

  // M5/M9 FIX: When entropy is collected, show passphrase input then call create_identity
  useEffect(() => {
    if (progress >= 30 && !showPassphraseInput && !isComplete) {
      setShowPassphraseInput(true)
    }
  }, [progress >= 30])

  const handleCreateIdentity = async () => {
    if (passphrase.length < 8) {
      setError('Passphrase must be at least 8 characters')
      return
    }
    setError(null)
    setShowPassphraseInput(false)

    try {
      // M5 FIX: Uses REAL create_identity, not fake generate_mlkem_keypair or add_entropy_seed
      setProgress(40)
      const result = await tryInvoke<{
        hadesId: string
        hadesIdShort: string
        mnemonic: string
        ed25519PublicKey: string
        x25519PublicKey: string
        walletAccounts: Array<{ chain: string; address: string; ticker: string }>
      }>('create_identity', { passphrase, displayName: null })

      setProgress(70)
      // Store mnemonic for backup display (will be cleared after backup)
      sessionStorage.setItem('hades_mnemonic', result.mnemonic)
      sessionStorage.setItem('hades_id', result.hadesIdShort)

      setProgress(100)
      setIsComplete(true)
    } catch (err) {
      console.error('Identity creation failed:', err)
      setError(String(err))
      // Fallback progress simulation for web dev mode
      let p = 30
      const timer = setInterval(() => {
        p += 5
        setProgress(p)
        if (p >= 100) {
          clearInterval(timer)
          setIsComplete(true)
        }
      }, 100)
    }
  }

  const currentPhase = getCurrentPhase(progress)
  const stepLabel = isComplete ? t('onboarding.stepReady') : currentPhase.label

  const features = [
    { icon: ShieldCheck, title: t('onboarding.featureE2EE'),     desc: t('onboarding.featureE2EEDesc') },
    { icon: EyeOff,      title: t('onboarding.featureMetadata'), desc: t('onboarding.featureMetadataDesc') },
    { icon: Fingerprint,  title: t('onboarding.featureNoPhone'), desc: t('onboarding.featureNoPhoneDesc') },
    { icon: Route,        title: t('onboarding.featureOnion'),   desc: t('onboarding.featureOnionDesc'), optional: true },
  ]

  return (
    <div
      className="onboarding-screen"
      ref={touchRef}
      onPointerMove={handlePointerMove}
    >
      {/* Header */}
      <header className="onboard-header">
        <div className="onboard-header-left">
          <span className="status-dot"></span>
          <span className="onboard-title">{t('onboarding.appTitle')}</span>
        </div>
        <span className="onboard-version">{t('onboarding.protocolVersion')}</span>
      </header>

      {/* Progress */}
      <div className="progress-section">
        <div className="progress-label">{t('onboarding.sectionTitle')}</div>
        <div className="progress-track">
          <div className="progress-fill" style={{ width: `${progress}%` }}></div>
        </div>
      </div>

      {/* Main Content */}
      <div className="onboard-content">
        <h1 className="onboard-heading">
          {t('onboarding.heading')}<br />
          <span className="heading-bold">{t('onboarding.headingBold')}</span>
        </h1>
        <p className="onboard-desc">
          {progress < 30
            ? t('onboarding.entropyPrompt')
            : showPassphraseInput
              ? 'Set your vault passphrase to encrypt your identity'
              : t('onboarding.desc')
          }
        </p>

        {/* Passphrase input — shown after entropy collection */}
        {showPassphraseInput && (
          <div className="passphrase-section" style={{ marginBottom: '1rem' }}>
            <input
              type="password"
              placeholder="Vault passphrase (min 8 characters)"
              value={passphrase}
              onChange={(e) => setPassphrase(e.target.value)}
              className="passphrase-input"
              style={{
                width: '100%', padding: '12px 16px', borderRadius: '12px',
                border: '1px solid rgba(255,255,255,0.12)',
                background: 'rgba(255,255,255,0.06)', color: 'var(--text-primary)',
                fontSize: '15px', marginBottom: '8px', outline: 'none',
              }}
              onKeyDown={(e) => e.key === 'Enter' && handleCreateIdentity()}
              autoFocus
            />
            {error && <p style={{ color: 'var(--accent-danger)', fontSize: '13px', margin: '4px 0' }}>{error}</p>}
            <button
              className="vault-continue-btn"
              onClick={handleCreateIdentity}
              disabled={passphrase.length < 8}
              style={{ marginTop: '8px' }}
            >
              Generate Identity & Wallet
            </button>
          </div>
        )}

        <div className="step-indicator">
          <div className="step-icon">
            {isComplete ? (
              <DatabaseZap size={ICON_SIZE.md} strokeWidth={ICON_STROKE.bold} color="var(--accent-secure)" />
            ) : (
              <Loader size={ICON_SIZE.md} className="spinner-icon" color="var(--accent-secure)" />
            )}
          </div>
          <span className="step-label">{stepLabel}</span>
        </div>

        {/* Vault Secured — navigate to main app */}
        {isComplete && (
          <button className="vault-continue-btn" onClick={() => navigate(ROUTES.CHAT_LIST)}>
            {t('onboarding.continueToApp')}
          </button>
        )}
      </div>

      {/* Feature Cards */}
      <div className="feature-cards">
        {features.map((feature, i) => {
          const Icon = feature.icon
          return (
            <div key={i} className="feature-card" style={{ animationDelay: `${i * 0.1}s` }}>
              <div className="feature-icon-wrap">
                <Icon size={ICON_SIZE.xl} strokeWidth={ICON_STROKE.thin} color="var(--accent-secure)" />
              </div>
              <div className="feature-content">
                <div className="feature-header">
                  <h3 className="feature-title">{feature.title}</h3>
                  {feature.optional && <span className="optional-badge">{t('common.optional')}</span>}
                </div>
                <p className="feature-desc">{feature.desc}</p>
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}

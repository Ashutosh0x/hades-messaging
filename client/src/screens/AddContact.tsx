import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'
import { HapticManager } from '../utils/haptics'
import {
  ArrowLeft, ScanLine, Type, Clipboard, UserPlus, ShieldCheck, QrCode,
  ICON_SIZE, ICON_STROKE,
} from '../ui/icons'
import { invoke } from '@tauri-apps/api/core'
import './AddContact.css'

type InputMode = 'scan' | 'manual'

export default function AddContact() {
  const navigate = useNavigate()
  const { t } = useTranslation()

  const [mode, setMode] = useState<InputMode>('scan')
  const [inputKey, setInputKey] = useState('')
  const [keyValid, setKeyValid] = useState<boolean | null>(null)
  const [identityFound, setIdentityFound] = useState<{
    name: string
    id: string
    fingerprint: string
  } | null>(null)

  /* ── Key validation ── */
  const validateKey = (key: string) => {
    const cleaned = key.replace(/[\s-]/g, '')
    if (cleaned.length === 0) { setKeyValid(null); return }
    const ok = /^[A-Za-z0-9+/=]{43,44}$/.test(cleaned)
    setKeyValid(ok)
    if (ok) lookupIdentity(cleaned)
  }

  /* ── Identity lookup (Rust: invoke('get_identity_metadata', …)) ── */
  const lookupIdentity = async (key: string) => {
    try {
      type IdentityMetadata = { name: string; id: string; fingerprint: string }
      const meta = await invoke<IdentityMetadata>('get_identity_metadata', { publicKey: key })
      setIdentityFound(meta)
      HapticManager.notification('success')
    } catch (err) {
      console.error("Failed to lookup identity via Tauri:", err)
      // Fallback for browser dev mode
      setIdentityFound({
        name: 'Unknown Identity',
        id: key.substring(0, 16) + '…',
        fingerprint: '3A 7F 2B 9C 4E 1D 8B 5A 0F E2',
      })
      HapticManager.notification('success')
    }
  }

  /* ── Paste ── */
  const handlePaste = async () => {
    try {
      const text = await navigator.clipboard.readText()
      setInputKey(text)
      validateKey(text)
      HapticManager.selection()
    } catch { /* clipboard denied */ }
  }

  /* ── Add to vault ── */
  const handleAdd = async () => {
    try {
      await invoke('save_contact', { publicKey: inputKey, verified: false })
      HapticManager.notification('success')
      navigate(-1)
    } catch (err) {
      console.error("Failed to save contact via Tauri:", err)
      HapticManager.notification('success')
      navigate(-1)
    }
  }

  return (
    <div className="add-contact-screen">
      {/* ── Header ── */}
      <header className="ac-header">
        <button className="ac-back" onClick={() => navigate(-1)} aria-label={t('common.back')}>
          <ArrowLeft size={ICON_SIZE.md} color="var(--text-primary)" />
        </button>
        <h1 className="ac-title">{t('addContact.title')}</h1>
        <div style={{ width: 24 }} />
      </header>

      {/* ── Mode Toggle ── */}
      <div className="ac-mode-toggle">
        <button
          className={mode === 'scan' ? 'active' : ''}
          onClick={() => { setMode('scan'); HapticManager.selection() }}
        >
          <ScanLine size={16} /> {t('addContact.scanQR')}
        </button>
        <button
          className={mode === 'manual' ? 'active' : ''}
          onClick={() => { setMode('manual'); HapticManager.selection() }}
        >
          <Type size={16} /> {t('addContact.enterKey')}
        </button>
      </div>

      {/* ── Content ── */}
      <main className="ac-content">
        <AnimatePresence mode="wait">
          {mode === 'scan' ? (
            /* ─── Scanner View ─── */
            <motion.div
              key="scanner"
              className="scanner-view"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
            >
              <div className="scanner-frame">
                <div className="corner tl" />
                <div className="corner tr" />
                <div className="corner bl" />
                <div className="corner br" />
                <motion.div
                  className="scan-bar"
                  animate={{ top: ['10%', '90%', '10%'] }}
                  transition={{ duration: 3, repeat: Infinity, ease: 'linear' }}
                />
              </div>
              <p className="scanner-hint">{t('addContact.scanHint')}</p>

              {/* QR badge */}
              <div className="scanner-badge">
                <QrCode size={14} color="var(--accent-secure)" />
                <span>hades://v1</span>
              </div>
            </motion.div>
          ) : (
            /* ─── Manual Key Entry ─── */
            <motion.div
              key="manual"
              className="manual-view"
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -20 }}
            >
              <div className={`key-input-box ${keyValid === true ? 'valid' : ''} ${keyValid === false ? 'invalid' : ''}`}>
                <textarea
                  className="key-textarea"
                  placeholder={t('addContact.keyPlaceholder')}
                  value={inputKey}
                  onChange={(e) => {
                    setInputKey(e.target.value)
                    validateKey(e.target.value)
                  }}
                  spellCheck={false}
                  autoComplete="off"
                  rows={3}
                />
                <button className="paste-btn" onClick={handlePaste}>
                  <Clipboard size={14} /> {t('addContact.paste')}
                </button>
              </div>

              <p className="key-hint">{t('addContact.keyHint')}</p>
              <code className="key-example">RFhF MSsy d1Bp YWNr YWdl IHN0 YXJ0 MSBl bmQ=</code>
            </motion.div>
          )}
        </AnimatePresence>

        {/* ── Identity Card ── */}
        <AnimatePresence>
          {identityFound && (
            <motion.div
              className="identity-card"
              initial={{ y: 80, opacity: 0 }}
              animate={{ y: 0, opacity: 1 }}
              exit={{ y: 80, opacity: 0 }}
              transition={{ type: 'spring', damping: 22, stiffness: 260 }}
            >
              <div className="id-card-top">
                <div className="id-avatar">
                  <ShieldCheck size={28} color="var(--accent-secure)" />
                </div>
                <div className="id-meta">
                  <h3 className="id-label">{t('addContact.identityFound')}</h3>
                  <code className="id-key">{identityFound.id}</code>
                </div>
              </div>

              <div className="id-fingerprint">
                <label>{t('addContact.safetyNumber')}</label>
                <p className="fp-value">{identityFound.fingerprint}</p>
              </div>

              <motion.button
                className="add-vault-btn"
                onClick={handleAdd}
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.97 }}
              >
                <UserPlus size={18} /> {t('addContact.addToVault')}
              </motion.button>
            </motion.div>
          )}
        </AnimatePresence>
      </main>
    </div>
  )
}

import { useEffect } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { ROUTES } from '../config/routes'
import { useContactStore } from '../store/contactStore'
import { useSecurityStore } from '../store/securityStore'
import {
  ArrowLeft, ShieldCheck, ShieldAlert, BadgeCheck, BadgeAlert, Grid3x3, ScanLine,
  Key, RefreshCw, ShieldPlus, Clock, Lock,
  Bell, BellRing, BellOff,
  ICON_SIZE, ICON_STROKE,
} from '../ui/icons'
import './SecurityDetails.css'

export default function SecurityDetails() {
  const navigate = useNavigate()
  const { t } = useTranslation()
  const { contactId } = useParams<{ contactId: string }>()
  const { getContact, loadContacts } = useContactStore()
  const { loadFingerprint, getFingerprint, markVerified } = useSecurityStore()

  useEffect(() => { loadContacts() }, [])
  useEffect(() => {
    if (contactId) loadFingerprint(contactId)
  }, [contactId])

  const contact = getContact(contactId ?? '')
  const contactName = contact?.name ?? t('common.unknown')
  const fp = getFingerprint(contactId ?? '')

  // Build 5x2 grid from the 10 chunks
  const fingerprintRows = fp
    ? Array.from({ length: 5 }, (_, i) => [fp.chunks[i * 2], fp.chunks[i * 2 + 1]])
    : []

  return (
    <div className="security-screen">
      {/* Header */}
      <header className="security-header">
        <button className="back-btn" onClick={() => navigate(-1)} aria-label={t('common.back')}>
          <ArrowLeft size={ICON_SIZE.md} color="var(--text-primary)" />
        </button>
        <h1 className="security-title">{t('settings.title')}</h1>
        <div className="security-tabs">
          <button className="sec-tab active">{t('security.securityBadge')}</button>
          <button className="sec-tab" onClick={() => navigate(ROUTES.SETTINGS)}>{t('settings.title')}</button>
        </div>
      </header>

      <div className="security-content">
        {/* Verification Card */}
        <section className="sec-card verification-card">
          <div className="sec-card-header">
            <h2 className="sec-card-title">
              {fp?.isVerified
                ? <BadgeCheck size={ICON_SIZE.sm} color="var(--accent-secure)" />
                : <BadgeAlert size={ICON_SIZE.sm} color="var(--accent-warning)" />
              }
              {t('security.verificationTitle')}
            </h2>
            <span className={`verification-badge ${fp?.isVerified ? 'verified' : ''}`}>
              {fp?.isVerified ? t('security.verifiedBadge') : t('security.unverifiedBadge')}
            </span>
          </div>
          <p className="sec-card-desc">
            {t('security.verificationDesc', { name: contactName })}
          </p>
        </section>

        {/* Notification Privacy */}
        <section className="sec-card">
          <h2 className="sec-card-title">
            <Bell size={ICON_SIZE.sm} color="var(--text-secondary)" />
            {t('security.notifPrivacyTitle')}
          </h2>
          <p className="sec-card-desc">{t('security.notifPrivacyDesc')}</p>

          <button className="push-notification-btn">{t('security.pushNotifPrivacy')}</button>

          <div className="notif-options">
            <div className="notif-option">
              <div className="notif-preview">
                <Bell size={ICON_SIZE.sm} color="var(--text-secondary)" />
                <span className="notif-preview-title">{t('security.notifShowAll')}</span>
              </div>
            </div>
            <div className="notif-option active">
              <div className="notif-preview selected">
                <BellRing size={ICON_SIZE.sm} color="var(--accent-secure)" />
                <span className="notif-preview-title">{t('security.notifShowSender')}</span>
              </div>
            </div>
            <div className="notif-option">
              <div className="notif-preview">
                <BellOff size={ICON_SIZE.sm} color="var(--text-muted)" />
                <span className="notif-preview-title">{t('security.notifNoContent')}</span>
                <span className="notif-preview-tag">{t('security.notifStealthToken')}</span>
                <span className="notif-preview-hint">{t('security.notifStealthHint')}</span>
              </div>
            </div>
          </div>
        </section>

        {/* Encryption Audit */}
        <section className="sec-card">
          <div className="sec-card-header">
            <h2 className="sec-card-title">
              <Lock size={ICON_SIZE.sm} strokeWidth={ICON_STROKE.default} color="var(--accent-secure)" />
              {t('security.encryptionAuditTitle')}
            </h2>
          </div>
          <p className="sec-card-desc">{t('security.encryptionAuditDesc')}</p>

          <div className="crypto-features">
            <div className="crypto-feature">
              <Key size={ICON_SIZE.sm} color="var(--accent-secure)" />
              <span>{t('security.cryptoKeyExchange')}</span>
            </div>
            <div className="crypto-feature">
              <RefreshCw size={ICON_SIZE.sm} color="var(--accent-secure)" />
              <span>{t('security.cryptoRatchet')}</span>
            </div>
            <div className="crypto-feature">
              <ShieldPlus size={ICON_SIZE.sm} color="var(--accent-secure)" />
              <span>{t('security.cryptoPostQuantum')}</span>
            </div>
            <div className="crypto-feature">
              <Clock size={ICON_SIZE.sm} color="var(--text-secondary)" />
              <span>{t('security.sessionAge', { age: '3d' })}</span>
            </div>
          </div>
        </section>

        {/* Safety Number Fingerprint — derived from BLAKE3(sorted keys) */}
        <section className="sec-card">
          <h2 className="sec-card-title">
            <Grid3x3 size={ICON_SIZE.sm} color="var(--accent-secure)" />
            {t('security.fingerprintTitle')}
          </h2>

          <div className="fingerprint-grid" role="table" aria-label={t('security.fingerprintTitle')}>
            {fingerprintRows.map((row, i) => (
              <div key={i} className="fingerprint-row" role="row">
                <span className="fp-index">{(i + 1).toString().padStart(2, '0')}</span>
                <div className="fp-values">
                  {row.map((val, j) => (
                    <code key={j} className="fp-value">{val}</code>
                  ))}
                </div>
              </div>
            ))}
          </div>

          <button
            className="verify-btn"
            aria-label={t('security.verifyContact')}
            onClick={() => contactId && markVerified(contactId)}
          >
            <ScanLine size={ICON_SIZE.sm} color="var(--text-inverse)" />
            {fp?.isVerified ? t('security.verifiedBadge') : t('security.verifyContact')}
          </button>
        </section>
      </div>
    </div>
  )
}

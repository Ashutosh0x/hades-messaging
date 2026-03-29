import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { ROUTES } from '../config/routes'
import { useSecurityStore, NotificationConfig } from '../store/securityStore'
import { useConnectionStore } from '../store/connectionStore'
import { useDeviceStore, LinkedDevice } from '../store/deviceStore'
import { useSettingsStore, ClipboardTimeout, SelfDestructTimer } from '../store/settingsStore'
import { useToastStore } from '../store/toastStore'
import { useSecureRoute } from '../hooks/useSecureRoute'
import {
  ArrowLeft, ShieldCheck, EyeOff, Eye, Clock, GlobeLock, Wifi, WifiOff, Laptop, Smartphone, Tablet,
  Flame, Trash2, Bell, Timer, ChevronRight, Ban, Key, UserRound, AlertTriangle,
  ICON_SIZE, ICON_STROKE,
} from '../ui/icons'
import './Settings.css'

// Safe invoke wrapper — no crash in browser dev mode
async function tryInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    return await invoke<T>(cmd, args)
  } catch {
    return null
  }
}

/** Cycle-through picker values */
const CLIPBOARD_OPTIONS: ClipboardTimeout[] = ['1 min', '3 min', '5 min', '10 min', 'Never']
const SELF_DESTRUCT_OPTIONS: SelfDestructTimer[] = ['Off', '5 min', '1 hour', '24 hours', '7 days']

export default function Settings() {
  const navigate = useNavigate()
  const { t, i18n } = useTranslation()

  // Stores
  const settings = useSettingsStore()
  const { notificationConfig, setNotificationConfig } = useSecurityStore()
  const { status: connectionStatus, stage: connectionStage, progress: connectionProgress } = useConnectionStore()
  const { devices, revokeDevice } = useDeviceStore()
  const { addToast } = useToastStore()
  const { establishRoute } = useSecureRoute()

  // Confirmation dialog state
  const [confirmAction, setConfirmAction] = useState<null | 'wipe' | 'delete'>(null)

  // ── Device helpers ──
  const getDeviceIcon = (type: LinkedDevice['type']) => {
    switch (type) {
      case 'desktop': return <Laptop size={ICON_SIZE.md} color="var(--text-secondary)" />
      case 'tablet': return <Tablet size={ICON_SIZE.md} color="var(--text-secondary)" />
      case 'phone': return <Smartphone size={ICON_SIZE.md} color="var(--text-secondary)" />
    }
  }

  // ── Relay helpers ──
  const getRelayStatusLabel = () => {
    switch (connectionStatus) {
      case 'established': return connectionStage || 'Connected to Hades Relay'
      case 'establishing':
      case 'connecting': return connectionStage || 'Connecting…'
      case 'error': return 'Connection failed'
      default: return 'Not connected'
    }
  }

  const getRelayBadge = () => {
    switch (connectionStatus) {
      case 'established': return <span className="status-badge connected">STABLE</span>
      case 'establishing':
      case 'connecting': return <span className="status-badge connecting">{connectionProgress}%</span>
      case 'error': return <span className="status-badge revoke">ERROR</span>
      default: return <span className="status-badge disconnected">OFFLINE</span>
    }
  }

  // ── Action handlers ──
  const handleRevokeDevice = (device: LinkedDevice) => {
    if (device.isCurrentDevice) {
      addToast('Cannot revoke current device', 'warning')
      return
    }
    revokeDevice(device.id)
    addToast(`${device.name} has been revoked`, 'security')
  }

  const handleToggleScreenshotGuard = () => {
    settings.toggleScreenshotGuard()
    addToast(
      settings.screenshotGuard ? 'Screenshot Guard Disabled' : 'Screenshot Guard Active',
      settings.screenshotGuard ? 'warning' : 'security'
    )
  }

  const handleToggleRouting = () => {
    settings.toggleAnonymityRouting()
    if (!settings.anonymityRouting) {
      addToast('Anonymity routing enabled', 'security')
    } else {
      addToast('Anonymity routing disabled', 'warning')
    }
  }

  const handleEnableRouting = () => {
    if (connectionStatus === 'established') {
      addToast('Already connected to secure route', 'security')
      return
    }
    establishRoute()
    addToast('Establishing secure route…', 'security')
  }

  const cycleClipboardTimeout = () => {
    const idx = CLIPBOARD_OPTIONS.indexOf(settings.clipboardTimeout)
    const next = CLIPBOARD_OPTIONS[(idx + 1) % CLIPBOARD_OPTIONS.length]
    settings.setClipboardTimeout(next)
    addToast(`Clipboard auto-wipe: ${next}`, 'security')
  }

  const cycleSelfDestructTimer = () => {
    const idx = SELF_DESTRUCT_OPTIONS.indexOf(settings.selfDestructTimer)
    const next = SELF_DESTRUCT_OPTIONS[(idx + 1) % SELF_DESTRUCT_OPTIONS.length]
    settings.setSelfDestructTimer(next)
    addToast(`Self-destruct timer: ${next}`, 'security')
  }

  const handleConfirmDangerAction = () => {
    if (confirmAction === 'wipe') {
      addToast('Emergency wipe initiated — all local data destroyed', 'warning')
      tryInvoke('emergency_wipe').catch(console.error)
    } else if (confirmAction === 'delete') {
      addToast('Account deletion requested — keys revoked', 'warning')
      tryInvoke('emergency_wipe').catch(console.error) // Uses the same wipe command
    }
    setConfirmAction(null)
  }

  return (
    <div className="settings-screen">
      {/* Header */}
      <header className="settings-header">
        <button className="back-btn" onClick={() => navigate(ROUTES.CHAT_LIST)} aria-label={t('common.back')}>
          <ArrowLeft size={ICON_SIZE.md} color="var(--text-primary)" />
        </button>
        <h1 className="settings-title">{t('settings.title')}</h1>
        <button className="settings-shield" aria-label={t('security.securityBadge')}>
          <ShieldCheck size={ICON_SIZE.sm} color="var(--accent-secure)" />
        </button>
      </header>

      {/* Scrollable Content */}
      <div className="settings-content">
        {/* Profile Card */}
        <section className="settings-section profile-summary-section" style={{ marginTop: 0 }}>
          <button className="setting-row" onClick={() => navigate(ROUTES.PROFILE)}>
            <div className="setting-info">
              <span className="setting-label">
                <UserRound size={ICON_SIZE.sm} color="var(--accent-secure)" />
                {t('settings.editProfile', 'Edit Profile & Identity')}
              </span>
              <span className="setting-hint">{t('settings.editProfileHint', 'Name and E2EE Avatar')}</span>
            </div>
            <ChevronRight size={16} color="var(--text-muted)" />
          </button>

          <div className="setting-row" style={{ borderBottom: 'none' }}>
            <div className="setting-info">
              <span className="setting-label">
                <GlobeLock size={ICON_SIZE.sm} color="var(--text-secondary)" />
                {t('settings.language', 'Language')}
              </span>
            </div>
            <select 
              className="language-select" 
              value={i18n.language?.split('-')[0] || 'en'} 
              onChange={(e) => {
                i18n.changeLanguage(e.target.value)
                addToast(`Language switched to ${e.target.selectedOptions[0].text}`, 'security')
              }}
              aria-label="Select application language"
            >
              <option value="en">English (US)</option>
              <option value="es">Español</option>
              <option value="fr">Français</option>
              <option value="de">Deutsch</option>
            </select>
          </div>
        </section>

        {/* Privacy Code Banner */}
        <div className="privacy-code-banner">
          <ShieldCheck size={ICON_SIZE.sm} color="var(--accent-secure)" />
          <span className="pcode-label">{t('settings.privacyCodeActive')}</span>
          <p className="pcode-desc">{t('settings.privacyCodeDesc')}</p>
        </div>

        {/* Privacy Controls */}
        <section className="settings-section">
          <h2 className="section-title">{t('settings.sectionPrivacy')}</h2>
          <p className="section-desc">{t('settings.sectionPrivacyDesc')}</p>

          <div className="setting-row">
            <div className="setting-info">
              <span className="setting-label">
                <EyeOff size={ICON_SIZE.sm} color="var(--text-secondary)" />
                {t('settings.readReceipts')}
              </span>
              <span className="setting-hint">{t('settings.readReceiptsHint')}</span>
            </div>
            <button className={`toggle ${settings.readReceipts ? 'active' : ''}`} onClick={settings.toggleReadReceipts} role="switch" aria-checked={settings.readReceipts}>
              <div className="toggle-thumb"></div>
            </button>
          </div>

          <div className="setting-row">
            <div className="setting-info">
              <span className="setting-label">
                <Eye size={ICON_SIZE.sm} color="var(--text-secondary)" />
                {t('settings.typingIndicators')}
              </span>
              <span className="setting-hint">{t('settings.typingIndicatorsHint')}</span>
            </div>
            <button className={`toggle ${settings.typingIndicators ? 'active' : ''}`} onClick={settings.toggleTypingIndicators} role="switch" aria-checked={settings.typingIndicators}>
              <div className="toggle-thumb"></div>
            </button>
          </div>

          <div className="setting-row">
            <div className="setting-info">
              <span className="setting-label">{t('settings.metadataMin')}</span>
              <span className="setting-hint">{t('settings.metadataMinHint')}</span>
            </div>
            <span className="active-badge">{t('common.active')}</span>
          </div>

          <button className="setting-row" onClick={cycleSelfDestructTimer}>
            <div className="setting-info">
              <span className="setting-label">
                <Timer size={ICON_SIZE.sm} color="var(--accent-secure)" />
                {t('settings.selfDestruct')}
              </span>
              <span className="setting-hint">Tap to cycle through options</span>
            </div>
            <span className="timer-badge">{settings.selfDestructTimer}</span>
          </button>
        </section>

        {/* Network & Nodes */}
        <section className="settings-section">
          <h2 className="section-title">{t('settings.sectionNetwork')}</h2>

          <div className="setting-row">
            <div className="setting-info">
              <span className="setting-label">
                <GlobeLock size={ICON_SIZE.sm} color="var(--accent-secure)" />
                {t('settings.anonymityRouting')}
              </span>
              <span className="setting-hint">{t('settings.anonymityRoutingHint')}</span>
            </div>
            <button className={`toggle ${settings.anonymityRouting ? 'active' : ''}`} onClick={handleToggleRouting} role="switch" aria-checked={settings.anonymityRouting}>
              <div className="toggle-thumb"></div>
            </button>
          </div>

          <button className="enable-routing-btn" onClick={handleEnableRouting}>
            {connectionStatus === 'established' ? '✓ ROUTE ACTIVE' : t('settings.enableRouting')}
          </button>

          <div className="setting-row">
            <div className="setting-info">
              <span className="setting-label">
                {connectionStatus === 'established'
                  ? <Wifi size={ICON_SIZE.sm} color="var(--accent-secure)" />
                  : <WifiOff size={ICON_SIZE.sm} color="var(--text-muted)" />
                }
                {t('settings.relayStatus')}
              </span>
              <span className="setting-hint">{getRelayStatusLabel()}</span>
            </div>
            {getRelayBadge()}
          </div>
        </section>

        {/* Linked Devices */}
        <section className="settings-section">
          <h2 className="section-title">{t('settings.sectionDevices')}</h2>

          {devices.length === 0 && (
            <div className="empty-devices">
              <p>No linked devices</p>
            </div>
          )}

          {devices.map((device) => (
            <div className="device-card" key={device.id}>
              <div className="device-icon-wrap">
                {getDeviceIcon(device.type)}
              </div>
              <div className="device-info">
                <span className="device-name">
                  {device.name}
                  {device.isCurrentDevice && <span className="this-device-tag">This device</span>}
                </span>
                <span className="device-key">{'0x' + device.publicKey.slice(0, 12)}</span>
              </div>
              {device.isCurrentDevice ? (
                <span className="status-badge connected">CONNECTED</span>
              ) : (
                <button
                  className="status-badge revoke"
                  onClick={() => handleRevokeDevice(device)}
                >
                  REVOKE
                </button>
              )}
            </div>
          ))}
        </section>

        {/* Security Hardening */}
        <section className="settings-section">
          <h2 className="section-title">{t('settings.sectionSecurity')}</h2>

          <div className="setting-row" style={{ flexDirection: 'column', alignItems: 'flex-start', gap: '8px' }}>
            <div className="setting-info" style={{ width: '100%' }}>
              <span className="setting-label">
                <Bell size={ICON_SIZE.sm} color="var(--accent-secure)" />
                {t('settings.sealedNotifications')}
              </span>
              <span className="setting-hint">{t('settings.sealedNotificationsHint')}</span>
            </div>
            <div className="segmented-control">
              {(['sealed', 'sender_only', 'full'] as NotificationConfig[]).map((cfg) => (
                <button 
                  key={cfg}
                  className={`segment-btn ${notificationConfig === cfg ? 'active' : ''}`}
                  onClick={() => {
                    setNotificationConfig(cfg)
                    addToast(`Notification mode: ${cfg.replace('_', ' ')}`, 'security')
                  }}
                >
                  {t(`settings.notif_${cfg}`)}
                </button>
              ))}
            </div>
          </div>

          <div className="setting-row">
            <div className="setting-info">
              <span className="setting-label">
                <Ban size={ICON_SIZE.sm} color="var(--accent-secure)" />
                {t('settings.screenshotGuard')}
              </span>
              <span className="setting-hint">{t('settings.screenshotGuardHint')}</span>
            </div>
            <button className={`toggle ${settings.screenshotGuard ? 'active' : ''}`} onClick={handleToggleScreenshotGuard} role="switch" aria-checked={settings.screenshotGuard}>
              <div className="toggle-thumb"></div>
            </button>
          </div>

          <div className="setting-row">
            <div className="setting-info">
              <span className="setting-label">
                <EyeOff size={ICON_SIZE.sm} color="var(--accent-secure)" />
                {t('settings.incognitoKeyboard')}
              </span>
              <span className="setting-hint">{t('settings.incognitoKeyboardHint')}</span>
            </div>
            <button className={`toggle ${settings.incognitoKeyboard ? 'active' : ''}`} onClick={() => {
              settings.toggleIncognitoKeyboard()
              addToast(settings.incognitoKeyboard ? 'Incognito Keyboard disabled' : 'Incognito Keyboard enabled', 'security')
            }} role="switch" aria-checked={settings.incognitoKeyboard}>
              <div className="toggle-thumb"></div>
            </button>
          </div>

          <button className="setting-row" onClick={cycleClipboardTimeout}>
            <div className="setting-info">
              <span className="setting-label">
                <Clock size={ICON_SIZE.sm} color="var(--text-secondary)" />
                {t('settings.clipboardTimer')}
              </span>
              <span className="setting-hint">{t('settings.clipboardTimerHint')}</span>
            </div>
            <span className="timer-badge">{settings.clipboardTimeout}</span>
          </button>

          <button className="setting-row recovery-row" onClick={() => navigate(ROUTES.RECOVERY_PHRASE)}>
            <div className="setting-info">
              <span className="setting-label">
                <Key size={ICON_SIZE.sm} color="var(--accent-secure)" />
                {t('settings.recoveryPhrase')}
              </span>
              <span className="setting-hint">{t('settings.recoveryPhraseHint')}</span>
            </div>
            <ChevronRight size={ICON_SIZE.sm} color="var(--text-muted)" />
          </button>
        </section>

        {/* Danger Zone */}
        <section className="settings-section danger-section">
          <h2 className="section-title">{t('settings.dangerZone', 'Danger Zone')}</h2>
          <button className="setting-row danger-row" onClick={() => setConfirmAction('wipe')}>
            <div className="setting-info">
              <span className="setting-label danger-label">
                <Flame size={ICON_SIZE.sm} color="var(--danger)" />
                {t('settings.emergencyWipe')}
              </span>
              <span className="setting-hint">{t('settings.emergencyWipeHint')}</span>
            </div>
            <ChevronRight size={ICON_SIZE.sm} color="var(--text-muted)" />
          </button>
          <button className="setting-row danger-row" onClick={() => setConfirmAction('delete')}>
            <div className="setting-info">
              <span className="setting-label danger-label">
                <Trash2 size={ICON_SIZE.sm} color="var(--danger)" />
                {t('settings.deleteAccount')}
              </span>
              <span className="setting-hint">{t('settings.deleteAccountHint')}</span>
            </div>
            <ChevronRight size={ICON_SIZE.sm} color="var(--text-muted)" />
          </button>
        </section>
      </div>

      {/* Confirmation Dialog */}
      {confirmAction && (
        <div className="confirm-overlay" onClick={() => setConfirmAction(null)}>
          <div className="confirm-dialog" onClick={(e) => e.stopPropagation()}>
            <div className="confirm-icon">
              <AlertTriangle size={32} color="var(--danger)" />
            </div>
            <h3 className="confirm-title">
              {confirmAction === 'wipe' ? 'Emergency Wipe' : 'Delete Account'}
            </h3>
            <p className="confirm-desc">
              {confirmAction === 'wipe'
                ? 'This will permanently destroy all local encryption keys, messages, and contacts. This action cannot be undone.'
                : 'This will revoke all device keys and delete your identity from the network. You will lose access to all conversations permanently.'
              }
            </p>
            <div className="confirm-actions">
              <button className="confirm-cancel" onClick={() => setConfirmAction(null)}>Cancel</button>
              <button className="confirm-danger" onClick={handleConfirmDangerAction}>
                {confirmAction === 'wipe' ? 'Wipe Everything' : 'Delete Forever'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

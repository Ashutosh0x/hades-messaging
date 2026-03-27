import { useState, useEffect } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { ROUTES, buildRoute } from '../config/routes'
import { UI } from '../config/constants'
import { useContactStore } from '../store/contactStore'
import { formatTimer } from '../utils/time'
import {
  UserRound, ShieldCheck, Mic, MicOff, Volume2, VolumeX, Video, PhoneOff,
  Plus, Pause, Signal,
  ICON_SIZE, ICON_STROKE,
} from '../ui/icons'
import './VoiceCall.css'

export default function VoiceCall() {
  const navigate = useNavigate()
  const { t } = useTranslation()
  const { contactId } = useParams<{ contactId: string }>()
  const { getContact, loadContacts } = useContactStore()
  const [elapsed, setElapsed] = useState(0)
  const [muted, setMuted] = useState(false)
  const [speaker, setSpeaker] = useState(false)

  useEffect(() => { loadContacts() }, [])

  useEffect(() => {
    const timer = setInterval(() => setElapsed(s => s + 1), 1000)
    return () => clearInterval(timer)
  }, [])

  const contact = getContact(contactId ?? '')

  return (
    <div className="voice-call-screen">
      <div className="voice-header">
        <div className="voice-secure-badge">
          <ShieldCheck size={12} color="var(--accent-secure)" />
          <span>{t('security.secured')}</span>
        </div>
        <Signal size={ICON_SIZE.sm} color="var(--accent-secure)" />
      </div>

      <div className="voice-center">
        <div className="voice-avatar-glow">
          <div className="voice-avatar">
            <UserRound size={56} strokeWidth={ICON_STROKE.thin} />
          </div>
        </div>
        <h1 className="voice-contact-name">{contact?.name ?? t('common.unknown')}</h1>
        <span className="voice-timer">{formatTimer(elapsed)}</span>
      </div>

      <div className="audio-visualizer" aria-hidden="true">
        {Array.from({ length: UI.AUDIO_BARS_COUNT }).map((_, i) => (
          <div
            key={i}
            className="audio-bar"
            style={{
              '--bar-h': `${8 + Math.random() * 28}px`,
              animationDelay: `${i * 0.05}s`,
            } as React.CSSProperties}
          />
        ))}
      </div>

      <div className="voice-controls">
        <button className={`voice-ctrl-btn ${muted ? 'active' : ''}`} onClick={() => setMuted(!muted)} aria-label={muted ? t('calls.unmute') : t('calls.mute')}>
          {muted ? <MicOff size={ICON_SIZE.md} /> : <Mic size={ICON_SIZE.md} />}
          <span>{muted ? t('calls.unmute') : t('calls.mute')}</span>
        </button>
        <button className={`voice-ctrl-btn ${speaker ? 'active' : ''}`} onClick={() => setSpeaker(!speaker)} aria-label={t('calls.speaker')}>
          {speaker ? <VolumeX size={ICON_SIZE.md} /> : <Volume2 size={ICON_SIZE.md} />}
          <span>{t('calls.speaker')}</span>
        </button>
        <button className="voice-ctrl-btn" onClick={() => navigate(buildRoute(ROUTES.VIDEO_CALL, { contactId: contactId ?? '' }))} aria-label={t('calls.video')}>
          <Video size={ICON_SIZE.md} />
          <span>{t('calls.video')}</span>
        </button>
        <button className="voice-ctrl-btn" aria-label={t('calls.add')}>
          <Plus size={ICON_SIZE.md} />
          <span>{t('calls.add')}</span>
        </button>
        <button className="voice-ctrl-btn" aria-label={t('calls.keypad')}>
          <span className="keypad-icon">⌨</span>
          <span>{t('calls.keypad')}</span>
        </button>
        <button className="voice-ctrl-btn" aria-label={t('calls.hold')}>
          <Pause size={ICON_SIZE.md} />
          <span>{t('calls.hold')}</span>
        </button>
      </div>

      <div className="voice-end-section">
        <button className="end-call-btn" onClick={() => navigate(ROUTES.CHAT_LIST)} aria-label={t('calls.endCall')}>
          <PhoneOff size={ICON_SIZE.lg} />
        </button>
      </div>
    </div>
  )
}

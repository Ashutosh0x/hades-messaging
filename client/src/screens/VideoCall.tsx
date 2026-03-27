import { useState, useEffect } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { ROUTES } from '../config/routes'
import { TIMERS } from '../config/constants'
import { useContactStore } from '../store/contactStore'
import { formatTimer } from '../utils/time'
import {
  Mic, MicOff, VideoOff, SwitchCamera, PhoneOff, MoreVertical,
  ShieldCheck, ArrowDown,
  ICON_SIZE,
} from '../ui/icons'
import './VideoCall.css'

export default function VideoCall() {
  const navigate = useNavigate()
  const { t } = useTranslation()
  const { contactId } = useParams<{ contactId: string }>()
  const { getContact, loadContacts } = useContactStore()
  const [elapsed, setElapsed] = useState(0)
  const [showControls, setShowControls] = useState(true)
  const [muted, setMuted] = useState(false)
  const [cameraOff, setCameraOff] = useState(false)

  useEffect(() => { loadContacts() }, [])

  useEffect(() => {
    const timer = setInterval(() => setElapsed(s => s + 1), 1000)
    return () => clearInterval(timer)
  }, [])

  useEffect(() => {
    if (!showControls) return
    const timeout = setTimeout(() => setShowControls(false), TIMERS.CONTROLS_AUTO_HIDE_MS)
    return () => clearTimeout(timeout)
  }, [showControls])

  const contact = getContact(contactId ?? '')

  return (
    <div className="video-call-screen" onClick={() => setShowControls(v => !v)}>
      <div className="remote-video">
        <div className="remote-video-placeholder">
          <div className="video-gradient" />
        </div>
      </div>

      <div className="local-pip">
        <div className="pip-placeholder">
          {cameraOff ? (
            <VideoOff size={ICON_SIZE.md} color="var(--text-muted)" />
          ) : (
            <div className="pip-gradient" />
          )}
        </div>
      </div>

      <div className={`video-top-overlay ${showControls ? 'visible' : ''}`}>
        <button className="video-top-btn" onClick={(e) => { e.stopPropagation(); navigate(-1) }} aria-label={t('common.back')}>
          <ArrowDown size={ICON_SIZE.md} />
        </button>
        <div className="video-top-center">
          <ShieldCheck size={12} color="var(--accent-secure)" />
          <span className="video-encrypted-label">{t('calls.encrypted')}</span>
        </div>
        <button className="video-top-btn" aria-label={t('common.switchCamera')} onClick={(e) => e.stopPropagation()}>
          <SwitchCamera size={ICON_SIZE.md} />
        </button>
      </div>

      <div className={`video-info-overlay ${showControls ? 'visible' : ''}`}>
        <span className="video-contact-name">{contact?.name ?? t('common.unknown')}</span>
        <span className="video-duration">{formatTimer(elapsed)}</span>
        <div className="video-pills">
          <span className="video-pill">
            <ShieldCheck size={10} color="var(--accent-secure)" />
            {t('calls.encrypted')}
          </span>
          <span className="video-pill">{t('calls.hd')}</span>
          <span className="video-pill">{t('calls.fps30')}</span>
        </div>
      </div>

      <div className={`video-bottom-controls ${showControls ? 'visible' : ''}`} onClick={(e) => e.stopPropagation()}>
        <button className={`video-ctrl-btn ${muted ? 'active' : ''}`} onClick={() => setMuted(!muted)} aria-label={muted ? t('calls.unmute') : t('calls.mute')}>
          {muted ? <MicOff size={ICON_SIZE.md} /> : <Mic size={ICON_SIZE.md} />}
        </button>
        <button className={`video-ctrl-btn ${cameraOff ? 'active' : ''}`} onClick={() => setCameraOff(!cameraOff)} aria-label={t('calls.video')}>
          <VideoOff size={ICON_SIZE.md} />
        </button>
        <button className="video-end-btn" onClick={() => navigate(ROUTES.CHAT_LIST)} aria-label={t('calls.endCall')}>
          <PhoneOff size={ICON_SIZE.lg} />
        </button>
        <button className="video-ctrl-btn" aria-label={t('common.moreOptions')}>
          <MoreVertical size={ICON_SIZE.md} />
        </button>
      </div>
    </div>
  )
}

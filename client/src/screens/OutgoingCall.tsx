import { useEffect } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { ROUTES, buildRoute } from '../config/routes'
import { useContactStore } from '../store/contactStore'
import {
  UserRound, ShieldCheck, Mic, Volume2, Video, PhoneOff, X,
  ICON_SIZE, ICON_STROKE,
} from '../ui/icons'
import './OutgoingCall.css'

export default function OutgoingCall() {
  const navigate = useNavigate()
  const { t } = useTranslation()
  const { contactId } = useParams<{ contactId: string }>()
  const { getContact, loadContacts } = useContactStore()

  useEffect(() => { loadContacts() }, [])

  const contact = getContact(contactId ?? '')

  return (
    <div className="outgoing-call-screen">
      <button className="outgoing-cancel" onClick={() => navigate(-1)} aria-label={t('common.cancel')}>
        <X size={ICON_SIZE.lg} />
      </button>

      <div className="outgoing-content">
        <div className="outgoing-avatar">
          <UserRound size={56} strokeWidth={ICON_STROKE.thin} />
        </div>
        <h1 className="outgoing-name">{contact?.name ?? t('common.unknown')}</h1>
        <p className="outgoing-status">{t('calls.calling')}<span className="calling-dots" /></p>
        <div className="outgoing-e2ee">
          <ShieldCheck size={12} color="var(--accent-secure)" />
          <span>{t('security.e2ee')}</span>
        </div>
      </div>

      <div className="outgoing-pre-controls">
        <button className="pre-control-btn" aria-label={t('calls.mute')}>
          <Mic size={ICON_SIZE.md} />
          <span>{t('calls.micOn')}</span>
        </button>
        <button className="pre-control-btn" aria-label={t('calls.speaker')}>
          <Volume2 size={ICON_SIZE.md} />
          <span>{t('calls.speaker')}</span>
        </button>
        <button className="pre-control-btn" aria-label={t('calls.video')} onClick={() => navigate(buildRoute(ROUTES.VIDEO_CALL, { contactId: contactId ?? '' }))}>
          <Video size={ICON_SIZE.md} />
          <span>{t('calls.video')}</span>
        </button>
      </div>

      <div className="outgoing-end-section">
        <button className="end-call-btn" onClick={() => navigate(ROUTES.CHAT_LIST)} aria-label={t('calls.endCall')}>
          <PhoneOff size={ICON_SIZE.lg} />
        </button>
        <span className="end-call-label">{t('calls.endCall')}</span>
      </div>
    </div>
  )
}

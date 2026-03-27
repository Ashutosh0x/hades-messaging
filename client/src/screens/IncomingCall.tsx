import { useEffect } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { ROUTES, buildRoute } from '../config/routes'
import { useContactStore } from '../store/contactStore'
import {
  UserRound, ShieldCheck, Phone, PhoneOff, MessageSquare,
  ICON_SIZE, ICON_STROKE,
} from '../ui/icons'
import './IncomingCall.css'

export default function IncomingCall() {
  const navigate = useNavigate()
  const { t } = useTranslation()
  const { contactId } = useParams<{ contactId: string }>()
  const { getContact, loadContacts } = useContactStore()

  useEffect(() => { loadContacts() }, [])

  const contact = getContact(contactId ?? '')

  return (
    <div className="incoming-call-screen">
      <div className="incoming-call-backdrop" />

      <div className="incoming-call-content">
        <div className="incoming-avatar-ring">
          <div className="incoming-avatar">
            <UserRound size={48} strokeWidth={ICON_STROKE.thin} />
          </div>
        </div>

        <h1 className="incoming-name">{contact?.name ?? t('common.unknown')}</h1>
        <p className="incoming-label">{t('calls.incomingVoiceCall')}</p>

        <div className="incoming-e2ee-badge">
          <ShieldCheck size={12} color="var(--accent-secure)" />
          <span>{t('security.e2ee')}</span>
        </div>

        <div className="incoming-actions">
          <button className="incoming-btn decline" onClick={() => navigate(ROUTES.CHAT_LIST)} aria-label={t('calls.endCall')}>
            <PhoneOff size={ICON_SIZE.lg} />
          </button>
          <button className="incoming-btn accept" onClick={() => navigate(buildRoute(ROUTES.VOICE_CALL, { contactId: contactId ?? '' }))} aria-label={t('calls.voiceCall')}>
            <Phone size={ICON_SIZE.lg} />
          </button>
        </div>

        <button className="incoming-message-btn" onClick={() => navigate(ROUTES.CHAT_LIST)}>
          <MessageSquare size={ICON_SIZE.sm} />
          {t('calls.messageInstead')}
        </button>
      </div>
    </div>
  )
}

import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { ROUTES, buildRoute } from '../config/routes'
import { useContactStore, type Contact } from '../store/contactStore'
import {
  ArrowLeft, Search, ShieldCheck, UserPlus, QrCode, Key,
  BadgeCheck, Phone, Video,
  ICON_SIZE,
} from '../ui/icons'
import './Contacts.css'

const STATUS_KEYS: Record<Contact['status'], string> = {
  online: 'contacts.online',
  recently: 'contacts.lastSeenRecently',
  offline: 'contacts.offline',
}

export default function Contacts() {
  const navigate = useNavigate()
  const { t } = useTranslation()
  const [query, setQuery] = useState('')

  const { loadContacts, verifiedContacts, unverifiedContacts } = useContactStore()

  useEffect(() => { loadContacts() }, [])

  const verified = verifiedContacts().filter(c => c.name.toLowerCase().includes(query.toLowerCase()))
  const unverified = unverifiedContacts().filter(c => c.name.toLowerCase().includes(query.toLowerCase()))

  const sections = [
    { key: 'sectionVerified',   contacts: verified },
    { key: 'sectionUnverified', contacts: unverified },
  ]

  return (
    <div className="contacts-screen">
      <div className="contacts-header">
        <button className="back-btn" onClick={() => navigate(ROUTES.CHAT_LIST)} aria-label={t('common.back')}>
          <ArrowLeft size={ICON_SIZE.md} color="var(--text-primary)" />
        </button>
        <h1 className="contacts-title">{t('contacts.title')}</h1>
        <div className="contacts-shield-badge">
          <ShieldCheck size={14} color="var(--accent-secure)" />
        </div>
      </div>

      <div className="contacts-search">
        <Search size={16} color="var(--text-muted)" />
        <input
          type="text"
          placeholder={t('contacts.searchPlaceholder')}
          value={query}
          onChange={e => setQuery(e.target.value)}
        />
      </div>

      <div className="contacts-list">
        {sections.map(({ key, contacts }) => {
          if (contacts.length === 0) return null
          return (
            <div key={key} className="contact-section">
              <div className="contact-section-label">
                {t(`contacts.${key}`)}
                <span className="contact-section-count">{contacts.length}</span>
              </div>

              {contacts.map(contact => (
                <div key={contact.id} className="contact-card">
                  <div className="contact-avatar">
                    <span>{contact.initial}</span>
                    {contact.verified && <div className="verified-dot" aria-label={t('security.verified')} />}
                  </div>
                  <div className="contact-info">
                    <span className="contact-name">
                      {contact.name}
                      {contact.verified && <BadgeCheck size={14} color="var(--accent-secure)" />}
                    </span>
                    <span className="contact-status">{t(STATUS_KEYS[contact.status])}</span>
                  </div>
                  <div className="contact-actions">
                    <button className="contact-action-btn" onClick={() => navigate(buildRoute(ROUTES.VOICE_CALL, { contactId: contact.id }))} aria-label={t('calls.voiceCall')}>
                      <Phone size={16} color="var(--text-secondary)" />
                    </button>
                    <button className="contact-action-btn" onClick={() => navigate(buildRoute(ROUTES.VIDEO_CALL, { contactId: contact.id }))} aria-label={t('calls.videoCall')}>
                      <Video size={16} color="var(--text-secondary)" />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )
        })}
      </div>

      <div className="contacts-add-section">
        <div className="add-contact-cta">
          <div className="add-cta-icon">
            <UserPlus size={24} color="var(--accent-secure)" />
          </div>
          <div className="add-cta-content">
            <span className="add-cta-title">{t('contacts.addNewContact')}</span>
            <span className="add-cta-desc">{t('contacts.addNewContactDesc')}</span>
          </div>
        </div>

        <div className="add-methods">
          <button className="add-method-btn">
            <QrCode size={16} color="var(--accent-secure)" />
            <span>{t('contacts.scanQr')}</span>
          </button>
          <button className="add-method-btn">
            <Key size={16} color="var(--text-secondary)" />
            <span>{t('contacts.enterKey')}</span>
          </button>
        </div>
      </div>
    </div>
  )
}

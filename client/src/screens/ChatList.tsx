import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { ROUTES, buildRoute } from '../config/routes'
import { useConversationStore } from '../store/conversationStore'
import { useSecurityStore } from '../store/securityStore'
import { useContactStore } from '../store/contactStore'
import MessageStatus from '../components/MessageStatus'
import SecureRouteIndicator from '../components/SecureRouteIndicator'
import {
  ShieldCheck, Search, Plus, MessageSquare, Users, Settings as SettingsIcon,
  Phone, ICON_SIZE, ICON_STROKE,
} from '../ui/icons'
import ActionSheet from './ActionSheet'
import AppLock from './AppLock'
import './ChatList.css'
import './Search.css'

export default function ChatList() {
  const navigate = useNavigate()
  const { t } = useTranslation()
  const [searchQuery, setSearchQuery] = useState('')
  const [showActionSheet, setShowActionSheet] = useState(false)

  const { vault } = useSecurityStore()
  const { conversations, loadConversations } = useConversationStore()
  const { loadContacts, verifiedContacts, unverifiedContacts } = useContactStore()

  useEffect(() => { 
    loadConversations()
    loadContacts()
  }, [])

  const filtered = searchQuery
    ? conversations.filter(c => c.name.toLowerCase().includes(searchQuery.toLowerCase()))
    : conversations

  const matchedContacts = searchQuery
    ? [...verifiedContacts(), ...unverifiedContacts()].filter(c => 
        c.name.toLowerCase().includes(searchQuery.toLowerCase()) &&
        !conversations.some(conv => conv.name === c.name) // Hide if already in conversations
      )
    : []

  // In Duress Mode, the vault appears completely empty
  const displayConversations = vault.isDuressMode ? [] : filtered
  const displayContacts = vault.isDuressMode ? [] : matchedContacts

  return (
    <div className="chatlist-screen">
      {/* Premium Lock Screen — replaces the old inline overlay */}
      {vault.isLocked && (
        <AppLock onUnlock={() => {/* vault.isLocked toggles via securityStore */}} />
      )}

      {/* Secure Route establishment HUD */}
      <SecureRouteIndicator />

      {/* Header */}
      <header className="chatlist-header">
        <div className="header-top">
          <span className="header-time">9:41</span>
          <div className="header-icons">
            <span className="signal-icon">●●●●</span>
          </div>
        </div>
        <div className="header-title-row">
          <h1 className="header-title">{t('chatList.appTitle')}</h1>
          <button className="header-badge" aria-label={t('security.securityBadge')}>
            <ShieldCheck size={ICON_SIZE.sm} color="var(--accent-secure)" />
          </button>
        </div>
      </header>

      {/* Search */}
      <div className="search-container">
        <div className="search-bar">
          <Search size={ICON_SIZE.sm} strokeWidth={ICON_STROKE.default} color="var(--text-muted)" />
          <input
            type="text"
            placeholder={t('chatList.searchPlaceholder')}
            className="search-input"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>
      </div>

      {/* Conversation List */}
      <div className="conversation-list">
        {searchQuery && displayConversations.length > 0 && (
          <div className="search-section-title">{t('chatList.sectionConversations', 'Conversations')}</div>
        )}
        
        {displayConversations.map((chat) => (
          <button 
            key={chat.id} 
            className={`conversation-card ${chat.unread > 0 ? 'has-unread' : ''}`} 
            onClick={() => navigate(buildRoute(ROUTES.CONVERSATION, { conversationId: chat.id }))}
            aria-label={`Open conversation with ${chat.name}`}
          >
            <div className="avatar" style={{ background: chat.color }}>
              {chat.initials}
            </div>
            <div className="conv-content">
              <div className="conv-top-row">
                <span className="conv-name">{chat.name}</span>
                <span className="conv-time">{chat.time}</span>
              </div>
              <div className="conv-bottom-row">
                {chat.lastMessageIsFromMe && (
                  <MessageStatus status={chat.lastMessageStatus} size={13} />
                )}
                <span className="conv-message">{chat.lastMessage}</span>
                {chat.unread > 0 && (
                  <span className="unread-badge">{chat.unread}</span>
                )}
              </div>
              <div className="conv-badges">
                <span className="e2ee-badge">
                  <ShieldCheck size={10} color="var(--accent-secure)" />
                  {t('security.encryptedTag')}
                </span>
              </div>
            </div>
          </button>
        ))}

        {/* Matched Contacts */}
        {searchQuery && displayContacts.length > 0 && (
          <>
            <div className="search-section-title">{t('chatList.sectionContacts', 'Contacts')}</div>
            {displayContacts.map((contact) => (
              <button 
                key={contact.id} 
                className="conversation-card" 
                onClick={() => navigate(buildRoute(ROUTES.CONVERSATION, { conversationId: `new_${contact.id}` }))}
              >
                <div className="avatar" style={{ background: 'var(--bg-surface-elevated)' }}>
                  {contact.initial}
                </div>
                <div className="conv-content">
                  <div className="conv-top-row">
                    <span className="conv-name">{contact.name}</span>
                  </div>
                  <div className="conv-bottom-row">
                    <span className="conv-message" style={{ color: 'var(--accent-secure)' }}>
                      Start secure conversation
                    </span>
                  </div>
                </div>
              </button>
            ))}
          </>
        )}

        {/* Global Message Search Simulation (FTS5) */}
        {searchQuery.length > 2 && !vault.isDuressMode && (
          <>
            <div className="search-section-title">{t('chatList.sectionMessages', 'Messages')}</div>
            <button 
              className="conversation-card" 
              onClick={() => navigate(buildRoute(ROUTES.CONVERSATION, { conversationId: 'conv1' }))}
            >
              <div className="avatar" style={{ background: 'var(--bg-surface-elevated)' }}>
                <Search size={ICON_SIZE.sm} color="var(--text-secondary)" />
              </div>
              <div className="conv-content">
                <div className="conv-top-row">
                  <span className="conv-name">Elias Thorne</span>
                  <span className="conv-time">Mar 27</span>
                </div>
                <div className="conv-bottom-row">
                  <span className="conv-message">
                    ...simulated FTS5 match for <span className="highlight">"{searchQuery}"</span>...
                  </span>
                </div>
              </div>
            </button>
          </>
        )}

        {displayConversations.length === 0 && displayContacts.length === 0 && !vault.isDuressMode && searchQuery.length <= 2 && (
          <div className="empty-state">
            <p>No conversations found.</p>
          </div>
        )}
      </div>

      {/* FAB */}
      <button className="fab" onClick={() => setShowActionSheet(true)} aria-label={t('actionSheet.createNew')}>
        <Plus size={ICON_SIZE.lg} color="var(--text-inverse)" strokeWidth={ICON_STROKE.bold} />
      </button>

      {/* Action Sheet */}
      <ActionSheet isOpen={showActionSheet} onClose={() => setShowActionSheet(false)} />

      {/* Bottom Nav */}
      <nav className="bottom-nav" aria-label={t('common.mainNav')}>
        <button className="nav-item active" aria-label={t('chatList.navChats')}>
          <MessageSquare size={22} />
          <span>{t('chatList.navChats')}</span>
        </button>
        <button className="nav-item" aria-label={t('chatList.navCalls')} onClick={() => navigate(ROUTES.CALL_HISTORY)}>
          <Phone size={22} />
          <span>{t('chatList.navCalls')}</span>
        </button>
        <button className="nav-item" aria-label={t('chatList.navContacts')} onClick={() => navigate(ROUTES.CONTACTS)}>
          <Users size={22} />
          <span>{t('chatList.navContacts')}</span>
        </button>
        <button className="nav-item" aria-label={t('chatList.navSettings')} onClick={() => navigate(ROUTES.SETTINGS)}>
          <SettingsIcon size={22} />
          <span>{t('chatList.navSettings')}</span>
        </button>
      </nav>
    </div>
  )
}

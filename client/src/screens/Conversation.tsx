import { useEffect, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { ROUTES } from '../config/routes'
import { useConversationStore } from '../store/conversationStore'
import MessageBubble from '../components/MessageBubble'
import ReplyPreview from '../components/ReplyPreview'
import VoiceRecorder from '../components/VoiceRecorder'
import InChatSendSheet from '../components/InChatSendSheet'
import CryptoTransferBubble from '../components/CryptoTransferBubble'
import { CryptoTransferMessage } from '../types/wallet'
import {
  ArrowLeft, ShieldCheck, FileArchive, Paperclip, Smile, Send,
  Phone, Video, Timer, Mic,
  ICON_SIZE, ICON_STROKE,
} from '../ui/icons'
import { Coins } from 'lucide-react'
import './Conversation.css'

export default function Conversation() {
  const navigate = useNavigate()
  const { t } = useTranslation()
  const { conversationId } = useParams<{ conversationId: string }>()
  const { getMessages, loadMessages, setActive, conversations, replyingTo, setReplyingTo, setConversationTtl } = useConversationStore()

  const convo = conversations.find(c => c.id === conversationId)
  const messages = getMessages(conversationId ?? '')

  const [inputText, setInputText] = useState('')
  const [isRecording, setIsRecording] = useState(false)
  const [showCryptoSheet, setShowCryptoSheet] = useState(false)

  useEffect(() => {
    if (conversationId) {
      setActive(conversationId)
      loadMessages(conversationId)
    }
    return () => setActive(null)
  }, [conversationId])

  const displayName = convo?.name ?? 'Elias Thorne'

  const handleToggleTtl = () => {
    if (!convo) return
    let nextTtl: number | undefined
    if (!convo.ttlConfig) nextTtl = 3600 // 1 hour
    else if (convo.ttlConfig === 3600) nextTtl = 86400 // 1 day
    else if (convo.ttlConfig === 86400) nextTtl = 604800 // 1 week
    else nextTtl = undefined // Off
    setConversationTtl(convo.id, nextTtl)
  }

  // Check if a message is a crypto transfer
  const isCryptoTransfer = (content: string): CryptoTransferMessage | null => {
    try {
      const parsed = JSON.parse(content)
      if (parsed.type === 'crypto_transfer') return parsed as CryptoTransferMessage
    } catch { /* not JSON */ }
    return null
  }

  return (
    <div className="conversation-screen">
      {/* Header */}
      <header className="conv-header">
        <button className="back-btn" onClick={() => navigate(ROUTES.CHAT_LIST)} aria-label={t('common.back')}>
          <ArrowLeft size={ICON_SIZE.md} color="var(--text-primary)" />
        </button>
        <div className="conv-header-info">
          <h1 className="conv-header-name">{displayName}</h1>
        </div>
        <div className="conv-header-actions">
          <button className="conv-call-btn" onClick={() => navigate(ROUTES.OUTGOING_CALL)} aria-label={t('calls.voiceCall')}>
            <Phone size={ICON_SIZE.sm} color="var(--text-secondary)" />
          </button>
          <button className="conv-call-btn" onClick={() => navigate(ROUTES.VIDEO_CALL)} aria-label={t('calls.videoCall')}>
            <Video size={ICON_SIZE.sm} color="var(--text-secondary)" />
          </button>
          <button 
            className={`conv-call-btn ${convo?.ttlConfig ? 'active-ttl' : ''}`} 
            onClick={handleToggleTtl} 
            aria-label="Disappearing Messages"
          >
            <Timer size={ICON_SIZE.sm} color={convo?.ttlConfig ? 'var(--accent-secure)' : 'var(--text-secondary)'} />
          </button>
        </div>
        <button className="security-badge-btn" onClick={() => navigate(ROUTES.SECURITY)} aria-label={t('security.securityBadge')}>
          <ShieldCheck size={12} color="var(--accent-secure)" />
          {t('security.securityBadge')}
        </button>
      </header>

      {/* Messages */}
      <div className="messages-area">
        <div className="date-divider">
          <span>{t('conversation.today')}, 14:23</span>
        </div>

        {messages.map((msg) => {
          const transfer = isCryptoTransfer(msg.text || '')
          if (transfer) {
            return (
              <CryptoTransferBubble
                key={msg.id}
                transfer={transfer}
                isMine={msg.sent}
              />
            )
          }
          return <MessageBubble key={msg.id} message={msg} />
        })}
      </div>

      {/* Input Area */}
      <div className="input-area-wrapper">
        {replyingTo && (
          <ReplyPreview 
            senderName={replyingTo.sent ? 'You' : convo?.name ?? 'Them'}
            text={replyingTo.text || 'Attachment'}
            onClear={() => setReplyingTo(null)}
          />
        )}
        
        {/* Input Bar */}
        {isRecording ? (
          <VoiceRecorder 
            onSend={(blob) => {
              setIsRecording(false)
            }} 
            onCancel={() => setIsRecording(false)} 
          />
        ) : (
          <div className="input-bar">
            <button className="input-action" aria-label={t('common.search')}>
              <Paperclip size={ICON_SIZE.md} color="var(--text-secondary)" />
            </button>
            <div className="input-field">
              <input 
                type="text" 
                placeholder={t('conversation.inputPlaceholder')} 
                value={inputText}
                onChange={(e) => setInputText(e.target.value)}
              />
            </div>
            <button className="input-action" aria-label={t('common.emoji')}>
              <Smile size={ICON_SIZE.md} color="var(--text-secondary)" />
            </button>
            {/* Crypto Send Button */}
            <button
              className="input-action crypto-send-action"
              onClick={() => setShowCryptoSheet(true)}
              title="Send crypto"
              aria-label="Send crypto"
            >
              <Coins size={ICON_SIZE.md} color="var(--accent-secure)" />
            </button>
            <button 
              className={`send-btn ${!inputText.trim() ? 'mic-mode' : ''}`} 
              aria-label={inputText.trim() ? t('common.sendMessage') : "Record Voice Message"}
              onClick={() => {
                if (!inputText.trim()) {
                  setIsRecording(true)
                } else {
                  setInputText('')
                }
              }}
            >
              {inputText.trim() ? (
                <Send size={18} color="var(--text-inverse)" />
              ) : (
                <Mic size={18} color="var(--text-inverse)" />
              )}
            </button>
          </div>
        )}
      </div>

      {/* Crypto Send Sheet */}
      <InChatSendSheet
        isOpen={showCryptoSheet}
        onClose={() => setShowCryptoSheet(false)}
        conversationId={conversationId ?? ''}
        contactName={displayName}
      />
    </div>
  )
}

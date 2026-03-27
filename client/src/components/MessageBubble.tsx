import { useState } from 'react'
import { motion, useMotionValue, useTransform } from 'framer-motion'
import { useTranslation } from 'react-i18next'
import { useConversationStore, Message } from '../store/conversationStore'
import { ShieldCheck, FileArchive, Reply, ICON_SIZE, ICON_STROKE } from '../ui/icons'
import MessageStatus from './MessageStatus'
import ReactionPicker from './ReactionPicker'
import BurnTimer from './BurnTimer'
import { HapticManager } from '../utils/haptics'

interface MessageBubbleProps {
  message: Message
}

export default function MessageBubble({ message }: MessageBubbleProps) {
  const { t } = useTranslation()
  const { setReplyingTo, addReaction, getMessages } = useConversationStore()
  
  const [reactionOpen, setReactionOpen] = useState(false)
  const [reactionPos, setReactionPos] = useState({ x: 0, y: 0 })

  const x = useMotionValue(0)
  const opacity = useTransform(x, [0, 60], [0, 1])
  const scale = useTransform(x, [0, 60], [0.5, 1])

  const handleDragEnd = (event: any, info: any) => {
    if (info.offset.x > 70) {
      setReplyingTo(message)
      HapticManager.selection()
    }
  }

  const handleLongPress = (event: React.MouseEvent | React.TouchEvent | PointerEvent) => {
    HapticManager.selection()
    let clientX = window.innerWidth / 2
    let clientY = window.innerHeight / 2

    // Approximate position for the reaction picker based on the event
    if ('touches' in event && event.touches.length > 0) {
      clientX = event.touches[0].clientX
      clientY = event.touches[0].clientY
    } else if ('clientX' in event) {
      clientX = (event as React.MouseEvent).clientX
      clientY = (event as React.MouseEvent).clientY
    }

    setReactionPos({ x: clientX, y: clientY })
    setReactionOpen(true)
  }

  // Find the original message if this is a reply
  const originalMessage = message.replyTo 
    ? getMessages(message.conversationId).find(m => m.id === message.replyTo)
    : null

  return (
    <div className={`message-row-container ${message.sent ? 'sent' : 'received'}`}>
      {/* Swipe to reply indicator */}
      {!message.sent && (
        <motion.div style={{ opacity, scale }} className="swipe-reply-indicator">
          <Reply size={18} color="var(--text-muted)" />
        </motion.div>
      )}

      <motion.div
        drag={message.sent ? false : "x"}
        dragConstraints={{ left: 0, right: 100 }}
        dragElastic={0.2}
        onDragEnd={message.sent ? undefined : handleDragEnd}
        onContextMenu={(e) => {
          e.preventDefault()
          handleLongPress(e.nativeEvent as any)
        }}
        className={`message-row ${message.sent ? 'sent' : 'received'}`}
        // Using onPointerDown/Up with a timer could also work for long-press, 
        // but pointer down/up logic is manually implemented here or via onContextMenu
      >
        <div className={`message-bubble ${message.sent ? 'sent' : 'received'}`}>
          
          {/* Reply Context Snippet */}
          {originalMessage && (
            <div className="reply-context-snippet">
              <span className="reply-sender">{originalMessage.sent ? 'You' : 'Them'}</span>
              <p className="reply-text">{originalMessage.text || 'Attachment...'}</p>
            </div>
          )}

          {message.attachment ? (
            <div className="attachment-card">
              <div className="attachment-preview">
                <div className="matrix-overlay"></div>
                <div className="file-icon">
                  <FileArchive size={ICON_SIZE.lg} strokeWidth={ICON_STROKE.thin} color="var(--accent-secure)" />
                </div>
              </div>
              <div className="attachment-info">
                <span className="attachment-name">{message.attachment.name}</span>
                <span className="attachment-size">{message.attachment.size}</span>
              </div>
              <div className="attachment-badges">
                <span className="encrypted-tag">
                  <ShieldCheck size={8} color="var(--accent-secure)" />
                  {t('security.encryptedTag')}
                </span>
              </div>
            </div>
          ) : (
            <p className="message-content message-text">{message.text}</p>
          )}

          {/* Footer: timestamp + delivery status + burn timer */}
          <span className="message-footer">
            <span className="message-time">{message.time}</span>
            {message.sent && (
              <MessageStatus status={message.status} size={14} />
            )}
            {message.expiresAt && message.ttl && (
              <BurnTimer expiresAt={message.expiresAt} ttl={message.ttl} size={12} />
            )}
          </span>

          {/* Reactions */}
          {message.reactions && message.reactions.length > 0 && (
            <div className={`reactions-row ${message.sent ? 'sent' : 'received'}`}>
              {message.reactions.map((r, i) => (
                <span key={`${r.emoji}-${i}`} className="reaction-pill">{r.emoji}</span>
              ))}
            </div>
          )}
        </div>
      </motion.div>

      <ReactionPicker 
        isOpen={reactionOpen} 
        position={reactionPos} 
        onReact={(emoji) => addReaction(message.id, emoji)} 
        onClose={() => setReactionOpen(false)} 
      />
    </div>
  )
}

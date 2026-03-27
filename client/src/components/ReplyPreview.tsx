import { motion } from 'framer-motion'
import { X, Reply, ICON_SIZE } from '../ui/icons'
import './ReplyPreview.css'

interface ReplyPreviewProps {
  senderName: string
  text: string
  onClear: () => void
}

/**
 * Compact preview bar pinned above the input bar when replying to a message.
 */
export default function ReplyPreview({ senderName, text, onClear }: ReplyPreviewProps) {
  return (
    <motion.div
      className="reply-preview"
      initial={{ height: 0, opacity: 0 }}
      animate={{ height: 'auto', opacity: 1 }}
      exit={{ height: 0, opacity: 0 }}
      transition={{ type: 'spring', damping: 22, stiffness: 300 }}
    >
      <div className="reply-accent-bar" />
      <Reply size={14} color="var(--accent-secure)" className="reply-icon" />
      <div className="reply-content">
        <span className="reply-sender">{senderName}</span>
        <span className="reply-text">{text}</span>
      </div>
      <button className="reply-clear" onClick={onClear} aria-label="Cancel reply">
        <X size={16} color="var(--text-muted)" />
      </button>
    </motion.div>
  )
}

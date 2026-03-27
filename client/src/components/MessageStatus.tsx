import { motion, AnimatePresence } from 'framer-motion'
import { DeliveryStatus } from '../types/message'
import { Clock, Check, CheckCheck, CircleAlert, ICON_SIZE } from '../ui/icons'
import './MessageStatus.css'

interface MessageStatusProps {
  status: DeliveryStatus
  /** Icon size in px. */
  size?: number
  /** Show group read count (e.g. "3/5"). */
  groupCount?: { read: number; total: number }
  className?: string
}

/**
 * Animated delivery status indicator — used in both message bubbles
 * and the chat list preview row.
 */
export default function MessageStatus({
  status,
  size = 14,
  groupCount,
  className,
}: MessageStatusProps) {
  const icon = () => {
    switch (status) {
      case DeliveryStatus.Pending:
      case DeliveryStatus.Sending:
        return <Clock size={size} className="status-icon status-pending" />
      case DeliveryStatus.Sent:
        return <Check size={size} className="status-icon status-sent" />
      case DeliveryStatus.Delivered:
        return <CheckCheck size={size} className="status-icon status-delivered" />
      case DeliveryStatus.Read:
        return <CheckCheck size={size} className="status-icon status-read" />
      case DeliveryStatus.Failed:
        return <CircleAlert size={size} className="status-icon status-failed" />
      default:
        return null
    }
  }

  return (
    <span className={`message-status-wrap ${className || ''}`}>
      <AnimatePresence mode="wait">
        <motion.span
          key={status}
          className="message-status-anim"
          initial={{ scale: 0.6, opacity: 0, x: 6 }}
          animate={{ scale: 1, opacity: 1, x: 0 }}
          exit={{ scale: 0.6, opacity: 0 }}
          transition={{ type: 'spring', damping: 18, stiffness: 400 }}
        >
          {icon()}
        </motion.span>
      </AnimatePresence>

      {/* Group read count badge */}
      {groupCount && status === DeliveryStatus.Read && (
        <motion.span
          className="group-read-count"
          initial={{ scale: 0 }}
          animate={{ scale: 1 }}
          transition={{ delay: 0.1, type: 'spring', stiffness: 500 }}
        >
          {groupCount.read}/{groupCount.total}
        </motion.span>
      )}
    </span>
  )
}

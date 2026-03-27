import { motion } from 'framer-motion'
import './TypingIndicator.css'

/**
 * Three-dot typing indicator with staggered bounce animation.
 * Renders inline in the chat list preview or the conversation view.
 */
export default function TypingIndicator() {
  return (
    <motion.span
      className="typing-indicator"
      initial={{ opacity: 0, y: 4 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -4 }}
    >
      <span className="typing-label">typing</span>
      <span className="typing-dots" aria-hidden="true">
        {[0, 1, 2].map(i => (
          <motion.span
            key={i}
            className="typing-dot"
            animate={{ y: [0, -4, 0], opacity: [0.4, 1, 0.4] }}
            transition={{ duration: 0.9, repeat: Infinity, delay: i * 0.18 }}
          />
        ))}
      </span>
    </motion.span>
  )
}

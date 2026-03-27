import { useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { HapticManager } from '../utils/haptics'
import './ReactionPicker.css'

const REACTIONS = ['👍', '❤️', '😂', '😮', '😢', '🔥']

interface ReactionPickerProps {
  isOpen: boolean
  position: { x: number; y: number }
  onReact: (emoji: string) => void
  onClose: () => void
}

/**
 * Floating emoji pill — appears on long-press of a message bubble.
 */
export default function ReactionPicker({ isOpen, position, onReact, onClose }: ReactionPickerProps) {
  const handleSelect = (emoji: string) => {
    HapticManager.selection()
    onReact(emoji)
    onClose()
  }

  return (
    <AnimatePresence>
      {isOpen && (
        <>
          <div className="reaction-backdrop" onClick={onClose} />
          <motion.div
            className="reaction-pill"
            style={{ left: position.x, top: position.y }}
            initial={{ scale: 0.5, opacity: 0, y: 10 }}
            animate={{ scale: 1, opacity: 1, y: 0 }}
            exit={{ scale: 0.5, opacity: 0, y: 10 }}
            transition={{ type: 'spring', damping: 20, stiffness: 400 }}
          >
            {REACTIONS.map((emoji, i) => (
              <motion.button
                key={emoji}
                className="reaction-btn"
                onClick={() => handleSelect(emoji)}
                initial={{ scale: 0 }}
                animate={{ scale: 1 }}
                transition={{ delay: i * 0.04, type: 'spring', stiffness: 500 }}
                whileHover={{ scale: 1.3 }}
                whileTap={{ scale: 0.85 }}
              >
                {emoji}
              </motion.button>
            ))}
          </motion.div>
        </>
      )}
    </AnimatePresence>
  )
}

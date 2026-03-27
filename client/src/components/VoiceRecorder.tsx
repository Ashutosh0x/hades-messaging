import { useState, useEffect, useRef } from 'react'
import { motion, useMotionValue, useTransform } from 'framer-motion'
import { Mic, Trash2, Send, ChevronLeft, ICON_SIZE } from '../ui/icons'
import { HapticManager } from '../utils/haptics'
import './VoiceRecorder.css'

interface VoiceRecorderProps {
  onSend: (audioBlob: Blob, duration: number) => void
  onCancel: () => void
}

export default function VoiceRecorder({ onSend, onCancel }: VoiceRecorderProps) {
  const [duration, setDuration] = useState(0)
  const [isLocked, setIsLocked] = useState(false) // Drag up to lock
  const x = useMotionValue(0)

  // As user drags left, opacity of "Slide to cancel" text fades, and trash icon turns red
  const textOpacity = useTransform(x, [-100, 0], [0, 1])
  const trashScale = useTransform(x, [-120, -100], [1.2, 1])
  
  useEffect(() => {
    HapticManager.selection()
    
    // Timer for display
    const timer = setInterval(() => {
      setDuration(prev => prev + 1)
    }, 1000)

    return () => clearInterval(timer)
  }, [])

  const handleDragEnd = (event: any, info: any) => {
    if (info.offset.x < -100) {
      HapticManager.impact('heavy')
      onCancel()
    } else {
      // Return to original position
    }
  }

  // Generate a fake waveform for visual flair
  const waveformBars = Array.from({ length: 24 }).map((_, i) => (
    <motion.div
      key={i}
      className="waveform-bar"
      initial={{ height: 4 }}
      animate={{ height: ['4px', `${Math.random() * 20 + 4}px`, '4px'] }}
      transition={{ 
        repeat: Infinity, 
        duration: 0.6 + Math.random() * 0.4, 
        ease: "easeInOut",
        delay: Math.random() * 0.5
      }}
    />
  ))

  const formatDuration = (sec: number) => {
    const m = Math.floor(sec / 60)
    const s = sec % 60
    return `${m}:${s.toString().padStart(2, '0')}`
  }

  return (
    <motion.div 
      className="voice-recorder-container"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: 20 }}
    >
      <div className="recorder-status">
        <div className="recording-dot" />
        <span className="duration">{formatDuration(duration)}</span>
      </div>

      <div className="waveform-container">
        {waveformBars}
      </div>

      <motion.div style={{ opacity: textOpacity }} className="slide-to-cancel">
        <ChevronLeft size={14} />
        <span>Slide to cancel</span>
      </motion.div>

      <motion.div 
        className="mic-btn recording"
        drag="x"
        dragConstraints={{ left: 0, right: 0 }}
        dragElastic={{ left: 1, right: 0 }}
        onDragEnd={handleDragEnd}
        style={{ x }}
        whileTap={{ scale: 1.1 }}
      >
         <Mic size={ICON_SIZE.md} color="var(--bg-base)" />
      </motion.div>
    </motion.div>
  )
}

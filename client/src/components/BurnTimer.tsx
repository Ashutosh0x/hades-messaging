import { useEffect, useState } from 'react'
import { motion } from 'framer-motion'
import './BurnTimer.css'

interface BurnTimerProps {
  expiresAt: number
  ttl: number // Original TTL in seconds
  size?: number
  onExpire?: () => void
}

export default function BurnTimer({ expiresAt, ttl, size = 14, onExpire }: BurnTimerProps) {
  const [remaining, setRemaining] = useState(Math.max(0, expiresAt - Date.now()))

  useEffect(() => {
    if (remaining <= 0) {
      if (onExpire) onExpire()
      return
    }

    const interval = setInterval(() => {
      const nowRemaining = Math.max(0, expiresAt - Date.now())
      setRemaining(nowRemaining)
      if (nowRemaining <= 0) {
        clearInterval(interval)
        if (onExpire) onExpire()
      }
    }, 1000)

    return () => clearInterval(interval)
  }, [expiresAt, onExpire, remaining])

  if (remaining <= 0) return null

  // Calculate generic fraction (0 to 1)
  const fraction = remaining / (ttl * 1000)

  return (
    <div className="burn-timer-wrapper" style={{ width: size, height: size }}>
      <svg 
        width={size} 
        height={size} 
        viewBox="0 0 24 24" 
        className="burn-timer-svg"
      >
        {/* Background track */}
        <circle 
          cx="12" 
          cy="12" 
          r="10" 
          stroke="var(--bg-surface-elevated)" 
          strokeWidth="3" 
          fill="none" 
        />
        {/* Animated Burn Path */}
        <motion.circle 
          cx="12" 
          cy="12" 
          r="10" 
          stroke="var(--accent-warning)" 
          strokeWidth="3" 
          fill="none" 
          strokeLinecap="round"
          initial={{ pathLength: fraction }}
          animate={{ pathLength: fraction }}
          transition={{ duration: 1, ease: 'linear' }}
          style={{
            rotate: -90,
            transformOrigin: '50% 50%'
          }}
        />
      </svg>
    </div>
  )
}

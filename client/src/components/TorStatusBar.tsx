import { motion } from 'framer-motion'
import { Shield, Wifi, WifiOff, ICON_SIZE } from '../ui/icons'
import { useNetworkStore, TorStatus } from '../store/networkStore'
import './TorStatusBar.css'

const STATUS_CONFIG: Record<TorStatus, { label: string; sublabel: string; icon: 'shield' | 'wifi' | 'wifioff'; color: string }> = {
  disconnected: { 
    label: 'No Secure Route', 
    sublabel: 'Messages queued locally',
    icon: 'wifioff',
    color: 'var(--danger)' 
  },
  building: { 
    label: 'Establishing Secure Route', 
    sublabel: 'Connecting to Tor network…',
    icon: 'shield',
    color: 'var(--accent-warning)' 
  },
  connected: { 
    label: 'Securely Routed via Tor', 
    sublabel: '',
    icon: 'wifi',
    color: 'var(--accent-secure)' 
  },
}

/**
 * Contextual status bar at the top of the Chat List.
 * Red = disconnected, amber = circuit building, green = routed (auto-hides).
 */
export default function TorStatusBar() {
  const { status, progress } = useNetworkStore()
  const cfg = STATUS_CONFIG[status]

  const icon = cfg.icon === 'wifioff' 
    ? <WifiOff size={13} color="var(--bg-base)" /> 
    : cfg.icon === 'wifi' 
      ? <Wifi size={13} color="var(--bg-base)" />
      : <Shield size={13} color="var(--bg-base)" />

  return (
    <motion.div
      className="tor-status-bar"
      style={{ background: cfg.color }}
      initial={{ height: 0, opacity: 0 }}
      animate={{ height: status === 'connected' ? 0 : 32, opacity: status === 'connected' ? 0 : 1 }}
      transition={{ type: 'spring', damping: 20, stiffness: 300 }}
    >
      {status === 'building' && (
        <motion.div
          className="tor-progress"
          style={{ width: `${progress ?? 0}%` }}
          transition={{ ease: 'linear' }}
        />
      )}
      <div className="tor-bar-inner">
        {icon}
        <span className="tor-label">
          {cfg.label}
          {status === 'building' && progress != null && ` — ${progress}%`}
        </span>
      </div>
    </motion.div>
  )
}

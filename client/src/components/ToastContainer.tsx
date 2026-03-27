import { AnimatePresence, motion } from 'framer-motion'
import { useToastStore, ToastType } from '../store/toastStore'
import { ShieldAlert, Info, Check, AlertTriangle, X, ICON_SIZE } from '../ui/icons'
import './Toast.css'

const getIcon = (type: ToastType) => {
  switch (type) {
    case 'security': return <ShieldAlert size={ICON_SIZE.sm} />
    case 'success': return <Check size={ICON_SIZE.sm} />
    case 'warning': return <AlertTriangle size={ICON_SIZE.sm} />
    case 'error': return <X size={ICON_SIZE.sm} />
    case 'info':
    default: return <Info size={ICON_SIZE.sm} />
  }
}

export default function ToastContainer() {
  const { toasts, removeToast } = useToastStore()

  return (
    <div className="toast-container">
      <AnimatePresence>
        {toasts.map((toast) => (
          <motion.div
            key={toast.id}
            initial={{ opacity: 0, y: -50, scale: 0.9 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, scale: 0.9, transition: { duration: 0.2 } }}
            className={`toast-notification toast-${toast.type}`}
          >
            <div className="toast-icon">
              {getIcon(toast.type)}
            </div>
            <span className="toast-message">{toast.message}</span>
            <button className="toast-close" onClick={() => removeToast(toast.id)}>
              <X size={14} />
            </button>
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  )
}

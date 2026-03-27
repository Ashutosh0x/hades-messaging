import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { ROUTES } from '../config/routes'
import {
  MessageSquare, Users, UserPlus, Upload, EyeOff, ShieldCheck, X,
  ICON_SIZE,
} from '../ui/icons'
import './ActionSheet.css'

interface ActionSheetProps {
  isOpen: boolean
  onClose: () => void
}

export default function ActionSheet({ isOpen, onClose }: ActionSheetProps) {
  const navigate = useNavigate()
  const { t } = useTranslation()
  const [selected, setSelected] = useState<string | null>(null)

  useEffect(() => {
    document.body.style.overflow = isOpen ? 'hidden' : ''
    return () => { document.body.style.overflow = '' }
  }, [isOpen])

  if (!isOpen) return null

  const ACTIONS = [
    {
      id: 'secure_chat',
      icon: MessageSquare,
      title: t('actionSheet.newSecureChat'),
      subtitle: t('actionSheet.newSecureChatDesc'),
      badge: t('actionSheet.badgeE2EE'),
      route: ROUTES.CONVERSATION,
    },
    {
      id: 'group_vault',
      icon: Users,
      title: t('actionSheet.newGroupVault'),
      subtitle: t('actionSheet.newGroupVaultDesc'),
      badge: t('actionSheet.badgeMLS'),
    },
    {
      id: 'add_contact',
      icon: UserPlus,
      title: t('actionSheet.addContact'),
      subtitle: t('actionSheet.addContactDesc'),
      route: ROUTES.ADD_CONTACT,
    },
    {
      id: 'import_contact',
      icon: Upload,
      title: t('actionSheet.importContact'),
      subtitle: t('actionSheet.importContactDesc'),
    },
    {
      id: 'anonymous_session',
      icon: EyeOff,
      title: t('actionSheet.anonymousSession'),
      subtitle: t('actionSheet.anonymousSessionDesc'),
      badge: t('actionSheet.badgeAdvanced'),
      badgeColor: 'warning',
    },
  ]

  const handleAction = (action: typeof ACTIONS[0]) => {
    if (navigator.vibrate) navigator.vibrate(10)
    setSelected(action.id)
    setTimeout(() => {
      if (action.route) navigate(action.route)
      onClose()
      setSelected(null)
    }, 150)
  }

  return (
    <div className="action-sheet-wrapper">
      <div className="action-sheet-backdrop" onClick={onClose} />
      <div className="action-sheet">
        <div className="sheet-handle" />

        <div className="sheet-header">
          <div>
            <h3 className="sheet-title">{t('actionSheet.createNew')}</h3>
            <p className="sheet-subtitle">{t('actionSheet.allSessionsEncrypted')}</p>
          </div>
          <button className="sheet-close-btn" onClick={onClose} aria-label={t('common.close')}>
            <X size={ICON_SIZE.md} />
          </button>
        </div>

        <div className="sheet-actions">
          {ACTIONS.map(action => (
            <button
              key={action.id}
              className={`sheet-action-item ${selected === action.id ? 'selected' : ''}`}
              onClick={() => handleAction(action)}
            >
              <div className={`sheet-action-icon ${action.badgeColor === 'warning' ? 'warning' : ''}`}>
                <action.icon size={22} />
              </div>
              <div className="sheet-action-content">
                <span className="sheet-action-title">
                  {action.title}
                  {action.badge && (
                    <span className={`sheet-action-badge ${action.badgeColor || ''}`}>
                      {action.badge}
                    </span>
                  )}
                </span>
                <span className="sheet-action-subtitle">{action.subtitle}</span>
              </div>
            </button>
          ))}
        </div>

        <div className="sheet-footer">
          <ShieldCheck size={14} color="var(--accent-secure)" />
          <p>{t('security.trustNote')}</p>
        </div>
      </div>
    </div>
  )
}

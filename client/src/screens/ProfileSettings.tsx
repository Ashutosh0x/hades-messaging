import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { ROUTES } from '../config/routes'
import { ArrowLeft, UserRound, Camera, ShieldCheck, Check, ICON_SIZE } from '../ui/icons'
import { useToastStore } from '../store/toastStore'
import './ProfileSettings.css'

export default function ProfileSettings() {
  const navigate = useNavigate()
  const { t } = useTranslation()
  const { addToast } = useToastStore()

  const [nickname, setNickname] = useState('Elias Thorne')
  const [avatarColor, setAvatarColor] = useState('linear-gradient(135deg, #10b981 0%, #047857 100%)')

  // Generate a new gradient
  const randomizeAvatar = () => {
    const hue1 = Math.floor(Math.random() * 360)
    const hue2 = (hue1 + 40) % 360
    setAvatarColor(`linear-gradient(135deg, hsl(${hue1}, 80%, 40%) 0%, hsl(${hue2}, 80%, 30%) 100%)`)
  }

  const handleSave = () => {
    // In a real app, this dispatches a PQXDH encrypted profile update packet
    addToast(t('profile.updateSuccess', 'Profile updated across network'), 'success')
    navigate(ROUTES.SETTINGS)
  }

  return (
    <div className="profile-screen">
      <header className="profile-header">
        <button className="back-btn" onClick={() => navigate(ROUTES.SETTINGS)} aria-label={t('common.back')}>
          <ArrowLeft size={ICON_SIZE.md} color="var(--text-primary)" />
        </button>
        <h1 className="profile-title">{t('profile.title', 'Profile & Identity')}</h1>
        <button className="save-btn" onClick={handleSave}>
          <Check size={ICON_SIZE.md} color="var(--accent-secure)" />
        </button>
      </header>

      <div className="profile-content">
        <div className="avatar-section">
          <div className="avatar-preview" style={{ background: avatarColor }}>
            <span className="avatar-initials">{nickname.substring(0, 2).toUpperCase() || <UserRound size={32} color="#fff" />}</span>
            <button className="avatar-edit-btn" aria-label="Edit Avatar" onClick={randomizeAvatar}>
              <Camera size={16} color="var(--text-primary)" />
            </button>
          </div>
          <p className="avatar-hint">{t('profile.avatarHint', 'Tap camera to generate new identity')}</p>
        </div>

        <section className="profile-form">
          <div className="form-group">
            <label className="form-label">{t('profile.nickname', 'Display Name')}</label>
            <input 
              type="text" 
              className="profile-input" 
              value={nickname} 
              onChange={(e) => setNickname(e.target.value)}
              placeholder="Enter your public nickname"
            />
          </div>
          
          <div className="profile-info-card">
            <ShieldCheck size={ICON_SIZE.sm} color="var(--accent-secure)" />
            <p className="profile-info-text">
              {t('profile.e2eeDisclosure', 'Your profile name and avatar are end-to-end encrypted. They are only visible to contacts you have approved.')}
            </p>
          </div>
        </section>
      </div>
    </div>
  )
}

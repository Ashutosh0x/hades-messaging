import { useEffect, useMemo } from 'react'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { useCallStore, type CallEntry } from '../store/callStore'
import { groupByDate, formatDateLabel, formatDuration, formatRelativeTime } from '../utils/time'
import { ROUTES, buildRoute } from '../config/routes'
import {
  ArrowLeft, PhoneIncoming, PhoneOutgoing, PhoneMissed, Phone, Video, Trash2,
  ICON_SIZE,
} from '../ui/icons'
import './CallHistory.css'

const TypeIcon = ({ type }: { type: CallEntry['type'] }) => {
  switch (type) {
    case 'incoming':  return <PhoneIncoming size={16} color="var(--accent-secure)" />
    case 'outgoing':  return <PhoneOutgoing size={16} color="var(--text-secondary)" />
    case 'missed':    return <PhoneMissed   size={16} color="var(--danger)" />
  }
}

function CallItem({ entry, onClick, t }: { entry: CallEntry; onClick: () => void; t: any }) {
  return (
    <button
      className={`call-entry ${entry.type === 'missed' ? 'missed' : ''}`}
      onClick={onClick}
    >
      <div className="call-entry-icon">
        <TypeIcon type={entry.type} />
      </div>
      <div className="call-entry-info">
        <span className="call-entry-name">{entry.name}</span>
        <span className="call-entry-meta">
          {entry.media === 'video' ? <Video size={12} /> : <Phone size={12} />}
          {t(`calls.${entry.type}`)}
          {entry.duration != null && ` · ${formatDuration(entry.duration)}`}
        </span>
      </div>
      <span className="call-entry-time">{formatRelativeTime(entry.timestamp)}</span>
    </button>
  )
}

function EmptyState({ t }: { t: any }) {
  return (
    <div className="call-empty-state">
      <Phone size={48} color="var(--text-muted)" />
      <h3>{t('calls.noCallsYet')}</h3>
      <p>{t('calls.startFirstCall')}</p>
    </div>
  )
}

export default function CallHistory() {
  const navigate = useNavigate()
  const { t } = useTranslation()
  const { history, isLoading, loadHistory, clearHistory } = useCallStore()

  useEffect(() => { loadHistory() }, [])

  const groups = useMemo(() => groupByDate(history), [history])

  const handleClick = (entry: CallEntry) => {
    const route = entry.media === 'video' ? ROUTES.VIDEO_CALL : ROUTES.VOICE_CALL
    navigate(buildRoute(route, { contactId: entry.contactId }))
  }

  if (isLoading && history.length === 0) {
    return (
      <div className="call-history-screen">
        <div className="call-history-header">
          <button className="back-btn" onClick={() => navigate(-1)} aria-label={t('common.back')}>
            <ArrowLeft size={ICON_SIZE.md} color="var(--text-primary)" />
          </button>
          <h1 className="call-history-title">{t('calls.historyTitle')}</h1>
        </div>
        <div className="call-empty-state"><p>{t('common.loading')}</p></div>
      </div>
    )
  }

  return (
    <div className="call-history-screen">
      <div className="call-history-header">
        <button className="back-btn" onClick={() => navigate(-1)} aria-label={t('common.back')}>
          <ArrowLeft size={ICON_SIZE.md} color="var(--text-primary)" />
        </button>
        <h1 className="call-history-title">{t('calls.historyTitle')}</h1>
      </div>

      <div className="call-history-list">
        {groups.length === 0 ? (
          <EmptyState t={t} />
        ) : (
          groups.map(group => (
            <div key={group.dateKey} className="call-group">
              <div className="call-date-label">{formatDateLabel(group.timestamp)}</div>
              {group.entries.map(entry => (
                <CallItem key={entry.id} entry={entry} onClick={() => handleClick(entry)} t={t} />
              ))}
            </div>
          ))
        )}
      </div>

      {history.length > 0 && (
        <div className="call-history-footer">
          <button className="clear-history-btn" onClick={clearHistory}>
            <Trash2 size={ICON_SIZE.sm} />
            {t('calls.clearHistory')}
          </button>
        </div>
      )}
    </div>
  )
}

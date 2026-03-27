import { format, isToday, isYesterday } from 'date-fns'
import i18n from '../config/i18n'

/**
 * Format a unix-ms timestamp into a human-readable relative time.
 * Today → "3:42 PM", Yesterday → "Yesterday 3:42 PM", older → "Mar 24, 3:42 PM"
 */
export function formatRelativeTime(timestamp: number): string {
  const date = new Date(timestamp)

  if (isToday(date)) {
    return format(date, 'p') // locale short time e.g. "3:42 PM"
  }

  if (isYesterday(date)) {
    return `${i18n.t('calls.yesterday')} ${format(date, 'p')}`
  }

  return format(date, 'MMM d, p') // "Mar 24, 3:42 PM"
}

/**
 * Format a duration in seconds → "4m 23s", "1h 12m", or "—" for 0/null.
 */
export function formatDuration(seconds: number | null): string {
  if (seconds == null || seconds <= 0) return '—'

  if (seconds < 60) {
    return `${seconds}s`
  }

  const m = Math.floor(seconds / 60)
  const s = seconds % 60

  if (m < 60) {
    return s > 0 ? `${m}m ${s.toString().padStart(2, '0')}s` : `${m}m`
  }

  const h = Math.floor(m / 60)
  const rm = m % 60
  return rm > 0 ? `${h}h ${rm}m` : `${h}h`
}

/**
 * Format a call-timer value (seconds elapsed) as "MM:SS".
 */
export function formatTimer(elapsed: number): string {
  const m = Math.floor(elapsed / 60).toString().padStart(2, '0')
  const s = (elapsed % 60).toString().padStart(2, '0')
  return `${m}:${s}`
}

/**
 * Return a translated date label for call history groups.
 */
export function formatDateLabel(timestamp: number): string {
  const date = new Date(timestamp)

  if (isToday(date)) return i18n.t('calls.today')
  if (isYesterday(date)) return i18n.t('calls.yesterday')

  return format(date, 'MMMM d, yyyy') // "March 25, 2026"
}

/**
 * Group an array of items with `timestamp` by date key.
 * Returns sorted groups (newest first), each group sorted newest first.
 */
export function groupByDate<T extends { timestamp: number }>(
  items: T[],
): { dateKey: string; timestamp: number; entries: T[] }[] {
  const map = new Map<string, { timestamp: number; entries: T[] }>()

  for (const item of items) {
    const key = format(new Date(item.timestamp), 'yyyy-MM-dd')
    if (!map.has(key)) {
      map.set(key, { timestamp: item.timestamp, entries: [] })
    }
    map.get(key)!.entries.push(item)
  }

  return Array.from(map.entries())
    .sort(([a], [b]) => b.localeCompare(a))
    .map(([dateKey, group]) => ({
      dateKey,
      timestamp: group.timestamp,
      entries: group.entries.sort((a, b) => b.timestamp - a.timestamp),
    }))
}

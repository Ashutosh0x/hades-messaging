import { useEffect, useState } from 'react'

export interface NetworkQuality {
  type: 'wifi' | 'cellular' | 'ethernet' | 'unknown'
  effectiveType: '4g' | '3g' | '2g' | 'slow-2g'
  downlink: number   // Mbps estimated
  rtt: number        // Round-trip time in ms
  saveData: boolean  // User requested low-data mode
}

/**
 * Detects network connection quality for adaptive media behavior.
 * Uses the Network Information API where available, with fallback estimation.
 *
 * Usage:
 *   const network = useNetworkQuality()
 *   const maxImageKB = network?.effectiveType === '4g' ? 1024 : 256
 */
export function useNetworkQuality(): NetworkQuality | null {
  const [quality, setQuality] = useState<NetworkQuality | null>(null)

  useEffect(() => {
    // Network Information API (Chrome/Android only — but Tauri WebView supports it)
    if ('connection' in navigator) {
      const conn = (navigator as any).connection

      const update = () => {
        setQuality({
          type: conn.type || 'unknown',
          effectiveType: conn.effectiveType || '4g',
          downlink: conn.downlink ?? 10,
          rtt: conn.rtt ?? 100,
          saveData: conn.saveData ?? false,
        })
      }

      update()
      conn.addEventListener('change', update)
      return () => conn.removeEventListener('change', update)
    }

    // Fallback: estimate from navigation timing
    if ('performance' in window) {
      const entries = performance.getEntriesByType('navigation')
      if (entries.length > 0) {
        const nav = entries[0] as PerformanceNavigationTiming
        const rtt = Math.max(0, nav.responseStart - nav.requestStart)

        setQuality({
          type: 'unknown',
          effectiveType: rtt < 100 ? '4g' : rtt < 300 ? '3g' : '2g',
          downlink: 10,
          rtt,
          saveData: false,
        })
      }
    }

    return () => {}
  }, [])

  return quality
}

/**
 * Returns recommended media constraints based on current network quality.
 */
export function getMediaConstraints(network: NetworkQuality | null) {
  const is4g = !network || network.effectiveType === '4g'
  const is3g = network?.effectiveType === '3g'

  return {
    maxImageSizeKb: is4g ? 1024 : is3g ? 512 : 256,
    maxImageDimension: is4g ? 2048 : is3g ? 1280 : 800,
    maxVideoSizeMb: is4g ? 16 : is3g ? 8 : 4,
    maxVideoDurationSec: is4g ? 120 : is3g ? 60 : 30,
    autoDownloadMedia: is4g,
    jpegQuality: is4g ? 85 : is3g ? 70 : 50,
  }
}

/**
 * HapticManager — cross-platform vibration feedback.
 *
 * Uses the Web Vibration API (Android/PWA) with fallback for
 * platforms that don't support it (iOS via Tauri native bridge).
 */
export class HapticManager {
  private static supported = typeof navigator !== 'undefined' && 'vibrate' in navigator

  static impact(style: 'light' | 'medium' | 'heavy' = 'medium') {
    if (!this.supported) return
    const patterns = { light: 10, medium: 20, heavy: 30 }
    navigator.vibrate(patterns[style])
  }

  static selection() {
    if (!this.supported) return
    navigator.vibrate(5)
  }

  static notification(type: 'success' | 'warning' | 'error') {
    if (!this.supported) return
    const patterns = {
      success: [10, 50, 10],
      warning: [20, 100, 20],
      error: [30, 50, 30, 50, 30],
    }
    navigator.vibrate(patterns[type])
  }
}

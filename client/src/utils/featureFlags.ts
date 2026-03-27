import ENV from '../config/env'

/**
 * Simple feature flag system.
 * Reads from env config. Can be extended with remote config later.
 */
const FeatureFlags = {
  isEnabled(feature: keyof typeof ENV.FEATURES): boolean {
    return ENV.FEATURES[feature] ?? false
  },

  /** Return `enabled` value if feature is on, `disabled` otherwise */
  when<T>(feature: keyof typeof ENV.FEATURES, enabled: T, disabled: T): T {
    return this.isEnabled(feature) ? enabled : disabled
  },

  /** Is mock data mode active? (dev only) */
  get useMockData(): boolean {
    return ENV.MOCK_DATA
  },
} as const

export default FeatureFlags

const ENV = {
  API_URL: import.meta.env.VITE_API_URL as string || 'http://localhost:8080',
  WS_URL: import.meta.env.VITE_WS_URL as string || 'ws://localhost:8080/v1/ws',
  TURN_URL: import.meta.env.VITE_TURN_URL as string || 'turn:localhost:3478',
  ENVIRONMENT: (import.meta.env.VITE_ENVIRONMENT || 'development') as 'development' | 'staging' | 'production',
  LOG_LEVEL: (import.meta.env.VITE_LOG_LEVEL || 'debug') as 'debug' | 'info' | 'warn' | 'error',
  MOCK_DATA: import.meta.env.VITE_MOCK_DATA === 'true',
  FEATURES: {
    CALLS: import.meta.env.VITE_FEATURE_CALLS !== 'false',
    GROUPS: import.meta.env.VITE_FEATURE_GROUPS !== 'false',
    ANONYMOUS: import.meta.env.VITE_FEATURE_ANONYMOUS === 'true',
  },
  LIMITS: {
    MAX_FILE_SIZE: parseInt(import.meta.env.VITE_MAX_FILE_SIZE || '104857600'),
    MAX_MESSAGE_LENGTH: parseInt(import.meta.env.VITE_MAX_MESSAGE_LENGTH || '5000'),
    MAX_GROUP_SIZE: parseInt(import.meta.env.VITE_MAX_GROUP_SIZE || '100'),
  },
} as const

export default ENV

export const ROUTES = {
  // Main
  CHAT_LIST: '/',
  CONVERSATION: '/conversation/:conversationId',
  ONBOARDING: '/onboarding',
  SETTINGS: '/settings',
  PROFILE: '/profile',
  SECURITY: '/security/:contactId',

  // Calls (parameterized — contact context)
  INCOMING_CALL: '/incoming-call/:contactId',
  OUTGOING_CALL: '/outgoing-call/:contactId',
  VOICE_CALL: '/voice-call/:contactId',
  VIDEO_CALL: '/video-call/:contactId',
  CALL_HISTORY: '/call-history',

  // Contacts
  CONTACTS: '/contacts',
  ADD_CONTACT: '/add-contact',
  RECOVERY_PHRASE: '/recovery-phrase',

  // Wallet
  WALLET: '/wallet',
  WALLET_SEND: '/wallet/send',
  WALLET_RECEIVE: '/wallet/receive',
  WALLET_HISTORY: '/wallet/history',
} as const

/** Build a route by replacing :param placeholders */
export function buildRoute(
  route: string,
  params: Record<string, string>,
): string {
  let result = route
  for (const [key, value] of Object.entries(params)) {
    result = result.replace(`:${key}`, encodeURIComponent(value))
  }
  return result
}

export type RoutePath = (typeof ROUTES)[keyof typeof ROUTES]

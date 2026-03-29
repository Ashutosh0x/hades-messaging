import { create } from 'zustand'

// ── Types ──
export interface Reaction {
  emoji: string
}

export interface Attachment {
  name: string
  size: string
}

export interface Message {
  id: string
  conversationId: string
  text: string
  sent: boolean
  time: string
  status: string
  reactions: Reaction[]
  replyTo?: string
  attachment?: Attachment
  expiresAt?: number
  ttl?: number
}

export interface Conversation {
  id: string
  name: string
  initials: string
  color: string
  lastMessage: string
  lastMessageStatus: 'sending' | 'sent' | 'delivered' | 'read' | 'failed'
  lastMessageIsFromMe: boolean
  time: string
  unread: number
  ttlConfig?: number
}

interface ReplyTarget {
  id: string
  text: string
  sent: boolean
}

interface ConversationState {
  conversations: Conversation[]
  messages: Record<string, Message[]>
  activeConversationId: string | null
  replyingTo: ReplyTarget | null
  loading: boolean
  error: string | null

  loadConversations: () => Promise<void>
  getMessages: (conversationId: string) => Message[]
  loadMessages: (conversationId: string) => Promise<void>
  fetchMessages: (conversationId: string) => Promise<void>
  setActive: (conversationId: string | null) => void
  setReplyingTo: (target: ReplyTarget | Message | null) => void
  setConversationTtl: (conversationId: string, ttl: number | undefined) => void
  addReaction: (messageId: string, emoji: string) => void
  sendMessage: (
    conversationId: string,
    content: string,
    burnAfter?: number,
    replyTo?: string
  ) => Promise<void>
  markRead: (messageId: string) => Promise<void>
  setupListeners: () => Promise<void>
}

// Helper: try to invoke Tauri command
async function tryInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    return await invoke<T>(cmd, args)
  } catch {
    return null
  }
}

// ── M1 FIX: Generate gradient colors deterministically from contact ID ──
const GRADIENT_PALETTE = [
  'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
  'linear-gradient(135deg, #f093fb 0%, #f5576c 100%)',
  'linear-gradient(135deg, #4facfe 0%, #00f2fe 100%)',
  'linear-gradient(135deg, #43e97b 0%, #38f9d7 100%)',
  'linear-gradient(135deg, #fa709a 0%, #fee140 100%)',
  'linear-gradient(135deg, #a18cd1 0%, #fbc2eb 100%)',
  'linear-gradient(135deg, #ffecd2 0%, #fcb69f 100%)',
  'linear-gradient(135deg, #ff9a9e 0%, #fecfef 100%)',
]

function hashToIndex(str: string, max: number): number {
  let hash = 0
  for (let i = 0; i < str.length; i++) {
    hash = ((hash << 5) - hash + str.charCodeAt(i)) | 0
  }
  return Math.abs(hash) % max
}

function formatTime(iso: string): string {
  try {
    const d = new Date(iso)
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  } catch {
    return ''
  }
}

function formatRelativeTime(iso: string): string {
  try {
    const d = new Date(iso)
    const now = Date.now()
    const diff = now - d.getTime()
    if (diff < 60_000) return 'now'
    if (diff < 3600_000) return `${Math.floor(diff / 60_000)}m`
    if (diff < 86400_000) return `${Math.floor(diff / 3600_000)}h`
    return 'Yesterday'
  } catch {
    return ''
  }
}

// ── Store ──
export const useConversationStore = create<ConversationState>((set, get) => ({
  conversations: [],     // M1 FIX: starts empty, no DEMO_CONVERSATIONS
  messages: {},          // M1 FIX: starts empty, no DEMO_MESSAGES
  activeConversationId: null,
  replyingTo: null,
  loading: false,
  error: null,

  loadConversations: async () => {
    set({ loading: true, error: null })

    // M1 FIX: Load contacts from Tauri backend — each contact is a conversation
    const contacts = await tryInvoke<Array<{
      id: string
      displayName: string
      identityKey: number[]
      safetyNumber: string | null
      verified: boolean
      createdAt: string
    }>>('get_contacts')

    if (!contacts) {
      // No Tauri backend available — show empty state (NOT mock data)
      set({ conversations: [], loading: false })
      return
    }

    const conversations: Conversation[] = []

    for (const contact of contacts) {
      const msgs = await tryInvoke<Array<{
        id: string
        conversationId: string
        senderId: string
        content: string
        timestamp: string
        status: string
        burnAfter: number | null
        replyTo: string | null
      }>>('get_messages', {
        conversationId: contact.id,
        limit: 1,
        offset: 0,
      })

      const lastMsg = msgs?.[0] ?? null
      const name = contact.displayName || 'Unknown'
      const initials = name.split(' ').map((w: string) => w[0]).join('').substring(0, 2).toUpperCase()

      conversations.push({
        id: contact.id,
        name,
        initials,
        color: GRADIENT_PALETTE[hashToIndex(contact.id, GRADIENT_PALETTE.length)],
        lastMessage: lastMsg?.content ?? '',
        lastMessageStatus: (lastMsg?.status as Conversation['lastMessageStatus']) ?? 'read',
        lastMessageIsFromMe: lastMsg?.senderId === 'self',
        time: lastMsg ? formatRelativeTime(lastMsg.timestamp) : '',
        unread: 0,
      })
    }

    // Sort by last message time (most recent first)
    conversations.sort((a, b) => {
      if (!a.time) return 1
      if (!b.time) return -1
      return 0  // preserve backend ordering
    })

    set({ conversations, loading: false })
  },

  getMessages: (conversationId: string) => {
    return get().messages[conversationId] ?? []
  },

  loadMessages: async (conversationId: string) => {
    set({ loading: true, error: null })
    try {
      const raw = await tryInvoke<any[]>('get_messages', {
        conversationId,
        limit: 50,
        offset: 0,
      })

      if (raw && raw.length > 0) {
        const msgs: Message[] = raw.map(m => ({
          id: m.id,
          conversationId,
          text: m.content || m.text || '',
          sent: m.senderId === 'self' || m.sent === true,
          time: m.time || formatTime(m.timestamp || ''),
          status: m.status || 'read',
          reactions: m.reactions ?? [],
          replyTo: m.replyTo,
        }))
        set((state) => ({
          messages: { ...state.messages, [conversationId]: msgs },
          loading: false,
        }))
      } else {
        // M1 FIX: Show empty list, NOT demo messages
        set((state) => ({
          messages: {
            ...state.messages,
            [conversationId]: [],
          },
          loading: false,
        }))
      }
    } catch {
      // M1 FIX: Show empty list on error, NOT demo messages
      set((state) => ({
        messages: {
          ...state.messages,
          [conversationId]: [],
        },
        loading: false,
      }))
    }
  },

  fetchMessages: async (conversationId: string) => {
    return get().loadMessages(conversationId)
  },

  setActive: (conversationId: string | null) => {
    set({ activeConversationId: conversationId })
  },

  setReplyingTo: (target: ReplyTarget | Message | null) => {
    if (!target) {
      set({ replyingTo: null })
      return
    }
    set({
      replyingTo: {
        id: target.id,
        text: target.text ?? '',
        sent: target.sent ?? false,
      }
    })
  },

  setConversationTtl: (conversationId: string, ttl: number | undefined) => {
    set((state) => ({
      conversations: state.conversations.map(c =>
        c.id === conversationId ? { ...c, ttlConfig: ttl } : c
      ),
    }))
  },

  addReaction: (messageId: string, emoji: string) => {
    set((state) => {
      const updated = { ...state.messages }
      for (const convId of Object.keys(updated)) {
        updated[convId] = updated[convId].map(m =>
          m.id === messageId
            ? { ...m, reactions: [...(m.reactions || []), { emoji }] }
            : m
        )
      }
      return { messages: updated }
    })
  },

  sendMessage: async (conversationId, content, burnAfter, replyTo) => {
    const newMsg: Message = {
      id: `local-${Date.now()}`,
      conversationId,
      text: content,
      sent: true,
      time: formatTime(new Date().toISOString()),
      status: 'sending',
      reactions: [],
      replyTo,
    }

    set((state) => ({
      messages: {
        ...state.messages,
        [conversationId]: [
          ...(state.messages[conversationId] || []),
          newMsg,
        ],
      },
    }))

    try {
      const result = await tryInvoke<any>('send_message', {
        contactId: conversationId,
        content,
        burnAfter: burnAfter ?? null,
        replyTo: replyTo ?? null,
      })

      if (result) {
        set((state) => ({
          messages: {
            ...state.messages,
            [conversationId]: (state.messages[conversationId] || []).map(m =>
              m.id === newMsg.id
                ? { ...m, id: result.id, status: result.status, time: formatTime(result.timestamp) }
                : m
            ),
          },
        }))
      } else {
        // No Tauri — mark as failed (not silently succeed)
        set((state) => ({
          messages: {
            ...state.messages,
            [conversationId]: (state.messages[conversationId] || []).map(m =>
              m.id === newMsg.id ? { ...m, status: 'failed' } : m
            ),
          },
        }))
      }
    } catch (err) {
      console.error('Send failed:', err)
      set((state) => ({
        messages: {
          ...state.messages,
          [conversationId]: (state.messages[conversationId] || []).map(m =>
            m.id === newMsg.id ? { ...m, status: 'failed' } : m
          ),
        },
        error: String(err),
      }))
    }
  },

  markRead: async (messageId: string) => {
    try {
      await tryInvoke('mark_message_read', { messageId })
    } catch (err) {
      console.error('Mark read failed:', err)
    }
  },

  setupListeners: async () => {
    try {
      const { listen } = await import('@tauri-apps/api/event')

      await listen<any>('new-message', (event) => {
        const raw = event.payload
        const msg: Message = {
          id: raw.id,
          conversationId: raw.conversationId,
          text: raw.content || raw.text || '',
          sent: raw.senderId === 'self',
          time: formatTime(raw.timestamp || ''),
          status: raw.status || 'read',
          reactions: [],
        }
        set((state) => ({
          messages: {
            ...state.messages,
            [msg.conversationId]: [
              ...(state.messages[msg.conversationId] || []),
              msg,
            ],
          },
        }))
      })

      await listen<{ id: string; status: string }>('message-status', (event) => {
        const { id, status } = event.payload
        set((state) => {
          const updated = { ...state.messages }
          for (const convId of Object.keys(updated)) {
            updated[convId] = updated[convId].map((m) =>
              m.id === id ? { ...m, status } : m
            )
          }
          return { messages: updated }
        })
      })

      // Listen for burn timer events — remove burned messages from UI
      await listen<{ count: number }>('messages-burned', () => {
        // Reload active conversation to reflect burned messages
        const activeId = get().activeConversationId
        if (activeId) {
          get().loadMessages(activeId)
        }
        // Reload conversation list to update previews
        get().loadConversations()
      })
    } catch {
      // Not in Tauri context
    }
  },
}))


import { create } from 'zustand'
import FeatureFlags from '../utils/featureFlags'
import { DeliveryStatus } from '../types/message'
import { invoke } from '@tauri-apps/api/core'

// ── Types ──
export interface Reaction {
  emoji: string
  senderId: string
}

export interface Message {
  id: string
  conversationId: string
  sent: boolean
  text: string
  time: string
  timestamp: number
  status: DeliveryStatus
  attachment?: {
    name: string
    size: string
    type: string
  }
  replyTo?: string
  reactions: Reaction[]
  expiresAt?: number // Timestamp when message should be deleted
  ttl?: number // Original TTL in seconds
}

export interface Conversation {
  id: string
  contactId: string
  name: string
  initials: string
  lastMessage: string
  lastMessageStatus: DeliveryStatus
  lastMessageIsFromMe: boolean
  time: string
  timestamp: number
  unread: number
  verified: boolean
  color: string
  ttlConfig?: number // Default TTL in seconds for new messages in this conv
}

interface ConversationStore {
  conversations: Conversation[]
  messages: Map<string, Message[]>  // keyed by conversationId
  activeId: string | null
  isLoading: boolean
  replyingTo: Message | null

  loadConversations: () => Promise<void>
  loadMessages: (conversationId: string) => Promise<void>
  setActive: (id: string | null) => void
  getActiveConversation: () => Conversation | undefined
  getMessages: (id: string) => Message[]
  setReplyingTo: (message: Message | null) => void
  addReaction: (messageId: string, emoji: string) => void
  setConversationTtl: (conversationId: string, ttl: number | undefined) => void
}

// ── Mock Data ──
const now = Date.now()
const HOUR = 3600_000

const MOCK_CONVOS: Conversation[] = [
  { id: 'conv1', contactId: 'c1', name: 'Elias Thorne',   initials: 'ET', lastMessage: 'Received. I\'ll run these through the local security audit...', lastMessageStatus: DeliveryStatus.Read, lastMessageIsFromMe: true, time: '14:26', timestamp: now - 2 * HOUR, unread: 0, verified: true, color: '#2ECC71', ttlConfig: 3600 },
  { id: 'conv2', contactId: 'c2', name: 'Julian Thorne',  initials: 'JT', lastMessage: 'Received. The vault security all...',     lastMessageStatus: DeliveryStatus.Delivered, lastMessageIsFromMe: false, time: 'Yesterday', timestamp: now - 26 * HOUR, unread: 2, verified: true, color: '#3498DB' },
  { id: 'conv3', contactId: 'c3', name: 'Alpha Team [7]', initials: 'AT', lastMessage: 'Mission parameters updated. Conf...',    lastMessageStatus: DeliveryStatus.Sent, lastMessageIsFromMe: true, time: 'Yesterday', timestamp: now - 30 * HOUR, unread: 0, verified: true, color: '#9B59B6', ttlConfig: 86400 },
  { id: 'conv4', contactId: 'c4', name: 'Elena Stone',    initials: 'ES', lastMessage: '"We\'re still waiting at the veri...',   lastMessageStatus: DeliveryStatus.Pending, lastMessageIsFromMe: true, time: '2d ago',    timestamp: now - 50 * HOUR, unread: 0, verified: true, color: '#E67E22' },
]

const MOCK_MESSAGES: Message[] = [
  { id: 'm1', conversationId: 'conv1', sent: false, text: 'The architectural blueprints for the Vault-X project are ready for your final review. I\'ve sent the encrypted files for your review.', time: '14:23', timestamp: now - 2 * HOUR, status: DeliveryStatus.Read, reactions: [] },
  { id: 'm2', conversationId: 'conv1', sent: false, text: '',     time: '14:24', timestamp: now - 2 * HOUR + 60000, status: DeliveryStatus.Read, attachment: { name: 'vault_x_blueprints_v4.pdf', size: '4.2 MB', type: 'file' }, reactions: [] },
  { id: 'm3', conversationId: 'conv1', sent: true,  text: 'Received. I\'ll run these through the local security audit before we proceed with the integration.', time: '14:26', timestamp: now - 2 * HOUR + 180000, status: DeliveryStatus.Read, replyTo: 'm1', reactions: [{ emoji: '👍', senderId: 'c1' }], expiresAt: now + 600000, ttl: 3600 },
]

// ── Store ──
export const useConversationStore = create<ConversationStore>()((set, get) => ({
  conversations: [],
  messages: new Map(),
  activeId: null,
  isLoading: false,
  replyingTo: null,

  loadConversations: async () => {
    set({ isLoading: true })
    try {
      if (FeatureFlags.useMockData) {
        await new Promise(r => setTimeout(r, 200))
        set({ conversations: MOCK_CONVOS, isLoading: false })
        return
      }
      // Production: const data = await invoke<Conversation[]>('get_active_sessions')
      set({ conversations: MOCK_CONVOS, isLoading: false })
    } catch {
      set({ isLoading: false })
    }
  },

  loadMessages: async (conversationId: string) => {
    try {
      let msgs: Message[]
      if (FeatureFlags.useMockData) {
        await new Promise(r => setTimeout(r, 100))
        msgs = MOCK_MESSAGES.filter(m => m.conversationId === conversationId)
      } else {
        try {
          msgs = await invoke('get_messages', { conversationId, limit: 50, offset: 0 })
        } catch (err) {
          console.error("Failed to fetch messages:", err)
          msgs = MOCK_MESSAGES.filter(m => m.conversationId === conversationId)
        }
      }
      set(state => {
        const map = new Map(state.messages)
        map.set(conversationId, msgs)
        return { messages: map }
      })
    } catch { /* skip */ }
  },

  setActive: (id) => set({ activeId: id }),

  getActiveConversation: () => {
    const { activeId, conversations } = get()
    return conversations.find(c => c.id === activeId)
  },

  getMessages: (id: string) => get().messages.get(id) ?? [],

  setReplyingTo: (message) => set({ replyingTo: message }),

  addReaction: (messageId, emoji) => set((state) => {
    const messages = new Map(state.messages)
    for (const [convId, msgs] of messages.entries()) {
      const msgIndex = msgs.findIndex(m => m.id === messageId)
      if (msgIndex !== -1) {
        const newMsgs = [...msgs]
        const reactions = [...newMsgs[msgIndex].reactions]
        const existing = reactions.findIndex(r => r.senderId === 'me' && r.emoji === emoji)
        if (existing !== -1) {
          reactions.splice(existing, 1)
        } else {
          reactions.push({ emoji, senderId: 'me' })
        }
        newMsgs[msgIndex] = { ...newMsgs[msgIndex], reactions }
          messages.set(convId, newMsgs)
          break
        }
      }
      return { messages }
    }),

  setConversationTtl: (conversationId: string, ttl: number | undefined) => set((state) => {
    const nextConvos = [...state.conversations]
    const idx = nextConvos.findIndex(c => c.id === conversationId)
    if (idx !== -1) {
      nextConvos[idx] = { ...nextConvos[idx], ttlConfig: ttl }
    }
    return { conversations: nextConvos }
  })
}))

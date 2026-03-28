import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// ── Types ──
export interface Message {
  id: string
  conversationId: string
  senderId: string
  content: string
  timestamp: string
  status: 'sending' | 'sent' | 'delivered' | 'read' | 'failed'
  burnAfter?: number
  replyTo?: string
}

interface ConversationState {
  messages: Record<string, Message[]>
  loading: boolean
  error: string | null

  fetchMessages: (conversationId: string) => Promise<void>
  sendMessage: (
    conversationId: string,
    content: string,
    burnAfter?: number,
    replyTo?: string
  ) => Promise<void>
  markRead: (messageId: string) => Promise<void>
  setupListeners: () => Promise<void>
}

// ── Store ──
export const useConversationStore = create<ConversationState>((set, get) => ({
  messages: {},
  loading: false,
  error: null,

  fetchMessages: async (conversationId: string) => {
    set({ loading: true, error: null })
    try {
      const msgs: Message[] = await invoke('get_messages', {
        conversationId,
        limit: 50,
        offset: 0,
      })
      set((state) => ({
        messages: { ...state.messages, [conversationId]: msgs },
        loading: false,
      }))
    } catch (err) {
      set({ error: String(err), loading: false })
    }
  },

  sendMessage: async (conversationId, content, burnAfter, replyTo) => {
    try {
      const msg: Message = await invoke('send_message', {
        contactId: conversationId,
        content,
        burnAfter: burnAfter ?? null,
        replyTo: replyTo ?? null,
      })

      set((state) => ({
        messages: {
          ...state.messages,
          [conversationId]: [
            ...(state.messages[conversationId] || []),
            msg,
          ],
        },
      }))
    } catch (err) {
      console.error('Send failed:', err)
      set({ error: String(err) })
    }
  },

  markRead: async (messageId: string) => {
    try {
      await invoke('mark_message_read', { messageId })
    } catch (err) {
      console.error('Mark read failed:', err)
    }
  },

  setupListeners: async () => {
    // Listen for incoming messages from the Rust backend
    await listen<Message>('new-message', (event) => {
      const msg = event.payload
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

    // Listen for status updates
    await listen<{ id: string; status: string }>('message-status', (event) => {
      const { id, status } = event.payload
      set((state) => {
        const updated = { ...state.messages }
        for (const convId of Object.keys(updated)) {
          updated[convId] = updated[convId].map((m) =>
            m.id === id ? { ...m, status: status as Message['status'] } : m
          )
        }
        return { messages: updated }
      })
    })
  },
}))

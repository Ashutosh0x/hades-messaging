/**
 * E2EE Message delivery status types for Hades Messaging.
 *
 * Status transitions:
 *   pending → sending → sent → delivered → read
 *                  ↘ failed
 */

export enum DeliveryStatus {
  /** Queued locally, not yet sent (Tor circuit building). */
  Pending   = 'pending',
  /** Currently being transmitted over Tor. */
  Sending   = 'sending',
  /** Relay ACK received — message reached the server. */
  Sent      = 'sent',
  /** Recipient device ACK — message downloaded by recipient. */
  Delivered = 'delivered',
  /** E2EE read receipt — recipient opened the conversation. */
  Read      = 'read',
  /** Send failed (network error, blocked circuit). */
  Failed    = 'failed',
}

/** A delivery/read receipt for a single message. */
export interface MessageReceipt {
  messageId: string
  status: DeliveryStatus
  timestamp: number
  /** Recipient who sent this receipt (for group chats). */
  recipientId?: string
  deviceId?: number
}

/** Aggregated group receipt for a single message. */
export interface GroupReceipt {
  messageId: string
  receipts: Map<string, MessageReceipt>
  readCount: number
  totalCount: number
}

/** Return true if `next` is a higher-priority status than `current`. */
export function isHigherPriority(next: DeliveryStatus, current: DeliveryStatus): boolean {
  const order: Record<DeliveryStatus, number> = {
    [DeliveryStatus.Failed]:    -1,
    [DeliveryStatus.Pending]:    0,
    [DeliveryStatus.Sending]:    1,
    [DeliveryStatus.Sent]:       2,
    [DeliveryStatus.Delivered]:  3,
    [DeliveryStatus.Read]:       4,
  }
  return order[next] > order[current]
}

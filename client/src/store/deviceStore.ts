import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

export type DeviceType = 'desktop' | 'phone' | 'tablet'
export type DeviceStatus = 'connected' | 'disconnected' | 'pending'

export interface LinkedDevice {
  id: string
  name: string
  type: DeviceType
  status: DeviceStatus
  lastSeen: number        // Unix timestamp
  publicKey: string       // Ed25519 device identity (truncated for display)
  isCurrentDevice: boolean
}

interface DeviceState {
  devices: LinkedDevice[]
  addDevice: (device: Omit<LinkedDevice, 'id' | 'lastSeen'>) => void
  removeDevice: (id: string) => void
  updateDeviceStatus: (id: string, status: DeviceStatus) => void
  revokeDevice: (id: string) => void
}

/**
 * Detects the current device type from User-Agent.
 * In production, this would come from the Tauri/native layer.
 */
function detectDeviceType(): DeviceType {
  const ua = navigator.userAgent.toLowerCase()
  if (/ipad|tablet|playbook|silk/.test(ua)) return 'tablet'
  if (/mobile|iphone|android/.test(ua)) return 'phone'
  return 'desktop'
}

function detectDeviceName(): string {
  const ua = navigator.userAgent
  if (/Mac/.test(ua)) return 'Mac'
  if (/Windows/.test(ua)) return 'Windows PC'
  if (/Linux/.test(ua)) return 'Linux'
  if (/iPhone/.test(ua)) return 'iPhone'
  if (/iPad/.test(ua)) return 'iPad'
  if (/Android/.test(ua)) return 'Android'
  return 'Unknown Device'
}

function generateDeviceFingerprint(): string {
  const arr = new Uint8Array(8)
  crypto.getRandomValues(arr)
  return Array.from(arr).map(b => b.toString(16).padStart(2, '0')).join('')
}

// Seed current device on first load
const currentDeviceId = `device_${generateDeviceFingerprint()}`

export const useDeviceStore = create<DeviceState>((set, get) => ({
  devices: [
    {
      id: currentDeviceId,
      name: detectDeviceName(),
      type: detectDeviceType(),
      status: 'connected' as DeviceStatus,
      lastSeen: Date.now(),
      publicKey: generateDeviceFingerprint(),
      isCurrentDevice: true,
    },
  ],

  addDevice: (device) => set((state) => ({
    devices: [
      ...state.devices,
      {
        ...device,
        id: `device_${generateDeviceFingerprint()}`,
        lastSeen: Date.now(),
      },
    ],
  })),

  removeDevice: (id) => set((state) => ({
    devices: state.devices.filter((d) => d.id !== id),
  })),

  updateDeviceStatus: (id, status) => set((state) => ({
    devices: state.devices.map((d) =>
      d.id === id ? { ...d, status, lastSeen: Date.now() } : d
    ),
  })),

  revokeDevice: (id) => {
    const device = get().devices.find((d) => d.id === id)
    if (device?.isCurrentDevice) return // Can't revoke current device

    invoke('hades_identity_revoke_device', { deviceId: id }).catch(console.error)
    set((state) => ({
      devices: state.devices.filter((d) => d.id !== id),
    }))
  },
}))

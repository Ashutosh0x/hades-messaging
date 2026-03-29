import { create } from 'zustand'

// M4 FIX: Safe invoke wrapper — no crash in browser dev mode
async function tryInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    return await invoke<T>(cmd, args)
  } catch {
    return null
  }
}

// Backend device shape (camelCase from #[serde(rename_all = "camelCase")])
export interface Device {
  deviceId: string
  deviceName: string
  lastSeen: string | null
  isCurrent: boolean
}

// Exported type for Settings.tsx backward compat
export interface LinkedDevice {
  id: string
  name: string
  type: 'desktop' | 'phone' | 'tablet'
  publicKey: string
  isCurrentDevice: boolean
  lastSeen: string | null
}

interface DeviceState {
  devices: LinkedDevice[]
  loading: boolean
  error: string | null

  loadDevices: () => Promise<void>
  revokeDevice: (deviceId: string) => Promise<void>
}

// Map backend Device to frontend LinkedDevice
function mapDevice(d: Device): LinkedDevice {
  // Heuristic to guess device type from name
  const nameLower = d.deviceName.toLowerCase()
  const type: LinkedDevice['type'] = nameLower.includes('phone') || nameLower.includes('mobile')
    ? 'phone'
    : nameLower.includes('tablet') || nameLower.includes('ipad')
      ? 'tablet'
      : 'desktop'

  return {
    id: d.deviceId,
    name: d.deviceName,
    type,
    publicKey: d.deviceId, // deviceId is the key
    isCurrentDevice: d.isCurrent,
    lastSeen: d.lastSeen,
  }
}

export const useDeviceStore = create<DeviceState>((set) => ({
  devices: [],
  loading: false,
  error: null,

  loadDevices: async () => {
    set({ loading: true, error: null })
    try {
      // M4 FIX: wrapped in tryInvoke, won't crash in browser
      const result = await tryInvoke<Device[]>('get_devices')
      set({
        devices: (result ?? []).map(mapDevice),
        loading: false,
      })
    } catch (err: any) {
      set({
        error: err?.message ?? 'Failed to load devices',
        loading: false,
      })
    }
  },

  revokeDevice: async (deviceId: string) => {
    try {
      // M4 FIX: correct command name is 'revoke_device'
      await tryInvoke('revoke_device', { deviceId })
      set((state) => ({
        devices: state.devices.filter((d) => d.id !== deviceId),
      }))
    } catch (err: any) {
      set({ error: err?.message ?? 'Failed to revoke device' })
    }
  },
}))

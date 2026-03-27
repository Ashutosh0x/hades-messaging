export type ConnectionStatusType = 'idle' | 'connecting' | 'establishing' | 'established' | 'error'

export interface ConnectionStage {
  progress: number
  stage: string
}

export const CONNECTION_STAGES: ConnectionStage[] = [
  { progress: 10,  stage: 'Initializing secure entropy' },
  { progress: 25,  stage: 'Connecting to guard node' },
  { progress: 40,  stage: 'Building first hop' },
  { progress: 55,  stage: 'Building second hop' },
  { progress: 70,  stage: 'Building end relay' },
  { progress: 80,  stage: 'Negotiating PQXDH key exchange' },
  { progress: 90,  stage: 'Verifying relay fingerprint' },
  { progress: 100, stage: 'Secure route established' },
]

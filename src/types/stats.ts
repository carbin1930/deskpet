export interface SystemStats {
  cpuUsage: number
  memUsed: number
  memTotal: number
  memUsage: number
  diskUsed: number
  diskTotal: number
  diskUsage: number
  netRx: number
  netTx: number
  sampledAt: number
  status: PetStatus
}

export type PetStatus = 'idle' | 'busy' | 'alert'

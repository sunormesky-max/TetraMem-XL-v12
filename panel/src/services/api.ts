const API_BASE = '/api'

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    headers: { 'Content-Type': 'application/json', ...options?.headers },
    ...options,
  })
  const text = await res.text()
  let data: any
  try {
    data = JSON.parse(text)
  } catch {
    throw new Error(`Server returned non-JSON: ${text.slice(0, 200)}`)
  }
  if (!data.success && data.error) {
    throw new Error(data.error)
  }
  return data as T
}

export interface StatsData {
  success: boolean
  data: {
    nodes: number
    manifested: number
    dark: number
    even: number
    odd: number
    total_energy: number
    allocated_energy: number
    available_energy: number
    physical_energy: number
    dark_energy: number
    utilization: number
    conservation_ok: boolean
    memory_count: number
    hebbian_edges: number
    hebbian_total_weight: number
  }
}

export interface HealthData {
  success: boolean
  data: {
    level: string
    conservation_ok: boolean
    energy_utilization: number
    node_count: number
    manifested_ratio: number
    hebbian_edge_count: number
    hebbian_avg_weight: number
    memory_count: number
    frontier_size: number
  }
}

export interface EncodeResult {
  success: boolean
  data: {
    anchor: string
    data_dim: number
    manifested: boolean
    created_at: number
  }
}

export interface DecodeResult {
  success: boolean
  data: {
    data: number[]
  }
}

export interface MemoryListResult {
  success: boolean
  data: string[]
}

export interface PulseResult {
  success: boolean
  data: {
    visited_nodes: number
    total_activation: number
    paths_recorded: number
    final_strength: number
  }
}

export interface DreamResult {
  success: boolean
  data: {
    paths_replayed: number
    paths_weakened: number
    memories_consolidated: number
    edges_before: number
    edges_after: number
    weight_before: number
    weight_after: number
  }
}

export interface ScaleResult {
  success: boolean
  data: {
    energy_expanded_by: number
    nodes_added: number
    nodes_removed: number
    reason: string
  }
}

export interface RegulateResult {
  success: boolean
  data: string[]
}

export interface HebbianNeighborsResult {
  success: boolean
  data: {
    node: string
    neighbors: { coord: string; weight: number }[]
  }
}

export interface BackupInfo {
  id: number
  timestamp_ms: number
  trigger: string
  node_count: number
  memory_count: number
  total_energy: number
  conservation_ok: boolean
  bytes: number
  generation: number
}

export interface CreateBackupResult {
  success: boolean
  data: {
    backup_id: number
    generation: number
    node_count: number
    memory_count: number
    bytes: number
    elapsed_ms: number
  }
}

export interface ListBackupsResult {
  success: boolean
  data: BackupInfo[]
}

export interface ClusterStatusResult {
  success: boolean
  data: {
    node_id: number
    role: string
    term: number
    leader_id: number | null
    log_index: number
    applied_count: number
    nodes: { id: number; addr: string; role: string }[]
  }
}

export interface ClusterProposeResult {
  success: boolean
  data: {
    log_index: number
  }
}

export interface ClusterNodeResult {
  success: boolean
  data: string
}

export interface TimelineDay {
  date: string
  count: number
  anchors: string[]
}

export interface TimelineResult {
  success: boolean
  data: TimelineDay[]
}

export interface TraceHop {
  anchor: string
  created_at: number
  data_dim: number
  confidence: number
  hop: number
}

export interface TraceResult {
  success: boolean
  data: TraceHop[]
}

export const api = {
  getStats: () => request<StatsData>('/stats'),
  getHealth: () => request<HealthData>('/health'),
  encodeMemory: (anchor: [number, number, number], data: number[]) =>
    request<EncodeResult>('/memory/encode', {
      method: 'POST',
      body: JSON.stringify({ anchor: Array.from(anchor), data }),
    }),
  decodeMemory: (anchor: [number, number, number], dataDim: number) =>
    request<DecodeResult>('/memory/decode', {
      method: 'POST',
      body: JSON.stringify({ anchor: Array.from(anchor), data_dim: dataDim }),
    }),
  listMemories: () => request<MemoryListResult>('/memory/list'),
  firePulse: (source: [number, number, number], pulseType: string) =>
    request<PulseResult>('/pulse', {
      method: 'POST',
      body: JSON.stringify({ source: Array.from(source), pulse_type: pulseType }),
    }),
  runDream: () => request<DreamResult>('/dream', { method: 'POST' }),
  autoScale: () => request<ScaleResult>('/scale', { method: 'POST' }),
  regulate: () => request<RegulateResult>('/regulate', { method: 'POST' }),
  getHebbianNeighbors: (x: number, y: number, z: number) =>
    request<HebbianNeighborsResult>(`/hebbian/neighbors/${x}/${y}/${z}`),
  frontierExpand: (maxNew: number) =>
    request<ScaleResult>(`/scale/frontier/${maxNew}`, { method: 'POST' }),
  createBackup: () => request<CreateBackupResult>('/backup/create', { method: 'POST' }),
  listBackups: () => request<ListBackupsResult>('/backup/list'),
  getClusterStatus: () => request<ClusterStatusResult>('/cluster/status'),
  initCluster: (nodeId?: number, addr?: string) =>
    request<ClusterStatusResult>('/cluster/init', {
      method: 'POST',
      body: JSON.stringify({ node_id: nodeId, addr }),
    }),
  clusterPropose: (key: string, value: string) =>
    request<ClusterProposeResult>('/cluster/propose', {
      method: 'POST',
      body: JSON.stringify({ key, value }),
    }),
  addClusterNode: (nodeId: number, addr: string) =>
    request<ClusterNodeResult>('/cluster/add-node', {
      method: 'POST',
      body: JSON.stringify({ node_id: nodeId, addr }),
    }),
  removeClusterNode: (nodeId: number) =>
    request<ClusterNodeResult>('/cluster/remove-node', {
      method: 'POST',
      body: JSON.stringify({ node_id: nodeId }),
    }),
  getTimeline: () => request<TimelineResult>('/memory/timeline'),
  traceMemory: (anchor: [number, number, number], maxHops?: number) =>
    request<TraceResult>('/memory/trace', {
      method: 'POST',
      body: JSON.stringify({ anchor: Array.from(anchor), max_hops: maxHops }),
    }),
}

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
    throw new Error(`Server returned non-JSON (${res.status}): ${text.slice(0, 200)}`)
  }
  if (!res.ok) {
    throw new Error(data?.error || `HTTP ${res.status}`)
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

export interface MemoryListItem {
  anchor: string
  data_dim: number
  created_at: number
  tags: string[]
  category: string | null
  description: string | null
  importance: number
}

export interface MemoryListResult {
  success: boolean
  data: MemoryListItem[]
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

export interface DarkQueryResult {
  success: boolean
  data: {
    nodes: { coord: string; energy: number; is_manifested: boolean }[]
    total: number
  }
}

export interface DarkFlowResult {
  success: boolean
  data: {
    from: string
    to: string
    amount: number
    conservation_ok: boolean
  }
}

export interface DarkTransferResult {
  success: boolean
  data: {
    source: string
    target: string
    amount: number
    conservation_ok: boolean
  }
}

export interface DarkMaterializeResult {
  success: boolean
  data: {
    coord: string
    energy: number
    physical_ratio: number
    conservation_ok: boolean
  }
}

export interface DarkPressureResult {
  success: boolean
  data: {
    total_dark_energy: number
    total_physical_energy: number
    pressure_ratio: number
    dimension_balance_ok: boolean
  }
}

export interface PhysicsStatusResult {
  success: boolean
  data: {
    total_nodes: number
    manifested_nodes: number
    dark_nodes: number
    total_energy: number
    physics_engine: string
  }
}

export interface PhysicsProfileResult {
  success: boolean
  data: {
    energy_distribution: { physical: number; dark: number }
    node_distribution: { manifested: number; dark: number }
    conservation_ok: boolean
  }
}

export interface PhysicsDistanceResult {
  success: boolean
  data: { distance_7d: number; distance_3d: number; dark_contribution: number }
}

export interface PhysicsProjectResult {
  success: boolean
  data: { physical: number[]; dark: number[] }
}

export interface RememberResult {
  success: boolean
  data: {
    success: boolean
    anchor: string
    manifested: boolean
    created_at: number
    conservation_ok: boolean
  }
}

export interface RecallResult {
  success: boolean
  data: {
    query: string
    results: {
      anchor: string
      similarity: number
      method: string
      dimensions: number
      hebbian_neighbors: number
      associated_memories: string[]
      description: string
      tags: string[]
      category: string
      importance: number
    }[]
    total: number
  }
}

export interface AssociateResult {
  success: boolean
  data: {
    topic: string
    seed_anchor: string
    associations: {
      source: string
      targets: { anchor: string; description: string }[]
      confidence: number
      hops: number
    }[]
    total: number
  }
}

export interface ConsolidateResult {
  success: boolean
  data: {
    dream_report: {
      paths_replayed: number
      paths_weakened: number
      memories_consolidated: number
      hebbian_edges_before: number
      hebbian_edges_after: number
      weight_before: number
      weight_after: number
    }
    maintenance: { weakened_edges: number; strengthened_edges: number }
    conservation_ok: boolean
  }
}

export interface ContextResult {
  success: boolean
  data: Record<string, any>
}

export interface ForgetResult {
  success: boolean
  data: {
    success: boolean
    erased_anchor: string
    description: string
    remaining_memories: number
    conservation_ok: boolean
  }
}

export interface AnnotateResult {
  success: boolean
  data: {
    anchor: string
    tags: string[]
    category: string | null
    description: string | null
    source: string | null
    importance: number
  }
}

export interface SemanticSearchResult {
  success: boolean
  data: {
    results: {
      anchor: string
      similarity: number
      distance: number
      tags: string[]
      category: string | null
      description: string | null
      importance: number
    }[]
  }
}

export interface PhaseDetectResult {
  success: boolean
  data: {
    phase: string
    crystal_count: number
    amorphous_count: number
    transition_ongoing: boolean
  }
}

export interface EmotionStatusResult {
  success: boolean
  data: {
    pad: { pleasure: number; arousal: number; dominance: number }
    quadrant: string
    functional_cluster: string
    recommendations: string[]
  }
}

export interface PerceptionStatusResult {
  success: boolean
  data: {
    total_budget: number
    allocated: number
    available: number
    spent: number
    returned: number
    utilization: number
  }
}

export interface SemanticStatusResult {
  success: boolean
  data: {
    embeddings_indexed: number
    relations_total: number
    concepts_extracted: number
  }
}

export interface ClusteringStatusResult {
  success: boolean
  data: {
    memories_clustered: number
    attractors_found: number
    tunnels_active: number
    bridges_active: number
  }
}

export interface ConstitutionStatusResult {
  success: boolean
  data: { rules_count: number; bounds_count: number; rules: string[] }
}

export interface EventsStatusResult {
  success: boolean
  data: { history_len: number; subscriber_count: number }
}

export interface WatchdogStatusResult {
  success: boolean
  data: { total_checkups: number; uptime_ms: number }
}

export interface WatchdogCheckupResult {
  success: boolean
  data: {
    level: string
    utilization: number
    conservation_ok: boolean
    actions: string[]
  }
}

export interface AgentExecuteResult {
  success: boolean
  data: { agent: string; success: boolean; duration_ms: number; details: string }
}

export interface MetricsResult {
  success: boolean
  data: string
}

export interface ConservationResult {
  success: boolean
  data: {
    conservation_ok: boolean
    energy_drift: number
    total_energy: number
    allocated_energy: number
    available_energy: number
    violation: number
  }
}

export interface PluginInfo {
  manifest: {
    name: string
    version: string
    author: string
    description: string
    api_version: number
    energy_budget: number
    permissions: {
      memory_read: boolean
      memory_write: boolean
      hebbian_read: boolean
      hebbian_write: boolean
      pulse_fire: boolean
      universe_read: boolean
      event_publish: boolean
      event_subscribe: boolean
    }
    tags: string[]
  }
  status: string
  energy_consumed: number
  executions: number
  last_execution: string | null
  installed_at: string
}

export interface PluginListResult {
  success: boolean
  data: PluginInfo[]
}

export interface PluginStatsResult {
  success: boolean
  data: {
    total: number
    enabled: number
    running: number
    suspended: number
    total_energy_consumed: number
    total_executions: number
    global_energy_budget: number
  }
}

export interface PluginExecuteResult {
  success: boolean
  data: {
    output: number[]
    energy_consumed: number
    execution_time_us: number
    success: boolean
    error: string | null
  }
}

export interface AdjustWeightResult {
  success: boolean
  data: {
    from: string
    to: string
    old_weight: number
    new_weight: number
    adjustment: number
  }
}

export interface CognitiveStateResult {
  success: boolean
  data: {
    overall_vigor: number
    dream_readiness: { should_dream: boolean; urgency: number }
    emotion_snapshot: { pleasure: number; arousal: number; dominance: number }
  }
}

export interface AttentionMapResult {
  success: boolean
  data: Record<string, number>
}

export interface DreamInsightsResult {
  success: boolean
  data: {
    contradictions: string[]
    weak_connections: number
    clusters_found: number
  }
}

export interface IdentityProfileResult {
  success: boolean
  data: {
    identity_memories: number
    total_importance: number
    coherence: number
  }
}

export interface MetaCognitiveResult {
  success: boolean
  data: {
    self_awareness: number
    domains_classified: number
    confidence_avg: number
  }
}

export interface PredictionStatusResult {
  success: boolean
  data: {
    active_predictions: number
    avg_confidence: number
    surprise_avg: number
    accuracy_avg: number
  }
}

export interface MemoryStreamEvent {
  event: string
  data: Record<string, any>
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

  // -- Dark Dimension --
  darkQuery: () => request<DarkQueryResult>('/dark/query'),
  darkFlow: (from: number[], to: number[], amount: number) =>
    request<DarkFlowResult>('/dark/flow', {
      method: 'POST',
      body: JSON.stringify({ from, to, amount }),
    }),
  darkTransfer: (source: number[], target: number[], amount: number) =>
    request<DarkTransferResult>('/dark/transfer', {
      method: 'POST',
      body: JSON.stringify({ source, target, amount }),
    }),
  darkMaterialize: (coord: number[], energy: number, physicalRatio: number) =>
    request<DarkMaterializeResult>('/dark/materialize', {
      method: 'POST',
      body: JSON.stringify({ coord, energy, physical_ratio: physicalRatio }),
    }),
  darkDematerialize: (coord: number[]) =>
    request<DarkMaterializeResult>('/dark/dematerialize', {
      method: 'POST',
      body: JSON.stringify({ coord }),
    }),
  darkPressure: () => request<DarkPressureResult>('/dark/pressure'),

  // -- Physics --
  physicsStatus: () => request<PhysicsStatusResult>('/physics/status'),
  physicsProfile: () => request<PhysicsProfileResult>('/physics/profile'),
  physicsDistance: (from: number[], to: number[]) =>
    request<PhysicsDistanceResult>('/physics/distance', {
      method: 'POST',
      body: JSON.stringify({ from, to }),
    }),
  physicsProject: (coord: number[]) =>
    request<PhysicsProjectResult>('/physics/project', {
      method: 'POST',
      body: JSON.stringify({ coord }),
    }),

  // -- AI Agent Memory --
  remember: (content: string, tags?: string[], category?: string, importance?: number, source?: string) =>
    request<RememberResult>('/memory/remember', {
      method: 'POST',
      body: JSON.stringify({ content, tags: tags || [], category, importance, source }),
    }),
  recall: (query: string, limit?: number) =>
    request<RecallResult>('/memory/recall', {
      method: 'POST',
      body: JSON.stringify({ query, limit }),
    }),
  associate: (topic: string, depth?: number, limit?: number) =>
    request<AssociateResult>('/memory/associate', {
      method: 'POST',
      body: JSON.stringify({ topic, depth, limit }),
    }),
  forget: (anchor: [number, number, number]) =>
    request<ForgetResult>('/memory/forget', {
      method: 'POST',
      body: JSON.stringify({ anchor: Array.from(anchor) }),
    }),
  consolidate: (importanceThreshold?: number) =>
    request<ConsolidateResult>('/dream/consolidate', {
      method: 'POST',
      body: JSON.stringify({ importance_threshold: importanceThreshold }),
    }),
  contextAction: (action: string, role?: string, content?: string) =>
    request<ContextResult>('/context', {
      method: 'POST',
      body: JSON.stringify({ action, role, content }),
    }),
  annotateMemory: (anchor: [number, number, number], tags?: string[], category?: string, description?: string, importance?: number) =>
    request<AnnotateResult>('/memory/annotate', {
      method: 'POST',
      body: JSON.stringify({ anchor: Array.from(anchor), tags: tags || [], category, description, importance }),
    }),

  // -- Semantic --
  semanticSearch: (data: number[], k?: number) =>
    request<SemanticSearchResult>('/semantic/search', {
      method: 'POST',
      body: JSON.stringify({ data, k }),
    }),
  semanticQuery: (text: string, k?: number) =>
    request<SemanticSearchResult>('/semantic/query', {
      method: 'POST',
      body: JSON.stringify({ text, k }),
    }),
  semanticRelations: (anchor: [number, number, number]) =>
    request<SemanticSearchResult>('/semantic/relations', {
      method: 'POST',
      body: JSON.stringify({ anchor: Array.from(anchor) }),
    }),
  semanticStatus: () => request<SemanticStatusResult>('/semantic/status'),
  semanticIndexAll: () =>
    request<AgentExecuteResult>('/semantic/index-all', { method: 'POST' }),
  semanticExtractConcepts: () =>
    request<AgentExecuteResult>('/semantic/extract-concepts', { method: 'POST' }),

  // -- Phase / Crystal --
  phaseDetect: () => request<PhaseDetectResult>('/phase/detect'),

  // -- Emotion --
  emotionStatus: () => request<EmotionStatusResult>('/emotion/status'),
  emotionPulse: (anchor: [number, number, number]) =>
    request<PulseResult>('/emotion/pulse', {
      method: 'POST',
      body: JSON.stringify({ anchor: Array.from(anchor) }),
    }),
  emotionDream: () => request<DreamResult>('/emotion/dream', { method: 'POST' }),

  // -- Perception --
  perceptionStatus: () => request<PerceptionStatusResult>('/perception/status'),
  perceptionReplenish: () =>
    request<AgentExecuteResult>('/perception/replenish', { method: 'POST' }),

  // -- Clustering --
  clusteringStatus: () => request<ClusteringStatusResult>('/clustering/status'),
  clusteringMaintenance: () =>
    request<AgentExecuteResult>('/clustering/maintenance', { method: 'POST' }),

  // -- Constitution --
  constitutionStatus: () => request<ConstitutionStatusResult>('/constitution/status'),

  // -- Events --
  eventsStatus: () => request<EventsStatusResult>('/events/status'),

  // -- Watchdog --
  watchdogStatus: () => request<WatchdogStatusResult>('/watchdog/status'),
  watchdogCheckup: () =>
    request<WatchdogCheckupResult>('/watchdog/checkup', { method: 'POST' }),

  // -- Agents --
  agentObserver: () => request<AgentExecuteResult>('/agent/observer'),
  agentEmotion: () => request<AgentExecuteResult>('/agent/emotion'),

  // -- Metrics / Conservation --
  getMetrics: () => request<MetricsResult>('/metrics'),
  conservationCheck: () => request<ConservationResult>('/conservation/status'),

  // -- Plugins --
  pluginList: () => request<PluginListResult>('/plugins/list'),
  pluginStats: () => request<PluginStatsResult>('/plugins/stats'),
  pluginStatus: (name: string) => request<PluginListResult>(`/plugins/${encodeURIComponent(name)}/status`),
  pluginInstall: (manifest: any, wasmBase64: string) =>
    request<{ success: boolean; data: { installed: boolean; status: string } }>('/plugins/install', {
      method: 'POST',
      body: JSON.stringify({ manifest, wasm_base64: wasmBase64 }),
    }),
  pluginUninstall: (name: string) =>
    request<{ success: boolean; data: { uninstalled: boolean; version: string } }>(`/plugins/${encodeURIComponent(name)}/uninstall`, {
      method: 'POST',
    }),
  pluginEnable: (name: string) =>
    request<{ success: boolean; data: { name: string; status: string } }>(`/plugins/${encodeURIComponent(name)}/enable`, {
      method: 'POST',
    }),
  pluginDisable: (name: string) =>
    request<{ success: boolean; data: { name: string; status: string } }>(`/plugins/${encodeURIComponent(name)}/disable`, {
      method: 'POST',
    }),
  pluginExecute: (name: string, func: string, input?: number[], energyLimit?: number) =>
    request<PluginExecuteResult>(`/plugins/${encodeURIComponent(name)}/execute`, {
      method: 'POST',
      body: JSON.stringify({ function: func, input: input || [], energy_limit: energyLimit }),
    }),
  pluginResetEnergy: (name: string) =>
    request<{ success: boolean; data: { name: string; energy_reset: boolean } }>(`/plugins/${encodeURIComponent(name)}/reset-energy`, {
      method: 'POST',
    }),

  // -- Memory Advanced --
  adjustWeight: (fromAnchor: number[], toAnchor: number[], boost: number) =>
    request<AdjustWeightResult>('/memory/adjust_weight', {
      method: 'POST',
      body: JSON.stringify({ from: fromAnchor, to: toAnchor, boost }),
    }),

  // -- Cognitive --
  getCognitiveState: () => request<CognitiveStateResult>('/cognitive/state'),
  getAttentionMap: () => request<AttentionMapResult>('/cognitive/attention'),
  getDreamInsights: () => request<DreamInsightsResult>('/cognitive/insights'),
  reflect: () =>
    request<AgentExecuteResult>('/cognitive/reflect', { method: 'POST' }),
  getIdentityProfile: () => request<IdentityProfileResult>('/cognitive/identity'),
  getMetaCognitive: () => request<MetaCognitiveResult>('/cognitive/meta'),
  getPredictionStatus: () => request<PredictionStatusResult>('/cognitive/prediction'),
}

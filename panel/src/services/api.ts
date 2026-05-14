const BASE = "/api";

export interface ApiResponse<T = unknown> {
  success: boolean;
  data: T;
  error?: string;
}

async function request<T>(path: string, options?: RequestInit): Promise<ApiResponse<T>> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { "Content-Type": "application/json" },
    ...options,
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}: ${res.statusText}`);
  return res.json();
}

function post<T>(path: string, body?: unknown): Promise<ApiResponse<T>> {
  return request<T>(path, {
    method: "POST",
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
}

function get<T>(path: string): Promise<ApiResponse<T>> {
  return request<T>(path);
}

/* ═══════════════════════════ TYPES ═══════════════════════════ */

export type StatsData = ApiResponse<{
  nodes: number;
  manifested: number;
  dark: number;
  conservation_ok: boolean;
  hebbian_edges: number;
  memory_count: number;
  utilization: number;
  available_energy: number;
  physical_energy: number;
  dark_energy: number;
  allocated_energy: number;
  total_energy: number;
  even: number;
  odd: number;
}>;

export type HealthData = ApiResponse<{
  conservation_ok: boolean;
  energy_utilization: number;
  node_count: number;
  manifested_ratio: number;
  hebbian_edge_count: number;
  hebbian_avg_weight: number;
  memory_count: number;
  frontier_size: number;
  level: string;
  hebbian_edges: number;
}>;

export type EncodeResult = ApiResponse<{
  anchor: string;
  data_dim: number;
  manifested: boolean;
  importance: number;
}>;

export type DecodeResult = ApiResponse<{
  anchor: string;
  data: number[];
}>;

export interface MemoryListItem {
  anchor: string;
  data_dim: number;
  tags: string[];
  category: string | null;
  description: string | null;
  importance: number;
  created_at: number;
}

export type HebbianNeighborsResult = ApiResponse<{
  neighbors: Array<{ coord: string; weight: number }>;
}>;

export type DarkQueryResult = ApiResponse<{
  nodes: Array<{
    coord: string | number[];
    is_manifested: boolean;
    energy: number;
  }>;
  total: number;
}>;

export interface DarkNodeInfo {
  exists: boolean;
  physical_energy: number;
  dark_energy: number;
  is_manifested: boolean;
}

export type DarkPressureResult = ApiResponse<{
  dimension_spread: number[];
  dark_node_count: number;
  physical_node_count: number;
  avg_dark_ratio: number;
  total_dark_energy: number;
  total_physical_energy: number;
  pressure_ratio: number;
  dimension_balance_ok: boolean;
}>;

export type ClusterStatusResult = ApiResponse<{
  node_id: string;
  state: string;
  nodes: Array<{ id: string; addr: string; role: string }>;
  leader: string | null;
  leader_id: string;
  role: string;
  term: number;
  applied_count: number;
  log_index: number;
  members: Array<{ id: string; addr: string }>;
}>;

export interface TimelineDay {
  anchor: string;
  timestamp: string;
  event_type: string;
  data_dim: number;
  importance: number;
  count: number;
  date: string;
  anchors: string[];
}

export interface TraceHop {
  anchor: string;
  coord: number[];
  weight: number;
  energy: number;
  hop: number;
  created_at: number;
  data_dim: number;
  confidence: number;
}

export type TraceResult = ApiResponse<TraceHop[]>;

export type PhaseDetectResult = ApiResponse<{
  phase: string;
  crystal_count: number;
  amorphous_count: number;
  transition_ongoing: boolean;
}>;

export type PhysicsStatusResult = ApiResponse<{
  total_nodes: number;
  manifested_nodes: number;
  dark_nodes: number;
  total_energy: number;
  physics_engine: string;
}>;

export type PhysicsProfileResult = ApiResponse<{
  energy_distribution: { physical: number; dark: number };
  conservation_ok: boolean;
}>;

export type PhysicsDistanceResult = ApiResponse<{
  distance_sq: number;
  distance_7d: number;
  distance_3d: number;
  dark_contribution: number;
}>;

export type PhysicsProjectResult = ApiResponse<{
  projected: number[];
  dimensions: number;
  physical: number[];
  dark: number[];
}>;

export type SemanticStatusResult = ApiResponse<{
  active: boolean;
  index_size: number;
  embedding_dim: number;
  model: string;
  embeddings_indexed: number;
  relations_total: number;
  concepts_extracted: number;
}>;

export type SemanticSearchResult = ApiResponse<{
  results: Array<{
    atom_key: string;
    anchor: string;
    similarity: number;
    distance: number;
    description: string;
    tags: string[];
    category: string;
    importance: number;
  }>;
}>;

export type RecallResult = ApiResponse<{
  results: Array<{
    anchor: string;
    content: string;
    similarity: number;
    tags: string[];
    method: string;
    description: string;
  }>;
  query: string;
  total: number;
}>;

export type AssociateResult = ApiResponse<{
  associations: Array<{
    topic: string;
    strength: number;
    related: string[];
    source: string;
    confidence: number;
    hops: number;
    targets: Array<{ name: string; description: string; anchor: string }>;
  }>;
  topic: string;
  seed_anchor: string;
  total: number;
}>;

export type WatchdogStatusResult = ApiResponse<{
  level: string;
  uptime: number;
  checks_total: number;
  checks_passed: number;
  last_check: string;
  total_checkups: number;
  uptime_ms: number;
}>;

export type WatchdogCheckupResult = ApiResponse<{
  level: string;
  checks: Array<{
    name: string;
    status: string;
    message: string;
  }>;
  utilization: number;
  conservation_ok: boolean;
  actions: string[];
}>;

export type ClusteringStatusResult = ApiResponse<Record<string, unknown>>;
export type ConstitutionStatusResult = ApiResponse<Record<string, unknown>>;
export type EventsStatusResult = ApiResponse<Record<string, unknown>>;

export interface PluginInfo {
  name: string;
  enabled: boolean;
  energy: number;
  status: string;
  manifest: {
    name: string;
    description: string;
    version: string;
    author: string;
    tags: string[];
    api_version: string;
    energy_budget: number;
    permissions: string[];
  };
  executions: number;
  energy_consumed: number;
  installed_at: string;
  last_execution: string;
}

export type PluginStatsResult = ApiResponse<{
  total: number;
  active: number;
  total_energy: number;
  available_energy: number;
  running: number;
  enabled: number;
  suspended: number;
  global_energy_budget: number;
  total_energy_consumed: number;
  total_executions: number;
}>;

export interface BackupInfo {
  backup_id: number;
  bytes: number;
  created_at: string;
  id: number;
  generation: number;
  node_count: number;
  memory_count: number;
  conservation_ok: boolean;
  trigger: string;
  timestamp_ms: number;
}

export type PulseResult = ApiResponse<{
  visited_nodes: number;
  total_activation: number;
  final_strength: number;
}>;

export type DreamResult = ApiResponse<{
  edges_before: number;
  edges_after: number;
  new_edges: number;
  paths_replayed: number;
  memories_consolidated: number;
  paths_weakened: number;
}>;

export type CognitiveState = ApiResponse<{
  regulation: { last_cycle: string; actions_count: number };
  prediction: { active: boolean };
  surprise: { last_score: number };
}>;

export type EmotionStatusResult = ApiResponse<{
  pad: { pleasure: number; arousal: number; dominance: number };
  quadrant: string;
  functional_cluster: string;
  recommendations: string[];
}>;

export type PerceptionStatusResult = ApiResponse<{
  total_budget: number;
  allocated: number;
  available: number;
  spent: number;
  returned: number;
  utilization: number;
}>;

/* ═══════════════════════════ API ═══════════════════════════ */

export const api = {
  getHealth: () => get<HealthData["data"]>("/health"),
  getStats: () => get<StatsData["data"]>("/stats"),

  encodeMemory: (
    anchor: number[],
    data: number[],
    tags?: string[],
    description?: string,
    importance?: number,
  ) =>
    post<EncodeResult["data"]>("/memory/encode", {
      anchor,
      data,
      tags: tags || [],
      description,
      importance: importance ?? 0.5,
    }),
  decodeMemory: (anchor: number[], dataDim: number) =>
    post<DecodeResult["data"]>("/memory/decode", {
      anchor,
      data_dim: dataDim,
    }),
  listMemories: () => get<MemoryListItem[]>("/memory/list"),
  annotateMemory: (
    anchor: number[],
    tags?: string[],
    category?: string,
    description?: string,
    importance?: number,
  ) => post<unknown>("/memory/annotate", { anchor, tags, category, description, importance }),
  remember: (
    content: string,
    tags: string[],
    category?: string,
    importance?: number,
  ) =>
    post<EncodeResult["data"]>("/memory/remember", {
      content,
      tags,
      category,
      importance: importance ?? 0.5,
    }),
  recall: (query: string, limit?: number) =>
    post<RecallResult["data"]>("/memory/recall", { query, limit: limit ?? 10 }),
  forget: (anchor: number[]) => post<unknown>("/memory/forget", { anchor }),
  adjustWeight: (from: number[], to: number[], boost: number) =>
    post<{ old_weight: number; new_weight: number }>("/memory/adjust_weight", {
      from,
      to,
      boost,
    }),
  traceMemory: (anchor: number[], maxHops?: number) =>
    post<TraceHop[]>("/memory/trace", { anchor, max_hops: maxHops }),
  getTimeline: () => get<TimelineDay[]>("/memory/timeline"),
  semanticRelations: (anchor: number[]) =>
    post<SemanticSearchResult["data"]>("/semantic/relations", { anchor }),

  firePulse: (source: number[], pulseType?: string) =>
    post<PulseResult["data"]>("/pulse", {
      source,
      pulse_type: pulseType || "exploratory",
    }),
  runDream: () => post<DreamResult["data"]>("/dream", {}),
  consolidate: (threshold?: number) =>
    post<unknown>("/dream/consolidate", { importance_threshold: threshold ?? 0.3 }),

  autoScale: () =>
    post<{ nodes_added: number; nodes_removed: number; reason: string }>("/scale", {}),
  frontierExpand: (maxNew: number) =>
    post<{ nodes_added: number; nodes_removed: number; reason: string }>(
      `/scale/frontier/${maxNew}`,
      {},
    ),
  regulate: () => post<string[]>("/regulate", {}),

  darkQuery: () => get<DarkQueryResult["data"]>("/dark/query"),
  darkMaterialize: (coord: number[], energy: number, physicalRatio?: number) =>
    post<{ coord: string; energy: number; physical_ratio: number }>("/dark/materialize", {
      coord,
      energy,
      physical_ratio: physicalRatio ?? 0.6,
    }),
  darkDematerialize: (coord: number[]) =>
    post<{ coord: string; energy: number }>("/dark/dematerialize", { coord }),
  darkFlow: (coord: number[], direction: string, amount: number) =>
    post<unknown>("/dark/flow", { coord, direction, amount }),
  darkTransfer: (from: number[], to: number[], amount: number) =>
    post<unknown>("/dark/transfer", { from, to, amount }),
  darkPressure: () => get<DarkPressureResult["data"]>("/dark/pressure"),

  getHebbianNeighbors: (x: number, y: number, z: number) =>
    get<HebbianNeighborsResult["data"]>(`/hebbian/neighbors/${x}/${y}/${z}`),

  getClusterStatus: () => get<ClusterStatusResult["data"]>("/cluster/status"),
  initCluster: () => post<ClusterStatusResult["data"]>("/cluster/init", {}),
  addClusterNode: (nodeId: number, addr: string) =>
    post<ClusterStatusResult["data"]>("/cluster/add-node", { node_id: nodeId, addr }),
  removeClusterNode: (nodeId: number) =>
    post<unknown>("/cluster/remove-node", { node_id: nodeId }),
  clusterPropose: (key: string, value: string) =>
    post<{ log_index: number }>("/cluster/propose", { key, value }),

  createBackup: () => post<BackupInfo>("/backup/create", {}),
  listBackups: () => get<BackupInfo[]>("/backup/list"),

  semanticSearch: (data: number[], k?: number) =>
    post<SemanticSearchResult["data"]>("/semantic/search", { data, k: k ?? 10 }),
  semanticStatus: () => get<SemanticStatusResult["data"]>("/semantic/status"),
  semanticQuery: (query: string) =>
    post<SemanticSearchResult["data"]>("/semantic/query", { query }),
  associate: (topic: string) =>
    post<AssociateResult["data"]>("/semantic/associate", { topic }),

  physicsStatus: () => get<PhysicsStatusResult["data"]>("/physics/status"),
  physicsProfile: () => get<PhysicsProfileResult["data"]>("/physics/profile"),
  physicsDistance: (from: number[], to: number[]) =>
    post<PhysicsDistanceResult["data"]>("/physics/distance", { from, to }),
  physicsProject: (coords: number[]) =>
    post<PhysicsProjectResult["data"]>("/physics/project", { coords }),

  emotionStatus: () => get<EmotionStatusResult["data"]>("/emotion/status"),
  perceptionStatus: () => get<PerceptionStatusResult["data"]>("/perception/status"),
  emotionPulse: (anchor: number[]) =>
    post<PulseResult["data"]>("/emotion/pulse", { anchor }),
  emotionDream: () => post<DreamResult["data"]>("/emotion/dream", {}),

  watchdogStatus: () => get<WatchdogStatusResult["data"]>("/watchdog/status"),
  watchdogCheckup: () => post<WatchdogCheckupResult["data"]>("/watchdog/checkup", {}),
  clusteringStatus: () => get<Record<string, unknown>>("/clustering/status"),
  constitutionStatus: () => get<Record<string, unknown>>("/constitution/status"),
  eventsStatus: () => get<Record<string, unknown>>("/events/status"),
  phaseDetect: () => get<PhaseDetectResult["data"]>("/phase/detect"),

  pluginList: () => get<PluginInfo[]>("/plugins/list"),
  pluginStats: () => get<PluginStatsResult["data"]>("/plugins/stats"),
  pluginInstall: (name: string) => post<unknown>("/plugins/install", { name }),
  pluginEnable: (name: string) => post<unknown>("/plugins/enable", { name }),
  pluginDisable: (name: string) => post<unknown>("/plugins/disable", { name }),
  pluginUninstall: (name: string) => post<unknown>("/plugins/uninstall", { name }),
  pluginResetEnergy: (name: string) =>
    post<unknown>("/plugins/reset-energy", { name }),
  pluginExecute: (name: string) => post<unknown>(`/plugins/${name}/execute`, {}),
};

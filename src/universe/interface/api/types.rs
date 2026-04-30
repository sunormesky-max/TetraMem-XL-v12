use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub nodes: usize,
    pub manifested: usize,
    pub dark: usize,
    pub even: usize,
    pub odd: usize,
    pub total_energy: f64,
    pub allocated_energy: f64,
    pub available_energy: f64,
    pub physical_energy: f64,
    pub dark_energy: f64,
    pub utilization: f64,
    pub conservation_ok: bool,
    pub energy_drift: f64,
    pub memory_count: usize,
    pub hebbian_edges: usize,
    pub hebbian_total_weight: f64,
}

#[derive(Deserialize)]
pub struct EncodeRequest {
    pub anchor: [i32; 3],
    pub data: Vec<f64>,
}

#[derive(Serialize)]
pub struct EncodeResponse {
    pub anchor: String,
    pub data_dim: usize,
    pub manifested: bool,
    pub created_at: u64,
}

#[derive(Deserialize)]
pub struct DecodeRequest {
    pub anchor: [i32; 3],
    pub data_dim: usize,
}

#[derive(Serialize)]
pub struct DecodeResponse {
    pub data: Vec<f64>,
}

#[derive(Serialize)]
pub struct BackupInfo {
    pub id: u64,
    pub timestamp_ms: u64,
    pub trigger: String,
    pub node_count: usize,
    pub memory_count: usize,
    pub total_energy: f64,
    pub conservation_ok: bool,
    pub bytes: usize,
    pub generation: u32,
}

#[derive(Serialize)]
pub struct CreateBackupResponse {
    pub backup_id: u64,
    pub generation: u32,
    pub node_count: usize,
    pub memory_count: usize,
    pub bytes: usize,
    pub elapsed_ms: f64,
}

#[derive(Deserialize)]
pub struct PulseRequest {
    pub source: [i32; 3],
    #[serde(default = "default_pulse_type")]
    pub pulse_type: String,
}

fn default_pulse_type() -> String {
    "exploratory".to_string()
}

#[derive(Serialize)]
pub struct PulseResponse {
    pub visited_nodes: usize,
    pub total_activation: f64,
    pub paths_recorded: usize,
    pub final_strength: f64,
}

#[derive(Serialize)]
pub struct DreamResponse {
    pub paths_replayed: usize,
    pub paths_weakened: usize,
    pub memories_consolidated: usize,
    pub edges_before: usize,
    pub edges_after: usize,
    pub weight_before: f64,
    pub weight_after: f64,
}

#[derive(Serialize)]
pub struct ScaleResponse {
    pub energy_expanded_by: f64,
    pub nodes_added: usize,
    pub nodes_removed: usize,
    pub reason: String,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub level: String,
    pub conservation_ok: bool,
    pub energy_utilization: f64,
    pub node_count: usize,
    pub manifested_ratio: f64,
    pub hebbian_edge_count: usize,
    pub hebbian_avg_weight: f64,
    pub memory_count: usize,
    pub frontier_size: usize,
}

#[derive(Serialize)]
pub struct HebbianNeighborsResponse {
    pub node: String,
    pub neighbors: Vec<NeighborInfo>,
}

#[derive(Serialize)]
pub struct NeighborInfo {
    pub coord: String,
    pub weight: f64,
}

#[derive(Serialize)]
pub struct OpenApiDoc {
    pub openapi: String,
    pub info: OpenApiInfo,
    pub paths: serde_json::Value,
}

#[derive(Serialize)]
pub struct OpenApiInfo {
    pub title: String,
    pub version: String,
    pub description: String,
}

#[derive(Serialize)]
pub struct TimelineDay {
    pub date: String,
    pub count: usize,
    pub anchors: Vec<String>,
}

#[derive(Serialize)]
pub struct TraceHop {
    pub anchor: String,
    pub created_at: u64,
    pub data_dim: usize,
    pub confidence: f64,
    pub hop: usize,
}

#[derive(Deserialize)]
pub struct TraceRequest {
    pub anchor: [i32; 3],
    pub max_hops: Option<usize>,
}

#[derive(Deserialize)]
pub struct ClusterInitRequest {
    pub node_id: Option<u64>,
    pub addr: Option<String>,
}

#[derive(Deserialize)]
pub struct PhaseConsensusRequest {
    #[serde(default)]
    pub force: bool,
}

#[derive(Deserialize)]
pub struct QuorumStartRequest {
    pub required_energy_budget: Option<f64>,
}

#[derive(Deserialize)]
pub struct QuorumExecuteRequest {
    #[serde(default)]
    pub force: bool,
}

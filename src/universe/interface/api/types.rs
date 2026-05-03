// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
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
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default = "default_importance")]
    pub importance: f64,
}

#[derive(Serialize)]
pub struct EncodeResponse {
    pub anchor: String,
    pub data_dim: usize,
    pub manifested: bool,
    pub created_at: u64,
}

#[derive(Deserialize)]
pub struct AnnotateRequest {
    pub anchor: [i32; 3],
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default = "default_importance")]
    pub importance: f64,
}

#[derive(Serialize)]
pub struct AnnotateResponse {
    pub anchor: String,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub importance: f64,
}

#[derive(Deserialize)]
pub struct SemanticSearchRequest {
    pub data: Vec<f64>,
    #[serde(default = "default_k")]
    pub k: usize,
}

#[derive(Serialize)]
pub struct SemanticSearchResponse {
    pub results: Vec<SemanticHit>,
}

#[derive(Serialize)]
pub struct SemanticHit {
    pub anchor: String,
    pub similarity: f64,
    pub distance: f64,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub importance: f64,
}

#[derive(Deserialize)]
pub struct SemanticTextQueryRequest {
    pub text: String,
    #[serde(default = "default_k")]
    pub k: usize,
}

#[derive(Deserialize)]
pub struct SemanticRelationRequest {
    pub anchor: [i32; 3],
}

#[derive(Serialize)]
pub struct SemanticRelationResponse {
    pub anchor: String,
    pub relations: Vec<RelationInfo>,
}

#[derive(Serialize)]
pub struct RelationInfo {
    pub from_anchor: String,
    pub to_anchor: String,
    pub relation_type: String,
    pub weight: f64,
}

fn default_k() -> usize {
    10
}

fn default_importance() -> f64 {
    0.5
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

pub fn default_pulse_type() -> String {
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

#[derive(Serialize)]
pub struct MemoryListItem {
    pub anchor: String,
    pub data_dim: usize,
    pub created_at: u64,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub importance: f64,
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

#[derive(Serialize)]
pub struct PerceptionStatusResponse {
    pub total_budget: f64,
    pub allocated: f64,
    pub available: f64,
    pub spent: f64,
    pub returned: f64,
    pub utilization: f64,
}

#[derive(Serialize)]
pub struct SemanticStatusResponse {
    pub embeddings_indexed: usize,
    pub relations_total: usize,
    pub concepts_extracted: usize,
}

#[derive(Serialize)]
pub struct ClusteringStatusResponse {
    pub memories_clustered: usize,
    pub attractors_found: usize,
    pub tunnels_active: usize,
    pub bridges_active: usize,
}

#[derive(Serialize)]
pub struct ConstitutionStatusResponse {
    pub rules_count: usize,
    pub bounds_count: usize,
    pub rules: Vec<String>,
}

#[derive(Serialize)]
pub struct EventsStatusResponse {
    pub history_len: usize,
    pub subscriber_count: usize,
}

#[derive(Serialize)]
pub struct WatchdogStatusResponse {
    pub total_checkups: u64,
    pub uptime_ms: f64,
}

#[derive(Serialize)]
pub struct WatchdogCheckupResponse {
    pub level: String,
    pub utilization: f64,
    pub conservation_ok: bool,
    pub actions: Vec<String>,
}

#[derive(Serialize)]
pub struct AgentExecuteResponse {
    pub agent: String,
    pub success: bool,
    pub duration_ms: f64,
    pub details: String,
}

#[derive(Deserialize)]
pub struct RememberRequest {
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_category")]
    pub category: Option<String>,
    #[serde(default = "default_importance")]
    pub importance: f64,
    #[serde(default = "default_source")]
    pub source: Option<String>,
}

fn default_category() -> Option<String> {
    Some("general".to_string())
}

fn default_source() -> Option<String> {
    Some("api".to_string())
}

#[derive(Deserialize)]
pub struct RecallRequest {
    pub query: String,
    #[serde(default = "default_k")]
    pub limit: usize,
}

#[derive(Deserialize)]
pub struct AssociateRequest {
    pub topic: String,
    #[serde(default = "default_depth")]
    pub depth: usize,
    #[serde(default = "default_k")]
    pub limit: usize,
}

fn default_depth() -> usize {
    3
}

#[derive(Deserialize)]
pub struct ConsolidateRequest {
    #[serde(default = "default_importance_threshold")]
    pub importance_threshold: f64,
}

fn default_importance_threshold() -> f64 {
    0.3
}

#[derive(Deserialize)]
pub struct ContextRequest {
    pub action: String,
    pub role: Option<String>,
    pub content: Option<String>,
}

#[derive(Deserialize)]
pub struct ForgetRequest {
    pub anchor: [i32; 3],
}

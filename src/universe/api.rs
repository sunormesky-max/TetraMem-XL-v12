use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::universe::autoscale::AutoScaler;
use crate::universe::coord::Coord7D;
use crate::universe::dream::DreamEngine;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::{MemoryAtom, MemoryCodec};
use crate::universe::node::DarkUniverse;
use crate::universe::observer::{SelfRegulator, UniverseObserver};
use crate::universe::pulse::{PulseEngine, PulseType};

pub struct AppState {
    pub universe: Mutex<DarkUniverse>,
    pub hebbian: Mutex<HebbianMemory>,
    pub memories: Mutex<Vec<MemoryAtom>>,
}

pub type SharedState = Arc<AppState>;

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

pub fn create_router(state: SharedState) -> Router {
    Router::new()
        .route("/stats", get(get_stats))
        .route("/health", get(get_health))
        .route("/memory/encode", post(encode_memory))
        .route("/memory/decode", post(decode_memory))
        .route("/memory/list", get(list_memories))
        .route("/pulse", post(fire_pulse))
        .route("/dream", post(run_dream))
        .route("/scale", post(auto_scale))
        .route("/scale/frontier/:max_new", post(frontier_expand))
        .route("/hebbian/neighbors/:x/:y/:z", get(get_hebbian_neighbors))
        .route("/regulate", post(regulate))
        .with_state(state)
}

async fn get_stats(State(state): State<SharedState>) -> Json<ApiResponse<StatsResponse>> {
    let u = state.universe.lock().await;
    let h = state.hebbian.lock().await;
    let mems = state.memories.lock().await;
    let stats = u.stats();

    Json(ApiResponse::ok(StatsResponse {
        nodes: stats.active_nodes,
        manifested: stats.manifested_nodes,
        dark: stats.dark_nodes,
        even: stats.even_nodes,
        odd: stats.odd_nodes,
        total_energy: stats.total_energy,
        allocated_energy: stats.allocated_energy,
        available_energy: stats.available_energy,
        physical_energy: stats.physical_energy,
        dark_energy: stats.dark_energy,
        utilization: stats.utilization,
        conservation_ok: u.verify_conservation(),
        memory_count: mems.len(),
        hebbian_edges: h.edge_count(),
        hebbian_total_weight: h.total_weight(),
    }))
}

async fn get_health(State(state): State<SharedState>) -> Json<ApiResponse<HealthResponse>> {
    let u = state.universe.lock().await;
    let h = state.hebbian.lock().await;
    let mems = state.memories.lock().await;

    let report = UniverseObserver::inspect(&u, &h, &mems);

    Json(ApiResponse::ok(HealthResponse {
        level: report.health_level().as_str().to_string(),
        conservation_ok: report.conservation_ok,
        energy_utilization: report.energy_utilization,
        node_count: report.node_count,
        manifested_ratio: report.manifested_ratio,
        hebbian_edge_count: report.hebbian_edge_count,
        hebbian_avg_weight: report.hebbian_avg_weight,
        memory_count: report.memory_count,
        frontier_size: report.frontier_size,
    }))
}

async fn encode_memory(
    State(state): State<SharedState>,
    Json(req): Json<EncodeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<EncodeResponse>>), StatusCode> {
    let mut u = state.universe.lock().await;
    let mut mems = state.memories.lock().await;

    let anchor = Coord7D::new_even([req.anchor[0], req.anchor[1], req.anchor[2], 0, 0, 0, 0]);

    match MemoryCodec::encode(&mut u, &anchor, &req.data) {
        Ok(atom) => {
            let manifested = atom.is_manifested(&u);
            let anchor_str = format!("{}", atom.anchor());
            mems.push(atom);
            Ok((
                StatusCode::OK,
                Json(ApiResponse::ok(EncodeResponse {
                    anchor: anchor_str,
                    data_dim: req.data.len(),
                    manifested,
                })),
            ))
        }
        Err(e) => Ok((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::err(format!("encode failed: {}", e))),
        )),
    }
}

async fn decode_memory(
    State(state): State<SharedState>,
    Json(req): Json<DecodeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<DecodeResponse>>), StatusCode> {
    let u = state.universe.lock().await;
    let mems = state.memories.lock().await;

    let anchor = Coord7D::new_even([req.anchor[0], req.anchor[1], req.anchor[2], 0, 0, 0, 0]);

    for mem in mems.iter() {
        if mem.anchor() == &anchor && mem.data_dim() == req.data_dim {
            match MemoryCodec::decode(&u, mem) {
                Ok(data) => {
                    return Ok((
                        StatusCode::OK,
                        Json(ApiResponse::ok(DecodeResponse { data })),
                    ));
                }
                Err(e) => {
                    return Ok((
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::err(format!("decode failed: {}", e))),
                    ));
                }
            }
        }
    }

    Ok((
        StatusCode::NOT_FOUND,
        Json(ApiResponse::err("memory not found")),
    ))
}

async fn list_memories(State(state): State<SharedState>) -> Json<ApiResponse<Vec<String>>> {
    let mems = state.memories.lock().await;
    let list: Vec<String> = mems.iter().map(|m| format!("{}", m)).collect();
    Json(ApiResponse::ok(list))
}

async fn fire_pulse(
    State(state): State<SharedState>,
    Json(req): Json<PulseRequest>,
) -> Json<ApiResponse<PulseResponse>> {
    let u = state.universe.lock().await;
    let mut h = state.hebbian.lock().await;

    let source = Coord7D::new_even([req.source[0], req.source[1], req.source[2], 0, 0, 0, 0]);
    let pt = match req.pulse_type.to_lowercase().as_str() {
        "reinforcing" => PulseType::Reinforcing,
        "cascade" => PulseType::Cascade,
        _ => PulseType::Exploratory,
    };

    let engine = PulseEngine::new();
    let result = engine.propagate(&source, pt, &u, &mut h);

    Json(ApiResponse::ok(PulseResponse {
        visited_nodes: result.visited_nodes,
        total_activation: result.total_activation,
        paths_recorded: result.paths_recorded,
        final_strength: result.final_strength,
    }))
}

async fn run_dream(State(state): State<SharedState>) -> Json<ApiResponse<DreamResponse>> {
    let u = state.universe.lock().await;
    let mut h = state.hebbian.lock().await;
    let mems = state.memories.lock().await;

    let dream = DreamEngine::new();
    let report = dream.dream(&u, &mut h, &mems);

    Json(ApiResponse::ok(DreamResponse {
        paths_replayed: report.paths_replayed,
        paths_weakened: report.paths_weakened,
        memories_consolidated: report.memories_consolidated,
        edges_before: report.hebbian_edges_before,
        edges_after: report.hebbian_edges_after,
        weight_before: report.weight_before,
        weight_after: report.weight_after,
    }))
}

async fn auto_scale(State(state): State<SharedState>) -> Json<ApiResponse<ScaleResponse>> {
    let mut u = state.universe.lock().await;
    let h = state.hebbian.lock().await;
    let mems = state.memories.lock().await;

    let scaler = AutoScaler::new();
    let report = scaler.auto_scale(&mut u, &h, &mems);

    Json(ApiResponse::ok(ScaleResponse {
        energy_expanded_by: report.energy_expanded_by,
        nodes_added: report.nodes_added,
        nodes_removed: report.nodes_removed,
        reason: format!("{:?}", report.reason),
    }))
}

async fn frontier_expand(
    State(state): State<SharedState>,
    Path(max_new): Path<usize>,
) -> Json<ApiResponse<ScaleResponse>> {
    let mut u = state.universe.lock().await;

    let scaler = AutoScaler::new();
    let report = scaler.frontier_expansion(&mut u, max_new);

    Json(ApiResponse::ok(ScaleResponse {
        energy_expanded_by: report.energy_expanded_by,
        nodes_added: report.nodes_added,
        nodes_removed: report.nodes_removed,
        reason: format!("{:?}", report.reason),
    }))
}

async fn get_hebbian_neighbors(
    State(state): State<SharedState>,
    Path((x, y, z)): Path<(i32, i32, i32)>,
) -> Json<ApiResponse<HebbianNeighborsResponse>> {
    let h = state.hebbian.lock().await;
    let coord = Coord7D::new_even([x, y, z, 0, 0, 0, 0]);
    let neighbors = h.get_neighbors(&coord);

    Json(ApiResponse::ok(HebbianNeighborsResponse {
        node: format!("{}", coord),
        neighbors: neighbors
            .into_iter()
            .map(|(c, w)| NeighborInfo {
                coord: format!("{}", c),
                weight: w,
            })
            .collect(),
    }))
}

async fn regulate(State(state): State<SharedState>) -> Json<ApiResponse<Vec<String>>> {
    let u = state.universe.lock().await;
    let mut h = state.hebbian.lock().await;
    let mems = state.memories.lock().await;

    let report = UniverseObserver::inspect(&u, &h, &mems);
    let regulator = SelfRegulator::new();
    let actions = regulator.regulate(&report, &mut h);

    let descriptions: Vec<String> = actions.iter().map(|a| a.description.clone()).collect();
    Json(ApiResponse::ok(descriptions))
}

pub async fn start_server(state: SharedState, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

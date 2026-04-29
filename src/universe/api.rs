use axum::{
    extract::{DefaultBodyLimit, Path, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::universe::auth::{JwtConfig, LoginRequest, LoginResponse};
use crate::universe::autoscale::AutoScaler;
use crate::universe::backup::{BackupScheduler, BackupTrigger};
use crate::universe::cluster::{
    AddNodeRequest as ClusterAddNodeRequest, ClusterManager, ClusterStatus, EnergyQuorumEntry,
    H6PhaseTransitionProposal,
    ProposeRequest, ProposeResponse as ClusterProposeResponse, QuorumStatus,
    RemoveNodeRequest as ClusterRemoveNodeRequest,
};
use crate::universe::config::AppConfig;
use crate::universe::coord::Coord7D;
use crate::universe::dream::DreamEngine;
use crate::universe::error::AppError;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::{MemoryAtom, MemoryCodec};
use crate::universe::metrics;
use crate::universe::node::DarkUniverse;
use crate::universe::observer::{SelfRegulator, UniverseObserver};
use crate::universe::pulse::{PulseEngine, PulseType};

pub struct AppState {
    pub universe: Mutex<DarkUniverse>,
    pub hebbian: Mutex<HebbianMemory>,
    pub memories: Mutex<Vec<MemoryAtom>>,
    pub crystal: Mutex<crate::universe::crystal::CrystalEngine>,
    pub backup: Mutex<BackupScheduler>,
    pub cluster: tokio::sync::Mutex<ClusterManager>,
    pub config: AppConfig,
    pub jwt: JwtConfig,
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

async fn metrics_middleware(req: Request, next: Next) -> Response {
    metrics::API_REQUESTS_TOTAL.inc();
    let start = std::time::Instant::now();
    let response = next.run(req).await;
    metrics::REQUEST_DURATION.observe(start.elapsed().as_secs_f64());
    response
}

async fn auth_middleware(
    State(state): State<SharedState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    if !state.config.auth.enabled {
        return Ok(next.run(req).await);
    }

    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match auth_header {
        Some(token) => {
            state.jwt.validate_token(token)?;
            Ok(next.run(req).await)
        }
        None => Err(AppError::Unauthorized("missing authorization header".to_string())),
    }
}

pub fn create_router(state: SharedState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let x_request_id = axum::http::HeaderName::from_static("x-request-id");

    let public_routes = Router::new()
        .route("/health", get(get_health))
        .route("/stats", get(get_stats))
        .route("/metrics", get(get_metrics))
        .route("/openapi.json", get(get_openapi))
        .route("/login", post(login));

    let protected_routes = Router::new()
        .route("/memory/encode", post(encode_memory))
        .route("/memory/decode", post(decode_memory))
        .route("/memory/list", get(list_memories))
        .route("/pulse", post(fire_pulse))
        .route("/dream", post(run_dream))
        .route("/scale", post(auto_scale))
        .route("/scale/frontier/:max_new", post(frontier_expand))
        .route("/hebbian/neighbors/:x/:y/:z", get(get_hebbian_neighbors))
        .route("/regulate", post(regulate))
        .route("/backup/create", post(create_backup))
        .route("/backup/list", get(list_backups))
        .route("/cluster/status", get(cluster_status))
        .route("/cluster/init", post(cluster_init))
        .route("/cluster/propose", post(cluster_propose))
        .route("/cluster/add-node", post(cluster_add_node))
        .route("/cluster/remove-node", post(cluster_remove_node))
        .route("/memory/timeline", get(memory_timeline))
        .route("/memory/trace", post(memory_trace))
        .route("/phase/detect", get(detect_phase_transition))
        .route("/phase/consensus", post(phase_consensus))
        .route("/phase/quorum/start", post(quorum_start))
        .route("/phase/quorum/confirm", post(quorum_confirm))
        .route("/phase/quorum/status", get(quorum_status_endpoint))
        .route("/phase/quorum/execute", post(quorum_execute))
        .layer(middleware::from_fn(metrics_middleware))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(DefaultBodyLimit::max(state.config.server.body_limit_bytes))
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::new(
                    x_request_id.clone(),
                    MakeRequestUuid,
                ))
                .layer(TraceLayer::new_for_http())
                .layer(PropagateRequestIdLayer::new(x_request_id))
                .layer(cors)
                .layer(TimeoutLayer::new(Duration::from_secs(state.config.server.timeout_secs))),
        )
        .with_state(state)
}

async fn login(
    State(state): State<SharedState>,
    Json(req): Json<LoginRequest>,
) -> Result<(StatusCode, Json<ApiResponse<LoginResponse>>), AppError> {
    if req.username.is_empty() || req.password.is_empty() {
        return Err(AppError::BadRequest("username and password required".to_string()));
    }

    tracing::info!(username = %req.username, "user login attempt");

    let token = state.jwt.create_token(&req.username, "user")?;
    let expires_in = state.config.auth.jwt_expiry_secs;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok(LoginResponse { token, expires_in })),
    ))
}

async fn get_metrics(State(state): State<SharedState>) -> String {
    let u = state.universe.lock().await;
    let h = state.hebbian.lock().await;
    let mems = state.memories.lock().await;
    let stats = u.stats();
    metrics::update_universe_metrics(
        stats.active_nodes,
        stats.manifested_nodes,
        stats.dark_nodes,
        stats.total_energy,
        stats.allocated_energy,
        stats.available_energy,
        mems.len(),
        h.edge_count(),
    );
    drop(u);
    drop(h);
    drop(mems);
    metrics::render_metrics()
}

async fn get_openapi() -> Json<OpenApiDoc> {
    let paths: serde_json::Value = serde_json::from_str(r#"{
        "/health": {"get":{"summary":"Health check","responses":{"200":{"description":"OK"}}}},
        "/stats": {"get":{"summary":"Universe statistics","responses":{"200":{"description":"OK"}}}},
        "/metrics": {"get":{"summary":"Prometheus metrics","responses":{"200":{"description":"OK"}}}},
        "/login": {"post":{"summary":"Authenticate","responses":{"200":{"description":"JWT token"}}}},
        "/memory/encode": {"post":{"summary":"Encode memory","responses":{"200":{"description":"OK"}}}},
        "/memory/decode": {"post":{"summary":"Decode memory","responses":{"200":{"description":"OK"}}}},
        "/memory/list": {"get":{"summary":"List memories","responses":{"200":{"description":"OK"}}}},
        "/pulse": {"post":{"summary":"Fire pulse","responses":{"200":{"description":"OK"}}}},
        "/dream": {"post":{"summary":"Run dream cycle","responses":{"200":{"description":"OK"}}}},
        "/scale": {"post":{"summary":"Auto-scale universe","responses":{"200":{"description":"OK"}}}},
        "/regulate": {"post":{"summary":"Run regulation cycle","responses":{"200":{"description":"OK"}}}}
    }"#).unwrap_or_default();

    Json(OpenApiDoc {
        openapi: "3.0.3".to_string(),
        info: OpenApiInfo {
            title: "TetraMem-XL v12.0 API".to_string(),
            version: "12.0.0".to_string(),
            description: "7D Dark Universe Memory System REST API".to_string(),
        },
        paths,
    })
}

async fn get_stats(State(state): State<SharedState>) -> Json<ApiResponse<StatsResponse>> {
    let u = state.universe.lock().await;
    let h = state.hebbian.lock().await;
    let mems = state.memories.lock().await;
    let stats = u.stats();

    tracing::debug!(nodes = stats.active_nodes, utilization = %format!("{:.1}%", stats.utilization * 100.0), "stats requested");

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
        energy_drift: u.energy_drift(),
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

    let level = report.health_level().as_str().to_string();
    if level == "WARNING" || level == "CRITICAL" {
        tracing::warn!(health = %level, nodes = report.node_count, "universe health degraded");
    }

    Json(ApiResponse::ok(HealthResponse {
        level,
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
) -> Result<(StatusCode, Json<ApiResponse<EncodeResponse>>), AppError> {
    let mut u = state.universe.lock().await;
    let mut mems = state.memories.lock().await;

    let anchor = Coord7D::new_even([req.anchor[0], req.anchor[1], req.anchor[2], 0, 0, 0, 0]);

    tracing::info!(anchor = %anchor, dims = req.data.len(), "encoding memory");
    metrics::API_ENCODE_TOTAL.inc();

    match MemoryCodec::encode(&mut u, &anchor, &req.data) {
        Ok(atom) => {
            let manifested = atom.is_manifested(&u);
            let anchor_str = format!("{}", atom.anchor());
            let created_at = atom.created_at();
            tracing::info!(anchor = %anchor_str, manifested, "memory encoded successfully");
            mems.push(atom);
            Ok((
                StatusCode::OK,
                Json(ApiResponse::ok(EncodeResponse {
                    anchor: anchor_str,
                    data_dim: req.data.len(),
                    manifested,
                    created_at,
                })),
            ))
        }
        Err(e) => {
            tracing::warn!(error = %e, "memory encode failed");
            Ok((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::err(format!("encode failed: {}", e))),
            ))
        }
    }
}

async fn decode_memory(
    State(state): State<SharedState>,
    Json(req): Json<DecodeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<DecodeResponse>>), AppError> {
    let u = state.universe.lock().await;
    let mems = state.memories.lock().await;

    metrics::API_DECODE_TOTAL.inc();
    let anchor = Coord7D::new_even([req.anchor[0], req.anchor[1], req.anchor[2], 0, 0, 0, 0]);

    for mem in mems.iter() {
        if mem.anchor() == &anchor && mem.data_dim() == req.data_dim {
            match MemoryCodec::decode(&u, mem) {
                Ok(data) => {
                    tracing::debug!(anchor = %anchor, dims = data.len(), "memory decoded");
                    return Ok((
                        StatusCode::OK,
                        Json(ApiResponse::ok(DecodeResponse { data })),
                    ));
                }
                Err(e) => {
                    tracing::warn!(error = %e, "memory decode failed");
                    return Ok((
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::err(format!("decode failed: {}", e))),
                    ));
                }
            }
        }
    }

    Ok((
        StatusCode::OK,
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

    metrics::API_PULSE_TOTAL.inc();
    let source = Coord7D::new_even([req.source[0], req.source[1], req.source[2], 0, 0, 0, 0]);
    let pt = match req.pulse_type.to_lowercase().as_str() {
        "reinforcing" => PulseType::Reinforcing,
        "cascade" => PulseType::Cascade,
        _ => PulseType::Exploratory,
    };

    tracing::info!(source = %source, pulse_type = ?pt, "firing pulse");
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

    metrics::API_DREAM_TOTAL.inc();
    tracing::info!("running dream cycle");

    let dream = DreamEngine::new();
    let report = dream.dream(&u, &mut h, &mems);

    tracing::info!(
        replayed = report.paths_replayed,
        weakened = report.paths_weakened,
        consolidated = report.memories_consolidated,
        "dream cycle complete"
    );

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

    tracing::info!(
        nodes_added = report.nodes_added,
        energy_expanded = report.energy_expanded_by,
        reason = ?report.reason,
        "auto-scale complete"
    );

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

    tracing::info!(actions = actions.len(), "regulation cycle complete");
    let descriptions: Vec<String> = actions.iter().map(|a| a.description.clone()).collect();
    Json(ApiResponse::ok(descriptions))
}

pub async fn start_server(state: SharedState, addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = create_router(state.clone());
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("API server listening on http://{}", addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    if state.config.backup.auto_persist {
        let persist_path = std::path::PathBuf::from(&state.config.backup.persist_path);
        let u = state.universe.lock().await;
        let h = state.hebbian.lock().await;
        let m = state.memories.lock().await;
        let crystal = crate::universe::crystal::CrystalEngine::new();
        match crate::universe::persist_file::PersistFile::save(&persist_path, &u, &h, &m, &crystal) {
            Ok(info) => tracing::info!("final persist on shutdown: {}", info),
            Err(e) => tracing::warn!("final persist failed: {}", e),
        }
    }

    tracing::info!("server shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .unwrap_or_else(|e| tracing::error!("ctrl_c handler error: {}", e));
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .unwrap_or_else(|e| {
                tracing::error!("signal handler error: {}", e);
                std::future::pending::<()>().await
            })
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("received Ctrl+C, shutting down gracefully...");
        },
        _ = terminate => {
            tracing::info!("received SIGTERM, shutting down gracefully...");
        },
    }
}

async fn create_backup(
    State(state): State<SharedState>,
) -> Result<(StatusCode, Json<ApiResponse<CreateBackupResponse>>), AppError> {
    let u = state.universe.lock().await;
    let h = state.hebbian.lock().await;
    let m = state.memories.lock().await;
    let mut bs = state.backup.lock().await;

    let crystal = crate::universe::crystal::CrystalEngine::new();
    let report = bs.create_backup(BackupTrigger::Manual, &u, &h, &m, &crystal)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    drop(u);
    drop(h);
    drop(m);
    drop(bs);

    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok(CreateBackupResponse {
            backup_id: report.metadata.id,
            generation: report.metadata.generation,
            node_count: report.metadata.node_count,
            memory_count: report.metadata.memory_count,
            bytes: report.metadata.bytes,
            elapsed_ms: report.elapsed_ms,
        })),
    ))
}

async fn list_backups(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<Vec<BackupInfo>>>, AppError> {
    let bs = state.backup.lock().await;
    let list: Vec<BackupInfo> = bs.list_backups().into_iter().map(|m| {
        let trigger = match m.trigger {
            BackupTrigger::Manual => "MANUAL",
            BackupTrigger::Timer => "TIMER",
            BackupTrigger::PreOperation => "PRE-OP",
            BackupTrigger::ConservationCheckpoint => "CONSERV",
        };
        BackupInfo {
            id: m.id,
            timestamp_ms: m.timestamp_ms,
            trigger: trigger.to_string(),
            node_count: m.node_count,
            memory_count: m.memory_count,
            total_energy: m.total_energy,
            conservation_ok: m.conservation_ok,
            bytes: m.bytes,
            generation: m.generation,
        }
    }).collect();
    Ok(Json(ApiResponse::ok(list)))
}

async fn cluster_status(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<ClusterStatus>>, AppError> {
    let cm = state.cluster.lock().await;
    let status = cm.status().await;
    Ok(Json(ApiResponse::ok(status)))
}

#[derive(Deserialize)]
struct ClusterInitRequest {
    node_id: Option<u64>,
    addr: Option<String>,
}

async fn cluster_init(
    State(state): State<SharedState>,
    Json(req): Json<ClusterInitRequest>,
) -> Result<(StatusCode, Json<ApiResponse<ClusterStatus>>), AppError> {
    let mut cm = state.cluster.lock().await;
    if let Some(node_id) = req.node_id {
        let addr = req.addr.unwrap_or_else(|| state.config.server.addr.clone());
        *cm = ClusterManager::new(node_id, addr);
    }
    cm.init_single_node().await.map_err(|e| AppError::Internal(e))?;
    let status = cm.status().await;
    Ok((StatusCode::OK, Json(ApiResponse::ok(status))))
}

async fn cluster_propose(
    State(state): State<SharedState>,
    Json(req): Json<ProposeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<ClusterProposeResponse>>), AppError> {
    let cm = state.cluster.lock().await;
    let resp = cm.propose(req).await.map_err(|e| AppError::Internal(e))?;
    Ok((StatusCode::OK, Json(ApiResponse::ok(resp))))
}

async fn cluster_add_node(
    State(state): State<SharedState>,
    Json(req): Json<ClusterAddNodeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), AppError> {
    let mut cm = state.cluster.lock().await;
    cm.add_peer(req.node_id, req.addr).await.map_err(|e| AppError::Internal(e))?;
    Ok((StatusCode::OK, Json(ApiResponse::ok("node added".to_string()))))
}

async fn cluster_remove_node(
    State(state): State<SharedState>,
    Json(req): Json<ClusterRemoveNodeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), AppError> {
    let mut cm = state.cluster.lock().await;
    cm.remove_peer(req.node_id).await.map_err(|e| AppError::Internal(e))?;
    Ok((StatusCode::OK, Json(ApiResponse::ok("node removed".to_string()))))
}

#[derive(Serialize)]
pub struct TimelineDay {
    pub date: String,
    pub count: usize,
    pub anchors: Vec<String>,
}

async fn memory_timeline(
    State(state): State<SharedState>,
) -> Json<ApiResponse<Vec<TimelineDay>>> {
    let mems = state.memories.lock().await;
    let mut day_map: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
    for m in mems.iter() {
        let ts = if m.created_at() > 0 { m.created_at() } else { 0 };
        let date = if ts > 0 {
            chrono::DateTime::from_timestamp_millis(ts as i64)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            "unknown".to_string()
        };
        day_map.entry(date).or_default().push(format!("{}", m.anchor()));
    }
    let max_days = state.config.universe.max_timeline_days;
    let timeline: Vec<TimelineDay> = day_map.into_iter()
        .rev()
        .take(max_days)
        .map(|(date, anchors)| TimelineDay {
            count: anchors.len(),
            date,
            anchors,
        })
        .collect();
    Json(ApiResponse::ok(timeline))
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

async fn memory_trace(
    State(state): State<SharedState>,
    Json(req): Json<TraceRequest>,
) -> Result<Json<ApiResponse<Vec<TraceHop>>>, AppError> {
    let u = state.universe.lock().await;
    let h = state.hebbian.lock().await;
    let mems = state.memories.lock().await;
    let c = state.crystal.lock().await;

    let source = Coord7D::new_even([req.anchor[0], req.anchor[1], req.anchor[2], 0, 0, 0, 0]);
    let max_hops = req.max_hops.unwrap_or(10);

    let associations = crate::universe::reasoning::ReasoningEngine::find_associations(
        &u, &h, &c, &source, max_hops,
    );

    let mut hops: Vec<TraceHop> = Vec::new();

    let source_mem = mems.iter().find(|m| m.anchor() == &source);
    if let Some(m) = source_mem {
        hops.push(TraceHop {
            anchor: format!("{}", m.anchor()),
            created_at: m.created_at(),
            data_dim: m.data_dim(),
            confidence: 1.0,
            hop: 0,
        });
    }

    for r in &associations {
        for target_str in &r.targets {
            if let Some(m) = mems.iter().find(|m| format!("{}", m.anchor()) == *target_str) {
                hops.push(TraceHop {
                    anchor: target_str.clone(),
                    created_at: m.created_at(),
                    data_dim: m.data_dim(),
                    confidence: r.confidence,
                    hop: r.hops,
                });
            }
        }
    }

    Ok(Json(ApiResponse::ok(hops)))
}

async fn detect_phase_transition(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<crate::universe::crystal::PhaseTransitionReport>>, AppError> {
    let u = state.universe.lock().await;
    let h = state.hebbian.lock().await;
    let c = state.crystal.lock().await;

    let report = c.detect_phase_transition(&h, &u);

    if report.requires_consensus {
        tracing::warn!(
            candidates = report.super_channel_candidates,
            existing = report.existing_super_channels,
            "H6 phase transition detected — consensus required"
        );
    }

    Ok(Json(ApiResponse::ok(report)))
}

#[derive(Deserialize)]
struct PhaseConsensusRequest {
    #[serde(default)]
    force: bool,
}

async fn phase_consensus(
    State(state): State<SharedState>,
    Json(req): Json<PhaseConsensusRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let u = state.universe.lock().await;
    let h = state.hebbian.lock().await;
    let mut c = state.crystal.lock().await;
    let cm = state.cluster.lock().await;

    let report = c.detect_phase_transition(&h, &u);

    if !report.requires_consensus && !req.force {
        return Ok(Json(ApiResponse::ok(serde_json::json!({
            "status": "no_transition",
            "phase_coherent": report.phase_coherent,
        }))));
    }

    if !cm.is_initialized() {
        tracing::warn!("phase consensus requested but cluster not initialized, proceeding locally");
        let crystal_report = c.crystallize(&h, &u);
        drop(u);
        drop(h);
        drop(cm);
        return Ok(Json(ApiResponse::ok(serde_json::json!({
            "status": "local_consensus",
            "new_crystals": crystal_report.new_crystals,
            "new_super_crystals": crystal_report.new_super_crystals,
            "cluster": "not_initialized",
        }))));
    }

    let proposal = H6PhaseTransitionProposal {
        proposer_node: cm.node_id(),
        super_candidates: report.super_channel_candidates,
        avg_edge_weight: report.avg_edge_weight,
        energy_budget: u.stats().available_energy,
        energy_sufficient: u.stats().available_energy > 100.0,
    };

    let propose_result = cm.propose(proposal.to_propose_request()).await;

    drop(cm);

    match propose_result {
        Ok(resp) => {
            let crystal_report = c.crystallize(&h, &u);
            drop(u);
            drop(h);
            Ok(Json(ApiResponse::ok(serde_json::json!({
                "status": "consensus_committed",
                "log_index": resp.log_index,
                "conservation_verified": resp.conservation_verified,
                "new_crystals": crystal_report.new_crystals,
                "new_super_crystals": crystal_report.new_super_crystals,
            }))))
        }
        Err(e) => {
            drop(u);
            drop(h);
            tracing::error!("phase consensus rejected: {}", e);
            Ok(Json(ApiResponse::ok(serde_json::json!({
                "status": "rejected",
                "reason": e,
            }))))
        }
    }
}

#[derive(Deserialize)]
struct QuorumStartRequest {
    required_energy_budget: Option<f64>,
}

async fn quorum_start(
    State(state): State<SharedState>,
    Json(req): Json<QuorumStartRequest>,
) -> Result<Json<ApiResponse<QuorumStatus>>, AppError> {
    let u = state.universe.lock().await;
    let budget = req.required_energy_budget.unwrap_or(100.0);
    drop(u);

    let mut cm = state.cluster.lock().await;
    let status = cm.start_energy_quorum(budget);

    tracing::info!(
        quorum_id = status.quorum_id,
        phase = ?status.phase,
        confirmations = status.confirming_count,
        "energy quorum started"
    );

    Ok(Json(ApiResponse::ok(status)))
}

async fn quorum_confirm(
    State(state): State<SharedState>,
    Json(entry): Json<EnergyQuorumEntry>,
) -> Result<Json<ApiResponse<QuorumStatus>>, AppError> {
    let mut cm = state.cluster.lock().await;
    let status = cm.confirm_energy_quorum(entry.clone());

    tracing::info!(
        quorum_id = status.quorum_id,
        node = entry.node_id,
        sufficient = entry.energy_sufficient,
        phase = ?status.phase,
        "energy quorum confirmation received"
    );

    Ok(Json(ApiResponse::ok(status)))
}

async fn quorum_status_endpoint(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<Option<QuorumStatus>>>, AppError> {
    let cm = state.cluster.lock().await;
    Ok(Json(ApiResponse::ok(cm.get_quorum_status())))
}

#[derive(Deserialize)]
struct QuorumExecuteRequest {
    #[serde(default)]
    force: bool,
}

async fn quorum_execute(
    State(state): State<SharedState>,
    Json(req): Json<QuorumExecuteRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let u = state.universe.lock().await;
    let h = state.hebbian.lock().await;
    let mut c = state.crystal.lock().await;
    let mut cm = state.cluster.lock().await;

    let report = c.detect_phase_transition(&h, &u);

    if !req.force && !report.requires_consensus {
        drop(u);
        drop(h);
        drop(c);
        drop(cm);
        return Ok(Json(ApiResponse::ok(serde_json::json!({
            "status": "no_transition_needed",
        }))));
    }

    let proposal = H6PhaseTransitionProposal {
        proposer_node: cm.node_id(),
        super_candidates: report.super_channel_candidates,
        avg_edge_weight: report.avg_edge_weight,
        energy_budget: u.stats().available_energy,
        energy_sufficient: u.stats().available_energy > 100.0,
    };

    match cm.quorum_propose(proposal).await {
        Ok(resp) => {
            let crystal_report = c.crystallize(&h, &u);
            drop(u);
            drop(h);
            drop(c);
            drop(cm);
            tracing::info!(
                crystals = crystal_report.new_crystals,
                super_crystals = crystal_report.new_super_crystals,
                "H6 phase transition executed after quorum consensus"
            );
            Ok(Json(ApiResponse::ok(serde_json::json!({
                "status": "quorum_executed",
                "log_index": resp.log_index,
                "conservation_verified": resp.conservation_verified,
                "new_crystals": crystal_report.new_crystals,
                "new_super_crystals": crystal_report.new_super_crystals,
            }))))
        }
        Err(e) => {
            drop(u);
            drop(h);
            drop(c);
            drop(cm);
            tracing::warn!("quorum execute failed: {}", e);
            Ok(Json(ApiResponse::ok(serde_json::json!({
                "status": "quorum_not_reached",
                "reason": e,
            }))))
        }
    }
}

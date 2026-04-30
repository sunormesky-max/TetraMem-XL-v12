use axum::{
    extract::{DefaultBodyLimit, Request, State},
    http::StatusCode as HttpStatusCode,
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::universe::error::AppError;
use crate::universe::metrics;

use super::backup_ops::{create_backup, list_backups};
use super::cluster_ops::{
    cluster_add_node, cluster_init, cluster_propose, cluster_remove_node, cluster_status,
};
use super::cognitive::{fire_pulse, regulate, run_dream};
use super::dark_dimension::{
    dark_dematerialize, dark_dimension_pressure, dark_flow, dark_materialize, dark_query,
    dark_transfer,
};
use super::health::{get_health, get_metrics, get_openapi, get_stats};
use super::memory_ops::{
    decode_memory, encode_memory, list_memories, memory_timeline, memory_trace,
};
use super::phase::{
    detect_phase_transition, phase_consensus, quorum_confirm, quorum_execute, quorum_start,
    quorum_status_endpoint,
};
use super::scale::{auto_scale, frontier_expand, get_hebbian_neighbors};
use super::server::login;
use super::state::SharedState;

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
        None => Err(AppError::Unauthorized(
            "missing authorization header".to_string(),
        )),
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
        .route("/dark/query", get(dark_query))
        .route("/dark/flow", post(dark_flow))
        .route("/dark/transfer", post(dark_transfer))
        .route("/dark/materialize", post(dark_materialize))
        .route("/dark/dematerialize", post(dark_dematerialize))
        .route("/dark/pressure", get(dark_dimension_pressure))
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
                .layer(TimeoutLayer::with_status_code(
                    HttpStatusCode::REQUEST_TIMEOUT,
                    Duration::from_secs(state.config.server.timeout_secs),
                )),
        )
        .with_state(state)
}

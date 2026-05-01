// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use axum::{
    extract::{DefaultBodyLimit, Extension, Request, State},
    http::{HeaderValue, StatusCode as HttpStatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::universe::auth::Claims;
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
use super::emotion_ops::{emotion_crystallize, emotion_dream, emotion_pulse, emotion_status};
use super::health::{get_health, get_metrics, get_openapi, get_stats};
use super::memory_ops::{
    decode_memory, encode_memory, list_memories, memory_timeline, memory_trace,
};
use super::phase::{
    detect_phase_transition, phase_consensus, quorum_confirm, quorum_execute, quorum_start,
    quorum_status_endpoint,
};
use super::physics_ops::{
    physics_configure, physics_distance, physics_profile, physics_project, physics_status,
};
use super::raft_rpc::{raft_append, raft_snapshot, raft_transfer, raft_vote};
use super::scale::{auto_scale, frontier_expand, get_hebbian_neighbors};
use super::server::login;
use super::state::SharedState;

struct RateLimiter {
    count: AtomicU64,
    max: u64,
    window_secs: u64,
    last_reset: std::sync::Mutex<std::time::Instant>,
}

impl RateLimiter {
    fn new(max: u64, _burst: u64, window_secs: u64) -> Self {
        Self {
            count: AtomicU64::new(0),
            max,
            window_secs,
            last_reset: std::sync::Mutex::new(std::time::Instant::now()),
        }
    }

    fn check_and_increment(&self) -> bool {
        {
            let mut last = self.last_reset.lock().unwrap();
            if last.elapsed() >= std::time::Duration::from_secs(self.window_secs) {
                self.count.store(0, Ordering::SeqCst);
                *last = std::time::Instant::now();
            }
        }
        let current = self.count.fetch_add(1, Ordering::SeqCst);
        current < self.max
    }
}

async fn rate_limit_middleware(
    Extension(limiter): Extension<Arc<RateLimiter>>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    if !limiter.check_and_increment() {
        return Err(AppError::TooManyRequests);
    }
    Ok(next.run(req).await)
}

async fn security_headers_middleware(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert("x-xss-protection", HeaderValue::from_static("0"));
    headers.insert(
        "referrer-policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        "content-security-policy",
        HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'"),
    );
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    headers.insert("cache-control", HeaderValue::from_static("no-store"));
    headers.insert("pragma", HeaderValue::from_static("no-cache"));
    response
}

async fn metrics_middleware(req: Request, next: Next) -> Response {
    if let Some(c) = metrics::API_REQUESTS_TOTAL.get() {
        c.inc();
    }
    let start = std::time::Instant::now();
    let response = next.run(req).await;
    if let Some(h) = metrics::REQUEST_DURATION.get() {
        h.observe(start.elapsed().as_secs_f64());
    }
    response
}

async fn auth_middleware(
    State(state): State<SharedState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    if !state.config.auth.enabled {
        tracing::warn!("⚠ AUTH IS DISABLED — all requests granted read-only user role");
        tracing::warn!(
            "⚠ Set TETRAMEM_ALLOW_NO_AUTH_ADMIN=1 to restore admin access (NOT for production)"
        );
        let role = if std::env::var("TETRAMEM_ALLOW_NO_AUTH_ADMIN").as_deref() == Ok("1") {
            tracing::warn!(
                "⚠ TETRAMEM_ALLOW_NO_AUTH_ADMIN=1 detected — granting admin (development only)"
            );
            "admin"
        } else {
            "user"
        };
        let claims = Claims::anonymous(role);
        req.extensions_mut().insert(claims);
        return Ok(next.run(req).await);
    }

    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match auth_header {
        Some(token) => {
            let claims = state.jwt.validate_token(token)?;
            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        }
        None => Err(AppError::Unauthorized(
            "missing authorization header".to_string(),
        )),
    }
}

async fn admin_middleware(req: Request, next: Next) -> Result<Response, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| AppError::Unauthorized("no auth claims found".to_string()))?;

    if claims.role() != "admin" {
        return Err(AppError::Forbidden(
            "admin role required for this operation".to_string(),
        ));
    }

    Ok(next.run(req).await)
}

async fn raft_auth_middleware(
    State(state): State<SharedState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let secret = &state.config.auth.raft_secret;

    let provided = req
        .headers()
        .get("x-raft-secret")
        .and_then(|v| v.to_str().ok());

    match provided {
        Some(s)
            if subtle::ConstantTimeEq::ct_eq(s.as_bytes(), secret.as_bytes()).unwrap_u8() == 1 =>
        {
            Ok(next.run(req).await)
        }
        _ => Err(AppError::Unauthorized(
            "invalid or missing raft secret".to_string(),
        )),
    }
}

fn build_cors_layer(origins: &[String]) -> CorsLayer {
    let methods = [
        axum::http::Method::GET,
        axum::http::Method::POST,
        axum::http::Method::OPTIONS,
    ];
    let headers = [
        axum::http::HeaderName::from_static("authorization"),
        axum::http::HeaderName::from_static("content-type"),
        axum::http::HeaderName::from_static("x-raft-secret"),
        axum::http::HeaderName::from_static("x-request-id"),
    ];

    if origins.len() == 1 && (origins[0] == "*" || origins[0] == "any") {
        tracing::warn!("CORS allows all origins — only use in development");
        return CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(methods)
            .allow_headers(headers);
    }

    let parsed: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|o| o.parse::<HeaderValue>().ok())
        .collect();

    if parsed.is_empty() {
        tracing::warn!("no valid CORS origins parsed, denying all cross-origin requests");
        return CorsLayer::new()
            .allow_origin(AllowOrigin::list(["http://localhost:5173"
                .parse::<HeaderValue>()
                .unwrap()]))
            .allow_methods(methods)
            .allow_headers(headers);
    }

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(parsed))
        .allow_methods(methods)
        .allow_headers(headers)
}

pub fn create_router(state: SharedState) -> Router {
    let cors = build_cors_layer(&state.config.server.cors_origins);

    let x_request_id = axum::http::HeaderName::from_static("x-request-id");

    let rpm = state.config.rate_limit.requests_per_minute;
    let burst = state.config.rate_limit.burst;
    let limiter = Arc::new(RateLimiter::new(rpm, burst, 60));

    let public_routes = Router::new()
        .route("/health", get(get_health))
        .route("/login", post(login))
        .layer(middleware::from_fn(rate_limit_middleware));

    let raft_routes = Router::new()
        .route("/raft/vote", post(raft_vote))
        .route("/raft/append", post(raft_append))
        .route("/raft/snapshot", post(raft_snapshot))
        .route("/raft/transfer", post(raft_transfer))
        .layer(middleware::from_fn(rate_limit_middleware))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            raft_auth_middleware,
        ));

    let user_routes = Router::new()
        .route("/stats", get(get_stats))
        .route("/metrics", get(get_metrics))
        .route("/openapi.json", get(get_openapi))
        .route("/memory/encode", post(encode_memory))
        .route("/memory/decode", post(decode_memory))
        .route("/memory/list", get(list_memories))
        .route("/pulse", post(fire_pulse))
        .route("/dream", post(run_dream))
        .route("/hebbian/neighbors/:x/:y/:z", get(get_hebbian_neighbors))
        .route("/dark/query", get(dark_query))
        .route("/dark/flow", post(dark_flow))
        .route("/dark/transfer", post(dark_transfer))
        .route("/dark/materialize", post(dark_materialize))
        .route("/dark/dematerialize", post(dark_dematerialize))
        .route("/dark/pressure", get(dark_dimension_pressure))
        .route("/physics/status", get(physics_status))
        .route("/physics/profile", get(physics_profile))
        .route("/physics/distance", post(physics_distance))
        .route("/physics/project", post(physics_project))
        .route("/memory/timeline", get(memory_timeline))
        .route("/memory/trace", post(memory_trace))
        .route("/phase/detect", get(detect_phase_transition))
        .route("/cluster/status", get(cluster_status))
        .route("/emotion/pulse", post(emotion_pulse))
        .route("/emotion/dream", post(emotion_dream))
        .route("/emotion/status", get(emotion_status))
        .layer(middleware::from_fn(rate_limit_middleware))
        .layer(middleware::from_fn(metrics_middleware))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let admin_routes = Router::new()
        .route("/scale", post(auto_scale))
        .route("/scale/frontier/:max_new", post(frontier_expand))
        .route("/regulate", post(regulate))
        .route("/backup/create", post(create_backup))
        .route("/backup/list", get(list_backups))
        .route("/cluster/init", post(cluster_init))
        .route("/cluster/propose", post(cluster_propose))
        .route("/cluster/add-node", post(cluster_add_node))
        .route("/cluster/remove-node", post(cluster_remove_node))
        .route("/phase/consensus", post(phase_consensus))
        .route("/phase/quorum/start", post(quorum_start))
        .route("/phase/quorum/confirm", post(quorum_confirm))
        .route("/phase/quorum/status", get(quorum_status_endpoint))
        .route("/phase/quorum/execute", post(quorum_execute))
        .route("/physics/configure", post(physics_configure))
        .route("/emotion/crystallize", post(emotion_crystallize))
        .layer(middleware::from_fn(rate_limit_middleware))
        .layer(middleware::from_fn(admin_middleware))
        .layer(middleware::from_fn(metrics_middleware))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    Router::new()
        .merge(public_routes)
        .merge(raft_routes)
        .merge(user_routes)
        .merge(admin_routes)
        .layer(Extension(limiter))
        .layer(middleware::from_fn(security_headers_middleware))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limiter_blocks_after_max() {
        let limiter = RateLimiter::new(5, 10, 60);
        for _ in 0..5 {
            assert!(limiter.check_and_increment());
        }
        assert!(!limiter.check_and_increment(), "should block after max");
        assert!(!limiter.check_and_increment(), "should stay blocked");
    }

    #[test]
    fn rate_limiter_resets_after_window() {
        let limiter = RateLimiter::new(2, 10, 60);
        assert!(limiter.check_and_increment());
        assert!(limiter.check_and_increment());
        assert!(!limiter.check_and_increment());
        {
            let mut last = limiter.last_reset.lock().unwrap();
            *last = std::time::Instant::now() - std::time::Duration::from_secs(61);
        }
        assert!(
            limiter.check_and_increment(),
            "should reset after window expiry"
        );
    }
}

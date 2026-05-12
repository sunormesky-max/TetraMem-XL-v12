// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use axum::http::StatusCode;
use axum::{extract::ConnectInfo, extract::State, Json};

use crate::universe::auth::{Claims, LoginRequest, LoginResponse};
use crate::universe::error::AppError;

use super::router::create_router;
use super::state::SharedState;
use super::types::ApiResponse;

struct LoginAttempt {
    count: u32,
    first_attempt: Instant,
}

static LOGIN_RATE_LIMIT: std::sync::OnceLock<Mutex<HashMap<String, LoginAttempt>>> =
    std::sync::OnceLock::new();

const MAX_LOGIN_ATTEMPTS: u32 = 10;
const LOGIN_WINDOW_SECS: u64 = 300;
const MAX_LOGIN_ENTRIES: usize = 10000;

fn check_login_rate(ip_key: &str) -> Result<(), AppError> {
    let map = LOGIN_RATE_LIMIT.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = map
        .lock()
        .map_err(|e| AppError::Internal(format!("login rate limit lock: {}", e)))?;
    let now = Instant::now();
    if map.len() > MAX_LOGIN_ENTRIES {
        map.retain(|_, v| now.duration_since(v.first_attempt).as_secs() < LOGIN_WINDOW_SECS);
    }
    if let Some(attempt) = map.get_mut(ip_key) {
        if attempt.first_attempt.elapsed().as_secs() > LOGIN_WINDOW_SECS {
            attempt.count = 0;
            attempt.first_attempt = now;
        }
        if attempt.count >= MAX_LOGIN_ATTEMPTS {
            return Err(AppError::TooManyRequests);
        }
        attempt.count += 1;
    } else {
        map.insert(
            ip_key.to_string(),
            LoginAttempt {
                count: 1,
                first_attempt: now,
            },
        );
    }
    Ok(())
}

pub async fn login(
    addr: Option<ConnectInfo<std::net::SocketAddr>>,
    State(state): State<SharedState>,
    Json(req): Json<LoginRequest>,
) -> Result<(StatusCode, Json<ApiResponse<LoginResponse>>), AppError> {
    if let Some(ConnectInfo(addr)) = addr {
        check_login_rate(&addr.ip().to_string())?;
    } else {
        check_login_rate("unknown")?;
    }

    if req.username.is_empty() || req.password.is_empty() {
        return Err(AppError::Unauthorized(
            "invalid username or password".to_string(),
        ));
    }
    if req.password.len() < 8 || req.password.len() > 128 {
        return Err(AppError::Unauthorized(
            "invalid username or password".to_string(),
        ));
    }

    tracing::info!(ip = ?addr.map(|ConnectInfo(a)| a.ip()), "login attempt");

    let role = state
        .users
        .verify(&req.username, &req.password)
        .ok_or_else(|| {
            tracing::warn!("login failed: invalid credentials");
            AppError::Unauthorized("invalid username or password".to_string())
        })?
        .to_string();

    let token = state.jwt.create_token(&req.username, &role)?;
    let expires_in = state.config.auth.jwt_expiry_secs;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok(LoginResponse { token, expires_in })),
    ))
}

pub async fn logout(
    claims: axum::Extension<Claims>,
    State(state): State<SharedState>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>), AppError> {
    let jti = claims.jti().to_string();
    state.token_blocklist.write().await.revoke(&jti);
    tracing::info!(jti = %jti, "token revoked via logout");
    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok(serde_json::json!({ "revoked": true }))),
    ))
}

pub async fn start_server(
    state: SharedState,
    addr: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = create_router(state.clone());
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("API server listening on http://{}", addr);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    state
        .shutdown
        .store(true, std::sync::atomic::Ordering::Relaxed);

    if state.config.backup.auto_persist {
        let persist_path = std::path::PathBuf::from(&state.config.backup.persist_path);
        let u = state.universe.read().await;
        let h = state.hebbian.read().await;
        let store = state.memory_store.read().await;
        let c = state.crystal.read().await;
        if state.config.backup.persist_backend == "sqlite" {
            let sqlite_path = persist_path.with_extension("db");
            match crate::universe::persist_sqlite::PersistSqlite::save(
                &sqlite_path,
                &u,
                &h,
                &store.memories,
                &c,
            ) {
                Ok(rows) => tracing::info!("final SQLite persist on shutdown: {} rows", rows),
                Err(e) => tracing::warn!("final SQLite persist failed: {}", e),
            }
        } else {
            match crate::universe::persist_file::PersistFile::save(
                &persist_path,
                &u,
                &h,
                &store.memories,
                &c,
            ) {
                Ok(info) => tracing::info!("final persist on shutdown: {}", info),
                Err(e) => tracing::warn!("final persist failed: {}", e),
            }
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
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(e) => {
                tracing::error!("signal handler error: {}", e);
                std::future::pending::<()>().await;
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_rate_limit_blocks_after_max_attempts() {
        let ip = "192.168.1.1";
        for _ in 0..MAX_LOGIN_ATTEMPTS {
            check_login_rate(ip).unwrap();
        }
        assert!(
            check_login_rate(ip).is_err(),
            "should block after max attempts"
        );
    }

    #[test]
    fn login_rate_limit_different_ips_independent() {
        check_login_rate("10.0.0.1").unwrap();
        for _ in 0..MAX_LOGIN_ATTEMPTS {
            check_login_rate("10.0.0.2").unwrap();
        }
        assert!(check_login_rate("10.0.0.2").is_err());
        assert!(
            check_login_rate("10.0.0.1").is_ok(),
            "different IP should be independent"
        );
    }

    #[test]
    fn login_rate_limit_respects_max_entries() {
        for i in 0..(MAX_LOGIN_ENTRIES + 100) {
            let _ = check_login_rate(&format!("172.16.0.{}", i % 256));
        }
        assert!(
            check_login_rate("172.16.1.1").is_ok(),
            "should still work after many entries"
        );
    }
}

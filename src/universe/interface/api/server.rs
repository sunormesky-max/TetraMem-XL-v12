use axum::http::StatusCode;
use axum::{extract::State, Json};

use crate::universe::auth::{LoginRequest, LoginResponse};
use crate::universe::error::AppError;

use super::router::create_router;
use super::state::SharedState;
use super::types::ApiResponse;

pub async fn login(
    State(state): State<SharedState>,
    Json(req): Json<LoginRequest>,
) -> Result<(StatusCode, Json<ApiResponse<LoginResponse>>), AppError> {
    if req.username.is_empty() || req.password.is_empty() {
        return Err(AppError::BadRequest(
            "username and password required".to_string(),
        ));
    }
    if req.password.len() < 8 {
        return Err(AppError::BadRequest(
            "password must be at least 8 characters".to_string(),
        ));
    }
    if req.password.len() > 128 {
        return Err(AppError::BadRequest(
            "password must be at most 128 characters".to_string(),
        ));
    }

    tracing::info!(username = %req.username, "user login attempt");

    let role = state
        .users
        .verify(&req.username, &req.password)
        .ok_or_else(|| {
            tracing::warn!(username = %req.username, "login failed: invalid credentials");
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

pub async fn start_server(
    state: SharedState,
    addr: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = create_router(state.clone());
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("API server listening on http://{}", addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    if state.config.backup.auto_persist {
        let persist_path = std::path::PathBuf::from(&state.config.backup.persist_path);
        let u = state.universe.read().await;
        let h = state.hebbian.read().await;
        let mems = state.memories.read().await;
        let c = state.crystal.read().await;
        match crate::universe::persist_file::PersistFile::save(&persist_path, &u, &h, &mems, &c)
        {
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

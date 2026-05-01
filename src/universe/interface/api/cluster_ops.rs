use axum::http::StatusCode;
use axum::{extract::State, Json};

use crate::universe::cluster::{
    AddNodeRequest as ClusterAddNodeRequest, ClusterManager, ClusterStatus, ProposeRequest,
    ProposeResponse as ClusterProposeResponse, RemoveNodeRequest as ClusterRemoveNodeRequest,
};
use crate::universe::error::AppError;

use super::state::SharedState;
use super::types::*;

fn validate_cluster_addr(addr: &str) -> Result<(), AppError> {
    let host = addr.split(':').next().unwrap_or(addr);
    if host.is_empty() {
        return Err(AppError::BadRequest("empty cluster address".to_string()));
    }
    if host == "0.0.0.0" {
        return Err(AppError::BadRequest(
            "0.0.0.0 is not a valid cluster address".to_string(),
        ));
    }
    if host.starts_with("127.") || host == "localhost" {
        return Err(AppError::BadRequest(
            "loopback addresses not allowed for cluster nodes".to_string(),
        ));
    }
    if let Some(rest) = host.strip_prefix("10.") {
        if rest.parse::<u8>().is_ok() || rest.split('.').count() >= 1 {
            return Err(AppError::BadRequest(
                "private network addresses not allowed".to_string(),
            ));
        }
    }
    if host.starts_with("192.168.") || host.starts_with("172.16.")
        || host.starts_with("172.17.") || host.starts_with("172.18.")
        || host.starts_with("172.19.") || host.starts_with("172.2")
        || host.starts_with("172.3")
    {
        return Err(AppError::BadRequest(
            "private network addresses not allowed".to_string(),
        ));
    }
    Ok(())
}

pub async fn cluster_status(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<ClusterStatus>>, AppError> {
    let cm = state.cluster.lock().await;
    let status = cm.status().await;
    Ok(Json(ApiResponse::ok(status)))
}

pub async fn cluster_init(
    State(state): State<SharedState>,
    Json(req): Json<ClusterInitRequest>,
) -> Result<(StatusCode, Json<ApiResponse<ClusterStatus>>), AppError> {
    let mut cm = state.cluster.lock().await;
    if let Some(node_id) = req.node_id {
        let addr = req.addr.unwrap_or_else(|| state.config.server.addr.clone());
        validate_cluster_addr(&addr)?;
        *cm = ClusterManager::new(node_id, addr);
    }
    cm.init_single_node().await.map_err(AppError::Internal)?;
    let status = cm.status().await;
    Ok((StatusCode::OK, Json(ApiResponse::ok(status))))
}

pub async fn cluster_propose(
    State(state): State<SharedState>,
    Json(req): Json<ProposeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<ClusterProposeResponse>>), AppError> {
    let cm = state.cluster.lock().await;
    let resp = cm.propose(req).await.map_err(AppError::Internal)?;
    Ok((StatusCode::OK, Json(ApiResponse::ok(resp))))
}

pub async fn cluster_add_node(
    State(state): State<SharedState>,
    Json(req): Json<ClusterAddNodeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), AppError> {
    validate_cluster_addr(&req.addr)?;
    let mut cm = state.cluster.lock().await;
    cm.add_peer(req.node_id, req.addr)
        .await
        .map_err(AppError::Internal)?;
    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok("node added".to_string())),
    ))
}

pub async fn cluster_remove_node(
    State(state): State<SharedState>,
    Json(req): Json<ClusterRemoveNodeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), AppError> {
    let mut cm = state.cluster.lock().await;
    cm.remove_peer(req.node_id)
        .await
        .map_err(AppError::Internal)?;
    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok("node removed".to_string())),
    ))
}

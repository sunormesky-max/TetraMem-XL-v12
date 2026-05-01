// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

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
    let parsed: SocketAddr = addr.parse().map_err(|_| {
        AppError::BadRequest(format!("invalid cluster address format: {}", addr))
    })?;
    let ip = parsed.ip();
    if ip.is_loopback() || ip.is_multicast() {
        return Err(AppError::BadRequest(
            "loopback and multicast addresses not allowed for cluster nodes".to_string(),
        ));
    }
    if let IpAddr::V4(v4) = ip {
        let octets = v4.octets();
        if octets[0] == 0 || octets[0] >= 224 || v4 == Ipv4Addr::new(0, 0, 0, 0) {
            return Err(AppError::BadRequest(
                "unusable/reserved network addresses not allowed for cluster nodes".to_string(),
            ));
        }
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

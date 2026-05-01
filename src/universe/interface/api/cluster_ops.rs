use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use axum::http::StatusCode;
use axum::{extract::State, Json};

use crate::universe::cluster::{
    AddNodeRequest as ClusterAddNodeRequest, ClusterManager, ClusterStatus, ProposeRequest,
    ProposeResponse as ClusterProposeResponse, RemoveNodeRequest as ClusterRemoveNodeRequest,
};
use crate::universe::error::AppError;

use super::state::SharedState;
use super::types::*;

fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            octets[0] == 0
                || octets[0] == 127
                || octets[0] == 10
                || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                || (octets[0] == 169 && octets[1] == 254)
                || (octets[0] == 192 && octets[1] == 168)
                || (octets[0] == 100 && (64..=127).contains(&octets[1]))
                || (octets[0] == 198 && (18..=19).contains(&octets[1]))
                || octets[0] >= 224
                || v4 == &Ipv4Addr::new(0, 0, 0, 0)
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_multicast()
                || is_ipv6_unique_local(v6)
                || is_ipv6_link_local(v6)
                || is_ipv6_ipv4_mapped(v6)
        }
    }
}

fn is_ipv6_unique_local(v6: &Ipv6Addr) -> bool {
    let segments = v6.segments();
    (segments[0] & 0xfe00) == 0xfc00
}

fn is_ipv6_link_local(v6: &Ipv6Addr) -> bool {
    let segments = v6.segments();
    (segments[0] & 0xffc0) == 0xfe80
}

fn is_ipv6_ipv4_mapped(v6: &Ipv6Addr) -> bool {
    let octets = v6.octets();
    octets[0..12] == [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff]
}

fn validate_cluster_addr(addr: &str) -> Result<(), AppError> {
    let parsed: SocketAddr = addr.parse().map_err(|_| {
        AppError::BadRequest(format!("invalid cluster address format: {}", addr))
    })?;
    if is_private_ip(&parsed.ip()) {
        return Err(AppError::BadRequest(
            "private/reserved network addresses not allowed for cluster nodes".to_string(),
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
    let _write_guard = state.write_guard.lock().await;
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
    let _write_guard = state.write_guard.lock().await;
    let cm = state.cluster.lock().await;
    let resp = cm.propose(req).await.map_err(AppError::Internal)?;
    Ok((StatusCode::OK, Json(ApiResponse::ok(resp))))
}

pub async fn cluster_add_node(
    State(state): State<SharedState>,
    Json(req): Json<ClusterAddNodeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), AppError> {
    validate_cluster_addr(&req.addr)?;
    let _write_guard = state.write_guard.lock().await;
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
    let _write_guard = state.write_guard.lock().await;
    let mut cm = state.cluster.lock().await;
    cm.remove_peer(req.node_id)
        .await
        .map_err(AppError::Internal)?;
    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok("node removed".to_string())),
    ))
}

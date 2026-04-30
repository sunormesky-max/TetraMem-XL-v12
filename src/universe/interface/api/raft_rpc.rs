use axum::extract::State;
use axum::Json;

use crate::universe::consensus::network::SnapshotTransport;
use crate::universe::consensus::raft_node::TypeName;
use crate::universe::error::AppError;

use openraft::raft::AppendEntriesRequest;
use openraft::raft::AppendEntriesResponse;
use openraft::raft::SnapshotResponse;
use openraft::raft::TransferLeaderRequest;
use openraft::raft::VoteRequest;
use openraft::raft::VoteResponse;

use super::state::SharedState;

pub async fn raft_vote(
    State(state): State<SharedState>,
    Json(req): Json<VoteRequest<TypeName>>,
) -> Result<Json<VoteResponse<TypeName>>, AppError> {
    let cm = state.cluster.lock().await;
    let raft = cm
        .raft_clone()
        .ok_or_else(|| AppError::Internal("raft not initialized".to_string()))?;
    drop(cm);
    let resp = raft
        .vote(req)
        .await
        .map_err(|e| AppError::Internal(format!("vote error: {}", e)))?;
    Ok(Json(resp))
}

pub async fn raft_append(
    State(state): State<SharedState>,
    Json(req): Json<AppendEntriesRequest<TypeName>>,
) -> Result<Json<AppendEntriesResponse<TypeName>>, AppError> {
    let cm = state.cluster.lock().await;
    let raft = cm
        .raft_clone()
        .ok_or_else(|| AppError::Internal("raft not initialized".to_string()))?;
    drop(cm);
    let resp = raft
        .append_entries(req)
        .await
        .map_err(|e| AppError::Internal(format!("append_entries error: {}", e)))?;
    Ok(Json(resp))
}

pub async fn raft_snapshot(
    State(state): State<SharedState>,
    Json(transport): Json<SnapshotTransport>,
) -> Result<Json<SnapshotResponse<TypeName>>, AppError> {
    let cm = state.cluster.lock().await;
    let raft = cm
        .raft_clone()
        .ok_or_else(|| AppError::Internal("raft not initialized".to_string()))?;
    drop(cm);
    let (vote, snapshot) = transport.into_parts();
    let resp = raft
        .install_full_snapshot(vote, snapshot)
        .await
        .map_err(|e| AppError::Internal(format!("install_snapshot error: {}", e)))?;
    Ok(Json(resp))
}

pub async fn raft_transfer(
    State(state): State<SharedState>,
    Json(req): Json<TransferLeaderRequest<TypeName>>,
) -> Result<Json<()>, AppError> {
    let cm = state.cluster.lock().await;
    let raft = cm
        .raft_clone()
        .ok_or_else(|| AppError::Internal("raft not initialized".to_string()))?;
    drop(cm);
    raft.handle_transfer_leader(req)
        .await
        .map_err(|e| AppError::Internal(format!("transfer_leader error: {}", e)))?;
    Ok(Json(()))
}

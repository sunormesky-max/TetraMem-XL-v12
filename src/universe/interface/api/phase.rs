use axum::{extract::State, Json};

use crate::universe::cluster::{EnergyQuorumEntry, H6PhaseTransitionProposal, QuorumStatus};
use crate::universe::error::AppError;

use super::state::SharedState;
use super::types::*;

pub async fn detect_phase_transition(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<crate::universe::crystal::PhaseTransitionReport>>, AppError> {
    let u = state.universe.read().await;
    let h = state.hebbian.read().await;
    let c = state.crystal.read().await;

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

pub async fn phase_consensus(
    State(state): State<SharedState>,
    Json(req): Json<PhaseConsensusRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let _write_guard = state.write_guard.lock().await;
    let u = state.universe.read().await;
    let h = state.hebbian.read().await;
    let crystal = state.crystal.read().await;
    let cm = state.cluster.lock().await;

    let report = crystal.detect_phase_transition(&h, &u);

    if !report.requires_consensus && !req.force {
        return Ok(Json(ApiResponse::ok(serde_json::json!({
            "status": "no_transition",
            "phase_coherent": report.phase_coherent,
        }))));
    }

    if !cm.is_initialized() {
        tracing::warn!("phase consensus requested but cluster not initialized, proceeding locally");
        drop(crystal);
        drop(cm);
        drop(u);
        drop(h);
        let mut crystal_w = state.crystal.write().await;
        let u_r = state.universe.read().await;
        let h_r = state.hebbian.read().await;
        let crystal_report = crystal_w.crystallize(&h_r, &u_r);
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
            drop(crystal);
            drop(u);
            drop(h);
            let mut crystal_w = state.crystal.write().await;
            let u_r = state.universe.read().await;
            let h_r = state.hebbian.read().await;
            let crystal_report = crystal_w.crystallize(&h_r, &u_r);
            Ok(Json(ApiResponse::ok(serde_json::json!({
                "status": "consensus_committed",
                "log_index": resp.log_index,
                "conservation_verified": resp.conservation_verified,
                "new_crystals": crystal_report.new_crystals,
                "new_super_crystals": crystal_report.new_super_crystals,
            }))))
        }
        Err(e) => {
            drop(crystal);
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

pub async fn quorum_start(
    State(state): State<SharedState>,
    Json(req): Json<QuorumStartRequest>,
) -> Result<Json<ApiResponse<QuorumStatus>>, AppError> {
    let budget = req.required_energy_budget.unwrap_or(100.0);
    if let Some(b) = req.required_energy_budget {
        if b <= 0.0 || !b.is_finite() {
            return Err(AppError::BadRequest(
                "required_energy_budget must be positive and finite".to_string(),
            ));
        }
    }

    let _write_guard = state.write_guard.lock().await;
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

pub async fn quorum_confirm(
    State(state): State<SharedState>,
    Json(entry): Json<EnergyQuorumEntry>,
) -> Result<Json<ApiResponse<QuorumStatus>>, AppError> {
    if entry.available_energy < 0.0 || !entry.available_energy.is_finite() {
        return Err(AppError::BadRequest(
            "available_energy must be non-negative and finite".to_string(),
        ));
    }
    let _write_guard = state.write_guard.lock().await;
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

pub async fn quorum_status_endpoint(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<Option<QuorumStatus>>>, AppError> {
    let cm = state.cluster.lock().await;
    Ok(Json(ApiResponse::ok(cm.get_quorum_status())))
}

pub async fn quorum_execute(
    State(state): State<SharedState>,
    Json(req): Json<QuorumExecuteRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let _write_guard = state.write_guard.lock().await;
    let u = state.universe.read().await;
    let h = state.hebbian.read().await;
    let crystal = state.crystal.read().await;
    let mut cm = state.cluster.lock().await;

    let report = crystal.detect_phase_transition(&h, &u);

    if !req.force && !report.requires_consensus {
        drop(u);
        drop(h);
        drop(crystal);
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
            drop(crystal);
            drop(u);
            drop(h);
            let mut crystal_w = state.crystal.write().await;
            let u_r = state.universe.read().await;
            let h_r = state.hebbian.read().await;
            let crystal_report = crystal_w.crystallize(&h_r, &u_r);
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
            drop(crystal);
            drop(cm);
            tracing::warn!("quorum execute failed: {}", e);
            Ok(Json(ApiResponse::ok(serde_json::json!({
                "status": "quorum_not_reached",
                "reason": e,
            }))))
        }
    }
}

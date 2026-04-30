use axum::{
    extract::{Path, State},
    Json,
};

use crate::universe::autoscale::AutoScaler;
use crate::universe::coord::Coord7D;

use super::state::SharedState;
use super::types::*;

pub async fn auto_scale(State(state): State<SharedState>) -> Json<ApiResponse<ScaleResponse>> {
    let mut u = state.universe.write().await;
    let h = state.hebbian.read().await;
    let mems = state.memories.read().await;

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

pub async fn frontier_expand(
    State(state): State<SharedState>,
    Path(max_new): Path<usize>,
) -> Json<ApiResponse<ScaleResponse>> {
    let mut u = state.universe.write().await;

    let scaler = AutoScaler::new();
    let report = scaler.frontier_expansion(&mut u, max_new);

    Json(ApiResponse::ok(ScaleResponse {
        energy_expanded_by: report.energy_expanded_by,
        nodes_added: report.nodes_added,
        nodes_removed: report.nodes_removed,
        reason: format!("{:?}", report.reason),
    }))
}

pub async fn get_hebbian_neighbors(
    State(state): State<SharedState>,
    Path((x, y, z)): Path<(i32, i32, i32)>,
) -> Json<ApiResponse<HebbianNeighborsResponse>> {
    let h = state.hebbian.read().await;
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

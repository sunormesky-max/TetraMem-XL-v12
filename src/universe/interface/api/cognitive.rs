use axum::{extract::State, Json};

use crate::universe::dream::DreamEngine;
use crate::universe::metrics;
use crate::universe::observer::{SelfRegulator, UniverseObserver};
use crate::universe::pulse::{PulseEngine, PulseType};

use super::state::SharedState;
use super::types::*;

pub async fn fire_pulse(
    State(state): State<SharedState>,
    Json(req): Json<PulseRequest>,
) -> Json<ApiResponse<PulseResponse>> {
    let u = state.universe.read().await;
    let mut h = state.hebbian.write().await;

    metrics::API_PULSE_TOTAL.inc();
    let source = crate::universe::coord::Coord7D::new_even([
        req.source[0],
        req.source[1],
        req.source[2],
        0,
        0,
        0,
        0,
    ]);
    let pt = match req.pulse_type.to_lowercase().as_str() {
        "reinforcing" => PulseType::Reinforcing,
        "cascade" => PulseType::Cascade,
        _ => PulseType::Exploratory,
    };

    tracing::info!(source = %source, pulse_type = ?pt, "firing pulse");
    let engine = PulseEngine::new();
    let result = engine.propagate(&source, pt, &u, &mut h);

    Json(ApiResponse::ok(PulseResponse {
        visited_nodes: result.visited_nodes,
        total_activation: result.total_activation,
        paths_recorded: result.paths_recorded,
        final_strength: result.final_strength,
    }))
}

pub async fn run_dream(State(state): State<SharedState>) -> Json<ApiResponse<DreamResponse>> {
    let u = state.universe.read().await;
    let mut h = state.hebbian.write().await;
    let mems = state.memories.read().await;

    metrics::API_DREAM_TOTAL.inc();
    tracing::info!("running dream cycle");

    let dream = DreamEngine::new();
    let report = dream.dream(&u, &mut h, &mems);

    tracing::info!(
        replayed = report.paths_replayed,
        weakened = report.paths_weakened,
        consolidated = report.memories_consolidated,
        "dream cycle complete"
    );

    Json(ApiResponse::ok(DreamResponse {
        paths_replayed: report.paths_replayed,
        paths_weakened: report.paths_weakened,
        memories_consolidated: report.memories_consolidated,
        edges_before: report.hebbian_edges_before,
        edges_after: report.hebbian_edges_after,
        weight_before: report.weight_before,
        weight_after: report.weight_after,
    }))
}

pub async fn regulate(State(state): State<SharedState>) -> Json<ApiResponse<Vec<String>>> {
    let u = state.universe.read().await;
    let mut h = state.hebbian.write().await;
    let mems = state.memories.read().await;

    let report = UniverseObserver::inspect(&u, &h, &mems);
    let regulator = SelfRegulator::new();
    let actions = regulator.regulate(&report, &mut h);

    tracing::info!(actions = actions.len(), "regulation cycle complete");
    let descriptions: Vec<String> = actions.iter().map(|a| a.description.clone()).collect();
    Json(ApiResponse::ok(descriptions))
}

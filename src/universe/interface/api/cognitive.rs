// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use axum::{extract::State, Json};

use crate::universe::dream::DreamEngine;
use crate::universe::error::AppError;
use crate::universe::metrics;
use crate::universe::observer::{SelfRegulator, UniverseObserver};
use crate::universe::pulse::{PulseEngine, PulseType};

use super::state::SharedState;
use super::types::*;

fn validate_coord_3(c: &[i32; 3]) -> Result<(), AppError> {
    for &v in c {
        if !(-10000..=10000).contains(&v) {
            return Err(AppError::BadRequest(format!(
                "coordinate value {} out of range [-10000, 10000]",
                v
            )));
        }
    }
    Ok(())
}

pub async fn fire_pulse(
    State(state): State<SharedState>,
    Json(req): Json<PulseRequest>,
) -> Result<Json<ApiResponse<PulseResponse>>, AppError> {
    validate_coord_3(&req.source)?;
    let u = state.universe.read().await;
    let mut h = state.hebbian.write().await;

    if let Some(c) = metrics::API_PULSE_TOTAL.get() {
        c.inc();
    }
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

    Ok(Json(ApiResponse::ok(PulseResponse {
        visited_nodes: result.visited_nodes,
        total_activation: result.total_activation,
        paths_recorded: result.paths_recorded,
        final_strength: result.final_strength,
    })))
}

pub async fn run_dream(State(state): State<SharedState>) -> Json<ApiResponse<DreamResponse>> {
    let _write_guard = state.write_guard.lock().await;
    let u = state.universe.read().await;
    let mut h = state.hebbian.write().await;
    let mems = state.memories.read().await;

    if let Some(c) = metrics::API_DREAM_TOTAL.get() {
        c.inc();
    }
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
    let _write_guard = state.write_guard.lock().await;
    let u = state.universe.read().await;
    let mems = state.memories.read().await;
    let mut h = state.hebbian.write().await;

    let report = UniverseObserver::inspect(&u, &h, &mems);
    let regulator = SelfRegulator::new();
    let actions = regulator.regulate(&report, &mut h);

    tracing::info!(actions = actions.len(), "regulation cycle complete");
    let descriptions: Vec<String> = actions.iter().map(|a| a.description.clone()).collect();
    Json(ApiResponse::ok(descriptions))
}

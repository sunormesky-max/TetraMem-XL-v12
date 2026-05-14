// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::universe::core::physics::UniversePhysics;
use crate::universe::error::AppError;

use super::state::SharedState;
use super::types::*;

#[derive(Serialize)]
pub struct PhysicsStatusResponse {
    pub total_nodes: usize,
    pub manifested_nodes: usize,
    pub dark_nodes: usize,
    pub total_energy: f64,
    pub physics_engine: String,
    pub active: bool,
    pub mode: String,
    pub metric_diagonal: [f64; 7],
    pub coupling_nonzero: bool,
    pub phase_threshold: f64,
    pub phase_mode: String,
    pub projection_has_dark_mixing: bool,
}

#[derive(Serialize)]
pub struct PhysicsProfileResponse {
    pub energy_distribution: EnergyDistribution,
    pub conservation_ok: bool,
    pub dimension_weights: [f64; 7],
    pub propagation_decays: [f64; 7],
    pub coupling_strengths: [f64; 7],
}

#[derive(Serialize)]
pub struct EnergyDistribution {
    pub physical: f64,
    pub dark: f64,
}

fn describe_physics(phys: &UniversePhysics) -> (String, bool) {
    let is_rich = phys.coupling.get(0, 3) > 0.0;
    let mode = if is_rich { "rich" } else { "flat" };
    (mode.to_string(), is_rich)
}

pub async fn physics_status(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<PhysicsStatusResponse>>, AppError> {
    let u = state.universe.read().await;
    let stats = u.stats();

    match u.physics() {
        Some(phys) => {
            let (mode, _is_rich) = describe_physics(phys);
            let mut diag = [0.0; 7];
            for (i, d) in diag.iter_mut().enumerate() {
                *d = phys.metric.get(i, i);
            }
            let coupling_nonzero = phys.coupling.get(0, 3) > 0.0;
            let phase_mode = if phys.phase.fluctuation_amplitude > 0.0 {
                "thermal"
            } else if phys.phase.sharpness == f64::INFINITY {
                "hard"
            } else {
                "sigmoid"
            };
            let projection_has_dark_mixing = phys.projection.get(0, 3) != 0.0;

            Ok(Json(ApiResponse::ok(PhysicsStatusResponse {
                total_nodes: stats.active_nodes,
                manifested_nodes: stats.manifested_nodes,
                dark_nodes: stats.dark_nodes,
                total_energy: stats.total_energy,
                physics_engine: mode.clone(),
                active: true,
                mode,
                metric_diagonal: diag,
                coupling_nonzero,
                phase_threshold: phys.phase.threshold,
                phase_mode: phase_mode.to_string(),
                projection_has_dark_mixing,
            })))
        }
        None => Ok(Json(ApiResponse::ok(PhysicsStatusResponse {
            total_nodes: stats.active_nodes,
            manifested_nodes: stats.manifested_nodes,
            dark_nodes: stats.dark_nodes,
            total_energy: stats.total_energy,
            physics_engine: "none".to_string(),
            active: false,
            mode: "none".to_string(),
            metric_diagonal: [1.0; 7],
            coupling_nonzero: false,
            phase_threshold: 0.5,
            phase_mode: "hard".to_string(),
            projection_has_dark_mixing: false,
        }))),
    }
}

#[derive(Deserialize)]
pub struct PhysicsConfigureRequest {
    pub mode: String,
}

#[derive(Serialize)]
pub struct PhysicsConfigureResponse {
    pub success: bool,
    pub mode: String,
}

pub async fn physics_configure(
    State(state): State<SharedState>,
    Json(req): Json<PhysicsConfigureRequest>,
) -> Result<Json<ApiResponse<PhysicsConfigureResponse>>, AppError> {
    let physics = match req.mode.as_str() {
        "flat" => UniversePhysics::flat(),
        "rich" => UniversePhysics::rich(),
        other => {
            return Err(AppError::BadRequest(format!(
                "unknown physics mode: '{}'. Available: flat, rich",
                other
            )))
        }
    };

    let mut u = state.universe.write().await;
    u.set_physics(physics);

    Ok(Json(ApiResponse::ok(PhysicsConfigureResponse {
        success: true,
        mode: req.mode,
    })))
}

pub async fn physics_profile(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<PhysicsProfileResponse>>, AppError> {
    let u = state.universe.read().await;
    let stats = u.stats();
    let conservation_ok = u.energy_drift() < 1e-6;

    match u.physics() {
        Some(phys) => {
            let weights = phys.profile.metric_weights();
            let decays = phys.profile.propagation_decays();
            let mut coupling = [0.0; 7];
            for (i, c) in coupling.iter_mut().enumerate() {
                *c = phys.profile.get(i).coupling_strength;
            }

            Ok(Json(ApiResponse::ok(PhysicsProfileResponse {
                energy_distribution: EnergyDistribution {
                    physical: stats.physical_energy,
                    dark: stats.dark_energy,
                },
                conservation_ok,
                dimension_weights: weights,
                propagation_decays: decays,
                coupling_strengths: coupling,
            })))
        }
        None => Ok(Json(ApiResponse::ok(PhysicsProfileResponse {
            energy_distribution: EnergyDistribution {
                physical: stats.physical_energy,
                dark: stats.dark_energy,
            },
            conservation_ok,
            dimension_weights: [1.0; 7],
            propagation_decays: [1.0; 7],
            coupling_strengths: [0.0; 7],
        }))),
    }
}

#[derive(Deserialize)]
pub struct PhysicsDistanceRequest {
    pub from: Vec<f64>,
    pub to: Vec<f64>,
}

#[derive(Serialize)]
pub struct PhysicsDistanceResponse {
    pub distance_sq: f64,
    pub distance_7d: f64,
    pub distance_3d: f64,
    pub dark_contribution: f64,
    pub uses_metric: bool,
}

fn pad7(v: &[f64]) -> [f64; 7] {
    let mut arr = [0.0; 7];
    for (i, &val) in v.iter().enumerate().take(7) {
        arr[i] = val;
    }
    arr
}

pub async fn physics_distance(
    State(state): State<SharedState>,
    Json(req): Json<PhysicsDistanceRequest>,
) -> Result<Json<ApiResponse<PhysicsDistanceResponse>>, AppError> {
    let u = state.universe.read().await;
    let from = pad7(&req.from);
    let to = pad7(&req.to);

    let d3_sq = (from[0] - to[0]).powi(2) + (from[1] - to[1]).powi(2) + (from[2] - to[2]).powi(2);

    let dark_sq: f64 = from[3..]
        .iter()
        .zip(&to[3..])
        .map(|(a, b)| (a - b).powi(2))
        .sum();

    match u.physics() {
        Some(phys) => {
            let dist = phys.weighted_distance_sq(&from, &to);
            Ok(Json(ApiResponse::ok(PhysicsDistanceResponse {
                distance_sq: dist,
                distance_7d: dist.sqrt(),
                distance_3d: d3_sq.sqrt(),
                dark_contribution: dark_sq.sqrt(),
                uses_metric: true,
            })))
        }
        None => {
            let dist: f64 = from.iter().zip(&to).map(|(a, b)| (a - b).powi(2)).sum();
            Ok(Json(ApiResponse::ok(PhysicsDistanceResponse {
                distance_sq: dist,
                distance_7d: dist.sqrt(),
                distance_3d: d3_sq.sqrt(),
                dark_contribution: dark_sq.sqrt(),
                uses_metric: false,
            })))
        }
    }
}

#[derive(Deserialize)]
pub struct PhysicsProjectRequest {
    pub coord: Vec<f64>,
}

#[derive(Serialize)]
pub struct PhysicsProjectResponse {
    pub projected: Vec<f64>,
    pub dimensions: usize,
    pub physical: Vec<f64>,
    pub dark: Vec<f64>,
    pub physical_3d: [f64; 3],
    pub uses_projection: bool,
}

pub async fn physics_project(
    State(state): State<SharedState>,
    Json(req): Json<PhysicsProjectRequest>,
) -> Result<Json<ApiResponse<PhysicsProjectResponse>>, AppError> {
    let u = state.universe.read().await;
    let coord = pad7(&req.coord);

    match u.physics() {
        Some(phys) => {
            let proj = phys.project_to_physical(&coord);
            let dark: Vec<f64> = coord[3..].to_vec();
            Ok(Json(ApiResponse::ok(PhysicsProjectResponse {
                projected: coord.to_vec(),
                dimensions: 7,
                physical: proj.to_vec(),
                dark,
                physical_3d: proj,
                uses_projection: true,
            })))
        }
        None => {
            let physical = vec![coord[0], coord[1], coord[2]];
            let dark: Vec<f64> = coord[3..].to_vec();
            Ok(Json(ApiResponse::ok(PhysicsProjectResponse {
                projected: coord.to_vec(),
                dimensions: 7,
                physical: physical.clone(),
                dark,
                physical_3d: [coord[0], coord[1], coord[2]],
                uses_projection: false,
            })))
        }
    }
}

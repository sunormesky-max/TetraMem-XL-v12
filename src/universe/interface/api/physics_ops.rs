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
    pub dimension_weights: [f64; 7],
    pub propagation_decays: [f64; 7],
    pub coupling_strengths: [f64; 7],
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

    match u.physics() {
        Some(phys) => {
            let weights = phys.profile.metric_weights();
            let decays = phys.profile.propagation_decays();
            let mut coupling = [0.0; 7];
            for (i, c) in coupling.iter_mut().enumerate() {
                *c = phys.profile.get(i).coupling_strength;
            }

            Ok(Json(ApiResponse::ok(PhysicsProfileResponse {
                dimension_weights: weights,
                propagation_decays: decays,
                coupling_strengths: coupling,
            })))
        }
        None => Ok(Json(ApiResponse::ok(PhysicsProfileResponse {
            dimension_weights: [1.0; 7],
            propagation_decays: [1.0; 7],
            coupling_strengths: [0.0; 7],
        }))),
    }
}

#[derive(Deserialize)]
pub struct PhysicsDistanceRequest {
    pub from: [f64; 7],
    pub to: [f64; 7],
}

#[derive(Serialize)]
pub struct PhysicsDistanceResponse {
    pub distance_sq: f64,
    pub uses_metric: bool,
}

pub async fn physics_distance(
    State(state): State<SharedState>,
    Json(req): Json<PhysicsDistanceRequest>,
) -> Result<Json<ApiResponse<PhysicsDistanceResponse>>, AppError> {
    let u = state.universe.read().await;

    match u.physics() {
        Some(phys) => {
            let dist = phys.weighted_distance_sq(&req.from, &req.to);
            Ok(Json(ApiResponse::ok(PhysicsDistanceResponse {
                distance_sq: dist,
                uses_metric: true,
            })))
        }
        None => {
            let mut dist = 0.0;
            for i in 0..7 {
                let d = req.from[i] - req.to[i];
                dist += d * d;
            }
            Ok(Json(ApiResponse::ok(PhysicsDistanceResponse {
                distance_sq: dist,
                uses_metric: false,
            })))
        }
    }
}

#[derive(Deserialize)]
pub struct PhysicsProjectRequest {
    pub coord: [f64; 7],
}

#[derive(Serialize)]
pub struct PhysicsProjectResponse {
    pub physical_3d: [f64; 3],
    pub uses_projection: bool,
}

pub async fn physics_project(
    State(state): State<SharedState>,
    Json(req): Json<PhysicsProjectRequest>,
) -> Result<Json<ApiResponse<PhysicsProjectResponse>>, AppError> {
    let u = state.universe.read().await;

    match u.physics() {
        Some(phys) => {
            let proj = phys.project_to_physical(&req.coord);
            Ok(Json(ApiResponse::ok(PhysicsProjectResponse {
                physical_3d: proj,
                uses_projection: true,
            })))
        }
        None => Ok(Json(ApiResponse::ok(PhysicsProjectResponse {
            physical_3d: [req.coord[0], req.coord[1], req.coord[2]],
            uses_projection: false,
        }))),
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::universe::coord::Coord7D;
use crate::universe::error::AppError;

use super::state::SharedState;
use super::types::*;

const MAX_FLOW_AMOUNT: f64 = 1e15;

fn validate_positive_finite(val: f64, name: &str) -> Result<(), AppError> {
    if val <= 0.0 || !val.is_finite() {
        return Err(AppError::BadRequest(format!(
            "{} must be positive and finite",
            name
        )));
    }
    if val > MAX_FLOW_AMOUNT {
        return Err(AppError::BadRequest(format!(
            "{} exceeds maximum allowed value",
            name
        )));
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct Coord7DRequest {
    pub coord: [i32; 7],
    #[serde(default = "default_parity")]
    pub parity: String,
}

fn default_parity() -> String {
    "even".to_string()
}

fn parse_coord(req: &Coord7DRequest) -> Coord7D {
    match req.parity.as_str() {
        "odd" => Coord7D::new_odd(req.coord),
        _ => Coord7D::new_even(req.coord),
    }
}

fn validate_coord_7(c: &[i32; 7]) -> Result<(), AppError> {
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

#[derive(Serialize)]
pub struct DarkQueryResponse {
    pub exists: bool,
    pub physical_energy: f64,
    pub dark_energy: f64,
    pub dims: [f64; 7],
    pub manifested: bool,
    pub manifestation_ratio: f64,
}

pub async fn dark_query(
    State(state): State<SharedState>,
    Json(req): Json<Coord7DRequest>,
) -> Result<Json<ApiResponse<DarkQueryResponse>>, AppError> {
    validate_coord_7(&req.coord)?;
    let coord = parse_coord(&req);
    let u = state.universe.read().await;

    match u.get_node(&coord) {
        Some(node) => {
            let e = node.energy();
            let dims = *e.dims();
            Ok(Json(ApiResponse::ok(DarkQueryResponse {
                exists: true,
                physical_energy: e.physical(),
                dark_energy: e.dark(),
                dims,
                manifested: node.is_manifested_with(u.manifestation_threshold()),
                manifestation_ratio: e.manifestation_ratio(),
            })))
        }
        None => Ok(Json(ApiResponse::ok(DarkQueryResponse {
            exists: false,
            physical_energy: 0.0,
            dark_energy: 0.0,
            dims: [0.0; 7],
            manifested: false,
            manifestation_ratio: 0.0,
        }))),
    }
}

#[derive(Serialize)]
pub struct DarkNodeSummary {
    pub coord: String,
    pub is_manifested: bool,
    pub energy: f64,
}

#[derive(Serialize)]
pub struct DarkListResponse {
    pub nodes: Vec<DarkNodeSummary>,
    pub total: usize,
}

pub async fn dark_list(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<DarkListResponse>>, AppError> {
    let u = state.universe.read().await;
    let threshold = u.manifestation_threshold();

    let nodes: Vec<DarkNodeSummary> = u
        .get_all_nodes()
        .iter()
        .map(|(coord, node)| DarkNodeSummary {
            coord: format!("{:?}", coord.basis()),
            is_manifested: node.is_manifested_with(threshold),
            energy: node.energy().total(),
        })
        .collect();

    let total = nodes.len();
    Ok(Json(ApiResponse::ok(DarkListResponse { nodes, total })))
}

#[derive(Deserialize)]
pub struct DarkFlowRequest {
    pub coord: [i32; 7],
    #[serde(default = "default_parity")]
    pub parity: String,
    pub direction: String,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct DarkFlowResponse {
    pub success: bool,
    pub physical_after: f64,
    pub dark_after: f64,
}

pub async fn dark_flow(
    State(state): State<SharedState>,
    Json(req): Json<DarkFlowRequest>,
) -> Result<Json<ApiResponse<DarkFlowResponse>>, AppError> {
    validate_positive_finite(req.amount, "amount")?;
    validate_coord_7(&req.coord)?;
    let coord = match req.parity.as_str() {
        "odd" => Coord7D::new_odd(req.coord),
        _ => Coord7D::new_even(req.coord),
    };
    let mut u = state.universe.write().await;

    let result = match req.direction.as_str() {
        "to_dark" => u.flow_node_physical_to_dark(&coord, req.amount),
        "to_physical" => u.flow_node_dark_to_physical(&coord, req.amount),
        _ => {
            return Err(AppError::BadRequest(format!(
                "unknown direction: {}",
                req.direction
            )))
        }
    };

    match result {
        Ok(()) => {
            let (phys, dark) = match u.get_node(&coord) {
                Some(node) => {
                    let e = node.energy();
                    (e.physical(), e.dark())
                }
                None => (0.0, 0.0),
            };
            Ok(Json(ApiResponse::ok(DarkFlowResponse {
                success: true,
                physical_after: phys,
                dark_after: dark,
            })))
        }
        Err(e) => Err(AppError::Energy(e)),
    }
}

#[derive(Deserialize)]
pub struct DarkTransferRequest {
    pub from: [i32; 7],
    pub to: [i32; 7],
    #[serde(default = "default_parity")]
    pub parity: String,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct DarkTransferResponse {
    pub success: bool,
    pub from_energy: f64,
    pub to_energy: f64,
}

pub async fn dark_transfer(
    State(state): State<SharedState>,
    Json(req): Json<DarkTransferRequest>,
) -> Result<Json<ApiResponse<DarkTransferResponse>>, AppError> {
    validate_positive_finite(req.amount, "amount")?;
    validate_coord_7(&req.from)?;
    validate_coord_7(&req.to)?;
    let parity_fn = match req.parity.as_str() {
        "odd" => Coord7D::new_odd as fn([i32; 7]) -> Coord7D,
        _ => Coord7D::new_even as fn([i32; 7]) -> Coord7D,
    };
    let from = parity_fn(req.from);
    let to = parity_fn(req.to);

    let mut u = state.universe.write().await;

    match u.transfer_energy(&from, &to, req.amount) {
        Ok(()) => {
            let from_energy = u.get_node(&from).map(|n| n.energy().total()).unwrap_or(0.0);
            let to_energy = u.get_node(&to).map(|n| n.energy().total()).unwrap_or(0.0);
            Ok(Json(ApiResponse::ok(DarkTransferResponse {
                success: true,
                from_energy,
                to_energy,
            })))
        }
        Err(e) => {
            tracing::warn!(
                from = ?from,
                to = ?to,
                amount = req.amount,
                error = %e,
                "dark transfer failed"
            );
            Err(AppError::Energy(e))
        }
    }
}

#[derive(Deserialize)]
pub struct DarkMaterializeRequest {
    pub coord: [i32; 7],
    #[serde(default = "default_parity")]
    pub parity: String,
    pub energy: f64,
    pub physical_ratio: f64,
}

#[derive(Serialize)]
pub struct DarkMaterializeResponse {
    pub success: bool,
    pub coord: String,
    pub manifested: bool,
    pub energy: f64,
    pub physical_ratio: f64,
}

pub async fn dark_materialize(
    State(state): State<SharedState>,
    Json(req): Json<DarkMaterializeRequest>,
) -> Result<Json<ApiResponse<DarkMaterializeResponse>>, AppError> {
    validate_positive_finite(req.energy, "energy")?;
    if req.physical_ratio < 0.0 || req.physical_ratio > 1.0 || !req.physical_ratio.is_finite() {
        return Err(AppError::BadRequest(
            "physical_ratio must be between 0.0 and 1.0 and finite".to_string(),
        ));
    }
    validate_coord_7(&req.coord)?;
    let coord = match req.parity.as_str() {
        "odd" => Coord7D::new_odd(req.coord),
        _ => Coord7D::new_even(req.coord),
    };
    let coord_str = format!("{:?}", coord.basis());

    let mut u = state.universe.write().await;

    match u.materialize_biased(coord, req.energy, req.physical_ratio) {
        Ok(()) => {
            let (manifested, energy) = match u.get_node(&coord) {
                Some(node) => (
                    node.is_manifested_with(u.manifestation_threshold()),
                    node.energy().total(),
                ),
                None => (false, 0.0),
            };
            Ok(Json(ApiResponse::ok(DarkMaterializeResponse {
                success: true,
                coord: coord_str,
                manifested,
                energy,
                physical_ratio: req.physical_ratio,
            })))
        }
        Err(e) => Err(AppError::Energy(e)),
    }
}

#[derive(Deserialize)]
pub struct DarkDematerializeRequest {
    pub coord: [i32; 7],
    #[serde(default = "default_parity")]
    pub parity: String,
}

#[derive(Serialize)]
pub struct DarkDematerializeResponse {
    pub success: bool,
    pub coord: String,
    pub energy: f64,
    pub recovered_energy: f64,
}

pub async fn dark_dematerialize(
    State(state): State<SharedState>,
    Json(req): Json<DarkDematerializeRequest>,
) -> Result<Json<ApiResponse<DarkDematerializeResponse>>, AppError> {
    validate_coord_7(&req.coord)?;
    let coord = match req.parity.as_str() {
        "odd" => Coord7D::new_odd(req.coord),
        _ => Coord7D::new_even(req.coord),
    };
    let coord_str = format!("{:?}", coord.basis());

    let mut u = state.universe.write().await;

    match u.dematerialize(&coord) {
        Some(field) => Ok(Json(ApiResponse::ok(DarkDematerializeResponse {
            success: true,
            coord: coord_str,
            energy: field.total(),
            recovered_energy: field.total(),
        }))),
        None => Err(AppError::NotFound(format!(
            "node at {:?} not found or protected",
            req.coord
        ))),
    }
}

#[derive(Serialize)]
pub struct DarkPressureResponse {
    pub dimension_spread: [f64; 7],
    pub avg_physical_ratio: f64,
    pub dark_node_count: usize,
    pub physical_node_count: usize,
    pub total_dark_energy: f64,
    pub total_physical_energy: f64,
    pub pressure_ratio: f64,
    pub dimension_balance_ok: bool,
}

pub async fn dark_dimension_pressure(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<DarkPressureResponse>>, AppError> {
    let u = state.universe.read().await;

    let stats = u.stats();
    let mut dim_totals = [0.0f64; 7];
    let mut count = 0usize;
    let mut total_ratio = 0.0f64;

    for node in u.get_all_nodes().values() {
        let e = node.energy();
        if e.is_empty() {
            continue;
        }
        let dims = e.dims();
        for i in 0..7 {
            dim_totals[i] += dims[i];
        }
        total_ratio += e.manifestation_ratio();
        count += 1;
    }

    let avg_physical_ratio = if count > 0 {
        total_ratio / count as f64
    } else {
        0.0
    };

    let total_physical = stats.physical_energy;
    let total_dark = stats.dark_energy;
    let pressure_ratio = if total_physical > 0.0 {
        total_dark / total_physical
    } else if total_dark > 0.0 {
        f64::INFINITY
    } else {
        1.0
    };
    let max_dim = dim_totals.iter().cloned().fold(0.0_f64, f64::max);
    let min_dim = dim_totals.iter().cloned().fold(f64::MAX, f64::min);
    let dimension_balance_ok = max_dim - min_dim < stats.total_energy * 0.1;

    Ok(Json(ApiResponse::ok(DarkPressureResponse {
        dimension_spread: dim_totals,
        avg_physical_ratio,
        dark_node_count: stats.dark_nodes,
        physical_node_count: stats.manifested_nodes,
        total_dark_energy: total_dark,
        total_physical_energy: total_physical,
        pressure_ratio,
        dimension_balance_ok,
    })))
}

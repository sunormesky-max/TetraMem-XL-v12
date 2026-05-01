use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::universe::coord::Coord7D;
use crate::universe::error::AppError;

use super::state::SharedState;
use super::types::*;

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
    if req.amount <= 0.0 || !req.amount.is_finite() {
        return Err(AppError::BadRequest(
            "amount must be positive and finite".to_string(),
        ));
    }
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
    if req.amount <= 0.0 || !req.amount.is_finite() {
        return Err(AppError::BadRequest(
            "amount must be positive and finite".to_string(),
        ));
    }
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
    pub manifested: bool,
    pub energy: f64,
}

pub async fn dark_materialize(
    State(state): State<SharedState>,
    Json(req): Json<DarkMaterializeRequest>,
) -> Result<Json<ApiResponse<DarkMaterializeResponse>>, AppError> {
    if req.energy <= 0.0 || !req.energy.is_finite() {
        return Err(AppError::BadRequest(
            "energy must be positive and finite".to_string(),
        ));
    }
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
                manifested,
                energy,
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

    let mut u = state.universe.write().await;

    match u.dematerialize(&coord) {
        Some(field) => Ok(Json(ApiResponse::ok(DarkDematerializeResponse {
            success: true,
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

    Ok(Json(ApiResponse::ok(DarkPressureResponse {
        dimension_spread: dim_totals,
        avg_physical_ratio,
        dark_node_count: stats.dark_nodes,
        physical_node_count: stats.manifested_nodes,
    })))
}

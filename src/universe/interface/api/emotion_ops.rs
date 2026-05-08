// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::universe::cognitive::emotion::PadVector;
use crate::universe::cognitive::functional_emotion::{EmotionSource, FunctionalEmotion};
use crate::universe::dream::DreamEngine;
use crate::universe::error::AppError;
use crate::universe::events::UniverseEvent;
use crate::universe::pulse::{EmotionPulseConfig, PulseEngine, PulseType};

use super::state::SharedState;
use super::types::default_pulse_type;
use super::types::*;

#[derive(Deserialize)]
pub struct EmotionPulseRequest {
    pub source: [i32; 3],
    #[serde(default = "default_pulse_type")]
    pub pulse_type: String,
    pub pleasure: f64,
    pub arousal: f64,
    pub dominance: f64,
    #[serde(default = "default_emotion_source")]
    pub emotion_source: String,
}

fn default_emotion_source() -> String {
    "Functional".to_string()
}

#[derive(Serialize)]
pub struct EmotionPulseResponse {
    pub visited_nodes: usize,
    pub total_activation: f64,
    pub paths_recorded: usize,
    pub final_strength: f64,
    pub emotion_cluster: String,
    pub valence: String,
    pub arousal_level: String,
}

pub async fn emotion_pulse(
    State(state): State<SharedState>,
    Json(req): Json<EmotionPulseRequest>,
) -> Result<Json<ApiResponse<EmotionPulseResponse>>, AppError> {
    for &v in &req.source {
        if !(-10000..=10000).contains(&v) {
            return Err(AppError::BadRequest(format!(
                "coordinate value {} out of range [-10000, 10000]",
                v
            )));
        }
    }

    let u = state.universe.read().await;
    let mut h = state.hebbian.write().await;

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
    let es = match req.emotion_source.as_str() {
        "Perceived" => EmotionSource::Perceived,
        _ => EmotionSource::Functional,
    };

    let pad = PadVector::new(
        req.pleasure.clamp(-1.0, 1.0),
        req.arousal.clamp(-1.0, 1.0),
        req.dominance.clamp(-1.0, 1.0),
    );
    let emotion = FunctionalEmotion::from_pad(pad, es);
    let config = EmotionPulseConfig::default();

    tracing::info!(source = %source, cluster = %emotion.cluster, "firing emotion pulse");

    let engine = PulseEngine::new();
    let result = engine.propagate_with_emotion(&source, pt, &u, &mut h, None, &config, &pad);

    Ok(Json(ApiResponse::ok(EmotionPulseResponse {
        visited_nodes: result.visited_nodes,
        total_activation: result.total_activation,
        paths_recorded: result.paths_recorded,
        final_strength: result.final_strength,
        emotion_cluster: emotion.cluster.name().to_string(),
        valence: format!("{:?}", emotion.valence),
        arousal_level: format!("{:?}", emotion.arousal),
    })))
}

#[derive(Deserialize)]
pub struct EmotionDreamRequest {
    pub pleasure: f64,
    pub arousal: f64,
    pub dominance: f64,
    #[serde(default = "default_emotion_source")]
    pub emotion_source: String,
}

#[derive(Serialize)]
pub struct EmotionDreamResponse {
    pub paths_replayed: usize,
    pub paths_weakened: usize,
    pub memories_consolidated: usize,
    pub edges_before: usize,
    pub edges_after: usize,
    pub weight_before: f64,
    pub weight_after: f64,
    pub emotion_cluster: String,
}

pub async fn emotion_dream(
    State(state): State<SharedState>,
    Json(req): Json<EmotionDreamRequest>,
) -> Result<Json<ApiResponse<EmotionDreamResponse>>, AppError> {
    let u = state.universe.read().await;
    let mut h = state.hebbian.write().await;
    let store = state.memory_store.read().await;

    let es = match req.emotion_source.as_str() {
        "Perceived" => EmotionSource::Perceived,
        _ => EmotionSource::Functional,
    };
    let pad = PadVector::new(
        req.pleasure.clamp(-1.0, 1.0),
        req.arousal.clamp(-1.0, 1.0),
        req.dominance.clamp(-1.0, 1.0),
    );
    let emotion = FunctionalEmotion::from_pad(pad, es);

    tracing::info!(cluster = %emotion.cluster, "running emotion dream cycle");

    let dream = DreamEngine::new();
    let report = dream.dream_with_emotion(&u, &mut h, &store.memories, &pad, es);

    Ok(Json(ApiResponse::ok(EmotionDreamResponse {
        paths_replayed: report.paths_replayed,
        paths_weakened: report.paths_weakened,
        memories_consolidated: report.memories_consolidated,
        edges_before: report.hebbian_edges_before,
        edges_after: report.hebbian_edges_after,
        weight_before: report.weight_before,
        weight_after: report.weight_after,
        emotion_cluster: emotion.cluster.name().to_string(),
    })))
}

#[derive(Deserialize)]
pub struct EmotionCrystallizeRequest {
    #[serde(default = "default_emotion_source")]
    pub emotion_source: String,
}

#[derive(Serialize)]
pub struct EmotionCrystallizeResponse {
    pub new_crystals: usize,
    pub new_super_crystals: usize,
    pub total_crystals: usize,
    pub total_super_crystals: usize,
    pub energy_locked: f64,
    pub emotion_source: String,
}

pub async fn emotion_crystallize(
    State(state): State<SharedState>,
    Json(req): Json<EmotionCrystallizeRequest>,
) -> Result<Json<ApiResponse<EmotionCrystallizeResponse>>, AppError> {
    {
        let con = state.constitution.read().await;
        let check = con.validate_operation("crystal_form");
        if !check.allowed {
            return Err(AppError::Forbidden(format!(
                "constitution blocks crystallize: {}",
                check.violations.join("; ")
            )));
        }
    }
    let u = state.universe.read().await;
    let h = state.hebbian.read().await;
    let mut crystal = state.crystal.write().await;

    let es = match req.emotion_source.as_str() {
        "Perceived" => EmotionSource::Perceived,
        _ => EmotionSource::Functional,
    };

    tracing::info!(source = ?es, "running emotion crystallization");

    let report = crystal.crystallize_emotion(&h, &u, es);

    state.event_sender.publish(UniverseEvent::CrystalFormed {
        new_crystals: report.new_crystals,
        new_super: report.new_super_crystals,
        total_crystals: report.total_crystals,
    });

    Ok(Json(ApiResponse::ok(EmotionCrystallizeResponse {
        new_crystals: report.new_crystals,
        new_super_crystals: report.new_super_crystals,
        total_crystals: report.total_crystals,
        total_super_crystals: report.total_super_crystals,
        energy_locked: report.energy_locked,
        emotion_source: format!("{:?}", es),
    })))
}

#[derive(Serialize)]
pub struct EmotionStatusResponse {
    pub hebbian_edges_functional: usize,
    pub hebbian_edges_perceived: usize,
    pub hebbian_edges_total: usize,
    pub clusters: Vec<String>,
}

pub async fn emotion_status(
    State(state): State<SharedState>,
) -> Json<ApiResponse<EmotionStatusResponse>> {
    let h = state.hebbian.read().await;

    let functional = h.edges_by_emotion(EmotionSource::Functional).len();
    let perceived = h.edges_by_emotion(EmotionSource::Perceived).len();
    let total = h.edge_count();

    let clusters: Vec<String> = crate::universe::functional_emotion::EmotionCluster::all()
        .iter()
        .map(|c| format!("{} ({:?}/{:?})", c.name(), c.valence(), c.arousal()))
        .collect();

    Json(ApiResponse::ok(EmotionStatusResponse {
        hebbian_edges_functional: functional,
        hebbian_edges_perceived: perceived,
        hebbian_edges_total: total,
        clusters,
    }))
}

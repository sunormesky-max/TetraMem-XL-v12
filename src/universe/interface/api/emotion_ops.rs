// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::universe::cognitive::emotion::PadVector;
use crate::universe::cognitive::functional_emotion::{EmotionSource, FunctionalEmotion};
use crate::universe::coord::Coord7D;
use crate::universe::dream::DreamEngine;
use crate::universe::error::AppError;
use crate::universe::events::UniverseEvent;
use crate::universe::pulse::{EmotionPulseConfig, PulseEngine, PulseType};

use super::state::SharedState;
use super::types::default_pulse_type;
use super::types::*;

#[derive(Deserialize)]
pub struct EmotionPulseRequest {
    #[serde(deserialize_with = "deserialize_anchor7d")]
    pub source: Coord7D,
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
    for v in req.source.basis() {
        if !(-10000..=10000).contains(&v) {
            return Err(AppError::BadRequest(format!(
                "coordinate value {} out of range [-10000, 10000]",
                v
            )));
        }
    }

    let mut h = state.hebbian.write().await;
    let u = state.universe.read().await;

    let source = req.source;
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
    let mut h = state.hebbian.write().await;
    let store = state.memory_store.read().await;
    let u = state.universe.read().await;

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
    let mut crystal = state.crystal.write().await;
    let h = state.hebbian.read().await;
    let u = state.universe.read().await;

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
    pub pad: PadSummary,
    pub quadrant: String,
    pub functional_cluster: String,
    pub recommendations: Vec<String>,
    pub hebbian_edges_total: usize,
}

#[derive(Serialize)]
pub struct PadSummary {
    pub pleasure: f64,
    pub arousal: f64,
    pub dominance: f64,
}

pub async fn emotion_status(
    State(state): State<SharedState>,
) -> Json<ApiResponse<EmotionStatusResponse>> {
    let u = state.universe.read().await;
    let h = state.hebbian.read().await;

    let reading = crate::universe::cognitive::emotion::EmotionMapper::read(&u);
    let pad = reading.pad;
    let quadrant = reading.quadrant.to_string();

    let func = crate::universe::cognitive::functional_emotion::FunctionalEmotion::from_pad(
        pad,
        EmotionSource::Functional,
    );
    let functional_cluster = func.cluster.name().to_string();

    let recommendations = generate_recommendations(&pad, &quadrant, &functional_cluster);

    let hebbian_edges_total = h.edge_count();

    Json(ApiResponse::ok(EmotionStatusResponse {
        pad: PadSummary {
            pleasure: pad.pleasure,
            arousal: pad.arousal,
            dominance: pad.dominance,
        },
        quadrant,
        functional_cluster,
        recommendations,
        hebbian_edges_total,
    }))
}

fn generate_recommendations(
    pad: &crate::universe::cognitive::emotion::PadVector,
    quadrant: &str,
    cluster: &str,
) -> Vec<String> {
    let mut recs = Vec::new();

    if pad.pleasure < -0.3 {
        recs.push("愉悦度偏低，建议注入正向记忆以稳定情绪平衡".to_string());
    }
    if pad.pleasure > 0.5 {
        recs.push("愉悦度较高，适合进行高强度学习与记忆整合".to_string());
    }
    if pad.arousal > 0.5 {
        recs.push("唤醒度偏高，推荐使用探索性脉冲扩展知识边界".to_string());
    }
    if pad.arousal < -0.3 {
        recs.push("唤醒度偏低，建议触发梦境循环激活潜在记忆路径".to_string());
    }
    if pad.dominance < -0.3 {
        recs.push("支配度不足，系统不确定性较高，建议运行认知反思".to_string());
    }
    if pad.dominance > 0.3 {
        recs.push("支配度良好，系统处于高置信状态，可执行巩固操作".to_string());
    }

    recs.push(format!("当前情绪聚类: {} — 象限: {}", cluster, quadrant));

    recs
}

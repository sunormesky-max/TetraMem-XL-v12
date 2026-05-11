// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use axum::{extract::State, Json};

use crate::universe::cognitive::agent::{
    AgentContext, AgentContextMut, CognitiveAgent, CrystalAgent, EmotionAgent, ObserverAgent,
};
use crate::universe::coord::Coord7D;
use crate::universe::dream::DreamEngine;
use crate::universe::error::AppError;
use crate::universe::events::UniverseEvent;
use crate::universe::metrics;
use crate::universe::observer::{SelfRegulator, UniverseObserver};
use crate::universe::pulse::{PulseEngine, PulseType};

use super::state::SharedState;
use super::types::*;

pub async fn fire_pulse(
    State(state): State<SharedState>,
    Json(req): Json<PulseRequest>,
) -> Result<Json<ApiResponse<PulseResponse>>, AppError> {
    {
        let con = state.constitution.read().await;
        let check = con.validate_operation("pulse_fire");
        if !check.allowed {
            return Err(AppError::Forbidden(format!(
                "constitution blocks pulse: {}",
                check.violations.join("; ")
            )));
        }
    }
    let u = state.universe.read().await;
    let mut h = state.hebbian.write().await;

    if let Some(c) = metrics::API_PULSE_TOTAL.get() {
        c.inc();
    }
    let source = req.source;
    let pt = match req.pulse_type.to_lowercase().as_str() {
        "reinforcing" => PulseType::Reinforcing,
        "cascade" => PulseType::Cascade,
        _ => PulseType::Exploratory,
    };

    tracing::info!(source = %source, pulse_type = ?pt, "firing pulse");
    let engine = PulseEngine::new();
    let result = engine.propagate(&source, pt, &u, &mut h);

    state.event_sender.publish(UniverseEvent::PulseCompleted {
        source: source.basis(),
        pulse_type: req.pulse_type,
        visited_nodes: result.visited_nodes,
        paths_recorded: result.paths_recorded,
    });

    Ok(Json(ApiResponse::ok(PulseResponse {
        visited_nodes: result.visited_nodes,
        total_activation: result.total_activation,
        paths_recorded: result.paths_recorded,
        final_strength: result.final_strength,
    })))
}

pub async fn run_dream(State(state): State<SharedState>) -> Json<ApiResponse<DreamResponse>> {
    {
        let con = state.constitution.read().await;
        let check = con.validate_operation("dream_cycle");
        if !check.allowed {
            return Json(ApiResponse::err(format!(
                "constitution blocks dream: {}",
                check.violations.join("; ")
            )));
        }
    }
    let u = state.universe.read().await;
    let mut h = state.hebbian.write().await;
    let store = state.memory_store.read().await;

    if let Some(c) = metrics::API_DREAM_TOTAL.get() {
        c.inc();
    }
    tracing::info!("running dream cycle");

    let dream = DreamEngine::new();
    let report = dream.dream(&u, &mut h, &store.memories);

    tracing::info!(
        replayed = report.paths_replayed,
        weakened = report.paths_weakened,
        consolidated = report.memories_consolidated,
        "dream cycle complete"
    );

    drop(store);
    let store = state.memory_store.read().await;
    {
        let mut sem = state.semantic.write().await;
        sem.sync_after_dream(&store.memories, &u);
    }

    state.event_sender.publish(UniverseEvent::DreamCompleted {
        phase: "default".to_string(),
        paths_replayed: report.paths_replayed,
        paths_weakened: report.paths_weakened,
        memories_consolidated: report.memories_consolidated,
        memories_merged: 0,
        edges_before: report.hebbian_edges_before,
        edges_after: report.hebbian_edges_after,
    });

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
    let store = state.memory_store.read().await;
    let mut h = state.hebbian.write().await;

    let report = UniverseObserver::inspect(&u, &h, &store.memories);
    let regulator = SelfRegulator::new();
    let actions = regulator.regulate(&report, &mut h);

    tracing::info!(actions = actions.len(), "regulation cycle complete");
    let descriptions: Vec<String> = actions.iter().map(|a| a.description.clone()).collect();
    state.event_sender.publish(UniverseEvent::RegulationCycle {
        stress_level: report.energy_utilization,
        entropy: report.density,
        actions_count: actions.len(),
    });
    Json(ApiResponse::ok(descriptions))
}

pub async fn perception_status(
    State(state): State<SharedState>,
) -> Json<ApiResponse<PerceptionStatusResponse>> {
    let p = state.perception.read().await;
    let report = p.report();
    Json(ApiResponse::ok(PerceptionStatusResponse {
        total_budget: report.total_budget,
        allocated: report.allocated,
        available: report.total_budget - report.allocated,
        spent: report.spent,
        returned: report.returned,
        utilization: report.utilization,
    }))
}

pub async fn perception_replenish(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<PerceptionStatusResponse>>, AppError> {
    {
        let u = state.universe.read().await;
        let mut p = state.perception.write().await;
        p.replenish(u.total_energy());
    }
    let p = state.perception.read().await;
    let report = p.report();
    Ok(Json(ApiResponse::ok(PerceptionStatusResponse {
        total_budget: report.total_budget,
        allocated: report.allocated,
        available: report.total_budget - report.allocated,
        spent: report.spent,
        returned: report.returned,
        utilization: report.utilization,
    })))
}

pub async fn assess_novelty(
    State(state): State<SharedState>,
    Json(req): Json<NoveltyAssessRequest>,
) -> Result<Json<ApiResponse<NoveltyAssessResponse>>, AppError> {
    if req.data.is_empty() || req.data.len() > 64 {
        return Err(AppError::BadRequest(format!(
            "data length must be between 1 and 64, got {}",
            req.data.len()
        )));
    }
    for &v in &req.data {
        if !v.is_finite() {
            return Err(AppError::BadRequest(
                "data values must be finite".to_string(),
            ));
        }
    }

    let anchor = req
        .anchor
        .unwrap_or_else(|| Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]));

    let (report, should_store) = {
        let sem = state.semantic.read().await;
        let store = state.memory_store.read().await;
        let h = state.hebbian.read().await;
        let knn = sem.search_similar(&req.data, 5);
        let knn_distances: Vec<(f64, usize)> = knn
            .iter()
            .filter_map(|r| {
                store
                    .memories
                    .iter()
                    .position(|m| {
                        let mk = crate::universe::memory::AtomKey::from_atom(m);
                        mk == r.atom_key
                    })
                    .map(|idx| (r.distance, idx))
            })
            .collect();
        let detector = crate::universe::memory::NoveltyDetector::default();
        let nr = detector.assess(&req.data, &knn_distances, &anchor, &h, &store.memories);
        let ss = detector.should_store(&nr);
        (nr, ss)
    };

    Ok(Json(ApiResponse::ok(NoveltyAssessResponse {
        score: report.score,
        level: format!("{}", report.level),
        suggested_importance: report.suggested_importance,
        wavelet_energy: report.wavelet_energy,
        detail_energy: report.detail_energy,
        anomaly_score: report.anomaly_score,
        should_store,
    })))
}

pub async fn semantic_status(
    State(state): State<SharedState>,
) -> Json<ApiResponse<SemanticStatusResponse>> {
    let sem = state.semantic.read().await;
    let report = sem.report();
    Json(ApiResponse::ok(SemanticStatusResponse {
        embeddings_indexed: report.embeddings_indexed,
        relations_total: report.relations_total,
        concepts_extracted: report.concepts_extracted,
    }))
}

pub async fn semantic_index_all(
    State(state): State<SharedState>,
) -> Json<ApiResponse<SemanticStatusResponse>> {
    let u = state.universe.read().await;
    let store = state.memory_store.read().await;
    let mut sem = state.semantic.write().await;
    for atom in store.memories.iter() {
        if let Ok(data) = crate::universe::memory::MemoryCodec::decode(&u, atom) {
            sem.index_memory(atom, &data);
        }
    }
    let report = sem.report();
    Json(ApiResponse::ok(SemanticStatusResponse {
        embeddings_indexed: report.embeddings_indexed,
        relations_total: report.relations_total,
        concepts_extracted: report.concepts_extracted,
    }))
}

pub async fn semantic_extract_concepts(
    State(state): State<SharedState>,
) -> Json<ApiResponse<SemanticStatusResponse>> {
    let store = state.memory_store.read().await;
    let mut sem = state.semantic.write().await;
    let atoms_by_key: std::collections::HashMap<
        crate::universe::memory::AtomKey,
        &crate::universe::memory::MemoryAtom,
    > = store
        .memories
        .iter()
        .map(|a| (crate::universe::memory::AtomKey::from_atom(a), a))
        .collect();
    sem.extract_concepts(&atoms_by_key);
    let report = sem.report();
    Json(ApiResponse::ok(SemanticStatusResponse {
        embeddings_indexed: report.embeddings_indexed,
        relations_total: report.relations_total,
        concepts_extracted: report.concepts_extracted,
    }))
}

pub async fn clustering_status(
    State(state): State<SharedState>,
) -> Json<ApiResponse<ClusteringStatusResponse>> {
    let cl = state.clustering.read().await;
    let report = cl.report();
    Json(ApiResponse::ok(ClusteringStatusResponse {
        memories_clustered: report.memories_in_attractors,
        attractors_found: report.attractors,
        tunnels_active: report.total_tunnels,
        bridges_active: report.total_bridges,
    }))
}

pub async fn clustering_maintenance(
    State(state): State<SharedState>,
) -> Json<ApiResponse<ClusteringStatusResponse>> {
    let u = state.universe.read().await;
    let store = state.memory_store.read().await;
    let mut h = state.hebbian.write().await;
    let mut cl = state.clustering.write().await;
    let report = cl.run_maintenance_cycle(&store.memories, &mut h, &u);
    Json(ApiResponse::ok(ClusteringStatusResponse {
        memories_clustered: report.memories_in_attractors,
        attractors_found: report.attractors,
        tunnels_active: report.total_tunnels,
        bridges_active: report.total_bridges,
    }))
}

pub async fn constitution_status(
    State(state): State<SharedState>,
) -> Json<ApiResponse<ConstitutionStatusResponse>> {
    let con = state.constitution.read().await;
    Json(ApiResponse::ok(ConstitutionStatusResponse {
        rules_count: con.rules().len(),
        bounds_count: con.bounds().len(),
        rules: con.rules().iter().map(|r| r.id.clone()).collect(),
    }))
}

pub async fn events_status(
    State(state): State<SharedState>,
) -> Json<ApiResponse<EventsStatusResponse>> {
    let ev = state.events.lock().await;
    Json(ApiResponse::ok(EventsStatusResponse {
        history_len: ev.history_len(),
        subscriber_count: ev.subscriber_count(),
    }))
}

pub async fn watchdog_status(
    State(state): State<SharedState>,
) -> Json<ApiResponse<WatchdogStatusResponse>> {
    let wd = state.watchdog.read().await;
    Json(ApiResponse::ok(WatchdogStatusResponse {
        total_checkups: wd.total_checkups(),
        uptime_ms: wd.uptime_ms(),
    }))
}

pub async fn watchdog_checkup(
    State(state): State<SharedState>,
) -> Json<ApiResponse<WatchdogCheckupResponse>> {
    let mut u = state.universe.write().await;
    let mut h = state.hebbian.write().await;
    let mut c = state.crystal.write().await;
    let store = state.memory_store.read().await;
    let mut wd = state.watchdog.write().await;
    let report = wd.checkup(&mut u, &mut h, &mut c, &store.memories);
    Json(ApiResponse::ok(WatchdogCheckupResponse {
        level: report.level.as_str().to_string(),
        utilization: report.utilization,
        conservation_ok: report.conservation_ok,
        actions: report
            .actions
            .iter()
            .map(|a| format!("{}: {}", a.action, a.detail))
            .collect(),
    }))
}

pub async fn agent_execute_observer(
    State(state): State<SharedState>,
) -> Json<ApiResponse<AgentExecuteResponse>> {
    let u = state.universe.read().await;
    let h = state.hebbian.read().await;
    let store = state.memory_store.read().await;
    let c = state.crystal.read().await;
    let con = state.constitution.read().await;
    let agent = ObserverAgent;
    let ctx = AgentContext {
        universe: &u,
        hebbian: &h,
        memories: &store.memories,
        crystal: &c,
        constitution: &con,
        event_sender: Some(&state.event_sender),
    };
    let report = agent.execute_readonly(&ctx);
    Json(ApiResponse::ok(AgentExecuteResponse {
        agent: format!("{}", report.agent),
        success: report.success,
        duration_ms: report.duration_ms,
        details: report.details,
    }))
}

pub async fn agent_execute_emotion(
    State(state): State<SharedState>,
) -> Json<ApiResponse<AgentExecuteResponse>> {
    let u = state.universe.read().await;
    let h = state.hebbian.read().await;
    let store = state.memory_store.read().await;
    let c = state.crystal.read().await;
    let con = state.constitution.read().await;
    let agent = EmotionAgent;
    let ctx = AgentContext {
        universe: &u,
        hebbian: &h,
        memories: &store.memories,
        crystal: &c,
        constitution: &con,
        event_sender: Some(&state.event_sender),
    };
    let report = agent.execute_readonly(&ctx);
    Json(ApiResponse::ok(AgentExecuteResponse {
        agent: format!("{}", report.agent),
        success: report.success,
        duration_ms: report.duration_ms,
        details: report.details,
    }))
}

pub async fn agent_execute_crystal(
    State(state): State<SharedState>,
) -> Json<ApiResponse<AgentExecuteResponse>> {
    let mut u = state.universe.write().await;
    let mut h = state.hebbian.write().await;
    let mut store = state.memory_store.write().await;
    let mut c = state.crystal.write().await;
    let con = state.constitution.read().await;
    let agent = CrystalAgent;
    let mut ctx = AgentContextMut {
        universe: &mut u,
        hebbian: &mut h,
        memories: &mut store.memories,
        crystal: &mut c,
        constitution: &con,
        event_sender: Some(&state.event_sender),
    };
    let report = agent.execute_mut(&mut ctx);
    Json(ApiResponse::ok(AgentExecuteResponse {
        agent: format!("{}", report.agent),
        success: report.success,
        duration_ms: report.duration_ms,
        details: report.details,
    }))
}

pub async fn memory_aging(
    State(state): State<SharedState>,
    Json(req): Json<super::types::AgingRequest>,
) -> Result<Json<ApiResponse<super::types::AgingResponse>>, AppError> {
    let accessed = req.accessed_anchors.unwrap_or_default();
    let mut store = state.memory_store.write().await;
    let engine = crate::universe::memory::AgingEngine::default();
    let report = engine.age(&mut store.memories, &accessed);
    let flagged = engine
        .flagged_memories(&store.memories)
        .iter()
        .map(|(_, m)| format!("{}", m.anchor()))
        .collect();
    Ok(Json(ApiResponse::ok(super::types::AgingResponse {
        aged_count: report.aged_count,
        flagged_for_forget: report.flagged_for_forget,
        boosted_count: report.boosted_count,
        min_importance: report.min_importance,
        avg_importance: report.avg_importance,
        flagged_anchors: flagged,
    })))
}

pub async fn detect_contradictions(
    State(state): State<SharedState>,
) -> Json<ApiResponse<serde_json::Value>> {
    let store = state.memory_store.read().await;
    let detector = crate::universe::memory::ContradictionDetector::default();
    let report = detector.detect(&store.memories);
    Json(ApiResponse::ok(serde_json::json!({
        "contradictions": report.contradictions.len(),
        "merge_candidates": report.merge_candidates.len(),
        "contradiction_details": report.contradictions,
        "merge_details": report.merge_candidates,
    })))
}

pub async fn get_cognitive_state(
    State(state): State<SharedState>,
) -> Json<ApiResponse<serde_json::Value>> {
    let u = state.universe.read().await;
    let h = state.hebbian.read().await;
    let store = state.memory_store.read().await;
    let cs = crate::universe::cognitive::cognitive_state::CognitiveStateEngine::assess(
        &u,
        &h,
        &store.memories,
    );
    let json =
        serde_json::to_value(&cs).unwrap_or(serde_json::json!({"error": "serialize failed"}));
    Json(ApiResponse::ok(json))
}

pub async fn get_attention_map(
    State(state): State<SharedState>,
) -> Json<ApiResponse<serde_json::Value>> {
    let u = state.universe.read().await;
    let h = state.hebbian.read().await;
    let store = state.memory_store.read().await;
    let map = crate::universe::cognitive::attention::AttentionEngine::new().compute(
        &u,
        &h,
        &store.memories,
    );
    let json =
        serde_json::to_value(&map).unwrap_or(serde_json::json!({"error": "serialize failed"}));
    Json(ApiResponse::ok(json))
}

pub async fn get_dream_insights(
    State(state): State<SharedState>,
) -> Json<ApiResponse<serde_json::Value>> {
    let u = state.universe.read().await;
    let h = state.hebbian.read().await;
    let store = state.memory_store.read().await;
    let sem = state.semantic.read().await;
    let report = crate::universe::cognitive::dream_insight::DreamInsightEngine::new()
        .generate_insights(&u, &h, &store.memories, &sem);
    let json =
        serde_json::to_value(&report).unwrap_or(serde_json::json!({"error": "serialize failed"}));
    Json(ApiResponse::ok(json))
}

pub async fn reflect(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<super::types::ReflectResponse>>, AppError> {
    let u = state.universe.read().await;
    let h = state.hebbian.read().await;
    let store = state.memory_store.read().await;
    let sem = state.semantic.read().await;

    let insights = crate::universe::cognitive::dream_insight::DreamInsightEngine::new()
        .generate_insights(&u, &h, &store.memories, &sem);

    let cs = crate::universe::cognitive::cognitive_state::CognitiveStateEngine::assess(
        &u,
        &h,
        &store.memories,
    );

    let conservation_ok = u.verify_conservation();

    drop(u);
    drop(h);
    drop(store);
    drop(sem);

    let insights_json = serde_json::to_value(&insights).unwrap_or(serde_json::json!({}));
    let cs_json = serde_json::to_value(&cs).unwrap_or(serde_json::json!({}));

    Ok(Json(ApiResponse::ok(super::types::ReflectResponse {
        dream_insights: insights_json,
        cognitive_state: cs_json,
        conservation_ok,
        total_insights: insights.total_insights,
    })))
}

pub async fn identity_profile(
    State(state): State<SharedState>,
) -> Json<ApiResponse<serde_json::Value>> {
    let store = state.memory_store.read().await;
    let guard = state.identity_guard.read().await;
    let profile = guard.profile(&store.memories);
    drop(guard);
    drop(store);
    match serde_json::to_value(profile) {
        Ok(v) => Json(ApiResponse::ok(v)),
        Err(e) => Json(ApiResponse::err(e.to_string())),
    }
}

pub async fn meta_cognitive_state(
    State(state): State<SharedState>,
) -> Json<ApiResponse<serde_json::Value>> {
    let u = state.universe.read().await;
    let h = state.hebbian.read().await;
    let store = state.memory_store.read().await;
    let model = crate::universe::cognitive::meta_cognitive::MetaCognitiveEngine::assess(
        &u,
        &h,
        &store.memories,
    );
    drop(store);
    drop(h);
    drop(u);
    match serde_json::to_value(model) {
        Ok(v) => Json(ApiResponse::ok(v)),
        Err(e) => Json(ApiResponse::err(e.to_string())),
    }
}

pub async fn prediction_status(
    State(state): State<SharedState>,
) -> Json<ApiResponse<serde_json::Value>> {
    let pred = state.prediction.read().await;
    Json(ApiResponse::ok(serde_json::json!({
        "active_predictions": pred.active_prediction_count(),
        "avg_surprise": pred.avg_surprise(),
        "prediction_accuracy": pred.prediction_accuracy(),
    })))
}

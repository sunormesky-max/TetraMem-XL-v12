// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use axum::{extract::State, Json};

use crate::universe::cognitive::agent::{
    AgentContext, AgentContextMut, CognitiveAgent, CrystalAgent, EmotionAgent, ObserverAgent,
};
use crate::universe::dream::DreamEngine;
use crate::universe::error::AppError;
use crate::universe::events::UniverseEvent;
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
    let mems = state.memories.read().await;
    let mut h = state.hebbian.write().await;

    let report = UniverseObserver::inspect(&u, &h, &mems);
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
    let mems = state.memories.read().await;
    let u = state.universe.read().await;
    let mut sem = state.semantic.write().await;
    for atom in mems.iter() {
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
    let mems = state.memories.read().await;
    let mut sem = state.semantic.write().await;
    let atoms_by_key: std::collections::HashMap<
        crate::universe::memory::AtomKey,
        &crate::universe::memory::MemoryAtom,
    > = mems
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
    let _cl = state.clustering.read().await;
    Json(ApiResponse::ok(ClusteringStatusResponse {
        memories_clustered: 0,
        attractors_found: 0,
        tunnels_active: 0,
        bridges_active: 0,
    }))
}

pub async fn clustering_maintenance(
    State(state): State<SharedState>,
) -> Json<ApiResponse<ClusteringStatusResponse>> {
    let mems = state.memories.read().await;
    let u = state.universe.read().await;
    let mut h = state.hebbian.write().await;
    let mut cl = state.clustering.write().await;
    let report = cl.run_maintenance_cycle(&mems, &mut h, &u);
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
    let ev = state.events.lock().unwrap();
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
    let mems = state.memories.read().await;
    let mut wd = state.watchdog.write().await;
    let report = wd.checkup(&mut u, &mut h, &mut c, &mems);
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
    let mems = state.memories.read().await;
    let c = state.crystal.read().await;
    let con = state.constitution.read().await;
    let agent = ObserverAgent;
    let ctx = AgentContext {
        universe: &u,
        hebbian: &h,
        memories: &mems,
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
    let mems = state.memories.read().await;
    let c = state.crystal.read().await;
    let con = state.constitution.read().await;
    let agent = EmotionAgent;
    let ctx = AgentContext {
        universe: &u,
        hebbian: &h,
        memories: &mems,
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
    let mut mems = state.memories.write().await;
    let mut c = state.crystal.write().await;
    let con = state.constitution.read().await;
    let agent = CrystalAgent;
    let mut ctx = AgentContextMut {
        universe: &mut u,
        hebbian: &mut h,
        memories: &mut mems,
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

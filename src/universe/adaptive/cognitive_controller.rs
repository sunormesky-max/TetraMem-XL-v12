// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use crate::universe::adaptive::spontaneous_drive::SpontaneousDrive;
use crate::universe::api::AppState;
use crate::universe::cognitive::cognitive_state::CognitiveStateEngine;
use crate::universe::config::MaintenanceConfig;
use crate::universe::config::SpontaneousConfig;
use crate::universe::dream::DreamEngine;
use crate::universe::events::UniverseEvent;
use crate::universe::memory::aging::AgingEngine;
use crate::universe::memory::MemoryCodec;
use crate::universe::regulation::RegulationEngine;
use crate::universe::watchdog::WatchdogLevel;

const WEAK_EDGE_THRESHOLD: f64 = 0.15;
const WEAK_EDGE_BOOST: f64 = 0.05;
const MAX_WEAK_REINFORCE: usize = 20;
const FORGET_THRESHOLD: f64 = 0.05;
const IDENTITY_RESTORE_MIN: f64 = 0.85;
const MIN_INTERVAL_SECS: u64 = 30;
const MAX_INTERVAL_SECS: u64 = 600;

struct ControllerState {
    forget_tracker: HashMap<String, u32>,
    last_elapsed_ms: f64,
    consecutive_failures: u32,
    spontaneous: Option<SpontaneousDrive>,
}

pub fn spawn_cognitive_controller(state: Arc<AppState>) -> Option<tokio::task::JoinHandle<()>> {
    let cfg = state.config.maintenance.clone();
    if !cfg.enabled {
        tracing::info!("cognitive controller: disabled by config");
        return None;
    }

    let spontaneous_cfg = state.config.spontaneous.clone();

    let base_interval = cfg.interval_secs.max(MIN_INTERVAL_SECS);
    tracing::info!(
        "cognitive controller: spawning (base_interval={}s, dream_urgency>={:.2}, auto_forget={}, max_memories={}, spontaneous={})",
        base_interval,
        cfg.dream_min_urgency,
        cfg.auto_forget_enabled,
        cfg.max_memories,
        spontaneous_cfg.enabled,
    );

    let handle = tokio::spawn(async move {
        let mut cycle: u64 = 0;
        let mut ctrl = ControllerState {
            forget_tracker: HashMap::new(),
            last_elapsed_ms: 0.0,
            consecutive_failures: 0,
            spontaneous: if spontaneous_cfg.enabled {
                Some(SpontaneousDrive::new(&spontaneous_cfg))
            } else {
                None
            },
        };

        loop {
            let adaptive_interval = compute_adaptive_interval(base_interval, &ctrl);
            let mut interval = tokio::time::interval(Duration::from_secs(adaptive_interval));
            interval.tick().await;
            interval.tick().await;
            cycle += 1;

            if state.shutdown.load(std::sync::atomic::Ordering::Relaxed) {
                tracing::info!(
                    cycle,
                    "cognitive controller: shutdown signal received, exiting"
                );
                break;
            }

            run_maintenance_cycle(&state, &cfg, &spontaneous_cfg, cycle, &mut ctrl).await;
        }
    });

    Some(handle)
}

fn compute_adaptive_interval(base: u64, ctrl: &ControllerState) -> u64 {
    if ctrl.consecutive_failures > 0 {
        return (base * 2).min(MAX_INTERVAL_SECS);
    }
    let elapsed = ctrl.last_elapsed_ms;
    if elapsed > 30_000.0 {
        return (base * 2).min(MAX_INTERVAL_SECS);
    }
    if elapsed > 10_000.0 {
        return ((base as f64 * 1.5) as u64).min(MAX_INTERVAL_SECS);
    }
    base.max(MIN_INTERVAL_SECS)
}

async fn run_maintenance_cycle(
    state: &Arc<AppState>,
    cfg: &MaintenanceConfig,
    spontaneous_cfg: &SpontaneousConfig,
    cycle: u64,
    ctrl: &mut ControllerState,
) {
    let start = std::time::Instant::now();

    let result = run_maintenance_inner(state, cfg, spontaneous_cfg, cycle, ctrl).await;

    match result {
        Ok(()) => {
            ctrl.consecutive_failures = 0;
        }
        Err(e) => {
            ctrl.consecutive_failures += 1;
            tracing::error!(
                cycle,
                consecutive_failures = ctrl.consecutive_failures,
                error = %e,
                "cognitive controller: error in maintenance cycle"
            );
        }
    }

    ctrl.last_elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    let adaptive_next = compute_adaptive_interval(cfg.interval_secs.max(MIN_INTERVAL_SECS), ctrl);
    tracing::info!(
        cycle,
        elapsed_ms = format!("{:.0}", ctrl.last_elapsed_ms),
        next_in_secs = adaptive_next,
        failures = ctrl.consecutive_failures,
        "cognitive controller: cycle complete"
    );
}

async fn run_maintenance_inner(
    state: &Arc<AppState>,
    cfg: &MaintenanceConfig,
    spontaneous_cfg: &SpontaneousConfig,
    cycle: u64,
    ctrl: &mut ControllerState,
) -> Result<(), String> {
    let start = std::time::Instant::now();

    let (cognitive_state, mem_count) = {
        let u = state.universe.read().await;
        let h = state.hebbian.read().await;
        let store = state.memory_store.read().await;
        let cs = CognitiveStateEngine::assess(&u, &h, &store.memories);
        let mc = store.memories.len();
        (cs, mc)
    };

    if mem_count == 0 {
        return Ok(());
    }

    let should_dream = cognitive_state.dream_readiness.should_dream
        && cognitive_state.dream_readiness.urgency >= cfg.dream_min_urgency;
    let vigor = cognitive_state.overall_vigor;

    restore_identity_importance(state).await;

    if should_dream {
        let mut h = state.hebbian.write().await;
        let store = state.memory_store.read().await;

        let mut reinforced = 0usize;
        for mem in store.memories.iter() {
            if reinforced >= MAX_WEAK_REINFORCE {
                break;
            }
            let anchor = mem.anchor();
            let neighbors = h.get_neighbors(anchor);
            for (coord, weight) in &neighbors {
                if *weight > 0.0 && *weight < WEAK_EDGE_THRESHOLD {
                    h.boost_edge(anchor, coord, WEAK_EDGE_BOOST);
                    reinforced += 1;
                    if reinforced >= MAX_WEAK_REINFORCE {
                        break;
                    }
                }
            }
        }
        drop(store);
        drop(h);

        if reinforced > 0 {
            tracing::debug!(
                cycle,
                reinforced,
                "cognitive controller: reinforced weak connections"
            );
        }
    }

    if should_dream {
        let u = state.universe.read().await;
        let store = state.memory_store.read().await;
        let mut h = state.hebbian.write().await;

        let dream = DreamEngine::new();
        let report = {
            let pred = state.prediction.read().await;
            let high_surprise_anchors: Vec<crate::universe::coord::Coord7D> = pred
                .predictions()
                .values()
                .filter(|p| p.confidence < 0.3)
                .map(|p| p.source)
                .collect();
            drop(pred);

            if high_surprise_anchors.is_empty() {
                dream.dream(&u, &mut h, &store.memories)
            } else {
                tracing::debug!(
                    cycle,
                    surprise_anchors = high_surprise_anchors.len(),
                    "cognitive controller: dream with prediction-surprise priority"
                );
                dream.dream_with_prediction_surprise(
                    &u,
                    &mut h,
                    &store.memories,
                    &high_surprise_anchors,
                )
            }
        };
        drop(store);
        drop(u);
        drop(h);

        state.event_sender.publish(UniverseEvent::DreamCompleted {
            phase: format!("{:?}", report.phase),
            paths_replayed: report.paths_replayed,
            paths_weakened: report.paths_weakened,
            memories_consolidated: report.memories_consolidated,
            memories_merged: report.memories_merged,
            edges_before: report.hebbian_edges_before,
            edges_after: report.hebbian_edges_after,
        });

        tracing::info!(
            cycle,
            "cognitive controller: dream — replayed={}, weakened={}, consolidated={}, edges {}→{}",
            report.paths_replayed,
            report.paths_weakened,
            report.memories_consolidated,
            report.hebbian_edges_before,
            report.hebbian_edges_after,
        );
    }

    if cfg.aging_enabled {
        let accessed: Vec<String> = {
            let store = state.memory_store.read().await;
            store
                .memories
                .iter()
                .filter(|m| m.importance() > 0.5)
                .map(|m| format!("{}", m.anchor()))
                .collect()
        };
        let mut store = state.memory_store.write().await;
        let report = AgingEngine::default().age(&mut store.memories, &accessed);
        drop(store);

        if report.flagged_for_forget > 0 {
            tracing::warn!(
                cycle,
                flagged = report.flagged_for_forget,
                "cognitive controller: aging flagged memories"
            );
        }
    }

    if cfg.aging_enabled && cfg.auto_forget_enabled && cycle > 2 {
        auto_forget_step(state, cfg, cycle, ctrl).await;
    }

    if cfg.clustering_enabled {
        let u = state.universe.read().await;
        let store = state.memory_store.read().await;
        let mut clustering = state.clustering.write().await;
        let mut h = state.hebbian.write().await;

        let report = clustering.run_maintenance_cycle(&store.memories, &mut h, &u);
        drop(h);
        drop(clustering);
        drop(store);
        drop(u);

        tracing::info!(
            cycle,
            attractors = report.attractors,
            tunnels = report.tunnels_discovered,
            bridges = report.bridges_created,
            "cognitive controller: clustering done"
        );
    }

    if cfg.crystal_decay_enabled {
        let active_nodes: HashSet<crate::universe::coord::Coord7D> = {
            let u = state.universe.read().await;
            u.coords().into_iter().collect()
        };
        let mut crystal = state.crystal.write().await;
        let removed = crystal.decay_unused(&active_nodes);
        drop(crystal);

        if removed > 0 {
            tracing::info!(
                cycle,
                removed,
                "cognitive controller: crystal decay removed orphaned channels"
            );
        }
    }

    if cfg.regulation_enabled && mem_count > 100 {
        let mut u = state.universe.write().await;
        let mut h = state.hebbian.write().await;
        let mut crystal = state.crystal.write().await;
        let store = state.memory_store.read().await;

        let report =
            RegulationEngine::new().regulate(&mut u, &mut h, &mut crystal, &store.memories);
        drop(store);
        drop(crystal);
        drop(h);
        drop(u);

        state.event_sender.publish(UniverseEvent::RegulationCycle {
            stress_level: report.stress_level,
            entropy: report.entropy,
            actions_count: report.actions.len(),
        });

        if report.stress_level > 0.7 {
            tracing::warn!(
                cycle,
                stress = format!("{:.2}", report.stress_level),
                imbalance = format!("{:.2}", report.dimension_pressure.imbalance),
                "cognitive controller: high stress regulation"
            );
        }
    }

    if cfg.watchdog_enabled && cycle.is_multiple_of(5) {
        let mut u = state.universe.write().await;
        let mut h = state.hebbian.write().await;
        let mut crystal = state.crystal.write().await;
        let store = state.memory_store.read().await;
        let mut watchdog = state.watchdog.write().await;
        let mut backup = state.backup.write().await;

        let report = watchdog.checkup_with_backup(
            &mut u,
            &mut h,
            &mut crystal,
            &store.memories,
            &mut backup,
        );
        drop(backup);
        drop(watchdog);
        drop(store);
        drop(crystal);
        drop(h);
        drop(u);

        if report.level >= WatchdogLevel::Warning {
            tracing::warn!(
                cycle,
                level = format!("{:?}", report.level),
                utilization = format!("{:.1}%", report.utilization * 100.0),
                conservation_ok = report.conservation_ok,
                "cognitive controller: watchdog"
            );
        }
    }

    if cfg.event_drain_enabled {
        let mut events = state.events.lock().await;
        let drained = events.drain();
        drop(events);

        if drained > 0 {
            tracing::debug!(cycle, drained, "cognitive controller: events drained");
        }
    }

    if cfg.interest_ttl_enabled {
        interest_ttl_cleanup(state, cfg).await;
    }

    run_prediction_surprise_cycle(state, &mut ctrl.spontaneous).await;

    if let Some(ref mut drive) = ctrl.spontaneous {
        drive
            .run_cycle_with_state(state, spontaneous_cfg, vigor, &cognitive_state)
            .await;
    }

    tracing::debug!(
        cycle,
        elapsed_ms = format!("{:.0}", start.elapsed().as_secs_f64() * 1000.0),
        vigor = format!("{:.3}", vigor),
        dream = should_dream,
        "cognitive controller: inner cycle done"
    );

    Ok(())
}

async fn run_prediction_surprise_cycle(
    state: &Arc<AppState>,
    drive: &mut Option<SpontaneousDrive>,
) {
    use crate::universe::cognitive::prediction::PredictionEngine;
    use crate::universe::cognitive::surprise::SurpriseComputer;

    let hebbian = state.hebbian.read().await;
    let store = state.memory_store.read().await;

    let active_anchors: Vec<crate::universe::coord::Coord7D> =
        store.memories.iter().map(|m| *m.anchor()).collect();
    drop(store);

    if active_anchors.is_empty() {
        drop(hebbian);
        return;
    }

    let predictions = PredictionEngine::generate_predictions(&hebbian, &active_anchors);
    drop(hebbian);

    let mut pred_state = state.prediction.write().await;

    let hebbian = state.hebbian.read().await;
    let old_predictions: Vec<_> = pred_state.predictions().values().cloned().collect();
    let mut cycle_surprises: Vec<f64> = Vec::new();

    for old_pred in &old_predictions {
        let actual_successors = hebbian.get_successors(&old_pred.source);
        if actual_successors.is_empty() {
            continue;
        }
        let surprise = PredictionEngine::compute_surprise(
            &mut pred_state,
            &old_pred.source,
            &actual_successors,
        );
        cycle_surprises.push(surprise);
    }
    drop(hebbian);

    if !cycle_surprises.is_empty() {
        let avg: f64 = cycle_surprises.iter().sum::<f64>() / cycle_surprises.len() as f64;
        let high_count = cycle_surprises.iter().filter(|&&s| s > 0.5).count();
        tracing::info!(
            validated = cycle_surprises.len(),
            avg_cycle_surprise = format!("{:.4}", avg),
            high_surprise_count = high_count,
            running_avg = format!("{:.4}", pred_state.avg_surprise()),
            accuracy = format!("{:.1}%", pred_state.prediction_accuracy() * 100.0),
            "prediction-surprise cycle: validated old predictions against reality"
        );
    }

    PredictionEngine::update_predictions(&mut pred_state, predictions.clone());

    for pred in &predictions {
        state.event_sender.publish(
            crate::universe::safety::events::UniverseEvent::PredictionMade {
                source: pred.source.basis(),
                predicted_count: pred.predicted_next.len(),
                confidence: pred.confidence,
            },
        );
    }

    let mut hebbian = state.hebbian.write().await;
    let store = state.memory_store.read().await;
    let adjustments = SurpriseComputer::apply_prediction_error_correction(
        &mut hebbian,
        &pred_state,
        &store.memories,
    );
    drop(store);
    drop(hebbian);

    if adjustments > 0 {
        state.event_sender.publish(
            crate::universe::safety::events::UniverseEvent::PredictionCorrected {
                hebbian_adjustments: adjustments,
                accuracy: pred_state.prediction_accuracy(),
            },
        );
    }

    if let Some(ref mut d) = drive {
        let uncertainty = PredictionEngine::find_high_uncertainty(&pred_state);
        d.on_correction_signal(uncertainty);
    }

    if pred_state.avg_surprise() > 0.5 {
        tracing::info!(
            avg_surprise = format!("{:.3}", pred_state.avg_surprise()),
            accuracy = format!("{:.1}%", pred_state.prediction_accuracy() * 100.0),
            predictions = pred_state.active_prediction_count(),
            "prediction-surprise cycle: high surprise detected"
        );
    }

    drop(pred_state);
}

async fn restore_identity_importance(state: &Arc<AppState>) {
    let guard = state.identity_guard.read().await;
    let mut store = state.memory_store.write().await;
    for mem in store.memories.iter_mut() {
        if guard.is_identity_memory(mem) && mem.importance() < IDENTITY_RESTORE_MIN {
            mem.set_importance(IDENTITY_RESTORE_MIN);
            tracing::debug!(
                anchor = %format!("{}", mem.anchor()),
                restored_to = IDENTITY_RESTORE_MIN,
                "identity guard: restored decayed identity memory importance"
            );
        }
    }
    drop(store);
    drop(guard);
}

async fn auto_forget_step(
    state: &Arc<AppState>,
    cfg: &MaintenanceConfig,
    cycle: u64,
    ctrl: &mut ControllerState,
) {
    let mut u = state.universe.write().await;
    let guard = state.identity_guard.read().await;
    let mut store = state.memory_store.write().await;

    let mut to_erase: Vec<usize> = Vec::new();
    let mut over_limit_count: usize = 0;

    for (i, mem) in store.memories.iter().enumerate() {
        let anchor_str = format!("{}", mem.anchor());

        if mem.importance() < FORGET_THRESHOLD {
            if guard.is_identity_memory(mem) {
                continue;
            }
            let count = ctrl.forget_tracker.entry(anchor_str.clone()).or_insert(0);
            *count += 1;
            if *count >= cfg.auto_forget_grace_cycles {
                to_erase.push(i);
            }
        } else {
            ctrl.forget_tracker.remove(&anchor_str);
        }
    }

    if store.memories.len() > cfg.max_memories {
        let excess = store.memories.len() - cfg.max_memories;
        let mut candidates: Vec<(usize, f64)> = store
            .memories
            .iter()
            .enumerate()
            .filter(|(i, m)| !to_erase.contains(i) && !guard.is_identity_memory(m))
            .map(|(i, m)| (i, m.importance()))
            .collect();
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        for (i, _) in candidates.into_iter().take(excess) {
            to_erase.push(i);
        }
        over_limit_count = excess;
    }
    drop(guard);

    if !to_erase.is_empty() {
        to_erase.sort_unstable();
        to_erase.dedup();

        for &i in to_erase.iter().rev() {
            if i < store.memories.len() {
                MemoryCodec::erase(&mut u, &store.memories[i]);
                let anchor_str = format!("{}", store.memories[i].anchor());
                ctrl.forget_tracker.remove(&anchor_str);
                store.remove_at(i);
            }
        }

        tracing::info!(
            cycle,
            erased = to_erase.len(),
            remaining = store.memories.len(),
            over_limit = over_limit_count,
            "cognitive controller: auto-forget erased memories"
        );
    }

    drop(store);
    drop(u);
}

async fn interest_ttl_cleanup(state: &Arc<AppState>, cfg: &MaintenanceConfig) {
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut interests = state.interests.write().await;
    let before = interests.len();

    interests.retain(|_id, profile| !profile.is_expired(now_secs));

    let expired = before - interests.len();

    if interests.len() > cfg.max_interests {
        let mut entries: Vec<_> = interests.iter().collect();
        entries.sort_by_key(|(_, p)| p.registered_at);
        let to_remove = entries.len() - cfg.max_interests;
        let oldest: Vec<String> = entries
            .iter()
            .take(to_remove)
            .map(|(id, _)| id.to_string())
            .collect();
        for id in oldest {
            interests.remove(&id);
        }
        tracing::warn!(
            expired,
            evicted = to_remove,
            remaining = interests.len(),
            "cognitive controller: interest TTL cleanup evicted excess interests"
        );
    } else if expired > 0 {
        tracing::info!(
            expired,
            remaining = interests.len(),
            "cognitive controller: interest TTL cleanup removed expired"
        );
    }

    drop(interests);
}

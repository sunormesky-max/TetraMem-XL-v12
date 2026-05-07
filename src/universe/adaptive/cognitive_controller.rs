// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use crate::universe::api::AppState;
use crate::universe::cognitive::cognitive_state::CognitiveStateEngine;
use crate::universe::config::MaintenanceConfig;
use crate::universe::dream::DreamEngine;
use crate::universe::events::UniverseEvent;
use crate::universe::memory::aging::AgingEngine;
use crate::universe::regulation::RegulationEngine;
use crate::universe::watchdog::WatchdogLevel;

const WEAK_EDGE_THRESHOLD: f64 = 0.15;
const WEAK_EDGE_BOOST: f64 = 0.05;
const MAX_WEAK_REINFORCE: usize = 20;

pub fn spawn_cognitive_controller(state: Arc<AppState>) -> Option<tokio::task::JoinHandle<()>> {
    let cfg = state.config.maintenance.clone();
    if !cfg.enabled {
        tracing::info!("cognitive controller: disabled by config");
        return None;
    }

    let interval_secs = cfg.interval_secs.max(30);
    tracing::info!(
        "cognitive controller: spawning (interval={}s, dream_urgency>={:.2})",
        interval_secs,
        cfg.dream_min_urgency,
    );

    let handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        interval.tick().await;
        let mut cycle: u64 = 0;
        loop {
            interval.tick().await;
            cycle += 1;
            run_maintenance_cycle(&state, &cfg, cycle).await;
        }
    });

    Some(handle)
}

async fn run_maintenance_cycle(state: &Arc<AppState>, cfg: &MaintenanceConfig, cycle: u64) {
    let start = std::time::Instant::now();

    let cognitive_state = {
        let u = state.universe.read().await;
        let h = state.hebbian.read().await;
        let mems = state.memories.read().await;
        CognitiveStateEngine::assess(&u, &h, &mems)
    };

    let mem_count = {
        let mems = state.memories.read().await;
        mems.len()
    };

    if mem_count == 0 {
        return;
    }

    let should_dream = cognitive_state.dream_readiness.should_dream
        && cognitive_state.dream_readiness.urgency >= cfg.dream_min_urgency;
    let vigor = cognitive_state.overall_vigor;

    if should_dream {
        let mut h = state.hebbian.write().await;
        let mems = state.memories.read().await;

        let mut reinforced = 0usize;
        for mem in mems.iter() {
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
        drop(mems);
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
        let mems = state.memories.read().await;
        let mut h = state.hebbian.write().await;

        let dream = DreamEngine::new();
        let report = dream.dream(&u, &mut h, &mems);
        drop(mems);
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
            "cognitive controller: dream completed — replayed={}, weakened={}, consolidated={}, edges {}→{}",
            report.paths_replayed,
            report.paths_weakened,
            report.memories_consolidated,
            report.hebbian_edges_before,
            report.hebbian_edges_after,
        );
    }

    if cfg.aging_enabled {
        let accessed: Vec<String> = {
            let mems = state.memories.read().await;
            mems.iter()
                .filter(|m| m.importance() > 0.5)
                .map(|m| format!("{}", m.anchor()))
                .collect()
        };
        let mut mems = state.memories.write().await;
        let report = AgingEngine::default().age(&mut mems, &accessed);
        drop(mems);

        if report.flagged_for_forget > 0 {
            tracing::warn!(
                cycle,
                flagged = report.flagged_for_forget,
                "cognitive controller: aging flagged memories for potential forget"
            );
        }
    }

    if cfg.clustering_enabled {
        let u = state.universe.read().await;
        let mems = state.memories.read().await;
        let mut clustering = state.clustering.write().await;
        let mut h = state.hebbian.write().await;

        let report = clustering.run_maintenance_cycle(&mems, &mut h, &u);
        drop(h);
        drop(clustering);
        drop(mems);
        drop(u);

        tracing::info!(
            cycle,
            attractors = report.attractors,
            tunnels = report.tunnels_discovered,
            bridges = report.bridges_created,
            "cognitive controller: clustering maintenance done"
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
        let mems = state.memories.read().await;

        let report = RegulationEngine::new().regulate(&mut u, &mut h, &mut crystal, &mems);
        drop(mems);
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
                "cognitive controller: high stress regulation applied"
            );
        }
    }

    if cfg.watchdog_enabled && cycle.is_multiple_of(5) {
        let mut u = state.universe.write().await;
        let mut h = state.hebbian.write().await;
        let mut crystal = state.crystal.write().await;
        let mems = state.memories.read().await;
        let mut watchdog = state.watchdog.write().await;
        let mut backup = state.backup.write().await;

        let report = watchdog.checkup_with_backup(&mut u, &mut h, &mut crystal, &mems, &mut backup);
        drop(backup);
        drop(watchdog);
        drop(mems);
        drop(crystal);
        drop(h);
        drop(u);

        if report.level >= WatchdogLevel::Warning {
            tracing::warn!(
                cycle,
                level = format!("{:?}", report.level),
                utilization = format!("{:.1}%", report.utilization * 100.0),
                conservation_ok = report.conservation_ok,
                "cognitive controller: watchdog checkup"
            );
        }
    }

    if cfg.event_drain_enabled {
        let mut events = state.events.lock().await;
        let drained = events.drain();
        drop(events);

        if drained > 0 {
            tracing::debug!(cycle, drained, "cognitive controller: event bus drained");
        }
    }

    let elapsed = start.elapsed();
    tracing::info!(
        cycle,
        elapsed_ms = elapsed.as_secs_f64() * 1000.0,
        vigor = format!("{:.3}", vigor),
        dream = should_dream,
        "cognitive controller: maintenance cycle complete"
    );
}

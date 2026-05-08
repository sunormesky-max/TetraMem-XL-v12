// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use std::sync::Arc;

use crate::universe::api::AppState;
use crate::universe::cognitive::cognitive_state::CognitiveState;
use crate::universe::cognitive::meta_cognitive::MetaCognitiveEngine;
use crate::universe::config::SpontaneousConfig;
use crate::universe::coord::Coord7D;
use crate::universe::events::UniverseEvent;
use crate::universe::memory::pulse::PulseEngine;
use crate::universe::memory::pulse::PulseType;

const MAX_PULSE_PER_CYCLE: usize = 3;
const MAX_RECALL_PER_CYCLE: usize = 5;
const MAX_CUROSITY_PROBES: usize = 3;
const MIN_VIGOR_FOR_PULSE: f64 = 0.15;
const MIN_VIGOR_FOR_RECALL: f64 = 0.10;

#[derive(Debug, Clone)]
pub struct DriveMetrics {
    pub pulses_fired: u64,
    pub discoveries_made: u64,
    pub recalls_performed: u64,
    pub contradictions_found: u64,
    pub probes_launched: u64,
    pub self_assessments: u64,
    pub events_reacted: u64,
    pub curiosity_level: f64,
    pub last_assessment_vigor: f64,
}

impl Default for DriveMetrics {
    fn default() -> Self {
        Self {
            pulses_fired: 0,
            discoveries_made: 0,
            recalls_performed: 0,
            contradictions_found: 0,
            probes_launched: 0,
            self_assessments: 0,
            events_reacted: 0,
            curiosity_level: 0.5,
            last_assessment_vigor: 0.0,
        }
    }
}

pub struct SpontaneousDrive {
    pub metrics: DriveMetrics,
    pub curiosity: f64,
    pub reflection_drive: f64,
    pub exploration_drive: f64,
    recent_pulse_origins: Vec<[i32; 7]>,
    recall_history: Vec<String>,
    probe_targets: Vec<String>,
}

impl SpontaneousDrive {
    pub fn new(config: &SpontaneousConfig) -> Self {
        Self {
            metrics: DriveMetrics::default(),
            curiosity: config.base_curiosity,
            reflection_drive: config.base_reflection,
            exploration_drive: config.base_exploration,
            recent_pulse_origins: Vec::new(),
            recall_history: Vec::new(),
            probe_targets: Vec::new(),
        }
    }

    pub async fn run_cycle_with_state(
        &mut self,
        state: &Arc<AppState>,
        config: &SpontaneousConfig,
        vigor: f64,
        _cognitive_state: &CognitiveState,
    ) {
        let mem_count = {
            let store = state.memory_store.read().await;
            store.memories.len()
        };

        if mem_count == 0 {
            return;
        }

        self.metrics.last_assessment_vigor = vigor;

        self.assess_and_adjust(vigor);

        if config.pulse_enabled && vigor >= MIN_VIGOR_FOR_PULSE {
            self.spontaneous_pulse(state).await;
        }

        if config.recall_enabled && vigor >= MIN_VIGOR_FOR_RECALL {
            self.spontaneous_recall(state).await;
        }

        if config.curiosity_enabled {
            self.curiosity_probe(state).await;
        }

        if config.event_reaction_enabled {
            self.event_reactor(state).await;
        }

        self.metrics.self_assessments += 1;

        if self.recent_pulse_origins.len() > 100 {
            self.recent_pulse_origins.drain(0..50);
        }
        if self.recall_history.len() > 200 {
            self.recall_history.drain(0..100);
        }
        if self.probe_targets.len() > 100 {
            self.probe_targets.drain(0..50);
        }

        tracing::debug!(
            curiosity = format!("{:.3}", self.curiosity),
            reflection = format!("{:.3}", self.reflection_drive),
            exploration = format!("{:.3}", self.exploration_drive),
            pulses = self.metrics.pulses_fired,
            recalls = self.metrics.recalls_performed,
            probes = self.metrics.probes_launched,
            "spontaneous drive: cycle complete"
        );
    }

    fn assess_and_adjust(&mut self, vigor: f64) {
        let curiosity_shift = (vigor - 0.5) * 0.1;
        self.curiosity = (self.curiosity + curiosity_shift).clamp(0.05, 1.0);

        self.exploration_drive = (self.exploration_drive * 0.95 + vigor * 0.05).clamp(0.05, 1.0);

        let contradictions_ratio = if self.metrics.recalls_performed > 0 {
            self.metrics.contradictions_found as f64 / self.metrics.recalls_performed as f64
        } else {
            0.0
        };
        self.reflection_drive =
            (self.reflection_drive + contradictions_ratio * 0.1).clamp(0.05, 1.0);

        self.metrics.curiosity_level = self.curiosity;
    }

    async fn spontaneous_pulse(&mut self, state: &Arc<AppState>) {
        let (seeds, cold_zones) =
            {
                let u = state.universe.read().await;
                let h = state.hebbian.read().await;
                let store = state.memory_store.read().await;
                let attention = crate::universe::cognitive::attention::AttentionEngine::new()
                    .compute(&u, &h, &store.memories);
                let seeds = attention.recommendation.suggested_pulse_anchors;
                let cold = attention.recommendation.cold_zones;
                (seeds, cold)
            };

        let mut origins: Vec<[i32; 7]> = Vec::new();

        if self.curiosity > 0.6 && !cold_zones.is_empty() {
            let idx = (self.metrics.pulses_fired as usize) % cold_zones.len();
            origins.push(cold_zones[idx]);
        }

        for seed in seeds.iter().take(MAX_PULSE_PER_CYCLE - origins.len()) {
            origins.push(*seed);
        }

        for origin_basis in &origins {
            let coord = Coord7D::new_even(*origin_basis);
            let pulse_type = if self.curiosity > 0.7 {
                PulseType::Exploratory
            } else if self.curiosity > 0.4 {
                PulseType::Reinforcing
            } else {
                PulseType::Cascade
            };

            let visited = {
                let u = state.universe.read().await;
                let mut h = state.hebbian.write().await;
                let engine = PulseEngine::new();
                let report = engine.propagate(&coord, pulse_type, &u, &mut h);
                drop(h);
                drop(u);
                report.visited_nodes
            };

            self.metrics.pulses_fired += 1;
            if visited > 5 {
                self.metrics.discoveries_made += 1;
            }

            self.recent_pulse_origins.push(*origin_basis);

            state.event_sender.publish(UniverseEvent::PulseCompleted {
                source: *origin_basis,
                pulse_type: "spontaneous".to_string(),
                visited_nodes: visited,
                paths_recorded: 0,
            });

            tracing::debug!(
                origin = format!("{:?}", origin_basis),
                pulse_type = format!("{:?}", pulse_type),
                visited,
                "spontaneous pulse: curiosity-driven exploration"
            );
        }
    }

    async fn spontaneous_recall(&mut self, state: &Arc<AppState>) {
        let mem_count = {
            let store = state.memory_store.read().await;
            store.memories.len()
        };

        if mem_count < 2 {
            return;
        }

        let candidates = {
            let store = state.memory_store.read().await;
            let h = state.hebbian.read().await;
            let mut scored: Vec<(usize, f64)> = Vec::new();
            for (i, mem) in store.memories.iter().enumerate() {
                let anchor_str = format!("{}", mem.anchor());
                if self.recall_history.contains(&anchor_str) {
                    continue;
                }
                let importance = mem.importance();
                let connectivity = (h.get_neighbors(mem.anchor()).len() as f64).ln_1p();
                let age_factor = {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    let age_ms = now.saturating_sub(mem.created_at());
                    (age_ms as f64 / 3_600_000.0).min(1.0)
                };
                let score = importance * 0.4
                    + age_factor * 0.2
                    + connectivity * 0.2
                    + self.reflection_drive * 0.2;
                scored.push((i, score));
            }
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            scored
                .into_iter()
                .take(MAX_RECALL_PER_CYCLE)
                .collect::<Vec<_>>()
        };

        if candidates.is_empty() {
            return;
        }

        let contradictions = {
            let store = state.memory_store.read().await;
            let h = state.hebbian.read().await;
            let mut found = 0usize;
            for &(idx, _score) in &candidates {
                let mem = &store.memories[idx];
                let neighbors = h.get_neighbors(mem.anchor());
                for (coord, _weight) in &neighbors {
                    if let Some(other_idx) = store.memories.iter().position(|m| m.anchor() == coord)
                    {
                        let other = &store.memories[other_idx];
                        let desc_a = mem.description().unwrap_or("");
                        let desc_b = other.description().unwrap_or("");
                        if crate::universe::memory::contradiction::descriptions_conflict(
                            Some(desc_a),
                            Some(desc_b),
                        ) {
                            found += 1;
                            break;
                        }
                    }
                }
            }
            found
        };

        if contradictions > 0 {
            self.metrics.contradictions_found += contradictions as u64;
            tracing::warn!(
                contradictions,
                "spontaneous recall: found contradictions during self-reflection"
            );
        }

        for &(idx, _) in &candidates {
            let store = state.memory_store.read().await;
            if let Some(mem) = store.memories.get(idx) {
                let anchor_str = format!("{}", mem.anchor());
                self.recall_history.push(anchor_str);
            }
        }

        self.metrics.recalls_performed += candidates.len() as u64;
    }

    async fn curiosity_probe(&mut self, state: &Arc<AppState>) {
        let self_model = {
            let u = state.universe.read().await;
            let h = state.hebbian.read().await;
            let store = state.memory_store.read().await;
            MetaCognitiveEngine::assess(&u, &h, &store.memories)
        };

        let blind_spots = &self_model.blind_spots;
        let unknown_areas = &self_model.unknown_areas;

        let mut probes = 0usize;
        let max_probes = (self.curiosity * MAX_CUROSITY_PROBES as f64).ceil() as usize;

        for spot in blind_spots.iter().take(max_probes) {
            if self.probe_targets.contains(spot) {
                continue;
            }
            self.probe_targets.push(spot.clone());
            probes += 1;
        }

        for area in unknown_areas.iter().take(max_probes - probes) {
            if self.probe_targets.contains(area) {
                continue;
            }
            self.probe_targets.push(area.clone());
            probes += 1;
        }

        if probes > 0 {
            self.metrics.probes_launched += probes as u64;

            let low_conf_tags: Vec<&str> = self_model
                .known_domains
                .iter()
                .filter(|d| d.confidence < 0.4)
                .take(3)
                .map(|d| d.tag.as_str())
                .collect();

            if !low_conf_tags.is_empty() {
                let mut store = state.memory_store.write().await;
                for mem in store.memories.iter_mut() {
                    for tag in &low_conf_tags {
                        if mem.tags().contains(&tag.to_string()) {
                            let boost = 0.02 * self.curiosity;
                            let new_imp = (mem.importance() + boost).min(1.0);
                            mem.set_importance(new_imp);
                        }
                    }
                }
                drop(store);
            }

            tracing::debug!(
                probes,
                blind_spots = blind_spots.len(),
                unknown_areas = unknown_areas.len(),
                low_conf_domains = low_conf_tags.len(),
                "curiosity probe: exploring knowledge gaps"
            );
        }
    }

    async fn event_reactor(&mut self, state: &Arc<AppState>) {
        let events = state.events.lock().await;
        let history: Vec<_> = events.history().into_iter().cloned().collect();
        let recent_len = history.len();

        if recent_len == 0 {
            drop(events);
            return;
        }

        let window_start = recent_len.saturating_sub(10);
        let mut high_stress_count = 0usize;
        let mut dream_count = 0usize;
        let mut violation_count = 0usize;

        for event in &history[window_start..] {
            match event {
                UniverseEvent::RegulationCycle { stress_level, .. } if *stress_level > 0.7 => {
                    high_stress_count += 1;
                }
                UniverseEvent::DreamCompleted { .. } => {
                    dream_count += 1;
                }
                UniverseEvent::ConservationViolation { .. } => {
                    violation_count += 1;
                }
                _ => {}
            }
        }
        drop(events);

        if high_stress_count > 0 {
            self.exploration_drive = (self.exploration_drive * 0.8).max(0.1);
            self.reflection_drive = (self.reflection_drive + 0.1).min(1.0);
            tracing::info!(
                high_stress_count,
                "event reactor: high stress detected, reducing exploration, increasing reflection"
            );
        }

        if dream_count > 0 {
            self.curiosity = (self.curiosity + 0.05).min(1.0);
            tracing::debug!(
                dream_count,
                "event reactor: dreams completed, boosting curiosity"
            );
        }

        if violation_count > 0 {
            self.exploration_drive = (self.exploration_drive * 0.5).max(0.05);
            tracing::warn!(
                violation_count,
                "event reactor: conservation violations, reducing exploration"
            );
        }

        self.metrics.events_reacted +=
            high_stress_count as u64 + dream_count as u64 + violation_count as u64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::config::SpontaneousConfig;

    fn default_config() -> SpontaneousConfig {
        SpontaneousConfig::default()
    }

    #[test]
    fn drive_metrics_default() {
        let m = DriveMetrics::default();
        assert_eq!(m.pulses_fired, 0);
        assert_eq!(m.curiosity_level, 0.5);
    }

    #[test]
    fn assess_and_adjust_high_vigor() {
        let config = default_config();
        let mut drive = SpontaneousDrive::new(&config);
        drive.assess_and_adjust(0.8);
        assert!(drive.curiosity > config.base_curiosity);
        assert!(drive.exploration_drive > 0.0);
    }

    #[test]
    fn assess_and_adjust_low_vigor() {
        let config = default_config();
        let mut drive = SpontaneousDrive::new(&config);
        drive.assess_and_adjust(0.1);
        assert!(drive.curiosity < config.base_curiosity);
    }

    #[test]
    fn assess_and_adjust_curiosity_bounded() {
        let config = default_config();
        let mut drive = SpontaneousDrive::new(&config);
        for _ in 0..1000 {
            drive.assess_and_adjust(1.0);
        }
        assert!(drive.curiosity <= 1.0);
        assert!(drive.curiosity >= 0.0);
    }

    #[test]
    fn assess_with_contradictions_boosts_reflection() {
        let config = default_config();
        let mut drive = SpontaneousDrive::new(&config);
        drive.metrics.recalls_performed = 10;
        drive.metrics.contradictions_found = 8;
        let before = drive.reflection_drive;
        drive.assess_and_adjust(0.5);
        assert!(drive.reflection_drive >= before);
    }

    #[test]
    fn event_reactor_logic() {
        let config = default_config();
        let mut drive = SpontaneousDrive::new(&config);
        drive.exploration_drive = 0.8;
        let before = drive.exploration_drive;

        drive.exploration_drive = (drive.exploration_drive * 0.8).max(0.1);
        assert!(drive.exploration_drive < before);
    }

    #[test]
    fn curiosity_history_truncation() {
        let config = default_config();
        let mut drive = SpontaneousDrive::new(&config);
        for i in 0..120 {
            drive.recent_pulse_origins.push([i, 0, 0, 0, 0, 0, 0]);
        }
        assert_eq!(drive.recent_pulse_origins.len(), 120);
        drive.recent_pulse_origins.drain(0..50);
        assert!(drive.recent_pulse_origins.len() < 120);
    }
}

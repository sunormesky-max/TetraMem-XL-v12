// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::cognitive::attention::AttentionEngine;
use crate::universe::cognitive::emotion::EmotionReport;
use crate::universe::cognitive::topology::TopologyEngine;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::aging::AgingEngine;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveState {
    pub emotion: EmotionSnapshot,
    pub attention_summary: AttentionSummary,
    pub topology_summary: TopologySummary,
    pub memory_health: MemoryHealth,
    pub pulse_recommendation: PulseRecommendation,
    pub dream_readiness: DreamReadiness,
    pub overall_vigor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionSnapshot {
    pub pleasure: f64,
    pub arousal: f64,
    pub dominance: f64,
    pub quadrant: String,
    pub functional_cluster: String,
    pub magnitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionSummary {
    pub hotspot_count: usize,
    pub total_heat: f64,
    pub coverage_ratio: f64,
    pub top_focus: Option<[i32; 7]>,
    pub cold_zone_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologySummary {
    pub connected_components: usize,
    pub betti_h0: usize,
    pub betti_h1: usize,
    pub euler_characteristic: i64,
    pub bridging_nodes: usize,
    pub isolated_nodes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHealth {
    pub total_memories: usize,
    pub hebbian_edges: usize,
    pub avg_importance: f64,
    pub flagged_for_forget: usize,
    pub contradiction_candidates: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseRecommendation {
    pub strategy: String,
    pub suggested_seeds: Vec<[i32; 7]>,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamReadiness {
    pub should_dream: bool,
    pub urgency: f64,
    pub reason: String,
}

pub struct CognitiveStateEngine;

impl CognitiveStateEngine {
    pub fn assess(
        universe: &DarkUniverse,
        hebbian: &HebbianMemory,
        memories: &[MemoryAtom],
    ) -> CognitiveState {
        let emotion_report = EmotionReport::analyze(universe);
        let attention_map = AttentionEngine::new().compute(universe, hebbian, memories);
        let topology_report = TopologyEngine::analyze(universe);

        let emotion = EmotionSnapshot {
            pleasure: emotion_report.pad.pleasure,
            arousal: emotion_report.pad.arousal,
            dominance: emotion_report.pad.dominance,
            quadrant: format!("{}", emotion_report.quadrant),
            functional_cluster: emotion_report.functional_cluster.clone(),
            magnitude: emotion_report.pad.magnitude(),
        };

        let attention_summary = AttentionSummary {
            hotspot_count: attention_map.hotspots.len(),
            total_heat: attention_map.total_heat,
            coverage_ratio: attention_map.coverage_ratio,
            top_focus: attention_map.hotspots.first().map(|h| h.anchor_basis),
            cold_zone_count: attention_map.recommendation.cold_zones.len(),
        };

        let topology_summary = TopologySummary {
            connected_components: topology_report.connected_components,
            betti_h0: topology_report.betti.get(0),
            betti_h1: topology_report.betti.get(1),
            euler_characteristic: topology_report.betti.euler_characteristic(),
            bridging_nodes: topology_report.bridging_nodes,
            isolated_nodes: topology_report.isolated_nodes,
        };

        let aging = AgingEngine::default();
        let flagged: Vec<usize> = aging
            .flagged_memories(memories)
            .into_iter()
            .map(|(idx, _)| idx)
            .collect();

        let avg_importance = if memories.is_empty() {
            0.0
        } else {
            memories.iter().map(|m| m.importance()).sum::<f64>() / memories.len() as f64
        };

        let memory_health = MemoryHealth {
            total_memories: memories.len(),
            hebbian_edges: hebbian.edge_count(),
            avg_importance,
            flagged_for_forget: flagged.len(),
            contradiction_candidates: 0,
        };

        let (strategy, reasoning) =
            Self::derive_pulse_strategy(&emotion_report, &attention_summary, &topology_summary);

        let suggested_seeds = attention_map
            .recommendation
            .suggested_pulse_anchors
            .into_iter()
            .take(3)
            .collect();

        let pulse_recommendation = PulseRecommendation {
            strategy,
            suggested_seeds,
            reasoning,
        };

        let dream_readiness =
            Self::assess_dream_readiness(&emotion_report, &memory_health, &topology_summary);

        let overall_vigor = Self::compute_vigor(&emotion, &attention_summary, &memory_health);

        CognitiveState {
            emotion,
            attention_summary,
            topology_summary,
            memory_health,
            pulse_recommendation,
            dream_readiness,
            overall_vigor,
        }
    }

    fn derive_pulse_strategy(
        emotion: &EmotionReport,
        attention: &AttentionSummary,
        topology: &TopologySummary,
    ) -> (String, String) {
        let mut reasons = Vec::new();

        let strategy = if emotion.pad.arousal > 0.5 {
            reasons.push("high arousal suggests exploration".to_string());
            "exploratory".to_string()
        } else if attention.cold_zone_count > attention.hotspot_count {
            reasons.push("more cold zones than hotspots".to_string());
            "cascade".to_string()
        } else if topology.isolated_nodes > topology.connected_components * 2 {
            reasons.push("many isolated nodes need bridging".to_string());
            "cascade".to_string()
        } else if emotion.pad.pleasure > 0.3 {
            reasons.push("positive state reinforces existing paths".to_string());
            "reinforcing".to_string()
        } else {
            reasons.push("balanced state".to_string());
            "balanced".to_string()
        };

        (strategy, reasons.join("; "))
    }

    fn assess_dream_readiness(
        emotion: &EmotionReport,
        health: &MemoryHealth,
        topology: &TopologySummary,
    ) -> DreamReadiness {
        let mut urgency = 0.0f64;
        let mut reasons = Vec::new();

        if health.flagged_for_forget > health.total_memories / 4 {
            urgency += 0.3;
            reasons.push("many memories flagged for forgetting");
        }
        if topology.isolated_nodes > 0 {
            urgency += 0.2;
            reasons.push("isolated nodes need consolidation");
        }
        if emotion.pad.pleasure < -0.3 {
            urgency += 0.2;
            reasons.push("negative state benefits from dream reflection");
        }
        if health.hebbian_edges > health.total_memories * 3 {
            urgency += 0.15;
            reasons.push("dense network needs pruning");
        }
        if health.avg_importance < 0.3 {
            urgency += 0.15;
            reasons.push("low average importance needs strengthening");
        }

        DreamReadiness {
            should_dream: urgency > 0.3,
            urgency: urgency.min(1.0),
            reason: if reasons.is_empty() {
                "system stable".to_string()
            } else {
                reasons.join("; ")
            },
        }
    }

    fn compute_vigor(
        emotion: &EmotionSnapshot,
        attention: &AttentionSummary,
        health: &MemoryHealth,
    ) -> f64 {
        let emotion_vigor = emotion.magnitude / 3.0f64.sqrt();
        let attention_vigor = if attention.hotspot_count > 0 {
            (attention.total_heat / attention.hotspot_count as f64).min(1.0)
        } else {
            0.0
        };
        let connectivity_vigor = if health.total_memories > 0 {
            (health.hebbian_edges as f64 / health.total_memories as f64)
                .ln_1p()
                .min(1.0)
        } else {
            0.0
        };

        (emotion_vigor * 0.35 + attention_vigor * 0.35 + connectivity_vigor * 0.30).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::coord::Coord7D;
    use crate::universe::memory::MemoryCodec;

    #[test]
    fn empty_universe_state() {
        let u = DarkUniverse::new(1000.0);
        let h = HebbianMemory::new();
        let mems = Vec::new();
        let state = CognitiveStateEngine::assess(&u, &h, &mems);
        assert_eq!(state.memory_health.total_memories, 0);
        assert!(state.overall_vigor >= 0.0);
    }

    #[test]
    fn state_with_memories() {
        let mut u = DarkUniverse::new(1_000_000.0);
        let mut h = HebbianMemory::new();
        let mut mems = Vec::new();
        for i in 0..5i32 {
            let a = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            let m = MemoryCodec::encode(&mut u, &a, &[i as f64]).unwrap();
            mems.push(m);
        }
        h.boost_edge(
            &Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]),
            &Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]),
            0.7,
        );
        let state = CognitiveStateEngine::assess(&u, &h, &mems);
        assert_eq!(state.memory_health.total_memories, 5);
        assert!(state.memory_health.hebbian_edges > 0);
        assert!(state.overall_vigor > 0.0);
    }

    #[test]
    fn emotion_snapshot_fields() {
        let u = DarkUniverse::new(1000.0);
        let h = HebbianMemory::new();
        let state = CognitiveStateEngine::assess(&u, &h, &[]);
        assert!(!state.emotion.quadrant.is_empty());
    }

    #[test]
    fn pulse_recommendation_with_cold_zones() {
        let mut u = DarkUniverse::new(1_000_000.0);
        let h = HebbianMemory::new();
        let mut mems = Vec::new();
        let a = Coord7D::new_even([50, 50, 50, 0, 0, 0, 0]);
        let m = MemoryCodec::encode(&mut u, &a, &[1.0]).unwrap();
        mems.push(m);
        let state = CognitiveStateEngine::assess(&u, &h, &mems);
        assert!(!state.pulse_recommendation.strategy.is_empty());
    }

    #[test]
    fn dream_readiness_empty() {
        let u = DarkUniverse::new(1000.0);
        let h = HebbianMemory::new();
        let state = CognitiveStateEngine::assess(&u, &h, &[]);
        assert!(!state.dream_readiness.should_dream);
    }

    #[test]
    fn cognitive_state_serde() {
        let u = DarkUniverse::new(1000.0);
        let h = HebbianMemory::new();
        let state = CognitiveStateEngine::assess(&u, &h, &[]);
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("overall_vigor"));
        let back: CognitiveState = serde_json::from_str(&json).unwrap();
        assert!(!back.emotion.quadrant.is_empty());
    }
}

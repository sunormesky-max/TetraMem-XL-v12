// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::constitution::Constitution;
use crate::universe::crystal::CrystalEngine;
use crate::universe::events::EventBusSender;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentKind {
    Pulse,
    Dream,
    Crystal,
    Reasoning,
    Observer,
    Regulation,
    Emotion,
}

impl fmt::Display for AgentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pulse => write!(f, "PulseAgent"),
            Self::Dream => write!(f, "DreamAgent"),
            Self::Crystal => write!(f, "CrystalAgent"),
            Self::Reasoning => write!(f, "ReasoningAgent"),
            Self::Observer => write!(f, "ObserverAgent"),
            Self::Regulation => write!(f, "RegulationAgent"),
            Self::Emotion => write!(f, "EmotionAgent"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReport {
    pub agent: AgentKind,
    pub success: bool,
    pub duration_ms: f64,
    pub details: String,
}

impl fmt::Display for AgentReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.success { "OK" } else { "FAIL" };
        write!(
            f,
            "{}[{} {:.1}ms] {}",
            self.agent, status, self.duration_ms, self.details
        )
    }
}

pub struct AgentContext<'a> {
    pub universe: &'a DarkUniverse,
    pub hebbian: &'a HebbianMemory,
    pub memories: &'a [MemoryAtom],
    pub crystal: &'a CrystalEngine,
    pub constitution: &'a Constitution,
    pub event_sender: Option<&'a EventBusSender>,
}

pub struct AgentContextMut<'a> {
    pub universe: &'a mut DarkUniverse,
    pub hebbian: &'a mut HebbianMemory,
    pub memories: &'a mut Vec<MemoryAtom>,
    pub crystal: &'a mut CrystalEngine,
    pub constitution: &'a Constitution,
    pub event_sender: Option<&'a EventBusSender>,
}

pub trait CognitiveAgent: Send + Sync {
    fn kind(&self) -> AgentKind;

    fn name(&self) -> &str;

    fn execute_readonly(&self, ctx: &AgentContext) -> AgentReport {
        let _ = ctx;
        AgentReport {
            agent: self.kind(),
            success: true,
            duration_ms: 0.0,
            details: "readonly not implemented".into(),
        }
    }

    fn execute_mut(&self, ctx: &mut AgentContextMut) -> AgentReport {
        let _ = ctx;
        AgentReport {
            agent: self.kind(),
            success: true,
            duration_ms: 0.0,
            details: "mut not implemented".into(),
        }
    }

    fn should_run(&self, _ctx: &AgentContext) -> bool {
        true
    }
}

pub struct PulseAgent {
    source_coords: Vec<crate::universe::coord::Coord7D>,
    pulse_type: crate::universe::pulse::PulseType,
}

impl PulseAgent {
    pub fn new(
        source_coords: Vec<crate::universe::coord::Coord7D>,
        pulse_type: crate::universe::pulse::PulseType,
    ) -> Self {
        Self {
            source_coords,
            pulse_type,
        }
    }
}

impl CognitiveAgent for PulseAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Pulse
    }
    fn name(&self) -> &str {
        "PulseAgent"
    }

    fn execute_mut(&self, ctx: &mut AgentContextMut) -> AgentReport {
        let start = Instant::now();
        let engine = crate::universe::pulse::PulseEngine::new();
        let mut total_visited = 0usize;
        let mut total_paths = 0usize;

        for source in &self.source_coords {
            if ctx.universe.get_node(source).is_some() {
                let r = engine.propagate(source, self.pulse_type, ctx.universe, ctx.hebbian);
                total_visited += r.visited_nodes;
                total_paths += r.paths_recorded;
            }
        }

        AgentReport {
            agent: self.kind(),
            success: true,
            duration_ms: start.elapsed().as_secs_f64() * 1000.0,
            details: format!(
                "{} pulses, visited {} nodes, {} paths",
                self.source_coords.len(),
                total_visited,
                total_paths
            ),
        }
    }
}

pub struct DreamAgent {
    config: crate::universe::dream::DreamConfig,
}

impl DreamAgent {
    pub fn new(config: crate::universe::dream::DreamConfig) -> Self {
        Self { config }
    }
}

impl CognitiveAgent for DreamAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Dream
    }
    fn name(&self) -> &str {
        "DreamAgent"
    }

    fn execute_mut(&self, ctx: &mut AgentContextMut) -> AgentReport {
        let start = Instant::now();
        let engine = crate::universe::dream::DreamEngine::with_config(self.config.clone());
        let report = engine.dream_with_merge(ctx.universe, ctx.hebbian, ctx.memories);

        AgentReport {
            agent: self.kind(),
            success: true,
            duration_ms: start.elapsed().as_secs_f64() * 1000.0,
            details: format!("{}", report),
        }
    }
}

pub struct CrystalAgent;

impl CognitiveAgent for CrystalAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Crystal
    }
    fn name(&self) -> &str {
        "CrystalAgent"
    }

    fn execute_mut(&self, ctx: &mut AgentContextMut) -> AgentReport {
        let start = Instant::now();
        let report = ctx.crystal.crystallize(ctx.hebbian, ctx.universe);

        AgentReport {
            agent: self.kind(),
            success: true,
            duration_ms: start.elapsed().as_secs_f64() * 1000.0,
            details: format!("{}", report),
        }
    }
}

pub struct EmotionAgent;

impl CognitiveAgent for EmotionAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Emotion
    }
    fn name(&self) -> &str {
        "EmotionAgent"
    }

    fn execute_readonly(&self, ctx: &AgentContext) -> AgentReport {
        let start = Instant::now();
        let legacy_reading = crate::universe::emotion::EmotionMapper::read(ctx.universe);

        let pad = crate::universe::emotion::PadVector::new(
            legacy_reading.pad.pleasure,
            legacy_reading.pad.arousal,
            legacy_reading.pad.dominance,
        );
        let functional = crate::universe::functional_emotion::FunctionalEmotion::from_pad(
            pad,
            crate::universe::functional_emotion::EmotionSource::Perceived,
        );

        let func_count = ctx.hebbian.edges_by_emotion(
            crate::universe::functional_emotion::EmotionSource::Functional,
        ).len();
        let perc_count = ctx.hebbian.edges_by_emotion(
            crate::universe::functional_emotion::EmotionSource::Perceived,
        ).len();

        AgentReport {
            agent: self.kind(),
            success: true,
            duration_ms: start.elapsed().as_secs_f64() * 1000.0,
            details: format!(
                "{} cluster={} valence={:?} arousal={:?} edges[func={} perc={}]",
                legacy_reading, functional.cluster.name(), functional.valence,
                functional.arousal, func_count, perc_count,
            ),
        }
    }
}

pub struct ObserverAgent;

impl CognitiveAgent for ObserverAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Observer
    }
    fn name(&self) -> &str {
        "ObserverAgent"
    }

    fn execute_readonly(&self, ctx: &AgentContext) -> AgentReport {
        let start = Instant::now();
        let health = crate::universe::observer::UniverseObserver::inspect(
            ctx.universe,
            ctx.hebbian,
            ctx.memories,
        );
        let cons_ok = ctx.universe.verify_conservation();

        let mut issues = Vec::new();
        if !cons_ok {
            issues.push("CONSERVATION_VIOLATION".to_string());
        }
        if health.energy_utilization > 0.95 {
            issues.push("HIGH_UTILIZATION".to_string());
        }

        AgentReport {
            agent: self.kind(),
            success: issues.is_empty(),
            duration_ms: start.elapsed().as_secs_f64() * 1000.0,
            details: if issues.is_empty() {
                format!(
                    "healthy: {} nodes, util={:.1}%",
                    health.node_count,
                    health.energy_utilization * 100.0
                )
            } else {
                format!("issues: {}", issues.join(", "))
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::coord::Coord7D;

    fn build_context() -> (
        DarkUniverse,
        HebbianMemory,
        Vec<MemoryAtom>,
        CrystalEngine,
        Constitution,
    ) {
        let mut u = DarkUniverse::new(1_000_000.0);
        let h = HebbianMemory::new();
        let c = CrystalEngine::new();
        let constitution = Constitution::tetramem_default();

        for i in 0..10i32 {
            let coord = Coord7D::new_even([i * 10, 0, 0, 0, 0, 0, 0]);
            u.materialize_biased(coord, 100.0, 0.6).ok();
        }

        (u, h, Vec::new(), c, constitution)
    }

    #[test]
    fn emotion_agent_readonly() {
        let (u, h, mems, c, cons) = build_context();
        let agent = EmotionAgent;
        let ctx = AgentContext {
            universe: &u,
            hebbian: &h,
            memories: &mems,
            crystal: &c,
            constitution: &cons,
            event_sender: None,
        };
        let report = agent.execute_readonly(&ctx);
        assert!(report.success);
        assert!(report.details.contains("PAD"));
    }

    #[test]
    fn observer_agent() {
        let (u, h, mems, c, cons) = build_context();
        let agent = ObserverAgent;
        let ctx = AgentContext {
            universe: &u,
            hebbian: &h,
            memories: &mems,
            crystal: &c,
            constitution: &cons,
            event_sender: None,
        };
        let report = agent.execute_readonly(&ctx);
        assert!(report.success);
    }

    #[test]
    fn crystal_agent_mut() {
        let (mut u, mut h, _mems, mut c, cons) = build_context();
        let engine = crate::universe::pulse::PulseEngine::new();
        let src = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        if u.get_node(&src).is_some() {
            engine.propagate(
                &src,
                crate::universe::pulse::PulseType::Exploratory,
                &u,
                &mut h,
            );
        }

        let agent = CrystalAgent;
        let mut ctx = AgentContextMut {
            universe: &mut u,
            hebbian: &mut h,
            memories: &mut Vec::new(),
            crystal: &mut c,
            constitution: &cons,
            event_sender: None,
        };
        let report = agent.execute_mut(&mut ctx);
        assert!(report.success);
    }

    #[test]
    fn agent_kind_display() {
        assert_eq!(format!("{}", AgentKind::Pulse), "PulseAgent");
        assert_eq!(format!("{}", AgentKind::Dream), "DreamAgent");
        assert_eq!(format!("{}", AgentKind::Emotion), "EmotionAgent");
    }

    #[test]
    fn agent_report_display() {
        let report = AgentReport {
            agent: AgentKind::Observer,
            success: true,
            duration_ms: 12.5,
            details: "healthy: 10 nodes".into(),
        };
        let s = format!("{}", report);
        assert!(s.contains("OK"));
        assert!(s.contains("ObserverAgent"));
    }
}

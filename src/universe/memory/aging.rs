// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use serde::{Deserialize, Serialize};

use crate::universe::memory::MemoryAtom;

const DEFAULT_AGING_RATE: f64 = 0.95;
const DEFAULT_AGING_THRESHOLD: f64 = 0.05;
const DEFAULT_ACCESS_BOOST: f64 = 0.1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgingConfig {
    pub decay_rate: f64,
    pub forget_threshold: f64,
    pub access_boost: f64,
}

impl Default for AgingConfig {
    fn default() -> Self {
        Self {
            decay_rate: DEFAULT_AGING_RATE,
            forget_threshold: DEFAULT_AGING_THRESHOLD,
            access_boost: DEFAULT_ACCESS_BOOST,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AgingReport {
    pub aged_count: usize,
    pub flagged_for_forget: usize,
    pub boosted_count: usize,
    pub min_importance: f64,
    pub avg_importance: f64,
}

#[derive(Default)]
pub struct AgingEngine {
    pub config: AgingConfig,
}

impl AgingEngine {
    pub fn age(&self, memories: &mut [MemoryAtom], accessed_anchors: &[String]) -> AgingReport {
        let access_set: std::collections::HashSet<&str> =
            accessed_anchors.iter().map(|s| s.as_str()).collect();

        let mut aged_count = 0usize;
        let mut flagged_for_forget = 0usize;
        let mut boosted_count = 0usize;

        for mem in memories.iter_mut() {
            let anchor_str = format!("{}", mem.anchor());
            let was_accessed = access_set.contains(anchor_str.as_str());

            if was_accessed {
                let current = mem.importance();
                let boosted = (current + self.config.access_boost).min(1.0);
                mem.set_importance(boosted);
                boosted_count += 1;
            } else {
                let current = mem.importance();
                let decayed = current * self.config.decay_rate;
                mem.set_importance(decayed);
                aged_count += 1;
                if decayed < self.config.forget_threshold {
                    flagged_for_forget += 1;
                }
            }
        }

        let min_importance = memories
            .iter()
            .map(|m| m.importance())
            .fold(f64::MAX, f64::min);
        let avg_importance = if memories.is_empty() {
            0.0
        } else {
            memories.iter().map(|m| m.importance()).sum::<f64>() / memories.len() as f64
        };

        AgingReport {
            aged_count,
            flagged_for_forget,
            boosted_count,
            min_importance,
            avg_importance,
        }
    }

    pub fn flagged_memories<'a>(&self, memories: &'a [MemoryAtom]) -> Vec<(usize, &'a MemoryAtom)> {
        memories
            .iter()
            .enumerate()
            .filter(|(_, m)| m.importance() < self.config.forget_threshold)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::coord::Coord7D;
    use crate::universe::memory::MemoryCodec;
    use crate::universe::DarkUniverse;

    fn make_memories(count: usize) -> (DarkUniverse, Vec<MemoryAtom>) {
        let mut u = DarkUniverse::new(100000.0);
        let mut mems = Vec::new();
        for i in 0..count {
            let anchor = Coord7D::new_even([i as i32 * 5, 0, 0, 0, 0, 0, 0]);
            if let Ok(mut atom) = MemoryCodec::encode(&mut u, &anchor, &[1.0, 2.0, 3.0]) {
                atom.set_importance(0.8);
                mems.push(atom);
            }
        }
        (u, mems)
    }

    #[test]
    fn aging_decays_unaccessed() {
        let (_, mut mems) = make_memories(5);
        let engine = AgingEngine::default();
        let report = engine.age(&mut mems, &[]);
        assert_eq!(report.aged_count, 5);
        assert_eq!(report.boosted_count, 0);
        assert!(mems.iter().all(|m| m.importance() < 0.8));
    }

    #[test]
    fn aging_boosts_accessed() {
        let (_, mut mems) = make_memories(3);
        let anchors: Vec<String> = mems.iter().map(|m| format!("{}", m.anchor())).collect();
        let engine = AgingEngine::default();
        let report = engine.age(&mut mems, &[anchors[0].clone()]);
        assert_eq!(report.boosted_count, 1);
        assert_eq!(report.aged_count, 2);
    }

    #[test]
    fn aging_flags_for_forget() {
        let engine = AgingEngine {
            config: AgingConfig {
                decay_rate: 0.1,
                forget_threshold: 0.5,
                access_boost: 0.1,
            },
        };
        let (_, mut mems) = make_memories(3);
        let report = engine.age(&mut mems, &[]);
        assert_eq!(report.flagged_for_forget, 3);
        assert!(mems.iter().all(|m| m.importance() < 0.5));
    }

    #[test]
    fn flagged_memories_returns_below_threshold() {
        let engine = AgingEngine {
            config: AgingConfig {
                decay_rate: 0.1,
                forget_threshold: 0.5,
                access_boost: 0.1,
            },
        };
        let (_, mut mems) = make_memories(3);
        engine.age(&mut mems, &[]);
        let flagged = engine.flagged_memories(&mems);
        assert_eq!(flagged.len(), 3);
    }

    #[test]
    fn aging_report_stats() {
        let (_, mut mems) = make_memories(5);
        let engine = AgingEngine::default();
        let report = engine.age(&mut mems, &[]);
        assert!(report.avg_importance > 0.0);
        assert!(report.min_importance > 0.0);
    }
}

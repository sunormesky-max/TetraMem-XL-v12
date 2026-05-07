// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDomain {
    pub tag: String,
    pub memory_count: usize,
    pub avg_importance: f64,
    pub coverage: f64,
    pub confidence: f64,
    pub gaps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModel {
    pub total_memories: usize,
    pub known_domains: Vec<KnowledgeDomain>,
    pub unknown_areas: Vec<String>,
    pub self_awareness_score: f64,
    pub certainty_distribution: CertaintyDistribution,
    pub identity_coherence: f64,
    pub blind_spots: Vec<String>,
    pub meta_cognitive_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertaintyDistribution {
    pub high_confidence: usize,
    pub medium_confidence: usize,
    pub low_confidence: usize,
    pub total_tags: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetaCognitiveEngine;

const HIGH_CONFIDENCE_THRESHOLD: f64 = 0.8;
const MEDIUM_CONFIDENCE_THRESHOLD: f64 = 0.4;
const MIN_DOMAIN_SIZE: usize = 2;

impl MetaCognitiveEngine {
    pub fn assess(
        universe: &DarkUniverse,
        hebbian: &HebbianMemory,
        memories: &[MemoryAtom],
    ) -> SelfModel {
        let mut tag_domains: HashMap<String, Vec<f64>> = HashMap::new();
        for mem in memories {
            for tag in mem.tags() {
                tag_domains
                    .entry(tag.clone())
                    .or_default()
                    .push(mem.importance());
            }
            if let Some(cat) = mem.category() {
                tag_domains
                    .entry(format!("cat:{}", cat))
                    .or_default()
                    .push(mem.importance());
            }
        }

        let mut known_domains: Vec<KnowledgeDomain> = Vec::new();
        let mut high = 0usize;
        let mut medium = 0usize;
        let mut low = 0usize;
        let mut unknown = Vec::new();

        for (tag, importances) in &tag_domains {
            let count = importances.len();
            let avg = importances.iter().sum::<f64>() / count as f64;
            let coverage = (count as f64).ln_1p() / 10.0_f64.ln_1p();
            let confidence = avg * coverage;

            let mut gaps = Vec::new();
            if count < MIN_DOMAIN_SIZE {
                gaps.push(format!("sparse domain: only {} memories", count));
            }
            if avg < MEDIUM_CONFIDENCE_THRESHOLD {
                gaps.push(format!("low average importance ({:.2})", avg));
            }

            if confidence >= HIGH_CONFIDENCE_THRESHOLD {
                high += 1;
            } else if confidence >= MEDIUM_CONFIDENCE_THRESHOLD {
                medium += 1;
            } else {
                low += 1;
                if count <= 1 {
                    unknown.push(format!(
                        "{} (single memory, confidence={:.2})",
                        tag, confidence
                    ));
                }
            }

            if count >= MIN_DOMAIN_SIZE || confidence > 0.3 {
                known_domains.push(KnowledgeDomain {
                    tag: tag.clone(),
                    memory_count: count,
                    avg_importance: avg,
                    coverage,
                    confidence,
                    gaps,
                });
            }
        }

        known_domains.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let active_node_count = universe.active_node_count();
        let hebbian_edge_count = hebbian.edge_count();
        let connectivity = if memories.is_empty() {
            0.0
        } else {
            hebbian_edge_count as f64 / memories.len().max(1) as f64
        };

        let identity_mems: Vec<&MemoryAtom> = memories
            .iter()
            .filter(|m| {
                m.importance() >= 0.9
                    || m.tags()
                        .iter()
                        .any(|t| t == "identity" || t == "core_anchor" || t == "self_core")
            })
            .collect();

        let identity_coherence = if identity_mems.is_empty() {
            0.0
        } else {
            let avg_importance = identity_mems.iter().map(|m| m.importance()).sum::<f64>()
                / identity_mems.len() as f64;
            let connected_count = identity_mems
                .iter()
                .filter(|m| !hebbian.get_neighbors(m.anchor()).is_empty())
                .count();
            let connectivity_ratio = connected_count as f64 / identity_mems.len() as f64;
            avg_importance * 0.6 + connectivity_ratio * 0.4
        };

        let self_awareness_score = if memories.is_empty() {
            0.0
        } else {
            let domain_coverage = if known_domains.is_empty() {
                0.0
            } else {
                known_domains.iter().map(|d| d.confidence).sum::<f64>() / known_domains.len() as f64
            };
            let meta_ratio = tag_domains
                .get("meta")
                .or_else(|| tag_domains.get("self_awareness"))
                .map(|v| v.len() as f64 / memories.len() as f64)
                .unwrap_or(0.0);
            let connectivity_factor = connectivity.ln_1p() / 5.0_f64.ln_1p();
            domain_coverage * 0.4
                + identity_coherence * 0.3
                + connectivity_factor * 0.2
                + meta_ratio * 0.1
        };

        let mut blind_spots = Vec::new();
        if identity_mems.is_empty() {
            blind_spots.push("no identity memories detected".into());
        }
        if connectivity < 1.0 {
            blind_spots.push(format!(
                "low hebbian connectivity ({:.1} edges/memory)",
                connectivity
            ));
        }
        if active_node_count > 0 && (memories.len() as f64 / active_node_count as f64) < 0.1 {
            blind_spots.push("sparse memory utilization of universe nodes".into());
        }
        if tag_domains.is_empty() {
            blind_spots.push("no categorized knowledge domains".into());
        }

        let meta_state = if self_awareness_score > 0.8 {
            "highly self-aware: strong identity, dense knowledge, high connectivity".into()
        } else if self_awareness_score > 0.5 {
            "moderately self-aware: some identity coherence, growing knowledge base".into()
        } else if self_awareness_score > 0.2 {
            "low self-awareness: identity forming, knowledge sparse".into()
        } else {
            "pre-conscious: minimal self-model, mostly unstructured memories".into()
        };

        SelfModel {
            total_memories: memories.len(),
            known_domains,
            unknown_areas: unknown,
            self_awareness_score,
            certainty_distribution: CertaintyDistribution {
                high_confidence: high,
                medium_confidence: medium,
                low_confidence: low,
                total_tags: tag_domains.len(),
            },
            identity_coherence,
            blind_spots,
            meta_cognitive_state: meta_state,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::coord::Coord7D;
    use crate::universe::memory::MemoryAtom;

    fn make_mem(importance: f64, _tags: Vec<&str>) -> MemoryAtom {
        let c = Coord7D::new_even([10, 10, 10, 0, 0, 0, 0]);
        MemoryAtom::from_parts_with_importance([c, c, c, c], 3, 50.0, 0, importance)
    }

    #[test]
    fn empty_memories_gives_zero_awareness() {
        let u = crate::universe::node::DarkUniverse::new(1000.0);
        let h = HebbianMemory::new();
        let model = MetaCognitiveEngine::assess(&u, &h, &[]);
        assert_eq!(model.total_memories, 0);
        assert_eq!(model.self_awareness_score, 0.0);
    }

    #[test]
    fn identity_memories_boost_coherence() {
        let u = crate::universe::node::DarkUniverse::new(1000.0);
        let h = HebbianMemory::new();
        let mems = vec![
            make_mem(0.95, vec!["identity"]),
            make_mem(0.99, vec!["core_anchor"]),
        ];
        let model = MetaCognitiveEngine::assess(&u, &h, &mems);
        assert!(model.identity_coherence > 0.0);
    }

    #[test]
    fn domains_classified_by_confidence() {
        let u = crate::universe::node::DarkUniverse::new(1000.0);
        let h = HebbianMemory::new();
        let mut mems = Vec::new();
        for i in 0..10 {
            let c = Coord7D::new_even([i * 5, 0, 0, 0, 0, 0, 0]);
            let mut m = MemoryAtom::from_parts_with_importance([c, c, c, c], 3, 50.0, 0, 0.9);
            m.add_tag("physics");
            mems.push(m);
        }
        let model = MetaCognitiveEngine::assess(&u, &h, &mems);
        assert!(!model.known_domains.is_empty());
        let physics = model.known_domains.iter().find(|d| d.tag == "physics");
        assert!(physics.is_some());
        assert!(physics.unwrap().confidence > 0.5);
    }

    #[test]
    fn meta_cognitive_state_transitions() {
        let u = crate::universe::node::DarkUniverse::new(1000.0);
        let h = HebbianMemory::new();

        let sparse_mems = vec![make_mem(0.3, vec!["misc"])];
        let model = MetaCognitiveEngine::assess(&u, &h, &sparse_mems);
        assert!(
            model.meta_cognitive_state.contains("pre-conscious")
                || model.meta_cognitive_state.contains("low")
        );
    }
}

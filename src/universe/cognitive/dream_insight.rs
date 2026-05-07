// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::aging::AgingEngine;
use crate::universe::memory::semantic::SemanticEngine;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamInsightReport {
    pub contradictions: Vec<ContradictionInsight>,
    pub forgotten_important: Vec<ForgottenInsight>,
    pub emerging_clusters: Vec<ClusterInsight>,
    pub weak_connections: Vec<WeakConnectionInsight>,
    pub total_insights: usize,
    pub insight_density: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionInsight {
    pub anchor_a: String,
    pub anchor_b: String,
    pub description_a: String,
    pub description_b: String,
    pub conflict_type: String,
    pub severity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgottenInsight {
    pub anchor: String,
    pub description: String,
    pub importance: f64,
    pub category: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInsight {
    pub center_anchor: String,
    pub member_count: usize,
    pub categories: Vec<String>,
    pub avg_importance: f64,
    pub radius: f64,
    pub discovery: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeakConnectionInsight {
    pub anchor_a: String,
    pub anchor_b: String,
    pub weight: f64,
    pub suggestion: String,
}

pub struct DreamInsightEngine {
    pub cluster_radius: f64,
    pub min_cluster_size: usize,
    pub weak_threshold: f64,
    pub important_forget_threshold: f64,
}

impl Default for DreamInsightEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DreamInsightEngine {
    pub fn new() -> Self {
        Self {
            cluster_radius: 2500.0,
            min_cluster_size: 3,
            weak_threshold: 0.15,
            important_forget_threshold: 0.3,
        }
    }

    pub fn generate_insights(
        &self,
        _universe: &DarkUniverse,
        hebbian: &HebbianMemory,
        memories: &[MemoryAtom],
        _semantic: &SemanticEngine,
    ) -> DreamInsightReport {
        let contradictions = self.detect_contradictions(memories);
        let forgotten_important = self.find_forgotten_important(memories);
        let emerging_clusters = self.find_clusters(memories);
        let weak_connections = self.find_weak_connections(memories, hebbian);

        let total = contradictions.len()
            + forgotten_important.len()
            + emerging_clusters.len()
            + weak_connections.len();

        let insight_density = if !memories.is_empty() {
            total as f64 / memories.len() as f64
        } else {
            0.0
        };

        DreamInsightReport {
            contradictions,
            forgotten_important,
            emerging_clusters,
            weak_connections,
            total_insights: total,
            insight_density,
        }
    }

    fn detect_contradictions(&self, memories: &[MemoryAtom]) -> Vec<ContradictionInsight> {
        let mut insights = Vec::new();

        for i in 0..memories.len() {
            for j in (i + 1)..memories.len() {
                let desc_a = memories[i].description().unwrap_or("");
                let desc_b = memories[j].description().unwrap_or("");
                if desc_a.is_empty() || desc_b.is_empty() {
                    continue;
                }

                if crate::universe::memory::contradiction::descriptions_conflict(
                    Some(desc_a),
                    Some(desc_b),
                ) {
                    insights.push(ContradictionInsight {
                        anchor_a: format!("{}", memories[i].anchor()),
                        anchor_b: format!("{}", memories[j].anchor()),
                        description_a: desc_a.to_string(),
                        description_b: desc_b.to_string(),
                        conflict_type: "description_negation".to_string(),
                        severity: 0.7,
                    });
                    continue;
                }

                let tags_a = memories[i].tags();
                let tags_b = memories[j].tags();
                if crate::universe::memory::contradiction::tags_conflict(tags_a, tags_b) {
                    insights.push(ContradictionInsight {
                        anchor_a: format!("{}", memories[i].anchor()),
                        anchor_b: format!("{}", memories[j].anchor()),
                        description_a: desc_a.to_string(),
                        description_b: desc_b.to_string(),
                        conflict_type: "tag_opposition".to_string(),
                        severity: 0.5,
                    });
                }
            }
            if insights.len() >= 10 {
                break;
            }
        }

        insights
    }

    fn find_forgotten_important(&self, memories: &[MemoryAtom]) -> Vec<ForgottenInsight> {
        let aging = AgingEngine::default();
        let flagged: Vec<usize> = aging
            .flagged_memories(memories)
            .into_iter()
            .map(|(idx, _)| idx)
            .collect();

        flagged
            .into_iter()
            .filter(|idx| {
                memories
                    .get(*idx)
                    .map(|m| m.importance() > 0.5)
                    .unwrap_or(false)
            })
            .map(|idx| {
                let mem = &memories[idx];
                ForgottenInsight {
                    anchor: format!("{}", mem.anchor()),
                    description: mem.description().unwrap_or("").to_string(),
                    importance: mem.importance(),
                    category: mem.category().unwrap_or("").to_string(),
                    message: format!(
                        "important memory ({:.1}) is aging and may be forgotten",
                        mem.importance()
                    ),
                }
            })
            .take(10)
            .collect()
    }

    fn find_clusters(&self, memories: &[MemoryAtom]) -> Vec<ClusterInsight> {
        let mut visited = vec![false; memories.len()];
        let mut clusters = Vec::new();

        for i in 0..memories.len() {
            if visited[i] {
                continue;
            }
            let mut members: Vec<usize> = Vec::new();
            let mut stack = vec![i];

            while let Some(ci) = stack.pop() {
                if visited[ci] {
                    continue;
                }
                visited[ci] = true;
                members.push(ci);

                for cj in 0..memories.len() {
                    if visited[cj] {
                        continue;
                    }
                    if memories[ci].anchor().distance_sq(memories[cj].anchor())
                        < self.cluster_radius
                    {
                        stack.push(cj);
                    }
                }
            }

            if members.len() >= self.min_cluster_size {
                let avg_imp = members
                    .iter()
                    .map(|&idx| memories[idx].importance())
                    .sum::<f64>()
                    / members.len() as f64;

                let categories: Vec<String> = members
                    .iter()
                    .filter_map(|&idx| memories[idx].category().map(String::from))
                    .filter(|c| !c.is_empty())
                    .collect();

                let center_idx = members
                    .iter()
                    .copied()
                    .max_by(|&a, &b| {
                        memories[a]
                            .importance()
                            .partial_cmp(&memories[b].importance())
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .unwrap_or(i);

                let radius = members
                    .iter()
                    .map(|&idx| {
                        memories[center_idx]
                            .anchor()
                            .distance_sq(memories[idx].anchor())
                    })
                    .fold(0.0f64, f64::max)
                    .sqrt();

                let unique_cats: Vec<String> = {
                    let mut cs = categories;
                    cs.sort();
                    cs.dedup();
                    cs
                };

                let cat_list = unique_cats.join("+");
                let discovery = if unique_cats.len() > 1 {
                    format!(
                        "cross-category cluster: {} memories across [{}]",
                        members.len(),
                        cat_list
                    )
                } else {
                    format!(
                        "dense cluster: {} memories in radius {:.0}",
                        members.len(),
                        radius
                    )
                };

                clusters.push(ClusterInsight {
                    center_anchor: format!("{}", memories[center_idx].anchor()),
                    member_count: members.len(),
                    categories: unique_cats,
                    avg_importance: avg_imp,
                    radius,
                    discovery,
                });
            }
        }

        clusters.sort_by_key(|b| std::cmp::Reverse(b.member_count));
        clusters.truncate(10);
        clusters
    }

    fn find_weak_connections(
        &self,
        memories: &[MemoryAtom],
        hebbian: &HebbianMemory,
    ) -> Vec<WeakConnectionInsight> {
        let mut weak = Vec::new();

        for mem in memories {
            let neighbors = hebbian.get_neighbors(mem.anchor());
            for (coord, weight) in &neighbors {
                if *weight > 0.0 && *weight < self.weak_threshold {
                    if let Some(other) = memories.iter().find(|m| m.anchor() == coord) {
                        let desc = other.description().unwrap_or("");
                        if !desc.is_empty() {
                            weak.push(WeakConnectionInsight {
                                anchor_a: format!("{}", mem.anchor()),
                                anchor_b: format!("{}", coord),
                                weight: *weight,
                                suggestion: format!(
                                    "weak edge ({:.3}) to '{}' — consider reinforcing",
                                    weight, desc
                                ),
                            });
                        }
                    }
                }
            }
            if weak.len() >= 10 {
                break;
            }
        }

        weak.sort_by(|a, b| {
            a.weight
                .partial_cmp(&b.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        weak
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::coord::Coord7D;
    use crate::universe::memory::semantic::SemanticConfig;
    use crate::universe::memory::MemoryCodec;

    fn setup() -> (DarkUniverse, HebbianMemory, Vec<MemoryAtom>, SemanticEngine) {
        let u = DarkUniverse::new(1_000_000.0);
        let h = HebbianMemory::new();
        let mems = Vec::new();
        let sem = SemanticEngine::new(SemanticConfig::default());
        (u, h, mems, sem)
    }

    #[test]
    fn empty_memories_no_insights() {
        let (u, h, mems, sem) = setup();
        let engine = DreamInsightEngine::new();
        let report = engine.generate_insights(&u, &h, &mems, &sem);
        assert_eq!(report.total_insights, 0);
        assert_eq!(report.insight_density, 0.0);
    }

    #[test]
    fn cluster_discovery() {
        let (mut u, h, _, sem) = setup();
        let mut mems = Vec::new();
        for i in 0..5i32 {
            let a = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            let mut m = MemoryCodec::encode(&mut u, &a, &[i as f64]).unwrap();
            m.set_category("test");
            m.set_description(format!("cluster member {}", i));
            m.set_importance(0.6);
            mems.push(m);
        }
        let engine = DreamInsightEngine::new();
        let report = engine.generate_insights(&u, &h, &mems, &sem);
        assert!(!report.emerging_clusters.is_empty());
        assert!(report.emerging_clusters[0].member_count >= 3);
    }

    #[test]
    fn weak_connections_detected() {
        let (mut u, mut h, _, sem) = setup();
        let a1 = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        let a2 = Coord7D::new_even([10, 0, 0, 0, 0, 0, 0]);
        let mut m1 = MemoryCodec::encode(&mut u, &a1, &[1.0]).unwrap();
        let mut m2 = MemoryCodec::encode(&mut u, &a2, &[2.0]).unwrap();
        m1.set_description("alpha");
        m2.set_description("beta");
        h.boost_edge(&a1, &a2, 0.1);
        let mems = vec![m1, m2];
        let engine = DreamInsightEngine::new();
        let report = engine.generate_insights(&u, &h, &mems, &sem);
        assert!(!report.weak_connections.is_empty());
        assert!(report.weak_connections[0].weight < 0.15);
    }

    #[test]
    fn contradiction_detection() {
        let (mut u, h, _, sem) = setup();
        let a1 = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        let a2 = Coord7D::new_even([50, 0, 0, 0, 0, 0, 0]);
        let mut m1 = MemoryCodec::encode(&mut u, &a1, &[1.0]).unwrap();
        let mut m2 = MemoryCodec::encode(&mut u, &a2, &[2.0]).unwrap();
        m1.set_description("the system is reliable");
        m2.set_description("the system is not reliable");
        let mems = vec![m1, m2];
        let engine = DreamInsightEngine::new();
        let report = engine.generate_insights(&u, &h, &mems, &sem);
        assert!(!report.contradictions.is_empty());
        assert_eq!(
            report.contradictions[0].conflict_type,
            "description_negation"
        );
    }

    #[test]
    fn insight_report_serde() {
        let report = DreamInsightReport {
            contradictions: Vec::new(),
            forgotten_important: Vec::new(),
            emerging_clusters: Vec::new(),
            weak_connections: Vec::new(),
            total_insights: 0,
            insight_density: 0.0,
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("total_insights"));
    }
}

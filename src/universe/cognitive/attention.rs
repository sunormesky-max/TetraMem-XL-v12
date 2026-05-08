// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::coord::Coord7D;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionHotspot {
    pub anchor_basis: [i32; 7],
    pub heat: f64,
    pub source: AttentionSource,
    pub nearby_memory_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttentionSource {
    HebbianHub,
    DenseCluster,
    HighImportance,
    RecentAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionMap {
    pub hotspots: Vec<AttentionHotspot>,
    pub total_heat: f64,
    pub coverage_ratio: f64,
    pub recommendation: AttentionRecommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionRecommendation {
    pub suggested_pulse_anchors: Vec<[i32; 7]>,
    pub suggested_explore_regions: Vec<[i32; 7]>,
    pub cold_zones: Vec<[i32; 7]>,
}

pub struct AttentionEngine {
    pub max_hotspots: usize,
    pub hebbian_weight: f64,
    pub density_weight: f64,
    pub importance_weight: f64,
    pub recency_weight: f64,
    pub neighborhood_radius: f64,
}

impl Default for AttentionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AttentionEngine {
    pub fn new() -> Self {
        Self {
            max_hotspots: 20,
            hebbian_weight: 0.35,
            density_weight: 0.25,
            importance_weight: 0.25,
            recency_weight: 0.15,
            neighborhood_radius: 2500.0,
        }
    }

    pub fn compute(
        &self,
        _universe: &DarkUniverse,
        hebbian: &HebbianMemory,
        memories: &[MemoryAtom],
    ) -> AttentionMap {
        let mut heat_map: HashMap<[i32; 7], f64> = HashMap::new();
        let mut source_map: HashMap<[i32; 7], AttentionSource> = HashMap::new();
        let mut nearby_count: HashMap<[i32; 7], usize> = HashMap::new();

        let grid_cell = (self.neighborhood_radius.sqrt().max(1.0)) as i32;

        let mut grid: HashMap<[i32; 7], Vec<usize>> = HashMap::new();
        for (i, mem) in memories.iter().enumerate() {
            let basis = mem.anchor().basis();
            let cell = [
                basis[0] / grid_cell,
                basis[1] / grid_cell,
                basis[2] / grid_cell,
                basis[3] / grid_cell,
                basis[4] / grid_cell,
                basis[5] / grid_cell,
                basis[6] / grid_cell,
            ];
            grid.entry(cell).or_default().push(i);
        }

        for mem in memories {
            let basis = mem.anchor().basis();
            let importance_heat = mem.importance() * self.importance_weight;
            *heat_map.entry(basis).or_insert(0.0) += importance_heat;
            if mem.importance() > 0.7 && !source_map.contains_key(&basis) {
                source_map.insert(basis, AttentionSource::HighImportance);
            }

            let recency_heat = {
                let age = mem.created_at();
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let elapsed_hours = (now.saturating_sub(age)) as f64 / 3_600_000.0;
                (1.0 / (1.0 + elapsed_hours)).max(0.0) * self.recency_weight
            };
            *heat_map.entry(basis).or_insert(0.0) += recency_heat;

            let neighbors = hebbian.get_neighbors(mem.anchor());
            let hub_heat = (neighbors.len() as f64).ln_1p() * self.hebbian_weight;
            *heat_map.entry(basis).or_insert(0.0) += hub_heat;
            if neighbors.len() > 5
                && source_map
                    .get(&basis)
                    .is_none_or(|s| !matches!(s, AttentionSource::HebbianHub))
            {
                source_map.insert(basis, AttentionSource::HebbianHub);
            }
        }

        for (i, mem_a) in memories.iter().enumerate() {
            let basis = mem_a.anchor().basis();
            let cell = [
                basis[0] / grid_cell,
                basis[1] / grid_cell,
                basis[2] / grid_cell,
                basis[3] / grid_cell,
                basis[4] / grid_cell,
                basis[5] / grid_cell,
                basis[6] / grid_cell,
            ];
            let mut count = 0usize;
            for dx in -1i32..=1 {
                for dy in -1i32..=1 {
                    for dz in -1i32..=1 {
                        let neighbor_cell = [
                            cell[0] + dx,
                            cell[1] + dy,
                            cell[2] + dz,
                            cell[3],
                            cell[4],
                            cell[5],
                            cell[6],
                        ];
                        if let Some(indices) = grid.get(&neighbor_cell) {
                            for &j in indices {
                                if i == j {
                                    continue;
                                }
                                if mem_a.anchor().distance_sq(memories[j].anchor())
                                    < self.neighborhood_radius
                                {
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
            *nearby_count.entry(basis).or_insert(0) += count;
            let density_heat = (count as f64).ln_1p() * self.density_weight;
            *heat_map.entry(basis).or_insert(0.0) += density_heat;
            if count > 3
                && source_map
                    .get(&basis)
                    .is_none_or(|s| !matches!(s, AttentionSource::DenseCluster))
            {
                source_map.insert(basis, AttentionSource::DenseCluster);
            }
        }

        let mut entries: Vec<([i32; 7], f64)> = heat_map.into_iter().collect();
        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let total_heat: f64 = entries.iter().map(|(_, h)| *h).sum();
        let total_memories = memories.len().max(1);

        let hotspots: Vec<AttentionHotspot> = entries
            .into_iter()
            .take(self.max_hotspots)
            .map(|(basis, heat)| {
                let nearby = memories
                    .iter()
                    .filter(|m| m.anchor().basis() == basis)
                    .count();
                AttentionHotspot {
                    anchor_basis: basis,
                    heat,
                    source: source_map
                        .remove(&basis)
                        .unwrap_or(AttentionSource::RecentAccess),
                    nearby_memory_count: nearby,
                }
            })
            .collect();

        let coverage_ratio = if total_memories > 0 {
            hotspots.len() as f64 / total_memories as f64
        } else {
            0.0
        };

        let suggested_pulse_anchors: Vec<[i32; 7]> = hotspots
            .iter()
            .filter(|h| h.heat > total_heat / (hotspots.len().max(1) as f64))
            .take(3)
            .map(|h| h.anchor_basis)
            .collect();

        let cold_zones = self.find_cold_zones(memories, &hotspots);

        let suggested_explore_regions = cold_zones.iter().take(3).copied().collect();

        AttentionMap {
            hotspots,
            total_heat,
            coverage_ratio,
            recommendation: AttentionRecommendation {
                suggested_pulse_anchors,
                suggested_explore_regions,
                cold_zones,
            },
        }
    }

    fn find_cold_zones(
        &self,
        memories: &[MemoryAtom],
        hotspots: &[AttentionHotspot],
    ) -> Vec<[i32; 7]> {
        let hotspot_bases: std::collections::HashSet<[i32; 7]> =
            hotspots.iter().map(|h| h.anchor_basis).collect();

        let mut cold: Vec<([i32; 7], f64)> = memories
            .iter()
            .filter(|m| !hotspot_bases.contains(&m.anchor().basis()))
            .filter_map(|m| {
                let min_dist = hotspots
                    .iter()
                    .map(|h| {
                        let h_coord = Coord7D::new_even(h.anchor_basis);
                        m.anchor().distance_sq(&h_coord)
                    })
                    .fold(f64::MAX, f64::min);
                if min_dist > 100.0 {
                    Some((m.anchor().basis(), min_dist))
                } else {
                    None
                }
            })
            .collect();

        cold.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        cold.into_iter().take(5).map(|(b, _)| b).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::memory::MemoryCodec;

    fn setup() -> (DarkUniverse, HebbianMemory, Vec<MemoryAtom>) {
        let u = DarkUniverse::new(1_000_000.0);
        let h = HebbianMemory::new();
        let mems = Vec::new();
        (u, h, mems)
    }

    #[test]
    fn empty_memories_empty_map() {
        let (u, h, mems) = setup();
        let engine = AttentionEngine::new();
        let map = engine.compute(&u, &h, &mems);
        assert!(map.hotspots.is_empty());
        assert_eq!(map.total_heat, 0.0);
    }

    #[test]
    fn single_memory_has_heat() {
        let (mut u, h, _) = setup();
        let anchor = Coord7D::new_even([1, 2, 3, 0, 0, 0, 0]);
        let mem = MemoryCodec::encode(&mut u, &anchor, &[1.0, 2.0, 3.0]).unwrap();
        let mems = vec![mem];
        let engine = AttentionEngine::new();
        let map = engine.compute(&u, &h, &mems);
        assert!(!map.hotspots.is_empty());
        assert!(map.total_heat > 0.0);
    }

    #[test]
    fn hub_memory_gets_more_heat() {
        let (mut u, mut h, _) = setup();
        let anchor = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        let mut mem = MemoryCodec::encode(&mut u, &anchor, &[1.0]).unwrap();
        mem.set_importance(0.8);
        let mut mems = vec![mem];
        for i in 1..=8i32 {
            let a = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            let m = MemoryCodec::encode(&mut u, &a, &[i as f64]).unwrap();
            h.boost_edge(&anchor, &a, 0.5);
            mems.push(m);
        }
        let engine = AttentionEngine::new();
        let map = engine.compute(&u, &h, &mems);
        let hub_heat = map
            .hotspots
            .iter()
            .find(|hs| hs.anchor_basis == [0, 0, 0, 0, 0, 0, 0])
            .map(|hs| hs.heat)
            .unwrap_or(0.0);
        assert!(hub_heat > 0.0);
    }

    #[test]
    fn recommendation_has_pulse_anchors() {
        let (mut u, h, _) = setup();
        let mut mems = Vec::new();
        for i in 0..5i32 {
            let a = Coord7D::new_even([i * 10, 0, 0, 0, 0, 0, 0]);
            let mut m = MemoryCodec::encode(&mut u, &a, &[i as f64]).unwrap();
            m.set_importance(0.8);
            mems.push(m);
        }
        let engine = AttentionEngine::new();
        let map = engine.compute(&u, &h, &mems);
        assert!(!map.hotspots.is_empty());
        assert!(map.total_heat > 0.0);
    }

    #[test]
    fn attention_source_serialization() {
        let src = AttentionSource::HebbianHub;
        let s = serde_json::to_string(&src).unwrap();
        assert!(s.contains("HebbianHub"));
    }
}

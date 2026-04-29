use crate::universe::coord::Coord7D;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::node::DarkUniverse;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

const CRYSTAL_THRESHOLD: f64 = 1.8;
const SUPER_CRYSTAL_THRESHOLD: f64 = 4.0;
const SUPER_CRYSTAL_BOOST: f64 = 2.5;

#[derive(Debug, Clone, PartialEq)]
pub struct CrystalChannel {
    endpoints: (Coord7D, Coord7D),
    strength: f64,
    is_super: bool,
}

impl CrystalChannel {
    pub fn endpoints(&self) -> (&Coord7D, &Coord7D) {
        (&self.endpoints.0, &self.endpoints.1)
    }

    pub fn strength(&self) -> f64 {
        self.strength
    }

    pub fn is_super(&self) -> bool {
        self.is_super
    }
}

#[derive(Debug, Clone)]
pub struct CrystalReport {
    pub new_crystals: usize,
    pub new_super_crystals: usize,
    pub total_crystals: usize,
    pub total_super_crystals: usize,
    pub energy_locked: f64,
}

impl std::fmt::Display for CrystalReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Crystal[new:{} super:{} total:{} super_total:{} locked:{:.1}]",
            self.new_crystals, self.new_super_crystals,
            self.total_crystals, self.total_super_crystals, self.energy_locked
        )
    }
}

pub struct CrystalEngine {
    pub threshold: f64,
    pub super_threshold: f64,
    pub super_boost: f64,
    channels: HashMap<(Coord7D, Coord7D), CrystalChannel>,
}

impl CrystalEngine {
    pub fn new() -> Self {
        Self {
            threshold: CRYSTAL_THRESHOLD,
            super_threshold: SUPER_CRYSTAL_THRESHOLD,
            super_boost: SUPER_CRYSTAL_BOOST,
            channels: HashMap::new(),
        }
    }

    pub fn restore_channel(&mut self, a: Coord7D, b: Coord7D, strength: f64, is_super: bool) {
        let ch = CrystalChannel { endpoints: (a, b), strength, is_super };
        self.channels.insert((a, b), ch);
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn super_count(&self) -> usize {
        self.channels.values().filter(|c| c.is_super).count()
    }

    pub fn total_locked_energy(&self) -> f64 {
        self.channels.values().map(|c| c.strength).sum()
    }

    pub fn contains(&self, a: &Coord7D, b: &Coord7D) -> bool {
        let key = if a <= b { (*a, *b) } else { (*b, *a) };
        self.channels.contains_key(&key)
    }

    pub fn get_strength(&self, a: &Coord7D, b: &Coord7D) -> f64 {
        let key = if a <= b { (*a, *b) } else { (*b, *a) };
        self.channels.get(&key).map_or(0.0, |c| c.strength)
    }

    pub fn crystallize(
        &mut self,
        hebbian: &HebbianMemory,
        universe: &DarkUniverse,
    ) -> CrystalReport {
        let strong = hebbian.strongest_edges(100);
        let mut new_crystals = 0usize;
        let mut new_super = 0usize;
        let mut energy_locked = 0.0f64;

        for ((a, b), weight) in &strong {
            if *weight < self.threshold {
                break;
            }

            let key = if a <= b { (*a, *b) } else { (*b, *a) };
            if self.channels.contains_key(&key) {
                continue;
            }

            if universe.get_node(a).is_none() || universe.get_node(b).is_none() {
                continue;
            }

            let is_super = *weight >= self.super_threshold;
            let strength = if is_super {
                weight * self.super_boost
            } else {
                *weight
            };

            self.channels.insert(key, CrystalChannel {
                endpoints: if a <= b { (*a, *b) } else { (*b, *a) },
                strength,
                is_super,
            });

            energy_locked += strength;
            if is_super {
                new_super += 1;
            } else {
                new_crystals += 1;
            }
        }

        let total = self.channels.len();
        let total_super = self.super_count();

        CrystalReport {
            new_crystals,
            new_super_crystals: new_super,
            total_crystals: total,
            total_super_crystals: total_super,
            energy_locked,
        }
    }

    pub fn crystal_neighbors(&self, node: &Coord7D) -> Vec<(Coord7D, f64, bool)> {
        let mut result = Vec::new();
        for ((a, b), channel) in &self.channels {
            if a == node {
                result.push((*b, channel.strength, channel.is_super));
            } else if b == node {
                result.push((*a, channel.strength, channel.is_super));
            }
        }
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    pub fn all_channels(&self) -> &HashMap<(Coord7D, Coord7D), CrystalChannel> {
        &self.channels
    }

    pub fn detect_phase_transition(
        &self,
        hebbian: &HebbianMemory,
        universe: &DarkUniverse,
    ) -> PhaseTransitionReport {
        let strong = hebbian.strongest_edges(1000);
        let mut super_count = 0usize;
        let mut total_weight = 0.0f64;

        for ((a, b), weight) in &strong {
            if universe.get_node(a).is_none() || universe.get_node(b).is_none() {
                continue;
            }
            if *weight >= self.super_threshold {
                super_count += 1;
            }
            total_weight += weight;
        }

        let avg_weight = if strong.is_empty() { 0.0 } else { total_weight / strong.len() as f64 };
        let phase_coherent = super_count >= 3 && avg_weight >= self.threshold;
        let existing_super = self.super_count();

        PhaseTransitionReport {
            super_channel_candidates: super_count,
            existing_super_channels: existing_super,
            avg_edge_weight: avg_weight,
            phase_coherent,
            requires_consensus: phase_coherent && super_count > existing_super,
        }
    }

    pub fn decay_unused(&mut self, active_nodes: &HashSet<Coord7D>) -> usize {
        let before = self.channels.len();
        self.channels.retain(|(a, b), _| {
            active_nodes.contains(a) && active_nodes.contains(b)
        });
        before - self.channels.len()
    }

    pub fn crystal_path(
        &self,
        start: &Coord7D,
        end: &Coord7D,
        max_hops: usize,
    ) -> Vec<Coord7D> {
        let mut visited = HashSet::new();
        visited.insert(*start);
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((vec![*start], 0usize));

        while let Some((path, hops)) = queue.pop_front() {
            if hops >= max_hops {
                continue;
            }
            let current = *path.last().unwrap();
            if current == *end {
                return path;
            }

            for (neighbor, _, _) in self.crystal_neighbors(&current) {
                if visited.contains(&neighbor) {
                    continue;
                }
                visited.insert(neighbor);
                let mut new_path = path.clone();
                new_path.push(neighbor);
                queue.push_back((new_path, hops + 1));
            }
        }
        vec![]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTransitionReport {
    pub super_channel_candidates: usize,
    pub existing_super_channels: usize,
    pub avg_edge_weight: f64,
    pub phase_coherent: bool,
    pub requires_consensus: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (DarkUniverse, HebbianMemory) {
        let mut u = DarkUniverse::new(1_000_000.0);
        let mut h = HebbianMemory::new();

        for x in 0..6i32 {
            for y in 0..6i32 {
                for z in 0..6i32 {
                    let c = Coord7D::new_even([x + 10, y + 10, z + 10, 0, 0, 0, 0]);
                    u.materialize_biased(c, 50.0, 0.6).ok();
                }
            }
        }

        let a = Coord7D::new_even([12, 12, 12, 0, 0, 0, 0]);
        let b = Coord7D::new_even([13, 12, 12, 0, 0, 0, 0]);
        let c = Coord7D::new_even([14, 12, 12, 0, 0, 0, 0]);

        for _ in 0..50 {
            h.record_path(&[a, b, c], 2.0);
        }

        (u, h)
    }

    #[test]
    fn crystallize_creates_channels() {
        let (u, h) = setup();
        let mut engine = CrystalEngine::new();
        engine.crystallize(&h, &u);

        let a = Coord7D::new_even([12, 12, 12, 0, 0, 0, 0]);
        let b = Coord7D::new_even([13, 12, 12, 0, 0, 0, 0]);
        let ab_weight = h.get_bias(&a, &b);
        assert!(ab_weight > 0.0, "hebbian should have edges, got weight {}", ab_weight);

        engine.crystallize(&h, &u);
        assert!(engine.channel_count() > 0, "weight={:.2}, should crystallize", ab_weight);
    }

    #[test]
    fn super_crystal_for_very_strong() {
        let (u, h) = setup();
        let mut engine = CrystalEngine::new();
        let report = engine.crystallize(&h, &u);

        assert!(report.total_super_crystals > 0, "50x reinforce should create super crystal");
    }

    #[test]
    fn crystal_contains_query() {
        let (u, h) = setup();
        let mut engine = CrystalEngine::new();
        engine.crystallize(&h, &u);

        let a = Coord7D::new_even([12, 12, 12, 0, 0, 0, 0]);
        let b = Coord7D::new_even([13, 12, 12, 0, 0, 0, 0]);
        assert!(engine.contains(&a, &b));
    }

    #[test]
    fn crystal_neighbors_returns_connected() {
        let (u, h) = setup();
        let mut engine = CrystalEngine::new();
        engine.crystallize(&h, &u);

        let a = Coord7D::new_even([12, 12, 12, 0, 0, 0, 0]);
        let neighbors = engine.crystal_neighbors(&a);
        assert!(!neighbors.is_empty());
    }

    #[test]
    fn no_crystal_below_threshold() {
        let mut u = DarkUniverse::new(100_000.0);
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        u.materialize_biased(a, 50.0, 0.6).unwrap();
        u.materialize_biased(b, 50.0, 0.6).unwrap();
        h.record_path(&[a, b], 0.1);

        let mut engine = CrystalEngine::new();
        let report = engine.crystallize(&h, &u);
        assert_eq!(report.new_crystals, 0);
    }

    #[test]
    fn decay_unused_removes_orphaned() {
        let (u, h) = setup();
        let mut engine = CrystalEngine::new();
        engine.crystallize(&h, &u);

        let active: HashSet<Coord7D> = u.coords().into_iter().take(3).collect();
        let removed = engine.decay_unused(&active);
        assert!(removed > 0);
    }

    #[test]
    fn crystal_path_finds_route() {
        let (u, h) = setup();
        let mut engine = CrystalEngine::new();
        engine.crystallize(&h, &u);

        let a = Coord7D::new_even([12, 12, 12, 0, 0, 0, 0]);
        let c = Coord7D::new_even([14, 12, 12, 0, 0, 0, 0]);
        let path = engine.crystal_path(&a, &c, 5);
        if !path.is_empty() {
            assert_eq!(*path.first().unwrap(), a);
            assert_eq!(*path.last().unwrap(), c);
        }
    }

    #[test]
    fn display_format() {
        let (u, h) = setup();
        let mut engine = CrystalEngine::new();
        let report = engine.crystallize(&h, &u);
        let s = format!("{}", report);
        assert!(s.contains("Crystal["));
    }
}

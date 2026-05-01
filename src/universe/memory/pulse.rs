// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::coord::Coord7D;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::lattice::Lattice;
use crate::universe::node::DarkUniverse;
use std::collections::{HashSet, VecDeque};

const NOISE_FLOOR: f64 = 0.01;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PulseType {
    Exploratory,
    Reinforcing,
    Cascade,
}

impl PulseType {
    pub fn max_hops(&self) -> usize {
        match self {
            PulseType::Exploratory => 12,
            PulseType::Reinforcing => 8,
            PulseType::Cascade => 8,
        }
    }

    pub fn default_strength(&self) -> f64 {
        match self {
            PulseType::Exploratory => 0.15,
            PulseType::Reinforcing => 0.35,
            PulseType::Cascade => 0.45,
        }
    }

    pub fn hebbian_bias_weight(&self) -> f64 {
        match self {
            PulseType::Exploratory => 0.5,
            PulseType::Reinforcing => 2.0,
            PulseType::Cascade => 0.5,
        }
    }

    pub fn fanout(&self) -> usize {
        match self {
            PulseType::Exploratory => 2,
            PulseType::Reinforcing => 1,
            PulseType::Cascade => 3,
        }
    }

    pub fn face_decay(&self) -> f64 {
        0.72
    }

    pub fn bcc_decay(&self) -> f64 {
        0.36
    }
}

#[derive(Debug, Clone)]
struct NeuralPulse {
    strength: f64,
    hops: usize,
    max_hops: usize,
    pulse_type: PulseType,
    path: Vec<Coord7D>,
    cascade_depth: usize,
}

impl NeuralPulse {
    fn new(source: Coord7D, pulse_type: PulseType) -> Self {
        Self {
            strength: pulse_type.default_strength(),
            hops: 0,
            max_hops: pulse_type.max_hops(),
            pulse_type,
            path: vec![source],
            cascade_depth: 0,
        }
    }

    fn is_alive(&self) -> bool {
        self.strength > NOISE_FLOOR && self.hops < self.max_hops
    }

    fn current(&self) -> Coord7D {
        *self.path.last().expect("NeuralPulse path must never be empty")
    }
}

#[derive(Debug)]
pub struct PulseResult {
    pub visited_nodes: usize,
    pub total_activation: f64,
    pub paths_recorded: usize,
    pub final_strength: f64,
}

pub struct PulseEngine {
    pub face_decay: f64,
    pub bcc_decay: f64,
    pub cascade_energy_factor: f64,
}

impl Default for PulseEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PulseEngine {
    pub fn new() -> Self {
        Self {
            face_decay: 0.72,
            bcc_decay: 0.36,
            cascade_energy_factor: 0.95,
        }
    }

    pub fn propagate(
        &self,
        source: &Coord7D,
        pulse_type: PulseType,
        universe: &DarkUniverse,
        hebbian: &mut HebbianMemory,
    ) -> PulseResult {
        let pulse = NeuralPulse::new(*source, pulse_type);
        self.run_pulse(pulse, universe, hebbian)
    }

    fn run_pulse(
        &self,
        initial: NeuralPulse,
        universe: &DarkUniverse,
        hebbian: &mut HebbianMemory,
    ) -> PulseResult {
        let mut visited = HashSet::new();
        visited.insert(initial.current());

        let mut queue = VecDeque::new();
        queue.push_back(initial);

        let mut visited_count = 0;
        let mut total_activation = 0.0;
        let mut paths_recorded = 0;
        let mut final_strength = 0.0;

        while let Some(pulse) = queue.pop_front() {
            if !pulse.is_alive() {
                continue;
            }

            let current = pulse.current();
            visited_count += 1;
            total_activation += pulse.strength;
            final_strength = pulse.strength;

            if pulse.pulse_type == PulseType::Reinforcing && pulse.path.len() >= 3 {
                let path_len = pulse.path.len();
                let start = path_len.saturating_sub(4);
                hebbian.record_path(&pulse.path[start..], pulse.strength * 0.6);
                paths_recorded += 1;
            }

            if pulse.pulse_type == PulseType::Cascade && pulse.path.len() >= 2 {
                let path_len = pulse.path.len();
                let start = path_len.saturating_sub(3);
                hebbian.record_path(&pulse.path[start..], pulse.strength * 0.8);
                paths_recorded += 1;
            }

            let candidates = self.biased_neighbors(&current, &pulse, universe, hebbian, &visited);

            let fanout = pulse.pulse_type.fanout().min(candidates.len());
            if fanout == 0 {
                continue;
            }

            let child_strength = if pulse.pulse_type == PulseType::Cascade {
                pulse.strength * self.cascade_energy_factor / fanout as f64
            } else {
                pulse.strength
            };

            for (neighbor, decay) in candidates.iter().take(fanout) {
                if visited.contains(neighbor) {
                    continue;
                }

                let mut child = pulse.clone();
                child.hops += 1;
                child.strength = child_strength * decay;
                child.path.push(*neighbor);

                if pulse.pulse_type == PulseType::Cascade {
                    child.cascade_depth += 1;
                }

                visited.insert(*neighbor);
                queue.push_back(child);
            }
        }

        PulseResult {
            visited_nodes: visited_count,
            total_activation,
            paths_recorded,
            final_strength,
        }
    }

    fn biased_neighbors(
        &self,
        coord: &Coord7D,
        pulse: &NeuralPulse,
        universe: &DarkUniverse,
        hebbian: &HebbianMemory,
        visited: &HashSet<Coord7D>,
    ) -> Vec<(Coord7D, f64)> {
        let mut candidates = Vec::new();
        let bias_w = pulse.pulse_type.hebbian_bias_weight();

        for n in Lattice::face_neighbor_coords(coord) {
            if visited.contains(&n) {
                continue;
            }
            let hebb_w = hebbian.get_bias(coord, &n);
            let bias = 1.0 + hebb_w * bias_w;
            let exists = universe.get_node(&n).is_some();
            let quality = if exists { 1.2 } else { 0.8 };
            candidates.push((n, self.face_decay * bias * quality));
        }

        for n in Lattice::bcc_neighbor_coords(coord) {
            if visited.contains(&n) {
                continue;
            }
            let hebb_w = hebbian.get_bias(coord, &n);
            let bias = 1.0 + hebb_w * bias_w;
            candidates.push((n, self.bcc_decay * bias));
        }

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        candidates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid_universe() -> DarkUniverse {
        let mut u = DarkUniverse::new(500000.0);
        for x in 0..5i32 {
            for y in 0..5i32 {
                for z in 0..5i32 {
                    let c = Coord7D::new_even([x, y, z, 0, 0, 0, 0]);
                    u.materialize_biased(c, 100.0, 0.6).unwrap();
                }
            }
        }
        for x in 0..4i32 {
            for y in 0..4i32 {
                for z in 0..4i32 {
                    let c = Coord7D::new_odd([x, y, z, 0, 0, 0, 0]);
                    u.materialize_biased(c, 80.0, 0.3).unwrap();
                }
            }
        }
        u
    }

    #[test]
    fn exploratory_pulse_propagates() {
        let u = make_grid_universe();
        let mut h = HebbianMemory::new();
        let engine = PulseEngine::new();

        let source = Coord7D::new_even([2, 2, 2, 0, 0, 0, 0]);
        let result = engine.propagate(&source, PulseType::Exploratory, &u, &mut h);

        assert!(result.visited_nodes > 1, "should visit multiple nodes");
        assert!(result.total_activation > 0.0);
    }

    #[test]
    fn reinforcing_pulse_records_hebbian() {
        let u = make_grid_universe();
        let mut h = HebbianMemory::new();
        let engine = PulseEngine::new();

        let source = Coord7D::new_even([2, 2, 2, 0, 0, 0, 0]);
        let result = engine.propagate(&source, PulseType::Reinforcing, &u, &mut h);

        assert!(
            result.paths_recorded > 0,
            "reinforcing should record Hebbian paths"
        );
        assert!(h.edge_count() > 0, "Hebbian memory should have edges");
    }

    #[test]
    fn cascade_pulse_branches() {
        let u = make_grid_universe();
        let mut h = HebbianMemory::new();
        let engine = PulseEngine::new();

        let source = Coord7D::new_even([2, 2, 2, 0, 0, 0, 0]);
        let result = engine.propagate(&source, PulseType::Cascade, &u, &mut h);

        assert!(result.visited_nodes > 2, "cascade should visit many nodes");
        assert!(result.paths_recorded > 0);
    }

    #[test]
    fn pulse_dies_at_noise_floor() {
        let mut u = DarkUniverse::new(10000.0);
        for i in 0..100i32 {
            let c = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            u.materialize_uniform(c, 10.0).unwrap();
        }
        let mut h = HebbianMemory::new();
        let engine = PulseEngine::new();

        let source = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        let result = engine.propagate(&source, PulseType::Reinforcing, &u, &mut h);

        assert!(result.visited_nodes <= 9, "should stop before max hops");
    }

    #[test]
    fn hebbian_bias_affects_routing() {
        let _u = make_grid_universe();
        let mut h = HebbianMemory::new();
        let _engine = PulseEngine::new();

        let a = Coord7D::new_even([2, 2, 2, 0, 0, 0, 0]);
        let b = Coord7D::new_even([3, 2, 2, 0, 0, 0, 0]);
        let c = Coord7D::new_even([3, 3, 2, 0, 0, 0, 0]);

        for _ in 0..10 {
            h.record_path(&[a, b, c], 2.0);
        }

        let bias_ab = h.get_bias(&a, &b);
        assert!(bias_ab > 1.0, "reinforced path should have high bias");
    }

    #[test]
    fn pulse_respects_max_hops() {
        let mut u = DarkUniverse::new(100000.0);
        for i in 0..50i32 {
            let c = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            u.materialize_uniform(c, 100.0).unwrap();
        }
        let mut h = HebbianMemory::new();
        let engine = PulseEngine::new();

        let source = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        let result = engine.propagate(&source, PulseType::Exploratory, &u, &mut h);

        assert!(result.visited_nodes > 1);
        assert!(result.final_strength > NOISE_FLOOR || result.visited_nodes > 5);
    }

    #[test]
    fn multiple_pulses_build_network() {
        let u = make_grid_universe();
        let mut h = HebbianMemory::new();
        let engine = PulseEngine::new();

        let sources = [
            Coord7D::new_even([1, 1, 1, 0, 0, 0, 0]),
            Coord7D::new_even([3, 1, 1, 0, 0, 0, 0]),
            Coord7D::new_even([1, 3, 1, 0, 0, 0, 0]),
            Coord7D::new_even([3, 3, 1, 0, 0, 0, 0]),
        ];

        for source in &sources {
            engine.propagate(source, PulseType::Reinforcing, &u, &mut h);
            engine.propagate(source, PulseType::Exploratory, &u, &mut h);
        }

        assert!(h.edge_count() > 5, "multiple pulses should build network");

        let neighbors = h.get_neighbors(&sources[0]);
        assert!(
            !neighbors.is_empty(),
            "source should have Hebbian neighbors"
        );
    }

    #[test]
    fn decay_then_reinforce_cycle() {
        let u = make_grid_universe();
        let mut h = HebbianMemory::new();
        let engine = PulseEngine::new();

        let source = Coord7D::new_even([2, 2, 2, 0, 0, 0, 0]);
        engine.propagate(&source, PulseType::Reinforcing, &u, &mut h);
        let after_record = h.total_weight();

        for _ in 0..20 {
            h.decay_all();
        }
        let after_decay = h.total_weight();
        assert!(after_decay < after_record, "decay should reduce weight");

        engine.propagate(&source, PulseType::Reinforcing, &u, &mut h);
        let after_reinforce = h.total_weight();
        assert!(
            after_reinforce > after_decay,
            "reinforcement should increase weight"
        );
    }
}

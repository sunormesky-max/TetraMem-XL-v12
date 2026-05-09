// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::cognitive::emotion::PadVector;
use crate::universe::coord::Coord7D;
use crate::universe::core::physics::UniversePhysics;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::lattice::Lattice;
use crate::universe::node::DarkUniverse;
use std::collections::{HashSet, VecDeque};

const NOISE_FLOOR: f64 = 0.01;
const DIM: usize = 7;

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
    hebbian_bias_override: Option<f64>,
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
            hebbian_bias_override: None,
        }
    }

    fn new_with_params(
        source: Coord7D,
        pulse_type: PulseType,
        strength: f64,
        max_hops: usize,
    ) -> Self {
        Self {
            strength,
            hops: 0,
            max_hops,
            pulse_type,
            path: vec![source],
            cascade_depth: 0,
            hebbian_bias_override: None,
        }
    }

    fn set_hebbian_bias_override(&mut self, bias: f64) {
        self.hebbian_bias_override = Some(bias);
    }

    fn effective_hebbian_bias(&self) -> f64 {
        self.hebbian_bias_override
            .unwrap_or_else(|| self.pulse_type.hebbian_bias_weight())
    }

    fn is_alive(&self) -> bool {
        self.strength > NOISE_FLOOR && self.hops < self.max_hops
    }

    fn current(&self) -> Coord7D {
        *self
            .path
            .last()
            .expect("NeuralPulse path must never be empty")
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

#[derive(Debug, Clone)]
pub struct EmotionPulseConfig {
    pub base_face_decay: f64,
    pub base_bcc_decay: f64,
    pub base_fanout_exploratory: usize,
    pub base_fanout_cascade: usize,
    pub base_strength_exploratory: f64,
    pub base_strength_reinforcing: f64,
    pub base_max_hops: usize,
}

#[derive(Debug, Clone)]
pub struct EmotionDecayParams {
    pub face_decay: f64,
    pub bcc_decay: f64,
}

impl Default for EmotionPulseConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl EmotionPulseConfig {
    pub fn new() -> Self {
        Self {
            base_face_decay: 0.72,
            base_bcc_decay: 0.36,
            base_fanout_exploratory: 2,
            base_fanout_cascade: 3,
            base_strength_exploratory: 0.15,
            base_strength_reinforcing: 0.35,
            base_max_hops: 12,
        }
    }

    pub fn modulated_face_decay(&self, pad: &PadVector) -> f64 {
        let valence_factor = 1.0 + 0.15 * pad.pleasure;
        let arousal_factor = 1.0 - 0.1 * pad.arousal;
        (self.base_face_decay * valence_factor * arousal_factor).clamp(0.1, 0.95)
    }

    pub fn modulated_bcc_decay(&self, pad: &PadVector) -> f64 {
        let valence_factor = 1.0 + 0.15 * pad.pleasure;
        let arousal_factor = 1.0 - 0.1 * pad.arousal;
        (self.base_bcc_decay * valence_factor * arousal_factor).clamp(0.05, 0.8)
    }

    pub fn modulated_fanout(&self, pulse_type: PulseType, pad: &PadVector) -> usize {
        let base = match pulse_type {
            PulseType::Exploratory => self.base_fanout_exploratory,
            PulseType::Reinforcing => 1,
            PulseType::Cascade => self.base_fanout_cascade,
        };
        let arousal_bonus = (pad.arousal * 2.0).round() as isize;
        let adjusted = base as isize + arousal_bonus;
        adjusted.max(1) as usize
    }

    pub fn modulated_strength(&self, pulse_type: PulseType, pad: &PadVector) -> f64 {
        let base = match pulse_type {
            PulseType::Exploratory => self.base_strength_exploratory,
            PulseType::Reinforcing => self.base_strength_reinforcing,
            PulseType::Cascade => 0.45,
        };
        let arousal_factor = 1.0 + 0.3 * pad.arousal;
        let valence_factor = 1.0 + 0.1 * pad.pleasure;
        (base * arousal_factor * valence_factor).clamp(0.01, 1.0)
    }

    pub fn modulated_max_hops(&self, pad: &PadVector) -> usize {
        let arousal_bonus = (pad.arousal * 4.0).round() as isize;
        (self.base_max_hops as isize + arousal_bonus).max(4) as usize
    }

    pub fn modulated_hebbian_bias(&self, pulse_type: PulseType, pad: &PadVector) -> f64 {
        let base = pulse_type.hebbian_bias_weight();
        let dominance_factor = 1.0 + 0.3 * pad.dominance;
        (base * dominance_factor).max(0.1)
    }
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
        self.run_pulse(pulse, universe, hebbian, None)
    }

    pub fn propagate_with_physics(
        &self,
        source: &Coord7D,
        pulse_type: PulseType,
        universe: &DarkUniverse,
        hebbian: &mut HebbianMemory,
        physics: &UniversePhysics,
    ) -> PulseResult {
        let pulse = NeuralPulse::new(*source, pulse_type);
        self.run_pulse(pulse, universe, hebbian, Some(physics))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn propagate_with_emotion(
        &self,
        source: &Coord7D,
        pulse_type: PulseType,
        universe: &DarkUniverse,
        hebbian: &mut HebbianMemory,
        physics: Option<&UniversePhysics>,
        emotion_config: &EmotionPulseConfig,
        pad: &PadVector,
    ) -> PulseResult {
        let strength = emotion_config.modulated_strength(pulse_type, pad);
        let max_hops = emotion_config.modulated_max_hops(pad);
        let mut pulse = NeuralPulse::new_with_params(*source, pulse_type, strength, max_hops);
        pulse.set_hebbian_bias_override(emotion_config.modulated_hebbian_bias(pulse_type, pad));

        let face_decay = emotion_config.modulated_face_decay(pad);
        let bcc_decay = emotion_config.modulated_bcc_decay(pad);
        let fanout_override = emotion_config.modulated_fanout(pulse_type, pad);

        self.run_pulse_emotion(
            pulse,
            universe,
            hebbian,
            physics,
            &EmotionDecayParams {
                face_decay,
                bcc_decay,
            },
            fanout_override,
        )
    }

    fn run_pulse(
        &self,
        initial: NeuralPulse,
        universe: &DarkUniverse,
        hebbian: &mut HebbianMemory,
        physics: Option<&UniversePhysics>,
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

            let candidates = match physics {
                Some(p) => {
                    self.biased_neighbors_physics(&current, &pulse, universe, hebbian, &visited, p)
                }
                None => self.biased_neighbors(&current, &pulse, universe, hebbian, &visited),
            };

            let fanout = pulse.pulse_type.fanout().min(candidates.len());
            if fanout == 0 {
                continue;
            }

            let child_strength = pulse.strength * self.cascade_energy_factor / fanout.max(1) as f64;

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

    #[allow(clippy::too_many_arguments)]
    fn run_pulse_emotion(
        &self,
        initial: NeuralPulse,
        universe: &DarkUniverse,
        hebbian: &mut HebbianMemory,
        physics: Option<&UniversePhysics>,
        decay: &EmotionDecayParams,
        fanout_override: usize,
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

            let candidates = match physics {
                Some(p) => self.biased_neighbors_emotion(
                    &current, &pulse, universe, hebbian, &visited, p, decay,
                ),
                None => self.biased_neighbors_emotion_nophysics(
                    &current, &pulse, universe, hebbian, &visited, decay,
                ),
            };

            let fanout = fanout_override.min(candidates.len());
            if fanout == 0 {
                continue;
            }

            let child_strength = pulse.strength * self.cascade_energy_factor / fanout.max(1) as f64;
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
            let score = self.face_decay * bias * quality;
            candidates.push((n, score.clamp(0.0, 1.0)));
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

    fn biased_neighbors_emotion_nophysics(
        &self,
        coord: &Coord7D,
        pulse: &NeuralPulse,
        universe: &DarkUniverse,
        hebbian: &HebbianMemory,
        visited: &HashSet<Coord7D>,
        decay: &EmotionDecayParams,
    ) -> Vec<(Coord7D, f64)> {
        let bias_w = pulse.effective_hebbian_bias();
        let mut candidates = Vec::new();

        for n in Lattice::face_neighbor_coords(coord) {
            if visited.contains(&n) {
                continue;
            }
            let hebb_w = hebbian.get_bias(coord, &n);
            let bias = 1.0 + hebb_w * bias_w;
            let exists = universe.get_node(&n).is_some();
            let quality = if exists { 1.2 } else { 0.8 };
            let score = decay.face_decay * bias * quality;
            candidates.push((n, score.clamp(0.0, 1.0)));
        }

        for n in Lattice::bcc_neighbor_coords(coord) {
            if visited.contains(&n) {
                continue;
            }
            let hebb_w = hebbian.get_bias(coord, &n);
            let bias = 1.0 + hebb_w * bias_w;
            candidates.push((n, decay.bcc_decay * bias));
        }

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        candidates
    }

    #[allow(clippy::too_many_arguments)]
    fn biased_neighbors_emotion(
        &self,
        coord: &Coord7D,
        pulse: &NeuralPulse,
        universe: &DarkUniverse,
        hebbian: &HebbianMemory,
        visited: &HashSet<Coord7D>,
        physics: &UniversePhysics,
        decay: &EmotionDecayParams,
    ) -> Vec<(Coord7D, f64)> {
        let decays = physics.profile.propagation_decays();
        let coord_f = coord.as_f64();
        let bias_w = pulse.effective_hebbian_bias();

        let mut candidates = Vec::new();

        for n in Lattice::face_neighbor_coords(coord) {
            if visited.contains(&n) {
                continue;
            }
            let n_f = n.as_f64();
            let modulation = self.dimension_modulation(&coord_f, &n_f, &decays);
            let d = decay.face_decay * modulation;
            let hebb_w = hebbian.get_bias(coord, &n);
            let bias = 1.0 + hebb_w * bias_w;
            let exists = universe.get_node(&n).is_some();
            let quality = if exists { 1.2 } else { 0.8 };
            let score = d * bias * quality;
            candidates.push((n, score.clamp(0.0, 1.0)));
        }

        for n in Lattice::bcc_neighbor_coords(coord) {
            if visited.contains(&n) {
                continue;
            }
            let n_f = n.as_f64();
            let modulation = self.dimension_modulation(&coord_f, &n_f, &decays);
            let d = decay.bcc_decay * modulation;
            let hebb_w = hebbian.get_bias(coord, &n);
            let bias = 1.0 + hebb_w * bias_w;
            candidates.push((n, d * bias));
        }

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        candidates
    }

    fn biased_neighbors_physics(
        &self,
        coord: &Coord7D,
        pulse: &NeuralPulse,
        universe: &DarkUniverse,
        hebbian: &HebbianMemory,
        visited: &HashSet<Coord7D>,
        physics: &UniversePhysics,
    ) -> Vec<(Coord7D, f64)> {
        let decays = physics.profile.propagation_decays();
        let coord_f = coord.as_f64();
        let bias_w = pulse.pulse_type.hebbian_bias_weight();

        let mut candidates = Vec::new();

        for n in Lattice::face_neighbor_coords(coord) {
            if visited.contains(&n) {
                continue;
            }
            let n_f = n.as_f64();
            let modulation = self.dimension_modulation(&coord_f, &n_f, &decays);
            let decay = self.face_decay * modulation;
            let hebb_w = hebbian.get_bias(coord, &n);
            let bias = 1.0 + hebb_w * bias_w;
            let exists = universe.get_node(&n).is_some();
            let quality = if exists { 1.2 } else { 0.8 };
            let score = decay * bias * quality;
            candidates.push((n, score.clamp(0.0, 1.0)));
        }

        for n in Lattice::bcc_neighbor_coords(coord) {
            if visited.contains(&n) {
                continue;
            }
            let n_f = n.as_f64();
            let modulation = self.dimension_modulation(&coord_f, &n_f, &decays);
            let decay = self.bcc_decay * modulation;
            let hebb_w = hebbian.get_bias(coord, &n);
            let bias = 1.0 + hebb_w * bias_w;
            candidates.push((n, decay * bias));
        }

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        candidates
    }

    fn dimension_modulation(&self, from: &[f64; DIM], to: &[f64; DIM], decays: &[f64; DIM]) -> f64 {
        let mut weighted = 0.0;
        let mut count = 0usize;
        for d in 0..DIM {
            let diff = (to[d] - from[d]).abs();
            if diff > 0.0 {
                weighted += decays[d];
                count += 1;
            }
        }
        if count > 0 {
            weighted / count as f64
        } else {
            1.0
        }
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

    #[test]
    fn physics_propagate_exploratory() {
        let u = make_grid_universe();
        let mut h = HebbianMemory::new();
        let engine = PulseEngine::new();
        let physics = UniversePhysics::rich();

        let source = Coord7D::new_even([2, 2, 2, 0, 0, 0, 0]);
        let result =
            engine.propagate_with_physics(&source, PulseType::Exploratory, &u, &mut h, &physics);

        assert!(result.visited_nodes > 1, "physics pulse should visit nodes");
        assert!(result.total_activation > 0.0);
    }

    #[test]
    fn physics_propagate_reinforcing() {
        let u = make_grid_universe();
        let mut h = HebbianMemory::new();
        let engine = PulseEngine::new();
        let physics = UniversePhysics::rich();

        let source = Coord7D::new_even([2, 2, 2, 0, 0, 0, 0]);
        let result =
            engine.propagate_with_physics(&source, PulseType::Reinforcing, &u, &mut h, &physics);

        assert!(
            result.paths_recorded > 0,
            "physics reinforcing should record paths"
        );
    }

    #[test]
    fn physics_propagate_cascade() {
        let u = make_grid_universe();
        let mut h = HebbianMemory::new();
        let engine = PulseEngine::new();
        let physics = UniversePhysics::rich();

        let source = Coord7D::new_even([2, 2, 2, 0, 0, 0, 0]);
        let result =
            engine.propagate_with_physics(&source, PulseType::Cascade, &u, &mut h, &physics);

        assert!(
            result.visited_nodes > 2,
            "physics cascade should visit many nodes"
        );
    }

    #[test]
    fn emotion_modulated_differs_from_flat() {
        let u = make_grid_universe();
        let source = Coord7D::new_even([2, 2, 2, 0, 0, 0, 0]);

        let mut h1 = HebbianMemory::new();
        let mut h2 = HebbianMemory::new();
        let engine = PulseEngine::new();
        let pad = PadVector {
            pleasure: 0.8,
            arousal: 0.6,
            dominance: 0.4,
        };
        let emotion_config = EmotionPulseConfig::new();

        let flat_result = engine.propagate(&source, PulseType::Exploratory, &u, &mut h1);
        let emotion_result = engine.propagate_with_emotion(
            &source,
            PulseType::Exploratory,
            &u,
            &mut h2,
            None,
            &emotion_config,
            &pad,
        );

        assert!(
            (flat_result.total_activation - emotion_result.total_activation).abs() > 0.001,
            "emotion-modulated pulse should differ from flat: flat={}, emotion={}",
            flat_result.total_activation,
            emotion_result.total_activation,
        );
    }

    #[test]
    fn flat_physics_similar_to_no_physics() {
        let u = make_grid_universe();
        let source = Coord7D::new_even([2, 2, 2, 0, 0, 0, 0]);

        let mut h1 = HebbianMemory::new();
        let mut h2 = HebbianMemory::new();
        let engine = PulseEngine::new();

        let no_phys = engine.propagate(&source, PulseType::Exploratory, &u, &mut h1);
        let flat_phys = engine.propagate_with_physics(
            &source,
            PulseType::Exploratory,
            &u,
            &mut h2,
            &UniversePhysics::flat(),
        );

        let ratio = no_phys.total_activation / flat_phys.total_activation.max(1e-15);
        assert!(
            (ratio - 1.0).abs() < 0.3,
            "flat physics should produce similar activation: no_phys={}, flat={}, ratio={}",
            no_phys.total_activation,
            flat_phys.total_activation,
            ratio,
        );
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::coord::Coord7D;
use crate::universe::core::physics::{CouplingMatrix, UniversePhysics};
use crate::universe::energy::{
    EnergyError, EnergyField, EnergyPool, EPSILON_NORMAL, EPSILON_RELATIVE,
};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeState {
    Dark,
    Manifested,
}

#[derive(Clone)]
pub struct DarkNode {
    coord: Coord7D,
    energy: EnergyField,
}

impl DarkNode {
    pub fn new(coord: Coord7D, energy: EnergyField) -> Self {
        Self { coord, energy }
    }

    pub fn coord(&self) -> &Coord7D {
        &self.coord
    }

    pub fn energy(&self) -> &EnergyField {
        &self.energy
    }

    pub fn energy_mut(&mut self) -> &mut EnergyField {
        &mut self.energy
    }

    pub fn state(&self, threshold: f64) -> NodeState {
        if self.energy.is_manifested(threshold) {
            NodeState::Manifested
        } else {
            NodeState::Dark
        }
    }

    pub fn is_manifested_with(&self, threshold: f64) -> bool {
        self.energy.is_manifested(threshold)
    }

    pub fn is_dark_with(&self, threshold: f64) -> bool {
        !self.is_manifested_with(threshold)
    }

    pub fn physical_coords(&self) -> [i32; 3] {
        self.coord.physical()
    }

    pub fn dark_coords(&self) -> [i32; 4] {
        self.coord.dark()
    }

    pub fn manifestation_ratio(&self) -> f64 {
        self.energy.manifestation_ratio()
    }
}

#[derive(Clone)]
pub struct DarkUniverse {
    pool: EnergyPool,
    nodes: HashMap<Coord7D, DarkNode>,
    protected: HashSet<Coord7D>,
    manifestation_threshold: f64,
    physics: Option<UniversePhysics>,
}

impl DarkUniverse {
    pub fn new(total_energy: f64) -> Self {
        Self::new_with_threshold(total_energy, 0.5)
    }

    pub fn new_with_threshold(total_energy: f64, manifestation_threshold: f64) -> Self {
        let pool = EnergyPool::new(total_energy)
            .unwrap_or_else(|_| EnergyPool::new(1.0).expect("EnergyPool::new(1.0) must succeed"));
        Self {
            pool,
            nodes: HashMap::new(),
            protected: HashSet::new(),
            manifestation_threshold: manifestation_threshold.clamp(0.0, 1.0),
            physics: None,
        }
    }

    pub fn new_with_physics(total_energy: f64, physics: UniversePhysics) -> Self {
        let pool = EnergyPool::new(total_energy)
            .unwrap_or_else(|_| EnergyPool::new(1.0).expect("EnergyPool::new(1.0) must succeed"));
        let threshold = physics.phase.threshold;
        Self {
            pool,
            nodes: HashMap::new(),
            protected: HashSet::new(),
            manifestation_threshold: threshold.clamp(0.0, 1.0),
            physics: Some(physics),
        }
    }

    pub fn manifestation_threshold(&self) -> f64 {
        self.manifestation_threshold
    }

    pub fn set_manifestation_threshold(&mut self, threshold: f64) {
        self.manifestation_threshold = threshold.clamp(0.0, 1.0);
    }

    pub fn physics(&self) -> Option<&UniversePhysics> {
        self.physics.as_ref()
    }

    pub fn set_physics(&mut self, physics: UniversePhysics) {
        self.manifestation_threshold = physics.phase.threshold.clamp(0.0, 1.0);
        self.physics = Some(physics);
    }

    fn is_manifested_internal(&self, ratio: f64) -> bool {
        if let Some(ref p) = self.physics {
            p.is_manifested(ratio)
        } else {
            ratio >= self.manifestation_threshold
        }
    }

    pub fn protect(&mut self, coords: &[Coord7D]) {
        for c in coords {
            self.protected.insert(*c);
        }
    }

    pub fn unprotect(&mut self, coords: &[Coord7D]) {
        for c in coords {
            self.protected.remove(c);
        }
    }

    pub fn is_protected(&self, coord: &Coord7D) -> bool {
        self.protected.contains(coord)
    }

    pub fn available_energy(&self) -> f64 {
        self.pool.available()
    }

    pub fn total_energy(&self) -> f64 {
        self.pool.total()
    }

    pub fn allocated_energy(&self) -> f64 {
        self.pool.allocated()
    }

    pub fn active_node_count(&self) -> usize {
        self.nodes.values().filter(|n| !n.energy.is_empty()).count()
    }

    pub fn manifested_node_count(&self) -> usize {
        self.nodes
            .values()
            .filter(|n| self.is_manifested_internal(n.manifestation_ratio()))
            .count()
    }

    pub fn utilization(&self) -> f64 {
        self.pool.utilization()
    }

    pub fn expand_energy_pool(&mut self, additional: f64) -> bool {
        self.pool.expand(additional).is_ok()
    }

    pub fn expand_energy_pool_with_cap(&mut self, additional: f64, max_total: f64) -> bool {
        self.pool.expand_with_cap(additional, max_total).is_ok()
    }

    pub fn shrink_energy_pool(&mut self, amount: f64) -> bool {
        self.pool.shrink(amount).is_ok()
    }

    pub fn materialize_uniform(
        &mut self,
        coord: Coord7D,
        energy_amount: f64,
    ) -> Result<(), EnergyError> {
        if self.nodes.contains_key(&coord) {
            return Err(EnergyError::AlreadyOccupied);
        }
        let allocated = self.pool.allocate(energy_amount)?;
        let field = EnergyField::uniform(allocated);
        self.nodes.insert(coord, DarkNode::new(coord, field));
        Ok(())
    }

    pub fn materialize_biased(
        &mut self,
        coord: Coord7D,
        energy_amount: f64,
        physical_ratio: f64,
    ) -> Result<(), EnergyError> {
        if self.nodes.contains_key(&coord) {
            return Err(EnergyError::AlreadyOccupied);
        }
        let allocated = self.pool.allocate(energy_amount)?;
        let field = EnergyField::with_physical_bias(allocated, physical_ratio);
        self.nodes.insert(coord, DarkNode::new(coord, field));
        Ok(())
    }

    pub fn materialize_field(
        &mut self,
        coord: Coord7D,
        field: EnergyField,
    ) -> Result<(), EnergyError> {
        if self.nodes.contains_key(&coord) {
            return Err(EnergyError::AlreadyOccupied);
        }
        self.pool.allocate(field.total())?;
        self.nodes.insert(coord, DarkNode::new(coord, field));
        Ok(())
    }

    pub fn dematerialize(&mut self, coord: &Coord7D) -> Option<EnergyField> {
        if self.protected.contains(coord) {
            return None;
        }
        if let Some(node) = self.nodes.remove(coord) {
            let field = node.energy;
            if let Err(e) = self.pool.release_field(&field) {
                tracing::error!(
                    "dematerialize: release_field failed for {:?}: {:?}, re-inserting node",
                    coord,
                    e
                );
                self.nodes.insert(*coord, DarkNode::new(*coord, field));
                return None;
            }
            self.protected.remove(coord);
            return Some(field);
        }
        None
    }

    pub fn transfer_energy(
        &mut self,
        from: &Coord7D,
        to: &Coord7D,
        amount: f64,
    ) -> Result<(), EnergyError> {
        if from == to {
            return Ok(());
        }
        let taken = {
            let from_node = self
                .nodes
                .get_mut(from)
                .ok_or(EnergyError::InsufficientEnergy {
                    requested: amount,
                    available: 0.0,
                })?;
            from_node.energy.split_amount(amount)?
        };

        if let Some(to_node) = self.nodes.get_mut(to) {
            to_node.energy.absorb(&taken);
        } else {
            self.nodes.insert(*to, DarkNode::new(*to, taken));
        }

        let should_remove = self.nodes.get(from).is_some_and(|n| n.energy.is_empty());

        if should_remove {
            if let Some(empty) = self.nodes.remove(from) {
                if let Err(e) = self.pool.release_field(&empty.energy) {
                    tracing::error!(
                        "transfer_energy: release_field failed for {:?}: {:?}",
                        from,
                        e
                    );
                }
            }
        }

        Ok(())
    }

    pub fn flow_node_physical_to_dark(
        &mut self,
        coord: &Coord7D,
        amount: f64,
    ) -> Result<(), EnergyError> {
        let node = self
            .nodes
            .get_mut(coord)
            .ok_or(EnergyError::InsufficientEnergy {
                requested: amount,
                available: 0.0,
            })?;
        node.energy.flow_physical_to_dark(amount)
    }

    pub fn flow_node_dark_to_physical(
        &mut self,
        coord: &Coord7D,
        amount: f64,
    ) -> Result<(), EnergyError> {
        let node = self
            .nodes
            .get_mut(coord)
            .ok_or(EnergyError::InsufficientEnergy {
                requested: amount,
                available: 0.0,
            })?;
        node.energy.flow_dark_to_physical(amount)
    }

    pub fn get_node(&self, coord: &Coord7D) -> Option<&DarkNode> {
        self.nodes.get(coord)
    }

    pub fn get_node_mut(&mut self, coord: &Coord7D) -> Option<&mut DarkNode> {
        self.nodes.get_mut(coord)
    }

    pub fn get_manifested_nodes(&self) -> Vec<&DarkNode> {
        self.nodes
            .values()
            .filter(|n| self.is_manifested_internal(n.manifestation_ratio()))
            .collect()
    }

    pub fn get_all_nodes(&self) -> &HashMap<Coord7D, DarkNode> {
        &self.nodes
    }

    pub fn contains(&self, coord: &Coord7D) -> bool {
        self.nodes.contains_key(coord)
    }

    pub fn coords(&self) -> Vec<Coord7D> {
        self.nodes.keys().copied().collect()
    }

    pub fn coords_iter(&self) -> impl Iterator<Item = Coord7D> + '_ {
        self.nodes.keys().copied()
    }

    pub fn coupled_flow(
        &mut self,
        coord: &Coord7D,
        from_dim: usize,
        amount: f64,
    ) -> Result<f64, EnergyError> {
        let node = self
            .nodes
            .get_mut(coord)
            .ok_or(EnergyError::InsufficientEnergy {
                requested: amount,
                available: 0.0,
            })?;
        let default_coupling = CouplingMatrix::new();
        let coupling = self
            .physics
            .as_ref()
            .map(|p| &p.coupling)
            .unwrap_or(&default_coupling);
        let available = node.energy.dim(from_dim);
        if available < amount - crate::universe::energy::EPSILON_STRICT {
            return Err(EnergyError::InsufficientEnergy {
                requested: amount,
                available,
            });
        }
        let actual = amount.min(available);
        let mut dims = *node.energy.dims();
        let net = coupling.coupled_flow(&mut dims, from_dim, actual);
        node.energy = EnergyField::from_dims(dims).map_err(|_| EnergyError::NegativeDimension {
            dim: from_dim,
            value: 0.0,
        })?;
        Ok(net)
    }

    pub fn weighted_distance_sq(&self, a: &Coord7D, b: &Coord7D) -> f64 {
        if let Some(ref p) = self.physics {
            p.weighted_distance_sq(&a.as_f64(), &b.as_f64())
        } else {
            a.distance_sq(b)
        }
    }

    pub fn project_to_physical(&self, coord: &Coord7D) -> [f64; 3] {
        if let Some(ref p) = self.physics {
            p.project_to_physical(&coord.as_f64())
        } else {
            let v = coord.as_f64();
            [v[0], v[1], v[2]]
        }
    }

    pub fn verify_conservation(&self) -> bool {
        if !self.pool.verify_conservation() {
            return false;
        }
        let mut node_total = 0.0;
        for node in self.nodes.values() {
            if !node.energy.verify_integrity() {
                return false;
            }
            node_total += node.energy.total();
        }
        let diff = (node_total - self.pool.allocated()).abs();
        let scale = self.pool.allocated().max(1.0);
        diff / scale < EPSILON_RELATIVE || diff < EPSILON_NORMAL
    }

    pub fn verify_conservation_with_tolerance(&self, tolerance: f64) -> bool {
        if !self.pool.verify_conservation_with_tolerance(tolerance) {
            return false;
        }
        let mut node_total = 0.0;
        for node in self.nodes.values() {
            if !node.energy.verify_integrity() {
                return false;
            }
            node_total += node.energy.total();
        }
        let diff = (node_total - self.pool.allocated()).abs();
        diff < tolerance
    }

    pub fn energy_drift(&self) -> f64 {
        self.pool.energy_drift()
    }

    pub fn stats(&self) -> UniverseStats {
        let mut active = 0usize;
        let mut manifested = 0usize;
        let mut dark = 0usize;
        let mut even_count = 0usize;
        let mut odd_count = 0usize;
        let mut total_physical_energy = 0.0f64;
        let mut total_dark_energy = 0.0f64;

        for node in self.nodes.values() {
            if !node.energy.is_empty() {
                active += 1;
                total_physical_energy += node.energy.physical();
                total_dark_energy += node.energy.dark();

                if self.is_manifested_internal(node.manifestation_ratio()) {
                    manifested += 1;
                } else {
                    dark += 1;
                }

                if node.coord.is_even() {
                    even_count += 1;
                } else {
                    odd_count += 1;
                }
            }
        }

        UniverseStats {
            active_nodes: active,
            manifested_nodes: manifested,
            dark_nodes: dark,
            even_nodes: even_count,
            odd_nodes: odd_count,
            total_energy: self.pool.total(),
            allocated_energy: self.pool.allocated(),
            available_energy: self.pool.available(),
            physical_energy: total_physical_energy,
            dark_energy: total_dark_energy,
            utilization: self.pool.utilization(),
        }
    }
}

#[derive(Debug)]
pub struct UniverseStats {
    pub active_nodes: usize,
    pub manifested_nodes: usize,
    pub dark_nodes: usize,
    pub even_nodes: usize,
    pub odd_nodes: usize,
    pub total_energy: f64,
    pub allocated_energy: f64,
    pub available_energy: f64,
    pub physical_energy: f64,
    pub dark_energy: f64,
    pub utilization: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn universe_starts_empty() {
        let u = DarkUniverse::new(1000.0);
        assert_eq!(u.active_node_count(), 0);
        assert_eq!(u.available_energy(), 1000.0);
        assert!(u.verify_conservation());
    }

    #[test]
    fn materialize_uniform() {
        let mut u = DarkUniverse::new(1000.0);
        let coord = Coord7D::new_even([0; 7]);
        u.materialize_uniform(coord, 100.0).unwrap();
        assert_eq!(u.active_node_count(), 1);
        assert_eq!(u.available_energy(), 900.0);
        assert!(u.verify_conservation());
    }

    #[test]
    fn materialize_biased_manifested() {
        let mut u = DarkUniverse::new(1000.0);
        let coord = Coord7D::new_even([0; 7]);
        u.materialize_biased(coord, 100.0, 0.8).unwrap();
        assert_eq!(u.manifested_node_count(), 1);
        assert!(u.verify_conservation());
    }

    #[test]
    fn materialize_biased_dark() {
        let mut u = DarkUniverse::new(1000.0);
        let coord = Coord7D::new_even([0; 7]);
        u.materialize_biased(coord, 100.0, 0.3).unwrap();
        assert_eq!(u.manifested_node_count(), 0);
        assert_eq!(u.active_node_count(), 1);
        assert!(u.verify_conservation());
    }

    #[test]
    fn dematerialize_returns_energy() {
        let mut u = DarkUniverse::new(1000.0);
        let coord = Coord7D::new_even([0; 7]);
        u.materialize_uniform(coord, 100.0).unwrap();
        let returned = u.dematerialize(&coord).unwrap();
        assert!((returned.total() - 100.0).abs() < 1e-10);
        assert_eq!(u.available_energy(), 1000.0);
        assert!(u.verify_conservation());
    }

    #[test]
    fn cannot_exceed_energy_budget() {
        let mut u = DarkUniverse::new(100.0);
        let coord = Coord7D::new_even([0; 7]);
        assert!(u.materialize_uniform(coord, 200.0).is_err());
    }

    #[test]
    fn transfer_between_nodes() {
        let mut u = DarkUniverse::new(1000.0);
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        u.materialize_biased(a, 100.0, 0.8).unwrap();
        u.materialize_biased(b, 50.0, 0.3).unwrap();
        u.transfer_energy(&a, &b, 30.0).unwrap();

        let na = u.get_node(&a).unwrap();
        let nb = u.get_node(&b).unwrap();
        assert!((na.energy().total() - 70.0).abs() < 1e-10);
        assert!((nb.energy().total() - 80.0).abs() < 1e-10);
        assert!(u.verify_conservation());
    }

    #[test]
    fn flow_physical_to_dark_changes_state() {
        let mut u = DarkUniverse::new(1000.0);
        let coord = Coord7D::new_even([0; 7]);
        u.materialize_biased(coord, 100.0, 0.8).unwrap();
        assert!(u
            .get_node(&coord)
            .unwrap()
            .is_manifested_with(u.manifestation_threshold()));

        u.flow_node_physical_to_dark(&coord, 50.0).unwrap();
        assert!(!u
            .get_node(&coord)
            .unwrap()
            .is_manifested_with(u.manifestation_threshold()));
        assert!(u.verify_conservation());
    }

    #[test]
    fn flow_dark_to_physical_changes_state() {
        let mut u = DarkUniverse::new(1000.0);
        let coord = Coord7D::new_even([0; 7]);
        u.materialize_biased(coord, 100.0, 0.2).unwrap();
        assert!(!u
            .get_node(&coord)
            .unwrap()
            .is_manifested_with(u.manifestation_threshold()));

        u.flow_node_dark_to_physical(&coord, 40.0).unwrap();
        assert!(u
            .get_node(&coord)
            .unwrap()
            .is_manifested_with(u.manifestation_threshold()));
        assert!(u.verify_conservation());
    }

    #[test]
    fn flow_preserves_total() {
        let mut u = DarkUniverse::new(1000.0);
        let coord = Coord7D::new_even([0; 7]);
        u.materialize_uniform(coord, 100.0).unwrap();
        let before = u.allocated_energy();

        u.flow_node_physical_to_dark(&coord, 20.0).unwrap();
        assert!((u.allocated_energy() - before).abs() < 1e-10);

        u.flow_node_dark_to_physical(&coord, 20.0).unwrap();
        assert!((u.allocated_energy() - before).abs() < 1e-10);
        assert!(u.verify_conservation());
    }

    #[test]
    fn even_and_odd_coexist() {
        let mut u = DarkUniverse::new(1000.0);
        u.materialize_uniform(Coord7D::new_even([0; 7]), 50.0)
            .unwrap();
        u.materialize_uniform(Coord7D::new_odd([0; 7]), 50.0)
            .unwrap();

        let stats = u.stats();
        assert_eq!(stats.even_nodes, 1);
        assert_eq!(stats.odd_nodes, 1);
        assert!(u.verify_conservation());
    }

    #[test]
    fn stats_physical_dark_split() {
        let mut u = DarkUniverse::new(10000.0);
        u.materialize_biased(Coord7D::new_even([0; 7]), 100.0, 0.8)
            .unwrap();
        u.materialize_biased(Coord7D::new_even([1; 7]), 100.0, 0.2)
            .unwrap();

        let stats = u.stats();
        assert!((stats.physical_energy - 100.0).abs() < 1e-10);
        assert!((stats.dark_energy - 100.0).abs() < 1e-10);
        assert!(u.verify_conservation());
    }

    #[test]
    fn full_stress_test() {
        let mut u = DarkUniverse::new(5000.0);
        let mut coords = Vec::new();

        for i in 0..20 {
            let c = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            let ratio = 0.3 + ((i % 5) as f64) * 0.1;
            u.materialize_biased(c, 100.0, ratio).unwrap();
            coords.push(c);
        }
        assert_eq!(u.active_node_count(), 20);
        assert!(u.verify_conservation());

        for i in (0..20).step_by(2) {
            u.flow_node_physical_to_dark(&coords[i], 30.0).unwrap();
        }
        assert!(u.verify_conservation());

        for i in (1..20).step_by(2) {
            u.flow_node_dark_to_physical(&coords[i], 20.0).unwrap();
        }
        assert!(u.verify_conservation());

        for c in &coords[15..] {
            u.dematerialize(c);
        }
        assert!(u.verify_conservation());

        let stats = u.stats();
        assert_eq!(stats.active_nodes, 15);
        assert!(u.verify_conservation());
    }
}

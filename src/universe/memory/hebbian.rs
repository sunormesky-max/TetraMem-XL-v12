// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::coord::Coord7D;
use std::collections::HashMap;

const DEFAULT_MAX_PATHS: usize = 4000;
const DEFAULT_DECAY: f64 = 0.98;
const DEFAULT_REINFORCE: f64 = 1.15;
const DEFAULT_MIN_WEIGHT: f64 = 0.01;
const GOLDEN_THRESHOLD: usize = 10;
const GOLDEN_MULTIPLIER: f64 = 1.5;
const MAX_EDGE_WEIGHT: f64 = 10.0;

#[derive(Debug, Clone)]
pub struct HebbianEdge {
    weight: f64,
    traversal_count: usize,
}

impl HebbianEdge {
    pub fn new(weight: f64) -> Self {
        Self {
            weight,
            traversal_count: 1,
        }
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }

    pub fn traversal_count(&self) -> usize {
        self.traversal_count
    }
}

fn canonical_edge(a: &Coord7D, b: &Coord7D) -> (Coord7D, Coord7D) {
    if a <= b {
        (*a, *b)
    } else {
        (*b, *a)
    }
}

#[derive(Clone)]
pub struct HebbianMemory {
    edges: HashMap<(Coord7D, Coord7D), HebbianEdge>,
    pub max_paths: usize,
    decay: f64,
    reinforce: f64,
    min_weight: f64,
}

impl Default for HebbianMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl HebbianMemory {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            max_paths: DEFAULT_MAX_PATHS,
            decay: DEFAULT_DECAY,
            reinforce: DEFAULT_REINFORCE,
            min_weight: DEFAULT_MIN_WEIGHT,
        }
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn total_weight(&self) -> f64 {
        self.edges.values().map(|e| e.weight).sum()
    }

    pub fn record_path(&mut self, path: &[Coord7D], strength: f64) {
        if path.len() < 2 || !strength.is_finite() || strength < 0.0 {
            return;
        }

        let edge_strength = (strength * self.reinforce / (path.len() - 1) as f64).min(2.0);

        for i in 0..path.len() - 1 {
            let key = canonical_edge(&path[i], &path[i + 1]);

            if let Some(edge) = self.edges.get_mut(&key) {
                edge.weight = (edge.weight + edge_strength).min(MAX_EDGE_WEIGHT);
                edge.traversal_count += 1;

                if edge.traversal_count == GOLDEN_THRESHOLD {
                    edge.weight = (edge.weight * GOLDEN_MULTIPLIER).min(MAX_EDGE_WEIGHT);
                }
            } else {
                self.edges.insert(key, HebbianEdge::new(edge_strength));
            }
        }

        if self.edges.len() > self.max_paths * 3 / 2 {
            self.prune();
        }
    }

    pub fn get_bias(&self, a: &Coord7D, b: &Coord7D) -> f64 {
        let key = canonical_edge(a, b);
        self.edges.get(&key).map_or(0.0, |e| e.weight)
    }

    pub fn get_neighbors(&self, node: &Coord7D) -> Vec<(Coord7D, f64)> {
        let mut result = Vec::new();
        for ((a, b), edge) in &self.edges {
            if a == node {
                result.push((*b, edge.weight));
            } else if b == node {
                result.push((*a, edge.weight));
            }
        }
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    pub fn decay_all(&mut self) {
        let min_w = self.min_weight;
        self.edges.retain(|_, edge| {
            edge.weight *= self.decay;
            edge.weight >= min_w
        });
    }

    pub fn prune(&mut self) {
        let target = self.max_paths * 4 / 5;
        if self.edges.len() <= target {
            return;
        }

        let mut entries: Vec<_> = self.edges.drain().collect();
        entries.sort_by(|a, b| {
            b.1.weight
                .partial_cmp(&a.1.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        entries.truncate(target);
        self.edges = entries.into_iter().collect();
    }

    pub fn strongest_edges(&self, n: usize) -> Vec<((Coord7D, Coord7D), f64)> {
        let mut entries: Vec<_> = self.edges.iter().map(|(k, e)| (*k, e.weight)).collect();
        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        entries.truncate(n);
        entries
    }

    pub fn edges_with_traversal(&self) -> Vec<((Coord7D, Coord7D), f64, usize)> {
        self.edges
            .iter()
            .map(|(k, e)| (*k, e.weight, e.traversal_count))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_path_creates_edges() {
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let c = Coord7D::new_even([2, 0, 0, 0, 0, 0, 0]);

        h.record_path(&[a, b, c], 1.0);
        assert_eq!(h.edge_count(), 2);
        assert!(h.get_bias(&a, &b) > 0.0);
        assert!(h.get_bias(&b, &c) > 0.0);
    }

    #[test]
    fn get_bias_unknown_returns_zero() {
        let h = HebbianMemory::new();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        assert_eq!(h.get_bias(&a, &b), 0.0);
    }

    #[test]
    fn get_bias_bidirectional() {
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        h.record_path(&[a, b], 1.0);

        assert!((h.get_bias(&a, &b) - h.get_bias(&b, &a)).abs() < 1e-10);
    }

    #[test]
    fn decay_all_reduces_weights() {
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        h.record_path(&[a, b], 1.0);

        let before = h.get_bias(&a, &b);
        h.decay_all();
        let after = h.get_bias(&a, &b);
        assert!(after < before);
    }

    #[test]
    fn decay_all_removes_weak_edges() {
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        h.record_path(&[a, b], 0.001);

        for _ in 0..200 {
            h.decay_all();
        }
        assert_eq!(h.edge_count(), 0);
    }

    #[test]
    fn prune_removes_weakest() {
        let mut h = HebbianMemory::new();
        h.max_paths = 5;

        for i in 0..10i32 {
            let a = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            let b = Coord7D::new_even([i, 1, 0, 0, 0, 0, 0]);
            h.record_path(&[a, b], (i + 1) as f64);
        }

        assert!(h.edge_count() >= 5);
        h.prune();
        assert!(h.edge_count() <= 4);
    }

    #[test]
    fn golden_path_boost() {
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);

        for _ in 0..9 {
            h.record_path(&[a, b], 0.1);
        }
        let before = h.get_bias(&a, &b);

        h.record_path(&[a, b], 0.1);
        let after = h.get_bias(&a, &b);
        assert!(after > before * 1.4, "golden boost should increase weight");
    }

    #[test]
    fn get_neighbors_returns_connected() {
        let mut h = HebbianMemory::new();
        let center = Coord7D::new_even([0; 7]);
        let n1 = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let n2 = Coord7D::new_even([0, 1, 0, 0, 0, 0, 0]);

        h.record_path(&[center, n1], 1.0);
        h.record_path(&[center, n2], 2.0);

        let neighbors = h.get_neighbors(&center);
        assert_eq!(neighbors.len(), 2);
        assert!(
            neighbors[0].1 >= neighbors[1].1,
            "should be sorted by weight desc"
        );
    }

    #[test]
    fn reinforcement_builds_strong_paths() {
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let c = Coord7D::new_even([2, 0, 0, 0, 0, 0, 0]);

        for _ in 0..5 {
            h.record_path(&[a, b, c], 1.0);
        }

        let ab = h.get_bias(&a, &b);
        let bc = h.get_bias(&b, &c);
        assert!(ab > 1.0, "reinforced path should be strong");
        assert!(bc > 1.0);
    }
}

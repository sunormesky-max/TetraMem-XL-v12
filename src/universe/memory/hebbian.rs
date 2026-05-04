// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::cognitive::functional_emotion::EmotionSource;
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
    emotion_tag: Option<EmotionSource>,
    emotion_weight: f64,
}

impl HebbianEdge {
    pub fn new(weight: f64) -> Self {
        Self {
            weight,
            traversal_count: 1,
            emotion_tag: None,
            emotion_weight: 0.0,
        }
    }

    pub fn with_emotion(weight: f64, source: EmotionSource) -> Self {
        Self {
            weight,
            traversal_count: 1,
            emotion_tag: Some(source),
            emotion_weight: weight,
        }
    }

    pub fn weight(&self) -> f64 {
        self.weight
    }

    pub fn traversal_count(&self) -> usize {
        self.traversal_count
    }

    pub fn emotion_tag(&self) -> Option<EmotionSource> {
        self.emotion_tag
    }

    pub fn emotion_weight(&self) -> f64 {
        self.emotion_weight
    }
}

fn canonical_edge(a: &Coord7D, b: &Coord7D) -> (Coord7D, Coord7D) {
    if a <= b {
        (*a, *b)
    } else {
        (*b, *a)
    }
}

pub struct HebbianEdgeFull {
    pub key: (Coord7D, Coord7D),
    pub weight: f64,
    pub traversal_count: usize,
    pub emotion_tag: Option<EmotionSource>,
    pub emotion_weight: f64,
}

#[derive(Clone)]
pub struct HebbianMemory {
    edges: HashMap<(Coord7D, Coord7D), HebbianEdge>,
    adj: HashMap<Coord7D, Vec<(Coord7D, f64)>>,
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
            adj: HashMap::new(),
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
        self.record_path_internal(path, strength, None);
    }

    pub fn record_path_emotion(&mut self, path: &[Coord7D], strength: f64, source: EmotionSource) {
        self.record_path_internal(path, strength, Some(source));
    }

    fn record_path_internal(
        &mut self,
        path: &[Coord7D],
        strength: f64,
        emotion: Option<EmotionSource>,
    ) {
        if path.len() < 2 || !strength.is_finite() || strength < 0.0 {
            return;
        }

        let edge_strength = (strength * self.reinforce / (path.len() - 1) as f64).min(2.0);

        for i in 0..path.len() - 1 {
            let key = canonical_edge(&path[i], &path[i + 1]);

            let new_weight = if let Some(edge) = self.edges.get_mut(&key) {
                edge.weight = (edge.weight + edge_strength).min(MAX_EDGE_WEIGHT);
                edge.traversal_count += 1;

                if let Some(src) = emotion {
                    edge.emotion_tag = Some(src);
                    edge.emotion_weight += edge_strength;
                }

                if edge.traversal_count == GOLDEN_THRESHOLD {
                    edge.weight = (edge.weight * GOLDEN_MULTIPLIER).min(MAX_EDGE_WEIGHT);
                }

                edge.weight
            } else {
                let edge = match emotion {
                    Some(src) => HebbianEdge::with_emotion(edge_strength, src),
                    None => HebbianEdge::new(edge_strength),
                };
                self.edges.insert(key, edge);
                self.add_adj_entry(&key.0, &key.1, edge_strength);
                continue;
            };

            self.update_adj_weight(&key.0, &key.1, new_weight);
        }

        if self.edges.len() > self.max_paths * 3 / 2 {
            self.prune();
        }
    }

    pub fn get_bias(&self, a: &Coord7D, b: &Coord7D) -> f64 {
        let key = canonical_edge(a, b);
        self.edges.get(&key).map_or(0.0, |e| e.weight)
    }

    pub fn boost_edge(&mut self, a: &Coord7D, b: &Coord7D, boost: f64) {
        let key = canonical_edge(a, b);
        if let Some(edge) = self.edges.get_mut(&key) {
            edge.weight = (edge.weight + boost).min(MAX_EDGE_WEIGHT);
            edge.traversal_count += 1;
        } else {
            self.edges
                .insert(key, HebbianEdge::new(boost.min(MAX_EDGE_WEIGHT)));
        }
    }

    fn add_adj_entry(&mut self, a: &Coord7D, b: &Coord7D, weight: f64) {
        self.adj.entry(*a).or_default().push((*b, weight));
        self.adj.entry(*b).or_default().push((*a, weight));
    }

    fn update_adj_weight(&mut self, a: &Coord7D, b: &Coord7D, weight: f64) {
        if let Some(neighbors) = self.adj.get_mut(a) {
            if let Some(entry) = neighbors.iter_mut().find(|(c, _)| c == b) {
                entry.1 = weight;
            }
        }
        if let Some(neighbors) = self.adj.get_mut(b) {
            if let Some(entry) = neighbors.iter_mut().find(|(c, _)| c == a) {
                entry.1 = weight;
            }
        }
    }

    fn remove_adj_entry(&mut self, a: &Coord7D, b: &Coord7D) {
        if let Some(neighbors) = self.adj.get_mut(a) {
            neighbors.retain(|(c, _)| c != b);
            if neighbors.is_empty() {
                self.adj.remove(a);
            }
        }
        if let Some(neighbors) = self.adj.get_mut(b) {
            neighbors.retain(|(c, _)| c != a);
            if neighbors.is_empty() {
                self.adj.remove(b);
            }
        }
    }

    pub fn get_neighbors(&self, node: &Coord7D) -> Vec<(Coord7D, f64)> {
        let mut result = self.adj.get(node).cloned().unwrap_or_default();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    pub fn decay_all(&mut self) {
        let min_w = self.min_weight;
        let mut to_remove = Vec::new();
        let mut to_update = Vec::new();
        for (k, e) in &mut self.edges {
            e.weight *= self.decay;
            if e.weight < min_w {
                to_remove.push(*k);
            } else {
                to_update.push((*k, e.weight));
            }
        }
        for k in &to_remove {
            self.edges.remove(k);
            self.remove_adj_entry(&k.0, &k.1);
        }
        for ((a, b), w) in &to_update {
            self.update_adj_weight(a, b, *w);
        }
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
        self.rebuild_adj_from_edges();
    }

    fn rebuild_adj_from_edges(&mut self) {
        let entries: Vec<_> = self
            .edges
            .iter()
            .map(|((a, b), e)| (*a, *b, e.weight))
            .collect();
        self.adj.clear();
        for (a, b, w) in &entries {
            self.adj.entry(*a).or_default().push((*b, *w));
            self.adj.entry(*b).or_default().push((*a, *w));
        }
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

    pub fn edges_full(&self) -> Vec<HebbianEdgeFull> {
        self.edges
            .iter()
            .map(|(k, e)| HebbianEdgeFull {
                key: *k,
                weight: e.weight,
                traversal_count: e.traversal_count,
                emotion_tag: e.emotion_tag,
                emotion_weight: e.emotion_weight,
            })
            .collect()
    }

    pub fn restore_edge(
        &mut self,
        a: Coord7D,
        b: Coord7D,
        weight: f64,
        traversal_count: usize,
        emotion_tag: Option<EmotionSource>,
        emotion_weight: f64,
    ) {
        let key = canonical_edge(&a, &b);
        let mut edge = match emotion_tag {
            Some(src) => HebbianEdge::with_emotion(weight, src),
            None => HebbianEdge::new(weight),
        };
        edge.traversal_count = traversal_count;
        edge.emotion_weight = emotion_weight;
        self.edges.insert(key, edge);
        self.add_adj_entry(&a, &b, weight);
    }

    pub fn edges_by_emotion(&self, source: EmotionSource) -> Vec<((Coord7D, Coord7D), f64)> {
        self.edges
            .iter()
            .filter(|(_, e)| e.emotion_tag == Some(source))
            .map(|(k, e)| (*k, e.emotion_weight))
            .collect()
    }

    pub fn get_edge_emotion(&self, a: &Coord7D, b: &Coord7D) -> Option<EmotionSource> {
        let key = canonical_edge(a, b);
        self.edges.get(&key).and_then(|e| e.emotion_tag)
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

    #[test]
    fn record_path_emotion_tags_edges() {
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);

        h.record_path_emotion(&[a, b], 1.0, EmotionSource::Functional);

        let edges = h.edges_by_emotion(EmotionSource::Functional);
        assert_eq!(edges.len(), 1);
        assert!(edges[0].1 > 0.0);
    }

    #[test]
    fn emotion_paths_separate_from_plain() {
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let c = Coord7D::new_even([2, 0, 0, 0, 0, 0, 0]);

        h.record_path(&[a, b], 1.0);
        h.record_path_emotion(&[b, c], 1.0, EmotionSource::Perceived);

        let perceived = h.edges_by_emotion(EmotionSource::Perceived);
        let functional = h.edges_by_emotion(EmotionSource::Functional);
        assert_eq!(perceived.len(), 1);
        assert_eq!(functional.len(), 0);
        assert_eq!(h.edge_count(), 2);
    }
}

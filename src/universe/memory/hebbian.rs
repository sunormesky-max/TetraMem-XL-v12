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
const STDP_LTD_RATE: f64 = 0.15;
const TEMPORAL_DECAY: f64 = 0.95;

#[derive(Debug, Clone)]
pub struct HebbianEdge {
    weight: f64,
    traversal_count: usize,
    emotion_tag: Option<EmotionSource>,
    emotion_weight: f64,
    avg_delay_ms: f64,
    temporal_strength: f64,
}

impl HebbianEdge {
    pub fn new(weight: f64) -> Self {
        Self {
            weight,
            traversal_count: 1,
            emotion_tag: None,
            emotion_weight: 0.0,
            avg_delay_ms: 0.0,
            temporal_strength: 0.0,
        }
    }

    pub fn with_emotion(weight: f64, source: EmotionSource) -> Self {
        Self {
            weight,
            traversal_count: 1,
            emotion_tag: Some(source),
            emotion_weight: weight,
            avg_delay_ms: 0.0,
            temporal_strength: 0.0,
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

    pub fn avg_delay_ms(&self) -> f64 {
        self.avg_delay_ms
    }

    pub fn temporal_strength(&self) -> f64 {
        self.temporal_strength
    }
}

pub struct HebbianEdgeFull {
    pub key: (Coord7D, Coord7D),
    pub weight: f64,
    pub traversal_count: usize,
    pub emotion_tag: Option<EmotionSource>,
    pub emotion_weight: f64,
    pub avg_delay_ms: f64,
    pub temporal_strength: f64,
}

#[derive(Clone)]
pub struct HebbianMemory {
    edges: HashMap<(Coord7D, Coord7D), HebbianEdge>,
    forward: HashMap<Coord7D, Vec<(Coord7D, f64)>>,
    backward: HashMap<Coord7D, Vec<(Coord7D, f64)>>,
    pub max_paths: usize,
    decay: f64,
    reinforce: f64,
    min_weight: f64,
    stdp_ltd_rate: f64,
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
            forward: HashMap::new(),
            backward: HashMap::new(),
            max_paths: DEFAULT_MAX_PATHS,
            decay: DEFAULT_DECAY,
            reinforce: DEFAULT_REINFORCE,
            min_weight: DEFAULT_MIN_WEIGHT,
            stdp_ltd_rate: STDP_LTD_RATE,
        }
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn total_weight(&self) -> f64 {
        self.edges.values().map(|e| e.weight).sum()
    }

    pub fn record_path(&mut self, path: &[Coord7D], strength: f64) {
        self.record_path_internal(path, strength, None, &[]);
    }

    pub fn record_path_emotion(&mut self, path: &[Coord7D], strength: f64, source: EmotionSource) {
        self.record_path_internal(path, strength, Some(source), &[]);
    }

    pub fn record_path_with_timestamps(
        &mut self,
        path: &[Coord7D],
        strength: f64,
        timestamps: &[u64],
    ) {
        self.record_path_internal(path, strength, None, timestamps);
    }

    pub fn record_path_emotion_with_timestamps(
        &mut self,
        path: &[Coord7D],
        strength: f64,
        source: EmotionSource,
        timestamps: &[u64],
    ) {
        self.record_path_internal(path, strength, Some(source), timestamps);
    }

    fn record_path_internal(
        &mut self,
        path: &[Coord7D],
        strength: f64,
        emotion: Option<EmotionSource>,
        timestamps: &[u64],
    ) {
        if path.len() < 2 || !strength.is_finite() || strength < 0.0 {
            return;
        }

        let edge_strength = (strength * self.reinforce / (path.len() - 1) as f64).min(2.0);
        let ltd_amount = edge_strength * self.stdp_ltd_rate;

        for i in 0..path.len() - 1 {
            let src = path[i];
            let tgt = path[i + 1];
            let fwd_key = (src, tgt);
            let rev_key = (tgt, src);

            let delay_ms = if i + 1 < timestamps.len() && timestamps[i + 1] > timestamps[i] {
                (timestamps[i + 1] - timestamps[i]) as f64
            } else {
                0.0
            };

            let new_weight = if let Some(edge) = self.edges.get_mut(&fwd_key) {
                edge.weight = (edge.weight + edge_strength).min(MAX_EDGE_WEIGHT);
                edge.traversal_count += 1;

                if delay_ms > 0.0 {
                    let n = edge.traversal_count as f64;
                    edge.avg_delay_ms = edge.avg_delay_ms * (n - 1.0) / n + delay_ms / n;
                    edge.temporal_strength =
                        (edge.temporal_strength + 1.0).min(edge.traversal_count as f64);
                }

                if let Some(src_kind) = emotion {
                    edge.emotion_tag = Some(src_kind);
                    edge.emotion_weight += edge_strength;
                }

                if edge.traversal_count == GOLDEN_THRESHOLD {
                    edge.weight = (edge.weight * GOLDEN_MULTIPLIER).min(MAX_EDGE_WEIGHT);
                }

                edge.weight
            } else {
                let mut edge = match emotion {
                    Some(s) => HebbianEdge::with_emotion(edge_strength, s),
                    None => HebbianEdge::new(edge_strength),
                };
                if delay_ms > 0.0 {
                    edge.avg_delay_ms = delay_ms;
                    edge.temporal_strength = 1.0;
                }
                self.edges.insert(fwd_key, edge);
                self.add_dir_adj_entry(&src, &tgt, edge_strength);
                0.0
            };

            if new_weight > 0.0 {
                self.update_dir_adj_weight(&src, &tgt, new_weight);
            }

            if let Some(rev) = self.edges.get_mut(&rev_key) {
                let rev_weight = {
                    rev.weight = (rev.weight - ltd_amount).max(0.0);
                    rev.weight
                };
                if rev_weight < self.min_weight {
                    self.edges.remove(&rev_key);
                    self.remove_dir_adj_entry(&tgt, &src);
                } else {
                    self.update_dir_adj_weight(&tgt, &src, rev_weight);
                }
            }
        }

        if self.edges.len() > self.max_paths * 3 / 2 {
            self.prune();
        }
    }

    pub fn get_bias(&self, a: &Coord7D, b: &Coord7D) -> f64 {
        self.edges.get(&(*a, *b)).map_or(0.0, |e| e.weight)
    }

    pub fn get_bias_max(&self, a: &Coord7D, b: &Coord7D) -> f64 {
        let fwd = self.edges.get(&(*a, *b)).map_or(0.0, |e| e.weight);
        let rev = self.edges.get(&(*b, *a)).map_or(0.0, |e| e.weight);
        fwd.max(rev)
    }

    pub fn boost_edge(&mut self, a: &Coord7D, b: &Coord7D, boost: f64) {
        let key = (*a, *b);
        let new_weight = if let Some(edge) = self.edges.get_mut(&key) {
            edge.weight = (edge.weight + boost).min(MAX_EDGE_WEIGHT);
            edge.traversal_count += 1;
            edge.weight
        } else {
            let w = boost.min(MAX_EDGE_WEIGHT);
            self.edges.insert(key, HebbianEdge::new(w));
            self.add_dir_adj_entry(a, b, w);
            return;
        };
        self.update_dir_adj_weight(a, b, new_weight);
    }

    fn add_dir_adj_entry(&mut self, src: &Coord7D, tgt: &Coord7D, weight: f64) {
        self.forward.entry(*src).or_default().push((*tgt, weight));
        self.backward.entry(*tgt).or_default().push((*src, weight));
    }

    fn update_dir_adj_weight(&mut self, src: &Coord7D, tgt: &Coord7D, weight: f64) {
        if let Some(list) = self.forward.get_mut(src) {
            if let Some(entry) = list.iter_mut().find(|(c, _)| c == tgt) {
                entry.1 = weight;
            }
        }
        if let Some(list) = self.backward.get_mut(tgt) {
            if let Some(entry) = list.iter_mut().find(|(c, _)| c == src) {
                entry.1 = weight;
            }
        }
    }

    fn remove_dir_adj_entry(&mut self, src: &Coord7D, tgt: &Coord7D) {
        if let Some(list) = self.forward.get_mut(src) {
            list.retain(|(c, _)| c != tgt);
            if list.is_empty() {
                self.forward.remove(src);
            }
        }
        if let Some(list) = self.backward.get_mut(tgt) {
            list.retain(|(c, _)| c != src);
            if list.is_empty() {
                self.backward.remove(tgt);
            }
        }
    }

    fn rebuild_adj_from_edges(&mut self) {
        let entries: Vec<_> = self
            .edges
            .iter()
            .map(|((src, tgt), e)| (*src, *tgt, e.weight))
            .collect();
        self.forward.clear();
        self.backward.clear();
        for (src, tgt, w) in &entries {
            self.forward.entry(*src).or_default().push((*tgt, *w));
            self.backward.entry(*tgt).or_default().push((*src, *w));
        }
    }

    pub fn get_neighbors(&self, node: &Coord7D) -> Vec<(Coord7D, f64)> {
        let mut seen: HashMap<Coord7D, f64> = HashMap::new();
        if let Some(list) = self.forward.get(node) {
            for &(c, w) in list {
                let e = seen.entry(c).or_default();
                *e = (*e).max(w);
            }
        }
        if let Some(list) = self.backward.get(node) {
            for &(c, w) in list {
                let e = seen.entry(c).or_default();
                *e = (*e).max(w);
            }
        }
        let mut result: Vec<_> = seen.into_iter().collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    pub fn get_successors(&self, node: &Coord7D) -> Vec<(Coord7D, f64)> {
        let mut result = self.forward.get(node).cloned().unwrap_or_default();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    pub fn get_predecessors(&self, node: &Coord7D) -> Vec<(Coord7D, f64)> {
        let mut result = self.backward.get(node).cloned().unwrap_or_default();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    pub fn get_temporal_sequence(&self, anchor: &Coord7D, max_steps: usize) -> Vec<(Coord7D, f64)> {
        let mut result = Vec::new();
        let mut current = *anchor;
        let mut visited = std::collections::HashSet::new();
        visited.insert(current);

        for _ in 0..max_steps {
            let successors = self.forward.get(&current).cloned().unwrap_or_default();
            let best = successors
                .into_iter()
                .filter(|(c, _)| !visited.contains(c))
                .filter(|(c, _)| {
                    self.edges
                        .get(&(current, *c))
                        .is_some_and(|e| e.temporal_strength > 0.0)
                })
                .max_by(|a, b| {
                    let sa = self
                        .edges
                        .get(&(current, a.0))
                        .map_or(0.0, |e| e.temporal_strength * e.weight);
                    let sb = self
                        .edges
                        .get(&(current, b.0))
                        .map_or(0.0, |e| e.temporal_strength * e.weight);
                    sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
                });

            match best {
                Some((next, w)) => {
                    result.push((next, w));
                    visited.insert(next);
                    current = next;
                }
                None => break,
            }
        }
        result
    }

    pub fn get_temporal_context(&self, anchor: &Coord7D, window_ms: f64) -> Vec<(Coord7D, f64)> {
        let all_neighbors = self.get_neighbors(anchor);
        all_neighbors
            .into_iter()
            .filter(|(c, _)| {
                let fwd = self.edges.get(&(*anchor, *c));
                let rev = self.edges.get(&(*c, *anchor));
                let delay = fwd
                    .map(|e| e.avg_delay_ms)
                    .or_else(|| rev.map(|e| e.avg_delay_ms))
                    .unwrap_or(0.0);
                delay > 0.0 && delay <= window_ms
            })
            .collect()
    }

    pub fn decay_all(&mut self) {
        let min_w = self.min_weight;
        let mut to_remove = Vec::new();
        let mut to_update = Vec::new();
        for (k, e) in &mut self.edges {
            e.weight *= self.decay;
            e.temporal_strength *= TEMPORAL_DECAY;
            if e.weight < min_w {
                to_remove.push(*k);
            } else {
                to_update.push((*k, e.weight));
            }
        }
        for (src, tgt) in &to_remove {
            self.edges.remove(&(*src, *tgt));
            self.remove_dir_adj_entry(src, tgt);
        }
        for ((src, tgt), w) in &to_update {
            self.update_dir_adj_weight(src, tgt, *w);
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
                avg_delay_ms: e.avg_delay_ms,
                temporal_strength: e.temporal_strength,
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
        let key = (a, b);
        let mut edge = match emotion_tag {
            Some(src) => HebbianEdge::with_emotion(weight, src),
            None => HebbianEdge::new(weight),
        };
        edge.traversal_count = traversal_count;
        edge.emotion_weight = emotion_weight;
        self.edges.insert(key, edge);
        self.add_dir_adj_entry(&a, &b, weight);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn restore_edge_full(
        &mut self,
        a: Coord7D,
        b: Coord7D,
        weight: f64,
        traversal_count: usize,
        emotion_tag: Option<EmotionSource>,
        emotion_weight: f64,
        avg_delay_ms: f64,
        temporal_strength: f64,
    ) {
        let key = (a, b);
        let mut edge = match emotion_tag {
            Some(src) => HebbianEdge::with_emotion(weight, src),
            None => HebbianEdge::new(weight),
        };
        edge.traversal_count = traversal_count;
        edge.emotion_weight = emotion_weight;
        edge.avg_delay_ms = avg_delay_ms;
        edge.temporal_strength = temporal_strength;
        self.edges.insert(key, edge);
        self.add_dir_adj_entry(&a, &b, weight);
    }

    pub fn edges_by_emotion(&self, source: EmotionSource) -> Vec<((Coord7D, Coord7D), f64)> {
        self.edges
            .iter()
            .filter(|(_, e)| e.emotion_tag == Some(source))
            .map(|(k, e)| (*k, e.emotion_weight))
            .collect()
    }

    pub fn get_edge_emotion(&self, a: &Coord7D, b: &Coord7D) -> Option<EmotionSource> {
        self.edges
            .get(&(*a, *b))
            .and_then(|e| e.emotion_tag)
            .or_else(|| self.edges.get(&(*b, *a)).and_then(|e| e.emotion_tag))
    }

    pub fn reconstruct_from_cue(
        &self,
        seed: &Coord7D,
        memories: &[crate::universe::memory::MemoryAtom],
        memory_index: &std::collections::HashMap<String, usize>,
        max_hops: usize,
    ) -> Option<Vec<(Coord7D, f64, Vec<f64>)>> {
        let seed_key = format!("{}", seed);
        let seed_idx = memory_index.get(&seed_key)?;
        let _seed_mem = memories.get(*seed_idx)?;
        let _seed_data: Vec<f64> = Vec::new();

        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        visited.insert(*seed);

        let mut frontier: Vec<(Coord7D, f64)> = vec![(*seed, 1.0)];

        for _ in 0..max_hops {
            let mut next_frontier = Vec::new();
            for (current, accumulated_weight) in frontier {
                let successors = self.get_successors(&current);
                for (succ, edge_w) in successors {
                    if visited.contains(&succ) {
                        continue;
                    }
                    visited.insert(succ);

                    let combined_w = accumulated_weight * edge_w;
                    let succ_key = format!("{}", succ);
                    let data = if let Some(&idx) = memory_index.get(&succ_key) {
                        if let Some(mem) = memories.get(idx) {
                            // We can't decode here without universe ref, store placeholder
                            vec![0.0; mem.data_dim()]
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    };

                    result.push((succ, combined_w, data));
                    next_frontier.push((succ, combined_w));
                }
            }
            frontier = next_frontier;
            if frontier.is_empty() {
                break;
            }
        }

        if result.is_empty() {
            None
        } else {
            result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            Some(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_path_creates_directed_edges() {
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
    fn directed_edges_are_asymmetric() {
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);

        h.record_path(&[a, b], 1.0);

        let fwd = h.get_bias(&a, &b);
        let rev = h.get_bias(&b, &a);
        assert!(fwd > 0.0, "forward edge should exist");
        assert!(
            rev < fwd * 0.5,
            "reverse should be weakened by STDP LTD: fwd={} rev={}",
            fwd,
            rev
        );
    }

    #[test]
    fn stdp_ltd_weakens_reverse_on_path_record() {
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);

        h.record_path(&[b, a], 1.0);
        let ba = h.get_bias(&b, &a);
        assert!(ba > 0.0);

        h.record_path(&[a, b], 1.0);
        let ba_after = h.get_bias(&b, &a);
        assert!(
            ba_after < ba,
            "STDP LTD should weaken B→A when A→B is recorded: before={} after={}",
            ba,
            ba_after
        );
    }

    #[test]
    fn get_bias_unknown_returns_zero() {
        let h = HebbianMemory::new();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        assert_eq!(h.get_bias(&a, &b), 0.0);
    }

    #[test]
    fn get_bias_max_returns_stronger_direction() {
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);

        for _ in 0..5 {
            h.record_path(&[a, b], 1.0);
        }
        let fwd = h.get_bias(&a, &b);
        let max_val = h.get_bias_max(&a, &b);
        assert!(
            (max_val - fwd).abs() < 1e-10,
            "max should return forward when it's stronger"
        );
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
    fn get_neighbors_includes_both_directions() {
        let mut h = HebbianMemory::new();
        let center = Coord7D::new_even([0; 7]);
        let n1 = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let n2 = Coord7D::new_even([0, 1, 0, 0, 0, 0, 0]);

        h.record_path(&[center, n1], 1.0);
        h.record_path(&[n2, center], 2.0);

        let neighbors = h.get_neighbors(&center);
        assert_eq!(
            neighbors.len(),
            2,
            "should see both successor and predecessor"
        );
    }

    #[test]
    fn get_successors_returns_only_outgoing() {
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let c = Coord7D::new_even([2, 0, 0, 0, 0, 0, 0]);

        h.record_path(&[a, b], 1.0);
        h.record_path(&[c, a], 2.0);

        let succ = h.get_successors(&a);
        assert_eq!(succ.len(), 1, "only b is a successor");
        assert_eq!(succ[0].0, b);

        let pred = h.get_predecessors(&a);
        assert_eq!(pred.len(), 1, "only c is a predecessor");
        assert_eq!(pred[0].0, c);
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

    #[test]
    fn temporal_sequence_follows_strongest_forward() {
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let c = Coord7D::new_even([2, 0, 0, 0, 0, 0, 0]);

        h.record_path_with_timestamps(&[a, b, c], 1.0, &[100, 200, 350]);

        let seq = h.get_temporal_sequence(&a, 5);
        assert_eq!(seq.len(), 2);
        assert_eq!(seq[0].0, b);
        assert_eq!(seq[1].0, c);
    }

    #[test]
    fn temporal_context_filters_by_delay() {
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let c = Coord7D::new_even([2, 0, 0, 0, 0, 0, 0]);

        h.record_path_with_timestamps(&[a, b], 1.0, &[0, 100]);
        h.record_path_with_timestamps(&[a, c], 1.0, &[0, 5000]);

        let ctx = h.get_temporal_context(&a, 200.0);
        assert_eq!(ctx.len(), 1, "only b should be within 200ms window");
        assert_eq!(ctx[0].0, b);
    }

    #[test]
    fn restore_edge_creates_directed() {
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);

        h.restore_edge(a, b, 1.5, 5, None, 0.0);

        assert!(h.get_bias(&a, &b) > 0.0, "A→B should exist");
        assert_eq!(h.get_bias(&b, &a), 0.0, "B→A should not exist");
    }
}

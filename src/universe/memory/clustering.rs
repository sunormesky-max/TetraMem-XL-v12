// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
// Memory Clustering Engine — 4-layer spatial intelligence
//
// L1: SemanticAnchorPlacer — data → dark coordinate mapping, gravity-based placement
// L2: DarkGravityField — energy-based metric curvature, attractor dynamics
// L3: ResonanceTunnel — long-range low-decay Hebbian bridge creation
// L4: TopologyBridge — Betti feedback, automatic wormhole creation

use crate::universe::coord::Coord7D;
use crate::universe::core::lattice::Lattice;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::{MemoryAtom, MemoryCodec, MemoryError};
use crate::universe::node::DarkUniverse;
use std::collections::HashMap;

const DIM: usize = 7;
const DARK_DIM: usize = 4;
const PHYSICAL_DIM: usize = 3;

#[derive(Debug, Clone)]
pub struct ClusteringConfig {
    pub gravity_strength: f64,
    pub gravity_radius: f64,
    pub tunnel_min_hops: usize,
    pub tunnel_max_per_cycle: usize,
    pub bridge_betti0_threshold: usize,
    pub bridge_max_per_cycle: usize,
    pub dark_quantum: f64,
    pub placement_search_radius: i32,
    pub placement_max_candidates: usize,
    pub semantic_hash_seed: u64,
}

impl Default for ClusteringConfig {
    fn default() -> Self {
        Self {
            gravity_strength: 0.3,
            gravity_radius: 20.0,
            tunnel_min_hops: 6,
            tunnel_max_per_cycle: 10,
            bridge_betti0_threshold: 2,
            bridge_max_per_cycle: 5,
            dark_quantum: 0.25,
            placement_search_radius: 8,
            placement_max_candidates: 200,
            semantic_hash_seed: 0x517c_c1b7_2722_0a95,
        }
    }
}

// ─── L1: Semantic Fingerprint → Dark Coordinate Mapping ───────────

fn data_fingerprint(data: &[f64]) -> [f64; DARK_DIM] {
    let n = data.len().max(1) as f64;
    let mut sum = 0.0f64;
    let mut sq_sum = 0.0f64;
    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;
    let mut autocorr = 0.0f64;
    let mut entropy_sum = 0.0f64;

    for &v in data {
        sum += v;
        sq_sum += v * v;
        if v < min_val {
            min_val = v;
        }
        if v > max_val {
            max_val = v;
        }
    }
    let mean = sum / n;
    let variance = (sq_sum / n) - (mean * mean);

    if data.len() > 1 {
        for i in 0..data.len() - 1 {
            autocorr += (data[i] - mean) * (data[i + 1] - mean);
        }
        autocorr /= (data.len() - 1) as f64 * variance.max(1e-10);
    }

    let range = (max_val - min_val).max(1e-10);
    let nbins = 10usize;
    let mut bins = vec![0usize; nbins];
    for &v in data {
        let idx = (((v - min_val) / range * (nbins as f64 - 1.0)).round() as usize).min(nbins - 1);
        bins[idx] += 1;
    }
    for &count in &bins {
        if count > 0 {
            let p = count as f64 / n;
            entropy_sum -= p * p.log2();
        }
    }

    let energy_dark = mean.abs().max(1e-10).ln().abs() * 2.0;
    let space_dark = variance.sqrt().max(1e-10).ln();
    let time_dark = autocorr * 3.0;
    let mu_dark = entropy_sum * 0.5;

    [energy_dark, space_dark, time_dark, mu_dark]
}

pub fn dark_coords_from_data(data: &[f64], quantum: f64) -> [i32; DARK_DIM] {
    let fp = data_fingerprint(data);
    let mut coords = [0i32; DARK_DIM];
    for i in 0..DARK_DIM {
        coords[i] = (fp[i] / quantum).round() as i32;
    }
    coords
}

pub fn semantic_distance(data_a: &[f64], data_b: &[f64]) -> f64 {
    let fa = data_fingerprint(data_a);
    let fb = data_fingerprint(data_b);
    let mut sum = 0.0;
    for i in 0..DARK_DIM {
        let d = fa[i] - fb[i];
        sum += d * d;
    }
    sum.sqrt()
}

// ─── L1: SemanticAnchorPlacer ──────────────────────────────────────

pub struct SemanticAnchorPlacer {
    config: ClusteringConfig,
    memory_anchors: Vec<Coord7D>,
    memory_data_cache: HashMap<Coord7D, Vec<f64>>,
}

impl SemanticAnchorPlacer {
    pub fn new(config: ClusteringConfig) -> Self {
        Self {
            config,
            memory_anchors: Vec::new(),
            memory_data_cache: HashMap::new(),
        }
    }

    pub fn register_memory(&mut self, anchor: Coord7D, data: &[f64]) {
        self.memory_anchors.push(anchor);
        self.memory_data_cache.insert(anchor, data.to_vec());
    }

    pub fn compute_ideal_anchor(&self, data: &[f64], universe: &DarkUniverse) -> Coord7D {
        let new_dark = dark_coords_from_data(data, self.config.dark_quantum);

        let mut best_anchor =
            Coord7D::new_even([0, 0, 0, new_dark[0], new_dark[1], new_dark[2], new_dark[3]]);
        let mut best_score = f64::NEG_INFINITY;

        let gravity_target = self.compute_gravity_target(data, universe);

        for dx in -self.config.placement_search_radius..=self.config.placement_search_radius {
            for dy in -self.config.placement_search_radius..=self.config.placement_search_radius {
                for dz in -self.config.placement_search_radius..=self.config.placement_search_radius
                {
                    if dx * dx + dy * dy + dz * dz > self.config.placement_search_radius.pow(2) {
                        continue;
                    }

                    let candidate = Coord7D::new_even([
                        gravity_target[0] + dx,
                        gravity_target[1] + dy,
                        gravity_target[2] + dz,
                        new_dark[0],
                        new_dark[1],
                        new_dark[2],
                        new_dark[3],
                    ]);

                    if universe.get_node(&candidate).is_some() {
                        continue;
                    }

                    let score = self.score_placement(&candidate, universe);
                    if score > best_score {
                        best_score = score;
                        best_anchor = candidate;
                    }

                    if self.memory_anchors.len() > 100 {
                        break;
                    }
                }
                if self.memory_anchors.len() > 200 {
                    break;
                }
            }
        }

        best_anchor
    }

    fn compute_gravity_target(&self, data: &[f64], universe: &DarkUniverse) -> [i32; PHYSICAL_DIM] {
        if self.memory_data_cache.is_empty() {
            return [0, 0, 0];
        }

        let mut weighted_sum = [0.0f64; PHYSICAL_DIM];
        let mut total_weight = 0.0f64;

        for anchor in &self.memory_anchors {
            let data_b = match self.memory_data_cache.get(anchor) {
                Some(d) => d,
                None => continue,
            };
            let mut sem_dist = semantic_distance(data, data_b);
            if sem_dist < 0.01 {
                sem_dist = 0.01;
            }
            let weight = 1.0 / (sem_dist * sem_dist);

            let phys = anchor.physical();
            for d in 0..PHYSICAL_DIM {
                weighted_sum[d] += phys[d] as f64 * weight;
            }
            total_weight += weight;
        }

        if total_weight < 1e-10 {
            return [0, 0, 0];
        }

        let centroid = universe
            .coords()
            .iter()
            .fold([0i64; PHYSICAL_DIM], |mut acc, c| {
                let p = c.physical();
                for d in 0..PHYSICAL_DIM {
                    acc[d] += p[d] as i64;
                }
                acc
            });
        let n = universe.active_node_count().max(1) as f64;

        let mut result = [0i32; PHYSICAL_DIM];
        for d in 0..PHYSICAL_DIM {
            let gravity_pos = weighted_sum[d] / total_weight;
            let centroid_pos = centroid[d] as f64 / n;
            let blended = gravity_pos * (1.0 - self.config.gravity_strength)
                + centroid_pos * self.config.gravity_strength;
            result[d] = blended.round() as i32;
        }
        result
    }

    fn score_placement(&self, candidate: &Coord7D, universe: &DarkUniverse) -> f64 {
        let face_neighbors = Lattice::face_neighbors_present(candidate, universe);
        let density_score = (face_neighbors.len() as f64) / 14.0;

        let occupied_neighbors = if !self.memory_anchors.is_empty() {
            let mut nearby = 0usize;
            for anchor in &self.memory_anchors {
                let d2 = candidate.distance_sq(anchor);
                if d2 < self.config.gravity_radius * self.config.gravity_radius {
                    nearby += 1;
                }
            }
            nearby as f64 / self.memory_anchors.len() as f64
        } else {
            0.0
        };

        let bcc_free = Lattice::bcc_neighbor_coords(candidate)
            .iter()
            .filter(|c| universe.get_node(c).is_none())
            .count();
        let room_score = (bcc_free as f64) / 128.0;

        0.3 * density_score + 0.5 * occupied_neighbors + 0.2 * room_score
    }

    pub fn encode_with_clustering(
        &mut self,
        universe: &mut DarkUniverse,
        data: &[f64],
    ) -> Result<MemoryAtom, MemoryError> {
        let anchor = self.compute_ideal_anchor(data, universe);
        let atom = MemoryCodec::encode(universe, &anchor, data)?;
        self.register_memory(anchor, data);
        Ok(atom)
    }
}

// ─── L2: DarkGravityField ─────────────────────────────────────────

pub struct DarkGravityField {
    config: ClusteringConfig,
    attractors: Vec<GravityAttractor>,
}

#[derive(Debug, Clone)]
struct GravityAttractor {
    center: Coord7D,
    strength: f64,
    radius: f64,
    memory_count: usize,
}

impl DarkGravityField {
    pub fn new(config: ClusteringConfig) -> Self {
        Self {
            config,
            attractors: Vec::new(),
        }
    }

    pub fn update_attractors(&mut self, memories: &[MemoryAtom], universe: &DarkUniverse) {
        self.attractors.clear();

        if memories.is_empty() {
            return;
        }

        let mut clusters: Vec<Vec<&MemoryAtom>> = Vec::new();
        let mut assigned: std::collections::HashSet<usize> = std::collections::HashSet::new();

        let radius_sq = self.config.gravity_radius * self.config.gravity_radius;

        for i in 0..memories.len() {
            if assigned.contains(&i) {
                continue;
            }
            let mut cluster = vec![&memories[i]];
            assigned.insert(i);

            for j in (i + 1)..memories.len() {
                if assigned.contains(&j) {
                    continue;
                }
                let d2 = memories[i].anchor().distance_sq(memories[j].anchor());
                if d2 < radius_sq {
                    cluster.push(&memories[j]);
                    assigned.insert(j);
                }
            }

            if cluster.len() >= 2 {
                clusters.push(cluster);
            }
        }

        for cluster in &clusters {
            let mut sum_coords = [0.0f64; DIM];
            let mut total_energy = 0.0f64;

            for mem in cluster {
                let f = mem.anchor().as_f64();
                for d in 0..DIM {
                    sum_coords[d] += f[d];
                }
                if let Some(node) = universe.get_node(mem.anchor()) {
                    total_energy += node.energy().total();
                }
            }

            let n = cluster.len() as f64;
            for v in sum_coords.iter_mut() {
                *v /= n;
            }

            let mut max_dist = 0.0f64;
            for mem in cluster {
                let d2 = mem.anchor().distance_sq(cluster[0].anchor());
                max_dist = max_dist.max(d2.sqrt());
            }

            let mut center_basis = [0i32; DIM];
            for d in 0..DIM {
                center_basis[d] = sum_coords[d].round() as i32;
            }
            let center = Coord7D::new_even(center_basis);

            self.attractors.push(GravityAttractor {
                center,
                strength: total_energy / n,
                radius: max_dist * 1.5,
                memory_count: cluster.len(),
            });
        }
    }

    pub fn gravitational_bias(&self, coord: &Coord7D) -> f64 {
        let mut total_bias = 0.0;
        for attractor in &self.attractors {
            let d = coord.distance_sq(&attractor.center).sqrt();
            if d < attractor.radius && d > 0.01 {
                let bias = attractor.strength * attractor.memory_count as f64 / (d * d + 1.0);
                total_bias += bias;
            }
        }
        total_bias * self.config.gravity_strength
    }

    pub fn warp_distance(&self, a: &Coord7D, b: &Coord7D, base_dist: f64) -> f64 {
        let mut contraction = 0.0f64;
        for attractor in &self.attractors {
            let d_a = a.distance_sq(&attractor.center).sqrt();
            let d_b = b.distance_sq(&attractor.center).sqrt();
            if d_a < attractor.radius && d_b < attractor.radius {
                let overlap = 1.0 - (d_a.max(d_b) / attractor.radius);
                contraction += overlap * attractor.strength * 0.1;
            }
        }
        (base_dist * (1.0 - contraction.min(0.5))).max(0.1)
    }

    pub fn attractor_count(&self) -> usize {
        self.attractors.len()
    }

    pub fn total_memories_in_attractors(&self) -> usize {
        self.attractors.iter().map(|a| a.memory_count).sum()
    }
}

// ─── L3: ResonanceTunnel ──────────────────────────────────────────

pub struct ResonanceTunnel {
    config: ClusteringConfig,
    tunnels: Vec<TunnelEdge>,
}

#[derive(Debug, Clone)]
pub struct TunnelEdge {
    pub from: Coord7D,
    pub to: Coord7D,
    pub strength: f64,
    pub semantic_similarity: f64,
    pub dark_distance: f64,
    pub physical_distance: f64,
}

impl ResonanceTunnel {
    pub fn new(config: ClusteringConfig) -> Self {
        Self {
            config,
            tunnels: Vec::new(),
        }
    }

    pub fn discover_tunnels(
        &mut self,
        memories: &[MemoryAtom],
        hebbian: &HebbianMemory,
        _gravity_field: &DarkGravityField,
    ) -> Vec<TunnelEdge> {
        let mut new_tunnels = Vec::new();
        let mut attempts = 0;

        for i in 0..memories.len() {
            if attempts >= self.config.tunnel_max_per_cycle {
                break;
            }
            for j in (i + 1)..memories.len() {
                if attempts >= self.config.tunnel_max_per_cycle {
                    break;
                }

                let a = memories[i].anchor();
                let b = memories[j].anchor();
                let d_sq = a.distance_sq(b);
                let d = d_sq.sqrt();

                let min_dist = self.config.tunnel_min_hops as f64;
                if d < min_dist {
                    continue;
                }

                if hebbian.get_bias_max(a, b) > 0.1 {
                    continue;
                }

                let dark_a = a.dark();
                let dark_b = b.dark();
                let mut dark_dist = 0.0f64;
                for d in 0..DARK_DIM {
                    let diff = dark_a[d] as f64 - dark_b[d] as f64;
                    dark_dist += diff * diff;
                }
                dark_dist = dark_dist.sqrt();

                let phys_a = a.physical();
                let phys_b = b.physical();
                let mut phys_dist = 0.0f64;
                for d in 0..PHYSICAL_DIM {
                    let diff = phys_a[d] as f64 - phys_b[d] as f64;
                    phys_dist += diff * diff;
                }
                phys_dist = phys_dist.sqrt();

                if dark_dist < 1.0 {
                    let similarity = 1.0 / (1.0 + dark_dist);
                    let strength = similarity * 0.5 / (1.0 + phys_dist * 0.01);

                    if strength > 0.05 {
                        let tunnel = TunnelEdge {
                            from: *a,
                            to: *b,
                            strength,
                            semantic_similarity: similarity,
                            dark_distance: dark_dist,
                            physical_distance: phys_dist,
                        };
                        new_tunnels.push(tunnel);
                        attempts += 1;
                    }
                }
            }
        }

        self.tunnels.extend(new_tunnels.clone());
        new_tunnels
    }

    pub fn apply_tunnels(&self, hebbian: &mut HebbianMemory) -> usize {
        let mut applied = 0;
        for tunnel in &self.tunnels {
            hebbian.record_path(&[tunnel.from, tunnel.to], tunnel.strength);
            applied += 1;
        }
        applied
    }

    pub fn tunnel_count(&self) -> usize {
        self.tunnels.len()
    }

    pub fn decay_tunnels(&mut self, rate: f64) {
        self.tunnels.retain(|t| t.strength * rate > 0.01);
        for tunnel in &mut self.tunnels {
            tunnel.strength *= rate;
        }
    }
}

// ─── L4: TopologyBridge ───────────────────────────────────────────

pub struct TopologyBridge {
    config: ClusteringConfig,
    bridges: Vec<BridgeEdge>,
}

#[derive(Debug, Clone)]
pub struct BridgeEdge {
    pub from: Coord7D,
    pub to: Coord7D,
    pub bridge_type: BridgeType,
    pub created_cycle: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BridgeType {
    Wormhole,
    Shortcut,
    EmergencyBridge,
}

impl TopologyBridge {
    pub fn new(config: ClusteringConfig) -> Self {
        Self {
            config,
            bridges: Vec::new(),
        }
    }

    pub fn detect_and_bridge(
        &mut self,
        memories: &[MemoryAtom],
        hebbian: &mut HebbianMemory,
        universe: &DarkUniverse,
        cycle: u64,
    ) -> Vec<BridgeEdge> {
        let mut new_bridges = Vec::new();
        let components = self.find_connected_components(memories, hebbian);

        if components.len() > self.config.bridge_betti0_threshold {
            let bridges_needed = (components.len() - 1).min(self.config.bridge_max_per_cycle);

            let centroids: Vec<Coord7D> = components
                .iter()
                .map(|comp| {
                    let mut sum = [0i64; DIM];
                    for anchor in comp {
                        let b = anchor.basis();
                        for d in 0..DIM {
                            sum[d] += b[d] as i64;
                        }
                    }
                    let n = comp.len() as i64;
                    let mut basis = [0i32; DIM];
                    for d in 0..DIM {
                        basis[d] = (sum[d] / n) as i32;
                    }
                    Coord7D::new_even(basis)
                })
                .collect();

            for i in 0..bridges_needed.min(centroids.len() - 1) {
                let from = centroids[i];
                let to = centroids[i + 1];

                let from_anchor = *components[i].first().unwrap_or(&from);
                let to_anchor = *components[i + 1].first().unwrap_or(&to);

                hebbian.record_path(&[from_anchor, to_anchor], 0.8);

                let bridge = BridgeEdge {
                    from: from_anchor,
                    to: to_anchor,
                    bridge_type: BridgeType::Wormhole,
                    created_cycle: cycle,
                };
                new_bridges.push(bridge);
            }
        }

        if new_bridges.len() < self.config.bridge_max_per_cycle {
            let shortcut_budget = self.config.bridge_max_per_cycle - new_bridges.len();
            let shortcuts =
                self.create_semantic_shortcuts(memories, hebbian, universe, shortcut_budget, cycle);
            new_bridges.extend(shortcuts);
        }

        self.bridges.extend(new_bridges.clone());
        new_bridges
    }

    fn find_connected_components(
        &self,
        memories: &[MemoryAtom],
        hebbian: &HebbianMemory,
    ) -> Vec<Vec<Coord7D>> {
        let mut parent: HashMap<Coord7D, Coord7D> = HashMap::new();

        for mem in memories {
            let anchor = *mem.anchor();
            parent.insert(anchor, anchor);
        }

        fn find(parent: &mut HashMap<Coord7D, Coord7D>, x: Coord7D) -> Coord7D {
            let mut root = x;
            while parent[&root] != root {
                root = parent[&root];
            }
            let mut curr = x;
            while curr != root {
                let next = parent[&curr];
                parent.insert(curr, root);
                curr = next;
            }
            root
        }

        fn union(parent: &mut HashMap<Coord7D, Coord7D>, a: Coord7D, b: Coord7D) {
            let ra = find(parent, a);
            let rb = find(parent, b);
            if ra != rb {
                parent.insert(ra, rb);
            }
        }

        for i in 0..memories.len() {
            for j in (i + 1)..memories.len() {
                let a = memories[i].anchor();
                let b = memories[j].anchor();
                if hebbian.get_bias_max(a, b) > 0.1 {
                    union(&mut parent, *a, *b);
                }
            }
        }

        let mut component_map: HashMap<Coord7D, Vec<Coord7D>> = HashMap::new();
        for mem in memories {
            let anchor = *mem.anchor();
            let root = find(&mut parent, anchor);
            component_map.entry(root).or_default().push(anchor);
        }

        component_map.into_values().collect()
    }

    fn create_semantic_shortcuts(
        &self,
        memories: &[MemoryAtom],
        hebbian: &mut HebbianMemory,
        _universe: &DarkUniverse,
        budget: usize,
        cycle: u64,
    ) -> Vec<BridgeEdge> {
        let mut shortcuts = Vec::new();

        if memories.len() < 2 {
            return shortcuts;
        }

        let sample_size = (memories.len() / 4).clamp(2, 50);
        let mut candidates: Vec<(usize, usize, f64)> = Vec::new();

        for _ in 0..sample_size {
            let i = (cycle as usize + shortcuts.len() * 7) % memories.len();
            let j = (i + memories.len() / 2) % memories.len();
            if i == j {
                continue;
            }

            let a = memories[i].anchor();
            let b = memories[j].anchor();

            if hebbian.get_bias_max(a, b) > 0.1 {
                continue;
            }

            let dark_a = a.dark();
            let dark_b = b.dark();
            let mut dark_dist = 0.0f64;
            for d in 0..DARK_DIM {
                let diff = dark_a[d] as f64 - dark_b[d] as f64;
                dark_dist += diff * diff;
            }
            dark_dist = dark_dist.sqrt();

            if dark_dist < 2.0 {
                candidates.push((i, j, dark_dist));
            }
        }

        candidates.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        for (i, j, dist) in candidates.iter().take(budget) {
            let a = memories[*i].anchor();
            let b = memories[*j].anchor();
            let strength = 0.6 / (1.0 + *dist);
            hebbian.record_path(&[*a, *b], strength);

            shortcuts.push(BridgeEdge {
                from: *a,
                to: *b,
                bridge_type: BridgeType::Shortcut,
                created_cycle: cycle,
            });
        }

        shortcuts
    }

    pub fn bridge_count(&self) -> usize {
        self.bridges.len()
    }
}

// ─── Unified Clustering Engine ─────────────────────────────────────

pub struct ClusteringEngine {
    pub placer: SemanticAnchorPlacer,
    pub gravity: DarkGravityField,
    pub tunnels: ResonanceTunnel,
    pub bridges: TopologyBridge,
    cycle_count: u64,
}

impl ClusteringEngine {
    pub fn new(config: ClusteringConfig) -> Self {
        let placer = SemanticAnchorPlacer::new(config.clone());
        let gravity = DarkGravityField::new(config.clone());
        let tunnels = ResonanceTunnel::new(config.clone());
        let bridges = TopologyBridge::new(config);
        Self {
            placer,
            gravity,
            tunnels,
            bridges,
            cycle_count: 0,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(ClusteringConfig::default())
    }

    pub fn encode_semantic(
        &mut self,
        universe: &mut DarkUniverse,
        data: &[f64],
    ) -> Result<MemoryAtom, MemoryError> {
        self.placer.encode_with_clustering(universe, data)
    }

    pub fn compute_ideal_anchor(&self, data: &[f64], universe: &DarkUniverse) -> Coord7D {
        self.placer.compute_ideal_anchor(data, universe)
    }

    pub fn register_memory(&mut self, anchor: Coord7D, data: &[f64]) {
        self.placer.register_memory(anchor, data);
    }

    pub fn run_maintenance_cycle(
        &mut self,
        memories: &[MemoryAtom],
        hebbian: &mut HebbianMemory,
        universe: &DarkUniverse,
    ) -> ClusteringReport {
        self.cycle_count += 1;

        self.placer.memory_anchors = memories.iter().map(|m| *m.anchor()).collect();
        self.placer.memory_data_cache.clear();
        for mem in memories {
            if let Ok(decoded) = crate::universe::memory::MemoryCodec::decode(universe, mem) {
                self.placer.memory_data_cache.insert(*mem.anchor(), decoded);
            }
        }

        self.gravity.update_attractors(memories, universe);

        let new_tunnels = self
            .tunnels
            .discover_tunnels(memories, hebbian, &self.gravity);
        let tunnels_applied = self.tunnels.apply_tunnels(hebbian);
        self.tunnels.decay_tunnels(0.95);

        let new_bridges =
            self.bridges
                .detect_and_bridge(memories, hebbian, universe, self.cycle_count);

        ClusteringReport {
            cycle: self.cycle_count,
            attractors: self.gravity.attractor_count(),
            memories_in_attractors: self.gravity.total_memories_in_attractors(),
            tunnels_discovered: new_tunnels.len(),
            tunnels_applied,
            total_tunnels: self.tunnels.tunnel_count(),
            bridges_created: new_bridges.len(),
            total_bridges: self.bridges.bridge_count(),
            total_memories: memories.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClusteringReport {
    pub cycle: u64,
    pub attractors: usize,
    pub memories_in_attractors: usize,
    pub tunnels_discovered: usize,
    pub tunnels_applied: usize,
    pub total_tunnels: usize,
    pub bridges_created: usize,
    pub total_bridges: usize,
    pub total_memories: usize,
}

impl std::fmt::Display for ClusteringReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Clustering[cycle={} attractors={}({}/{}mems) tunnels=+{}({}total) bridges=+{}({}total)]",
            self.cycle,
            self.attractors,
            self.memories_in_attractors,
            self.total_memories,
            self.tunnels_discovered,
            self.total_tunnels,
            self.bridges_created,
            self.total_bridges,
        )
    }
}

// ─── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_deterministic() {
        let data = [1.0, 2.0, 3.0, 4.0];
        let fp1 = data_fingerprint(&data);
        let fp2 = data_fingerprint(&data);
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn similar_data_similar_fingerprint() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [1.01, 2.01, 3.01, 4.01];
        let dist = semantic_distance(&a, &b);
        assert!(
            dist < 0.5,
            "similar data should have small distance: {}",
            dist
        );
    }

    #[test]
    fn different_data_different_fingerprint() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [100.0, -200.0, 300.0, -400.0];
        let dist = semantic_distance(&a, &b);
        assert!(
            dist > 1.0,
            "different data should have large distance: {}",
            dist
        );
    }

    #[test]
    fn dark_coords_deterministic() {
        let data = [1.0, 2.0, 3.0];
        let c1 = dark_coords_from_data(&data, 0.25);
        let c2 = dark_coords_from_data(&data, 0.25);
        assert_eq!(c1, c2);
    }

    #[test]
    fn similar_data_same_dark_coords() {
        let a = [1.0, 2.0, 3.0];
        let b = [1.01, 2.01, 3.01];
        let ca = dark_coords_from_data(&a, 0.25);
        let cb = dark_coords_from_data(&b, 0.25);
        assert_eq!(ca, cb, "similar data should quantize to same dark coords");
    }

    #[test]
    fn placer_gravity_target_falls_near_existing() {
        let mut u = DarkUniverse::new(1_000_000.0);
        let config = ClusteringConfig::default();
        let mut placer = SemanticAnchorPlacer::new(config.clone());

        let data1 = vec![1.0, 2.0, 3.0];
        let mem1 =
            MemoryCodec::encode(&mut u, &Coord7D::new_even([10, 10, 10, 0, 0, 0, 0]), &data1)
                .unwrap();
        placer.register_memory(*mem1.anchor(), &data1);

        let data2 = vec![1.01, 2.01, 3.01];
        let target = placer.compute_gravity_target(&data2, &u);

        assert!(
            (target[0] - 10).abs() <= 3
                && (target[1] - 10).abs() <= 3
                && (target[2] - 10).abs() <= 3,
            "gravity target should be near existing memory: {:?}",
            target,
        );
    }

    #[test]
    fn clustering_engine_encode_creates_clustered_memories() {
        let mut u = DarkUniverse::new(10_000_000.0);
        let mut engine = ClusteringEngine::with_default_config();

        let similar_data: Vec<Vec<f64>> = (0..5)
            .map(|i| {
                vec![
                    (i as f64 * 0.1 + 1.0).sin(),
                    (i as f64 * 0.1 + 2.0).cos(),
                    (i as f64 * 0.1 + 3.0),
                ]
            })
            .collect();

        let mut anchors = Vec::new();
        for data in &similar_data {
            let mem = engine.encode_semantic(&mut u, data).unwrap();
            anchors.push(*mem.anchor());
        }

        let mut max_dist = 0.0f64;
        for i in 0..anchors.len() {
            for j in (i + 1)..anchors.len() {
                let d = anchors[i].distance_sq(&anchors[j]).sqrt();
                max_dist = max_dist.max(d);
            }
        }
        assert!(
            max_dist < 20.0,
            "similar memories should cluster within radius: max_dist={}",
            max_dist,
        );
    }

    #[test]
    fn gravity_field_creates_attractors() {
        let mut u = DarkUniverse::new(1_000_000.0);
        let config = ClusteringConfig::default();
        let mut gravity = DarkGravityField::new(config);

        let mut mems = Vec::new();
        for i in 0..5 {
            let anchor = Coord7D::new_even([i * 2, i * 2, i * 2, 0, 0, 0, 0]);
            let mem = MemoryCodec::encode(&mut u, &anchor, &[1.0, 2.0]).unwrap();
            mems.push(mem);
        }

        gravity.update_attractors(&mems, &u);
        assert!(gravity.attractor_count() >= 1, "should detect cluster");
        assert!(gravity.total_memories_in_attractors() >= 2);
    }

    #[test]
    fn tunnel_discovery_connects_distant_similar_memories() {
        let mut u = DarkUniverse::new(10_000_000.0);
        let config = ClusteringConfig {
            tunnel_min_hops: 3,
            ..Default::default()
        };
        let mut tunnels = ResonanceTunnel::new(config);
        let hebbian = HebbianMemory::new();
        let gravity = DarkGravityField::new(ClusteringConfig::default());

        let data_a = vec![1.0, 2.0, 3.0];
        let data_b = vec![1.01, 2.01, 3.01];

        let mem_a = MemoryCodec::encode(&mut u, &Coord7D::new_even([0, 0, 0, 5, 5, 5, 5]), &data_a)
            .unwrap();
        let mem_b = MemoryCodec::encode(
            &mut u,
            &Coord7D::new_even([20, 20, 20, 5, 5, 5, 5]),
            &data_b,
        )
        .unwrap();

        let new_tunnels = tunnels.discover_tunnels(&[mem_a, mem_b], &hebbian, &gravity);
        assert!(
            !new_tunnels.is_empty(),
            "should discover tunnel between distant similar memories",
        );
    }

    #[test]
    fn topology_bridge_connects_disconnected_components() {
        let mut u = DarkUniverse::new(10_000_000.0);
        let config = ClusteringConfig {
            bridge_betti0_threshold: 2,
            bridge_max_per_cycle: 5,
            ..Default::default()
        };
        let mut bridges = TopologyBridge::new(config);
        let mut hebbian = HebbianMemory::new();

        let mem_a = MemoryCodec::encode(
            &mut u,
            &Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]),
            &[1.0, 2.0],
        )
        .unwrap();
        let mem_b = MemoryCodec::encode(
            &mut u,
            &Coord7D::new_even([100, 100, 100, 0, 0, 0, 0]),
            &[3.0, 4.0],
        )
        .unwrap();

        let new_bridges =
            bridges.detect_and_bridge(&[mem_a.clone(), mem_b.clone()], &mut hebbian, &u, 1);
        assert!(
            !new_bridges.is_empty(),
            "should create bridge between disconnected components",
        );
        assert!(
            hebbian.get_bias_max(mem_a.anchor(), mem_b.anchor()) > 0.0,
            "bridge should create Hebbian edge",
        );
    }

    #[test]
    fn full_clustering_cycle_works() {
        let mut u = DarkUniverse::new(10_000_000.0);
        let mut engine = ClusteringEngine::with_default_config();
        let mut hebbian = HebbianMemory::new();

        let mut mems = Vec::new();
        for i in 0..10 {
            let data = vec![
                ((i as f64 * 0.5 + 1.0).sin() * 10.0),
                ((i as f64 * 0.5 + 2.0).cos() * 10.0),
            ];
            let mem = engine.encode_semantic(&mut u, &data).unwrap();
            mems.push(mem);
        }

        let report = engine.run_maintenance_cycle(&mems, &mut hebbian, &u);
        assert_eq!(report.total_memories, 10);
        assert!(u.verify_conservation(), "must conserve after clustering");
    }
}

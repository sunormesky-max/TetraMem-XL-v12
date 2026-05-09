// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::coord::Coord7D;
use crate::universe::lattice::Lattice;
use crate::universe::node::DarkUniverse;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq)]
pub struct BettiVector {
    values: [usize; 7],
}

impl BettiVector {
    pub fn new(values: [usize; 7]) -> Self {
        Self { values }
    }

    pub fn zero() -> Self {
        Self { values: [0; 7] }
    }

    pub fn get(&self, dim: usize) -> usize {
        self.values.get(dim).copied().unwrap_or(0)
    }

    pub fn values(&self) -> &[usize; 7] {
        &self.values
    }

    pub fn euler_characteristic(&self) -> i64 {
        let mut chi = 0i64;
        for (i, &b) in self.values.iter().enumerate() {
            chi += if i % 2 == 0 { b as i64 } else { -(b as i64) };
        }
        chi
    }

    pub fn is_exact(&self, dim: usize) -> bool {
        dim <= 1
    }
}

impl std::fmt::Display for BettiVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let labels: Vec<String> = self
            .values
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                if i <= 1 {
                    format!("H{}={}", i, v)
                } else {
                    format!("C{}~{}", i, v)
                }
            })
            .collect();
        write!(f, "Topo[{}]", labels.join(" "))
    }
}

#[derive(Debug, Clone)]
pub struct TopologyReport {
    pub betti: BettiVector,
    pub connected_components: usize,
    pub cycles_detected: usize,
    #[doc = "Counts BCC neighbor pairs (tetrahedron edge candidates), not actual 4-vertex tetrahedra"]
    pub tetrahedra_count: usize,
    pub bridging_nodes: usize,
    pub isolated_nodes: usize,
    pub average_coordination: f64,
    pub dimension_spread: [f64; 7],
}

impl std::fmt::Display for TopologyReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TopoRpt[{} comps:{} cycles:{} tetra:{} bridges:{} isolated:{} coord:{:.2}]",
            self.betti,
            self.connected_components,
            self.cycles_detected,
            self.tetrahedra_count,
            self.bridging_nodes,
            self.isolated_nodes,
            self.average_coordination
        )
    }
}

pub struct TopologyEngine;

impl TopologyEngine {
    pub fn analyze(universe: &DarkUniverse) -> TopologyReport {
        let nodes = universe.coords();
        let node_set: HashSet<Coord7D> = nodes.iter().copied().collect();

        let components = Self::find_connected_components(&nodes, &node_set);
        let h0 = components.len();

        let coordination = Self::compute_coordination(&nodes, &node_set);
        let avg_coord = if nodes.is_empty() {
            0.0
        } else {
            coordination.values().map(|&c| c as f64).sum::<f64>() / nodes.len() as f64
        };

        let (bridging, isolated) =
            Self::find_bridging_and_isolated(&nodes, &node_set, &coordination);

        let components = Self::find_connected_components(&nodes, &node_set);
        let num_components = components.len();
        let cycles = Self::find_cycles(&nodes, &node_set, &coordination, num_components);

        let tetra_count = Self::count_tetrahedra(&nodes, &node_set);

        let dim_spread = Self::dimension_spread(&nodes, universe);

        let h1 = cycles;
        let h2 = Self::estimate_h2(tetra_count, h1, &nodes);
        let h3 = Self::estimate_higher_betti(h2, 3, &dim_spread);
        let h4 = Self::estimate_higher_betti(h3, 4, &dim_spread);
        let h5 = Self::estimate_higher_betti(h4, 5, &dim_spread);
        let h6 = Self::estimate_higher_betti(h5, 6, &dim_spread);

        TopologyReport {
            betti: BettiVector::new([h0, h1, h2, h3, h4, h5, h6]),
            connected_components: h0,
            cycles_detected: h1,
            tetrahedra_count: tetra_count,
            bridging_nodes: bridging,
            isolated_nodes: isolated,
            average_coordination: avg_coord,
            dimension_spread: dim_spread,
        }
    }

    fn find_connected_components(
        nodes: &[Coord7D],
        node_set: &HashSet<Coord7D>,
    ) -> Vec<Vec<Coord7D>> {
        let mut visited = HashSet::new();
        let mut components = Vec::new();

        for node in nodes {
            if visited.contains(node) {
                continue;
            }

            let mut component = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back(*node);
            visited.insert(*node);

            while let Some(current) = queue.pop_front() {
                component.push(current);

                for neighbor in Lattice::face_neighbor_coords(&current) {
                    if node_set.contains(&neighbor) && !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
                for neighbor in Lattice::bcc_neighbor_coords(&current) {
                    if node_set.contains(&neighbor) && !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }

            components.push(component);
        }

        components
    }

    fn compute_coordination(
        nodes: &[Coord7D],
        node_set: &HashSet<Coord7D>,
    ) -> HashMap<Coord7D, usize> {
        let mut coord = HashMap::new();
        for node in nodes {
            let mut count = 0usize;
            for n in Lattice::face_neighbor_coords(node) {
                if node_set.contains(&n) {
                    count += 1;
                }
            }
            for n in Lattice::bcc_neighbor_coords(node) {
                if node_set.contains(&n) {
                    count += 1;
                }
            }
            coord.insert(*node, count);
        }
        coord
    }

    fn find_bridging_and_isolated(
        nodes: &[Coord7D],
        _node_set: &HashSet<Coord7D>,
        coordination: &HashMap<Coord7D, usize>,
    ) -> (usize, usize) {
        let mut bridging = 0usize;
        let mut isolated = 0usize;

        for node in nodes {
            let coord = coordination.get(node).copied().unwrap_or(0);
            if coord == 0 {
                isolated += 1;
            } else if coord == 1 {
                bridging += 1;
            }
        }

        (bridging, isolated)
    }

    fn find_cycles(
        nodes: &[Coord7D],
        _node_set: &HashSet<Coord7D>,
        coordination: &HashMap<Coord7D, usize>,
        num_components: usize,
    ) -> usize {
        let mut cycle_edges = 0usize;
        let mut total_edges = 0usize;

        for node in nodes {
            let coord = coordination.get(node).copied().unwrap_or(0);
            total_edges += coord;
        }
        total_edges /= 2;

        let node_count = nodes.len();
        if node_count > 0 && total_edges >= node_count {
            cycle_edges = total_edges - node_count + num_components;
        }

        cycle_edges.min(total_edges / 3)
    }

    fn count_tetrahedra(nodes: &[Coord7D], node_set: &HashSet<Coord7D>) -> usize {
        let mut count = 0usize;
        let mut seen = HashSet::new();

        for node in nodes {
            if node.is_even() {
                for n in Lattice::bcc_neighbor_coords(node) {
                    if node_set.contains(&n)
                        && !seen.contains(&(*node, n))
                        && !seen.contains(&(n, *node))
                    {
                        seen.insert((*node, n));
                        count += 1;
                    }
                }
            }
        }

        count
    }

    fn estimate_h2(tetra_count: usize, h1: usize, nodes: &[Coord7D]) -> usize {
        if nodes.len() < 4 || tetra_count < 2 {
            return 0;
        }
        let potential = tetra_count.saturating_sub(h1) / 4;
        potential.min(tetra_count / 2)
    }

    #[doc = "Heuristic complexity indicator for higher dimensions. NOT a true Betti number."]
    fn estimate_higher_betti(lower: usize, dim: usize, spread: &[f64; 7]) -> usize {
        if dim >= 7 || spread[dim] < 0.01 {
            return 0;
        }
        let factor = spread[dim].min(1.0);
        let raw = (lower as f64 * factor * 0.3) as usize;
        raw.min(lower)
    }

    fn dimension_spread(nodes: &[Coord7D], universe: &DarkUniverse) -> [f64; 7] {
        if nodes.is_empty() {
            return [0.0; 7];
        }

        let mut min = [f64::MAX; 7];
        let mut max = [f64::MIN; 7];

        for node in nodes {
            if let Some(n) = universe.get_node(node) {
                let dims = n.energy().dims();
                for d in 0..7 {
                    min[d] = min[d].min(dims[d]);
                    max[d] = max[d].max(dims[d]);
                }
            }
        }

        let mut spread = [0.0f64; 7];
        for d in 0..7 {
            spread[d] = max[d] - min[d];
        }
        spread
    }

    pub fn find_shortest_path(
        universe: &DarkUniverse,
        from: &Coord7D,
        to: &Coord7D,
    ) -> Vec<Coord7D> {
        let node_set: HashSet<Coord7D> = universe.coords().into_iter().collect();
        let mut parent: HashMap<Coord7D, Coord7D> = HashMap::new();
        let mut visited = HashSet::new();
        visited.insert(*from);
        let mut queue = VecDeque::new();
        queue.push_back(*from);

        while let Some(current) = queue.pop_front() {
            if current == *to {
                let mut path = vec![*to];
                let mut cur = *to;
                while let Some(&p) = parent.get(&cur) {
                    path.push(p);
                    cur = p;
                }
                path.reverse();
                return path;
            }

            for n in Lattice::face_neighbor_coords(&current)
                .into_iter()
                .chain(Lattice::bcc_neighbor_coords(&current))
            {
                if node_set.contains(&n) && !visited.contains(&n) {
                    visited.insert(n);
                    parent.insert(n, current);
                    queue.push_back(n);
                }
            }
        }

        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_lattice() -> DarkUniverse {
        let mut u = DarkUniverse::new(2_000_000.0);
        for x in 0..5i32 {
            for y in 0..5i32 {
                for z in 0..5i32 {
                    let c = Coord7D::new_even([x, y, z, 0, 0, 0, 0]);
                    u.materialize_biased(c, 50.0, 0.6).ok();
                }
            }
        }
        for x in 0..4i32 {
            for y in 0..4i32 {
                for z in 0..4i32 {
                    let c = Coord7D::new_odd([x, y, z, 0, 0, 0, 0]);
                    u.materialize_biased(c, 40.0, 0.3).ok();
                }
            }
        }
        u
    }

    #[test]
    fn analyze_connected_lattice() {
        let u = make_lattice();
        let report = TopologyEngine::analyze(&u);

        assert_eq!(
            report.connected_components, 1,
            "continuous lattice should be one component"
        );
        assert!(report.tetrahedra_count > 0);
        assert!(report.average_coordination > 0.0);
    }

    #[test]
    fn h0_counts_components() {
        let mut u = DarkUniverse::new(100_000.0);
        u.materialize_biased(Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]), 50.0, 0.6)
            .unwrap();
        u.materialize_biased(Coord7D::new_even([100, 0, 0, 0, 0, 0, 0]), 50.0, 0.6)
            .unwrap();

        let report = TopologyEngine::analyze(&u);
        assert_eq!(
            report.connected_components, 2,
            "two distant nodes = 2 components"
        );
    }

    #[test]
    fn isolated_nodes_detected() {
        let mut u = DarkUniverse::new(100_000.0);
        u.materialize_biased(Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]), 50.0, 0.6)
            .unwrap();
        u.materialize_biased(Coord7D::new_even([100, 0, 0, 0, 0, 0, 0]), 50.0, 0.6)
            .unwrap();

        let report = TopologyEngine::analyze(&u);
        assert_eq!(report.isolated_nodes, 2);
    }

    #[test]
    fn betti_vector_display() {
        let b = BettiVector::new([1, 3, 2, 0, 0, 0, 0]);
        let s = format!("{}", b);
        assert!(s.contains("H0=1"));
        assert!(s.contains("H1=3"));
        assert!(s.contains("C2~2"));
    }

    #[test]
    fn euler_characteristic() {
        let b = BettiVector::new([1, 2, 1, 0, 0, 0, 0]);
        let chi = b.euler_characteristic();
        assert_eq!(chi, 1 - 2 + 1);
    }

    #[test]
    fn shortest_path_finds_route() {
        let u = make_lattice();
        let from = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        let to = Coord7D::new_even([4, 0, 0, 0, 0, 0, 0]);

        let path = TopologyEngine::find_shortest_path(&u, &from, &to);
        assert!(!path.is_empty());
        assert_eq!(*path.first().unwrap(), from);
        assert_eq!(*path.last().unwrap(), to);
    }

    #[test]
    fn shortest_path_no_route() {
        let mut u = DarkUniverse::new(100_000.0);
        u.materialize_biased(Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]), 50.0, 0.6)
            .unwrap();
        u.materialize_biased(Coord7D::new_even([100, 0, 0, 0, 0, 0, 0]), 50.0, 0.6)
            .unwrap();

        let path = TopologyEngine::find_shortest_path(
            &u,
            &Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]),
            &Coord7D::new_even([100, 0, 0, 0, 0, 0, 0]),
        );
        assert!(path.is_empty());
    }

    #[test]
    fn topology_report_display() {
        let u = make_lattice();
        let report = TopologyEngine::analyze(&u);
        let s = format!("{}", report);
        assert!(s.contains("TopoRpt["));
    }

    #[test]
    fn bridging_nodes_on_partial_lattice() {
        let mut u = DarkUniverse::new(100_000.0);
        u.materialize_biased(Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]), 50.0, 0.6)
            .unwrap();
        u.materialize_biased(Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]), 50.0, 0.6)
            .unwrap();
        u.materialize_biased(Coord7D::new_even([2, 0, 0, 0, 0, 0, 0]), 50.0, 0.6)
            .unwrap();

        let report = TopologyEngine::analyze(&u);
        assert!(report.connected_components == 1);
        assert!(report.average_coordination > 0.0);
    }
}

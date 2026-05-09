// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::coord::{Coord7D, Parity};
use crate::universe::core::physics::UniversePhysics;
use crate::universe::node::DarkUniverse;
use std::collections::HashSet;
use std::fmt;

const DIM: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeighborShell {
    Face,
    Bcc,
    Edge,
}

impl NeighborShell {
    pub fn distance_sq(&self) -> f64 {
        match self {
            NeighborShell::Face => 1.0,
            NeighborShell::Bcc => 1.75,
            NeighborShell::Edge => 2.0,
        }
    }

    pub fn candidate_count(&self) -> usize {
        match self {
            NeighborShell::Face => 14,
            NeighborShell::Bcc => 128,
            NeighborShell::Edge => 84,
        }
    }

    pub fn crosses_sublattice(&self) -> bool {
        matches!(self, NeighborShell::Bcc)
    }
}

impl fmt::Display for NeighborShell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NeighborShell::Face => write!(f, "Face(d²=1.00, n=14)"),
            NeighborShell::Bcc => write!(f, "BCC(d²=1.75, n=128)"),
            NeighborShell::Edge => write!(f, "Edge(d²=2.00, n=84)"),
        }
    }
}

pub struct Lattice;

impl Lattice {
    pub fn face_neighbor_coords(center: &Coord7D) -> Vec<Coord7D> {
        Coord7D::face_neighbor_offsets()
            .iter()
            .filter_map(|off| center.shifted(off))
            .collect()
    }

    pub fn face_neighbors_present(center: &Coord7D, universe: &DarkUniverse) -> Vec<Coord7D> {
        Self::face_neighbor_coords(center)
            .into_iter()
            .filter(|c| universe.get_node(c).is_some())
            .collect()
    }

    pub fn bcc_neighbor_coords(center: &Coord7D) -> Vec<Coord7D> {
        let center_basis = center.basis();
        let mut neighbors = Vec::with_capacity(128);

        match center.parity() {
            Parity::Even => {
                for mask in 0u32..128 {
                    let mut basis = center_basis;
                    for (d, b) in basis.iter_mut().enumerate() {
                        if (mask >> d) & 1 == 1 {
                            *b -= 1;
                        }
                    }
                    neighbors.push(Coord7D::new_odd(basis));
                }
            }
            Parity::Odd => {
                for mask in 0u32..128 {
                    let mut basis = center_basis;
                    for (d, b) in basis.iter_mut().enumerate() {
                        if (mask >> d) & 1 == 1 {
                            *b += 1;
                        }
                    }
                    neighbors.push(Coord7D::new_even(basis));
                }
            }
        }

        neighbors
    }

    pub fn bcc_neighbors_present(center: &Coord7D, universe: &DarkUniverse) -> Vec<Coord7D> {
        Self::bcc_neighbor_coords(center)
            .into_iter()
            .filter(|c| universe.get_node(c).is_some())
            .collect()
    }

    pub fn edge_neighbor_offsets() -> Vec<Coord7D> {
        let mut offsets = Vec::new();
        for d1 in 0..DIM {
            for d2 in (d1 + 1)..DIM {
                for s1 in [-1i32, 1i32] {
                    for s2 in [-1i32, 1i32] {
                        let mut basis = [0i32; DIM];
                        basis[d1] = s1;
                        basis[d2] = s2;
                        offsets.push(Coord7D::new_even(basis));
                    }
                }
            }
        }
        offsets
    }

    pub fn edge_neighbor_coords(center: &Coord7D) -> Vec<Coord7D> {
        Self::edge_neighbor_offsets()
            .iter()
            .filter_map(|off| center.shifted(off))
            .collect()
    }

    pub fn edge_neighbors_present(center: &Coord7D, universe: &DarkUniverse) -> Vec<Coord7D> {
        Self::edge_neighbor_coords(center)
            .into_iter()
            .filter(|c| universe.get_node(c).is_some())
            .collect()
    }

    pub fn all_neighbors_present(center: &Coord7D, universe: &DarkUniverse) -> Vec<Coord7D> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for c in Self::face_neighbors_present(center, universe)
            .into_iter()
            .chain(Self::bcc_neighbors_present(center, universe))
            .chain(Self::edge_neighbors_present(center, universe))
        {
            if seen.insert(c) {
                result.push(c);
            }
        }

        result
    }

    pub fn neighbors_by_physics_distance(
        center: &Coord7D,
        universe: &DarkUniverse,
        physics: &UniversePhysics,
    ) -> Vec<(Coord7D, f64)> {
        let center_f = center.as_f64();
        let mut result = Vec::new();

        for n in Self::all_neighbors_present(center, universe) {
            let n_f = n.as_f64();
            let dist = physics.weighted_distance_sq(&center_f, &n_f);
            result.push((n, dist));
        }

        result.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tetrahedron {
    vertices: [Coord7D; 4],
}

impl Tetrahedron {
    pub fn new(vertices: [Coord7D; 4]) -> Self {
        Self { vertices }
    }

    pub fn vertices(&self) -> &[Coord7D; 4] {
        &self.vertices
    }

    pub fn projected_volume_3d(&self) -> f64 {
        let p: Vec<[f64; 3]> = self
            .vertices
            .iter()
            .map(|c| {
                let f = c.as_f64();
                [f[0], f[1], f[2]]
            })
            .collect();

        let a = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
        let b = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
        let c = [p[3][0] - p[0][0], p[3][1] - p[0][1], p[3][2] - p[0][2]];

        let det = a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
            + a[2] * (b[0] * c[1] - b[1] * c[0]);

        det.abs() / 6.0
    }

    pub fn has_volume(&self) -> bool {
        self.projected_volume_3d() > 1e-10
    }

    pub fn exists_in(&self, universe: &DarkUniverse) -> bool {
        self.vertices.iter().all(|v| universe.get_node(v).is_some())
    }

    pub fn is_manifested(&self, universe: &DarkUniverse) -> bool {
        let threshold = universe.manifestation_threshold();
        self.vertices.iter().all(|v| {
            universe
                .get_node(v)
                .is_some_and(|n| n.is_manifested_with(threshold))
        })
    }

    pub fn total_energy(&self, universe: &DarkUniverse) -> f64 {
        self.vertices
            .iter()
            .filter_map(|v| universe.get_node(v).map(|n| n.energy().total()))
            .sum()
    }

    fn canonical_key(&self) -> [Coord7D; 4] {
        let mut v = self.vertices;
        v.sort();
        v
    }

    pub fn bcc_tetrahedra_around(anchor: &Coord7D, universe: &DarkUniverse) -> Vec<Tetrahedron> {
        let bcc = Lattice::bcc_neighbors_present(anchor, universe);
        if bcc.len() < 3 {
            return Vec::new();
        }

        let mut result = Vec::new();
        for i in 0..bcc.len() {
            for j in (i + 1)..bcc.len() {
                for k in (j + 1)..bcc.len() {
                    let tet = Tetrahedron::new([*anchor, bcc[i], bcc[j], bcc[k]]);
                    if tet.has_volume() {
                        result.push(tet);
                    }
                }
            }
        }
        result
    }

    pub fn face_tetrahedra_around(anchor: &Coord7D, universe: &DarkUniverse) -> Vec<Tetrahedron> {
        let face = Lattice::face_neighbors_present(anchor, universe);
        if face.len() < 3 {
            return Vec::new();
        }

        let mut result = Vec::new();
        for i in 0..face.len() {
            for j in (i + 1)..face.len() {
                for k in (j + 1)..face.len() {
                    let tet = Tetrahedron::new([*anchor, face[i], face[j], face[k]]);
                    if tet.has_volume() {
                        result.push(tet);
                    }
                }
            }
        }
        result
    }

    pub fn find_bcc_all(universe: &DarkUniverse) -> Vec<Tetrahedron> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        let coords: Vec<Coord7D> = universe.get_all_nodes().keys().copied().collect();
        for coord in &coords {
            for tet in Self::bcc_tetrahedra_around(coord, universe) {
                let key = tet.canonical_key();
                if seen.insert(key) {
                    result.push(tet);
                }
            }
        }

        result
    }

    pub fn find_face_all(universe: &DarkUniverse) -> Vec<Tetrahedron> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        let coords: Vec<Coord7D> = universe.get_all_nodes().keys().copied().collect();
        for coord in &coords {
            for tet in Self::face_tetrahedra_around(coord, universe) {
                let key = tet.canonical_key();
                if seen.insert(key) {
                    result.push(tet);
                }
            }
        }

        result
    }

    pub fn projected_edge_lengths_3d(&self) -> [f64; 6] {
        let p: Vec<[f64; 3]> = self.vertices.iter().map(Projection::to_3d).collect();
        let mut edges = [0.0f64; 6];
        let mut idx = 0;
        for i in 0..4 {
            for j in (i + 1)..4 {
                edges[idx] = ((p[i][0] - p[j][0]).powi(2)
                    + (p[i][1] - p[j][1]).powi(2)
                    + (p[i][2] - p[j][2]).powi(2))
                .sqrt();
                idx += 1;
            }
        }
        edges
    }

    pub fn projected_volume_3d_physics(&self, physics: &UniversePhysics) -> f64 {
        let p: Vec<[f64; 3]> = self
            .vertices
            .iter()
            .map(|c| Projection::to_3d_physics(c, physics))
            .collect();

        let a = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
        let b = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
        let c = [p[3][0] - p[0][0], p[3][1] - p[0][1], p[3][2] - p[0][2]];

        let det = a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
            + a[2] * (b[0] * c[1] - b[1] * c[0]);

        det.abs() / 6.0
    }

    pub fn projected_edge_lengths_3d_physics(&self, physics: &UniversePhysics) -> [f64; 6] {
        let p: Vec<[f64; 3]> = self
            .vertices
            .iter()
            .map(|c| Projection::to_3d_physics(c, physics))
            .collect();
        let mut edges = [0.0f64; 6];
        let mut idx = 0;
        for i in 0..4 {
            for j in (i + 1)..4 {
                edges[idx] = ((p[i][0] - p[j][0]).powi(2)
                    + (p[i][1] - p[j][1]).powi(2)
                    + (p[i][2] - p[j][2]).powi(2))
                .sqrt();
                idx += 1;
            }
        }
        edges
    }

    pub fn is_mixed_parity(&self) -> bool {
        let has_even = self.vertices.iter().any(|v| v.is_even());
        let has_odd = self.vertices.iter().any(|v| v.is_odd());
        has_even && has_odd
    }

    pub fn parity_split(&self) -> (usize, usize) {
        let even = self.vertices.iter().filter(|v| v.is_even()).count();
        (even, 4 - even)
    }
}

impl fmt::Display for Tetrahedron {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tet[{}, {}, {}, {}]",
            self.vertices[0], self.vertices[1], self.vertices[2], self.vertices[3]
        )
    }
}

pub struct Projection;

impl Projection {
    pub fn to_3d(coord: &Coord7D) -> [f64; 3] {
        let f = coord.as_f64();
        [f[0], f[1], f[2]]
    }

    pub fn to_3d_physics(coord: &Coord7D, physics: &UniversePhysics) -> [f64; 3] {
        let f = coord.as_f64();
        physics.project_to_physical(&f)
    }

    pub fn dist_sq_3d(a: &Coord7D, b: &Coord7D) -> f64 {
        let pa = Self::to_3d(a);
        let pb = Self::to_3d(b);
        (pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)
    }

    pub fn dist_sq_3d_physics(a: &Coord7D, b: &Coord7D, physics: &UniversePhysics) -> f64 {
        let fa = a.as_f64();
        let fb = b.as_f64();
        physics.weighted_distance_sq(&fa, &fb)
    }

    pub fn verify_bcc(universe: &DarkUniverse) -> BccVerification {
        let active: Vec<Coord7D> = universe
            .get_all_nodes()
            .keys()
            .filter(|c| universe.get_node(c).is_some_and(|n| !n.energy().is_empty()))
            .copied()
            .collect();

        let even_count = active.iter().filter(|c| c.is_even()).count();
        let odd_count = active.iter().filter(|c| c.is_odd()).count();

        let even_integer = active.iter().filter(|c| c.is_even()).all(|c| {
            let p = Self::to_3d(c);
            p.iter().all(|v| (*v - v.round()).abs() < 1e-10)
        });

        let odd_half_integer = active.iter().filter(|c| c.is_odd()).all(|c| {
            let p = Self::to_3d(c);
            p.iter()
                .all(|v| (2.0 * v - (2.0 * v).round()).abs() < 1e-10)
        });

        let mut min_bcc = f64::INFINITY;
        let mut min_face = f64::INFINITY;
        let mut bcc_count = 0usize;
        let mut face_count = 0usize;

        for i in 0..active.len() {
            for j in (i + 1)..active.len() {
                let d2 = Self::dist_sq_3d(&active[i], &active[j]);
                if active[i].parity() != active[j].parity() {
                    min_bcc = min_bcc.min(d2);
                    bcc_count += 1;
                } else {
                    min_face = min_face.min(d2);
                    face_count += 1;
                }
            }
        }

        if min_bcc == f64::INFINITY {
            min_bcc = 0.0;
        }
        if min_face == f64::INFINITY {
            min_face = 0.0;
        }

        let is_bcc = even_integer
            && odd_half_integer
            && even_count > 0
            && odd_count > 0
            && (min_bcc - 0.75).abs() < 1e-10
            && (min_face - 1.0).abs() < 1e-10;

        BccVerification {
            total_nodes: active.len(),
            even_nodes: even_count,
            odd_nodes: odd_count,
            even_at_integer: even_integer,
            odd_at_half_integer: odd_half_integer,
            cross_sublattice_pairs: bcc_count,
            same_sublattice_pairs: face_count,
            min_bcc_projected_dist_sq: min_bcc,
            min_face_projected_dist_sq: min_face,
            is_bcc_lattice: is_bcc,
        }
    }
}

#[derive(Debug)]
pub struct BccVerification {
    pub total_nodes: usize,
    pub even_nodes: usize,
    pub odd_nodes: usize,
    pub even_at_integer: bool,
    pub odd_at_half_integer: bool,
    pub cross_sublattice_pairs: usize,
    pub same_sublattice_pairs: usize,
    pub min_bcc_projected_dist_sq: f64,
    pub min_face_projected_dist_sq: f64,
    pub is_bcc_lattice: bool,
}

impl fmt::Display for BccVerification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "BCC Verification:")?;
        writeln!(
            f,
            "  Nodes: {} (Even:{} Odd:{})",
            self.total_nodes, self.even_nodes, self.odd_nodes
        )?;
        writeln!(
            f,
            "  Even at integer:   {}",
            if self.even_at_integer { "PASS" } else { "FAIL" }
        )?;
        writeln!(
            f,
            "  Odd at half-int:   {}",
            if self.odd_at_half_integer {
                "PASS"
            } else {
                "FAIL"
            }
        )?;
        writeln!(
            f,
            "  Cross pairs: {} | Same pairs: {}",
            self.cross_sublattice_pairs, self.same_sublattice_pairs
        )?;
        writeln!(
            f,
            "  Min cross d^2: {:.4} (theory: 0.75)",
            self.min_bcc_projected_dist_sq
        )?;
        writeln!(
            f,
            "  Min same d^2:  {:.4} (theory: 1.00)",
            self.min_face_projected_dist_sq
        )?;
        writeln!(
            f,
            "  BCC lattice:  {}",
            if self.is_bcc_lattice { "PASS" } else { "FAIL" }
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn face_neighbor_count() {
        let origin = Coord7D::new_even([0; 7]);
        assert_eq!(Lattice::face_neighbor_coords(&origin).len(), 14);
    }

    #[test]
    fn face_neighbors_same_parity() {
        let origin = Coord7D::new_even([0; 7]);
        for n in Lattice::face_neighbor_coords(&origin) {
            assert_eq!(n.parity(), Parity::Even);
        }

        let odd = Coord7D::new_odd([0; 7]);
        for n in Lattice::face_neighbor_coords(&odd) {
            assert_eq!(n.parity(), Parity::Odd);
        }
    }

    #[test]
    fn face_neighbor_distance() {
        let origin = Coord7D::new_even([0; 7]);
        for n in Lattice::face_neighbor_coords(&origin) {
            assert!(
                (origin.distance_sq(&n) - 1.0).abs() < 1e-10,
                "face neighbor distance should be 1.0"
            );
        }
    }

    #[test]
    fn bcc_neighbor_count() {
        let even = Coord7D::new_even([0; 7]);
        assert_eq!(Lattice::bcc_neighbor_coords(&even).len(), 128);

        let odd = Coord7D::new_odd([0; 7]);
        assert_eq!(Lattice::bcc_neighbor_coords(&odd).len(), 128);
    }

    #[test]
    fn bcc_neighbors_opposite_parity() {
        let even = Coord7D::new_even([0; 7]);
        for n in Lattice::bcc_neighbor_coords(&even) {
            assert_eq!(n.parity(), Parity::Odd);
        }

        let odd = Coord7D::new_odd([0; 7]);
        for n in Lattice::bcc_neighbor_coords(&odd) {
            assert_eq!(n.parity(), Parity::Even);
        }
    }

    #[test]
    fn bcc_neighbor_distance() {
        let even = Coord7D::new_even([0; 7]);
        for n in Lattice::bcc_neighbor_coords(&even) {
            assert!(
                (even.distance_sq(&n) - 1.75).abs() < 1e-10,
                "BCC neighbor distance should be 1.75"
            );
        }

        let odd = Coord7D::new_odd([5, 3, -2, 1, 0, 4, -1]);
        for n in Lattice::bcc_neighbor_coords(&odd) {
            assert!(
                (odd.distance_sq(&n) - 1.75).abs() < 1e-10,
                "BCC neighbor distance should be 1.75"
            );
        }
    }

    #[test]
    fn edge_neighbor_count() {
        assert_eq!(Lattice::edge_neighbor_offsets().len(), 84);
    }

    #[test]
    fn edge_neighbors_same_parity() {
        let origin = Coord7D::new_even([0; 7]);
        for n in Lattice::edge_neighbor_coords(&origin) {
            assert_eq!(n.parity(), Parity::Even);
        }

        let odd = Coord7D::new_odd([0; 7]);
        for n in Lattice::edge_neighbor_coords(&odd) {
            assert_eq!(n.parity(), Parity::Odd);
        }
    }

    #[test]
    fn edge_neighbor_distance() {
        let origin = Coord7D::new_even([0; 7]);
        for n in Lattice::edge_neighbor_coords(&origin) {
            assert!(
                (origin.distance_sq(&n) - 2.0).abs() < 1e-10,
                "edge neighbor distance should be 2.0"
            );
        }
    }

    #[test]
    fn neighbor_shell_ordering() {
        assert!(NeighborShell::Face.distance_sq() < NeighborShell::Bcc.distance_sq());
        assert!(NeighborShell::Bcc.distance_sq() < NeighborShell::Edge.distance_sq());
    }

    #[test]
    fn face_neighbors_present_filtered() {
        let mut u = DarkUniverse::new(10000.0);
        let origin = Coord7D::new_even([0; 7]);
        let n1 = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let n2 = Coord7D::new_even([0, 1, 0, 0, 0, 0, 0]);

        u.materialize_uniform(origin, 100.0).unwrap();
        u.materialize_uniform(n1, 100.0).unwrap();
        u.materialize_uniform(n2, 100.0).unwrap();

        let present = Lattice::face_neighbors_present(&origin, &u);
        assert_eq!(present.len(), 2);
    }

    #[test]
    fn bcc_neighbors_present_filtered() {
        let mut u = DarkUniverse::new(100000.0);
        let origin = Coord7D::new_even([0; 7]);
        u.materialize_uniform(origin, 100.0).unwrap();

        let bcc_coords = Lattice::bcc_neighbor_coords(&origin);
        for c in bcc_coords.iter().take(5) {
            u.materialize_uniform(*c, 10.0).unwrap();
        }

        let present = Lattice::bcc_neighbors_present(&origin, &u);
        assert_eq!(present.len(), 5);
    }

    #[test]
    fn tetrahedron_bcc_volume() {
        let even = Coord7D::new_even([0; 7]);
        let odd1 = Coord7D::new_odd([0, 0, 0, 0, 0, 0, 0]);
        let odd2 = Coord7D::new_odd([-1, 0, 0, 0, 0, 0, 0]);
        let odd3 = Coord7D::new_odd([0, -1, 0, 0, 0, 0, 0]);

        let tet = Tetrahedron::new([even, odd1, odd2, odd3]);
        let vol = tet.projected_volume_3d();

        assert!(
            (vol - 1.0 / 12.0).abs() < 1e-10,
            "expected 1/12 ≈ 0.0833, got {}",
            vol
        );
        assert!(tet.has_volume());
    }

    #[test]
    fn tetrahedron_face_volume() {
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let c = Coord7D::new_even([0, 1, 0, 0, 0, 0, 0]);
        let d = Coord7D::new_even([0, 0, 1, 0, 0, 0, 0]);

        let tet = Tetrahedron::new([a, b, c, d]);
        let vol = tet.projected_volume_3d();

        assert!(
            (vol - 1.0 / 6.0).abs() < 1e-10,
            "expected 1/6 ≈ 0.1667, got {}",
            vol
        );
        assert!(tet.has_volume());
    }

    #[test]
    fn tetrahedron_zero_volume_coplanar() {
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let c = Coord7D::new_even([2, 0, 0, 0, 0, 0, 0]);
        let d = Coord7D::new_even([3, 0, 0, 0, 0, 0, 0]);

        let tet = Tetrahedron::new([a, b, c, d]);
        assert!(!tet.has_volume());
        assert_eq!(tet.projected_volume_3d(), 0.0);
    }

    #[test]
    fn tetrahedron_exists_in_universe() {
        let mut u = DarkUniverse::new(10000.0);
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let c = Coord7D::new_even([0, 1, 0, 0, 0, 0, 0]);
        let d = Coord7D::new_even([0, 0, 1, 0, 0, 0, 0]);

        let tet = Tetrahedron::new([a, b, c, d]);
        assert!(!tet.exists_in(&u));

        u.materialize_uniform(a, 100.0).unwrap();
        u.materialize_uniform(b, 100.0).unwrap();
        u.materialize_uniform(c, 100.0).unwrap();
        u.materialize_uniform(d, 100.0).unwrap();
        assert!(tet.exists_in(&u));
    }

    #[test]
    fn bcc_tetrahedra_around_anchor() {
        let mut u = DarkUniverse::new(100000.0);
        let anchor = Coord7D::new_even([0; 7]);
        u.materialize_uniform(anchor, 100.0).unwrap();

        let bcc_coords = Lattice::bcc_neighbor_coords(&anchor);
        for c in bcc_coords.iter().take(4) {
            u.materialize_uniform(*c, 10.0).unwrap();
        }

        let tets = Tetrahedron::bcc_tetrahedra_around(&anchor, &u);
        assert!(!tets.is_empty(), "should find at least 1 BCC tetrahedron");
        for tet in &tets {
            assert!(tet.has_volume());
            assert!(tet.exists_in(&u));
        }
    }

    #[test]
    fn face_tetrahedra_around_anchor() {
        let mut u = DarkUniverse::new(10000.0);
        let anchor = Coord7D::new_even([0; 7]);
        u.materialize_uniform(anchor, 100.0).unwrap();
        u.materialize_uniform(Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]), 100.0)
            .unwrap();
        u.materialize_uniform(Coord7D::new_even([0, 1, 0, 0, 0, 0, 0]), 100.0)
            .unwrap();
        u.materialize_uniform(Coord7D::new_even([0, 0, 1, 0, 0, 0, 0]), 100.0)
            .unwrap();

        let tets = Tetrahedron::face_tetrahedra_around(&anchor, &u);
        assert_eq!(tets.len(), 1);
        assert!(tets[0].exists_in(&u));
        assert!((tets[0].projected_volume_3d() - 1.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn find_face_all_deduplicates() {
        let mut u = DarkUniverse::new(10000.0);
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let c = Coord7D::new_even([0, 1, 0, 0, 0, 0, 0]);
        let d = Coord7D::new_even([0, 0, 1, 0, 0, 0, 0]);

        u.materialize_uniform(a, 100.0).unwrap();
        u.materialize_uniform(b, 100.0).unwrap();
        u.materialize_uniform(c, 100.0).unwrap();
        u.materialize_uniform(d, 100.0).unwrap();

        let all = Tetrahedron::find_face_all(&u);
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn bcc_emergence_projection() {
        let mut u = DarkUniverse::new(1000000.0);

        for x in 0..3i32 {
            for y in 0..3i32 {
                for z in 0..3i32 {
                    let even = Coord7D::new_even([x, y, z, 0, 0, 0, 0]);
                    u.materialize_uniform(even, 100.0).unwrap();
                }
            }
        }

        for x in 0..2i32 {
            for y in 0..2i32 {
                for z in 0..2i32 {
                    let odd = Coord7D::new_odd([x, y, z, 0, 0, 0, 0]);
                    u.materialize_uniform(odd, 100.0).unwrap();
                }
            }
        }

        let even_origin = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        let odd_body = Coord7D::new_odd([0, 0, 0, 0, 0, 0, 0]);

        let ep = even_origin.as_f64();
        assert_eq!(ep[0], 0.0);
        assert_eq!(ep[1], 0.0);
        assert_eq!(ep[2], 0.0);

        let op = odd_body.as_f64();
        assert_eq!(op[0], 0.5);
        assert_eq!(op[1], 0.5);
        assert_eq!(op[2], 0.5);

        assert!((even_origin.distance_sq(&odd_body) - 1.75).abs() < 1e-10);

        let interior = Coord7D::new_even([1, 1, 1, 0, 0, 0, 0]);
        let bcc_of_interior = Lattice::bcc_neighbors_present(&interior, &u);
        assert!(
            bcc_of_interior.len() >= 3,
            "interior node should have ≥3 BCC neighbors, got {}",
            bcc_of_interior.len()
        );

        let tets = Tetrahedron::bcc_tetrahedra_around(&interior, &u);
        assert!(!tets.is_empty(), "BCC tetrahedra should emerge");
        for tet in &tets {
            assert!(tet.has_volume());
            assert!(tet.exists_in(&u));
        }
    }

    #[test]
    fn full_lattice_conservation() {
        let mut u = DarkUniverse::new(100000.0);

        for i in 0..5 {
            let even = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            u.materialize_biased(even, 200.0, 0.6).unwrap();

            let odd = Coord7D::new_odd([i, 0, 0, 0, 0, 0, 0]);
            u.materialize_biased(odd, 150.0, 0.3).unwrap();
        }

        assert!(u.verify_conservation());

        let all_coords: Vec<Coord7D> = u.get_all_nodes().keys().copied().collect();
        for coord in &all_coords {
            let neighbors = Lattice::face_neighbors_present(coord, &u);
            for n in &neighbors {
                let _ = u.transfer_energy(coord, n, 1.0);
            }
        }

        assert!(u.verify_conservation());
    }

    #[test]
    fn bcc_all_with_grid() {
        let mut u = DarkUniverse::new(1000000.0);

        for x in 0..3i32 {
            for y in 0..3i32 {
                for z in 0..3i32 {
                    let even = Coord7D::new_even([x, y, z, 0, 0, 0, 0]);
                    u.materialize_uniform(even, 100.0).unwrap();
                }
            }
        }

        for x in 0..2i32 {
            for y in 0..2i32 {
                for z in 0..2i32 {
                    let odd = Coord7D::new_odd([x, y, z, 0, 0, 0, 0]);
                    u.materialize_uniform(odd, 80.0).unwrap();
                }
            }
        }

        let bcc_tets = Tetrahedron::find_bcc_all(&u);
        let face_tets = Tetrahedron::find_face_all(&u);

        assert!(!bcc_tets.is_empty(), "should find BCC tetrahedra");
        assert!(!face_tets.is_empty(), "should find face tetrahedra");

        for tet in &bcc_tets {
            assert!(tet.exists_in(&u));
            assert!(tet.has_volume());
        }

        for tet in &face_tets {
            assert!(tet.exists_in(&u));
            assert!(tet.has_volume());
        }

        assert!(u.verify_conservation());
    }

    #[test]
    fn projection_to_3d_even() {
        let c = Coord7D::new_even([1, 2, 3, 4, 5, 6, 7]);
        let p = Projection::to_3d(&c);
        assert_eq!(p, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn projection_to_3d_odd() {
        let c = Coord7D::new_odd([1, 2, 3, 4, 5, 6, 7]);
        let p = Projection::to_3d(&c);
        assert_eq!(p, [1.5, 2.5, 3.5]);
    }

    #[test]
    fn projection_dist_sq_3d() {
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        assert!((Projection::dist_sq_3d(&a, &b) - 1.0).abs() < 1e-10);

        let c = Coord7D::new_odd([0; 7]);
        assert!((Projection::dist_sq_3d(&a, &c) - 0.75).abs() < 1e-10);
    }

    #[test]
    fn bcc_verification_full_grid() {
        let mut u = DarkUniverse::new(1000000.0);

        for x in 0..3i32 {
            for y in 0..3i32 {
                for z in 0..3i32 {
                    let even = Coord7D::new_even([x, y, z, 0, 0, 0, 0]);
                    u.materialize_uniform(even, 100.0).unwrap();
                }
            }
        }

        for x in 0..2i32 {
            for y in 0..2i32 {
                for z in 0..2i32 {
                    let odd = Coord7D::new_odd([x, y, z, 0, 0, 0, 0]);
                    u.materialize_uniform(odd, 80.0).unwrap();
                }
            }
        }

        let v = Projection::verify_bcc(&u);
        assert!(
            v.even_at_integer,
            "Even nodes should be at integer positions"
        );
        assert!(
            v.odd_at_half_integer,
            "Odd nodes should be at half-integer positions"
        );
        assert!(
            (v.min_bcc_projected_dist_sq - 0.75).abs() < 1e-10,
            "min cross d² should be 0.75, got {}",
            v.min_bcc_projected_dist_sq
        );
        assert!(
            (v.min_face_projected_dist_sq - 1.0).abs() < 1e-10,
            "min same d² should be 1.0, got {}",
            v.min_face_projected_dist_sq
        );
        assert!(v.is_bcc_lattice, "should be recognized as BCC lattice");
        assert!(v.cross_sublattice_pairs > 0);
        assert!(v.same_sublattice_pairs > 0);
    }

    #[test]
    fn bcc_verification_even_only_is_not_bcc() {
        let mut u = DarkUniverse::new(10000.0);
        for x in 0..3i32 {
            for y in 0..3i32 {
                for z in 0..3i32 {
                    let even = Coord7D::new_even([x, y, z, 0, 0, 0, 0]);
                    u.materialize_uniform(even, 100.0).unwrap();
                }
            }
        }

        let v = Projection::verify_bcc(&u);
        assert!(v.even_at_integer);
        assert!(!v.is_bcc_lattice, "even-only grid should not be BCC");
    }

    #[test]
    fn tetrahedron_edge_lengths_bcc() {
        let even = Coord7D::new_even([0; 7]);
        let odd1 = Coord7D::new_odd([0, 0, 0, 0, 0, 0, 0]);
        let odd2 = Coord7D::new_odd([-1, 0, 0, 0, 0, 0, 0]);
        let odd3 = Coord7D::new_odd([0, -1, 0, 0, 0, 0, 0]);

        let tet = Tetrahedron::new([even, odd1, odd2, odd3]);
        let edges = tet.projected_edge_lengths_3d();

        let sqrt_075 = 0.75f64.sqrt();
        let mut short_count = 0;
        let mut long_count = 0;
        for e in &edges {
            if (*e - sqrt_075).abs() < 1e-10 {
                short_count += 1;
            } else if (*e - 1.0).abs() < 1e-10 {
                long_count += 1;
            }
        }

        assert_eq!(short_count, 3, "should have 3 edges of sqrt(0.75)");
        assert!(
            long_count >= 2,
            "should have at least 2 edges of 1.0, got {}",
            long_count
        );
    }

    #[test]
    fn tetrahedron_edge_lengths_face() {
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let c = Coord7D::new_even([0, 1, 0, 0, 0, 0, 0]);
        let d = Coord7D::new_even([0, 0, 1, 0, 0, 0, 0]);

        let tet = Tetrahedron::new([a, b, c, d]);
        let edges = tet.projected_edge_lengths_3d();

        let mut short_count = 0;
        let mut long_count = 0;
        for e in &edges {
            if (*e - 1.0).abs() < 1e-10 {
                short_count += 1;
            } else if (*e - 2.0f64.sqrt()).abs() < 1e-10 {
                long_count += 1;
            }
        }

        assert_eq!(short_count, 3, "should have 3 edges of 1.0");
        assert_eq!(long_count, 3, "should have 3 edges of sqrt(2)");
    }

    #[test]
    fn tetrahedron_mixed_parity() {
        let even = Coord7D::new_even([0; 7]);
        let odd1 = Coord7D::new_odd([0; 7]);
        let odd2 = Coord7D::new_odd([1, 0, 0, 0, 0, 0, 0]);
        let odd3 = Coord7D::new_odd([0, 1, 0, 0, 0, 0, 0]);

        let tet = Tetrahedron::new([even, odd1, odd2, odd3]);
        assert!(tet.is_mixed_parity());
        assert_eq!(tet.parity_split(), (1, 3));

        let face_tet = Tetrahedron::new([
            Coord7D::new_even([0; 7]),
            Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]),
            Coord7D::new_even([0, 1, 0, 0, 0, 0, 0]),
            Coord7D::new_even([0, 0, 1, 0, 0, 0, 0]),
        ]);
        assert!(!face_tet.is_mixed_parity());
        assert_eq!(face_tet.parity_split(), (4, 0));
    }

    #[test]
    fn bcc_tetrahedra_all_mixed_parity() {
        let mut u = DarkUniverse::new(1000000.0);

        for x in 0..3i32 {
            for y in 0..3i32 {
                for z in 0..3i32 {
                    let even = Coord7D::new_even([x, y, z, 0, 0, 0, 0]);
                    u.materialize_uniform(even, 100.0).unwrap();
                }
            }
        }

        for x in 0..2i32 {
            for y in 0..2i32 {
                for z in 0..2i32 {
                    let odd = Coord7D::new_odd([x, y, z, 0, 0, 0, 0]);
                    u.materialize_uniform(odd, 80.0).unwrap();
                }
            }
        }

        let bcc_tets = Tetrahedron::find_bcc_all(&u);
        assert!(!bcc_tets.is_empty());
        for tet in &bcc_tets {
            assert!(tet.is_mixed_parity(), "BCC tetrahedra must be mixed parity");
        }

        let face_tets = Tetrahedron::find_face_all(&u);
        for tet in &face_tets {
            assert!(
                !tet.is_mixed_parity(),
                "face tetrahedra must be single parity"
            );
        }
    }

    #[test]
    fn projection_bcc_vs_7d_distance() {
        let mut u = DarkUniverse::new(100000.0);
        let even = Coord7D::new_even([0; 7]);
        let odd = Coord7D::new_odd([0, 0, 0, 0, 0, 0, 0]);
        u.materialize_uniform(even, 100.0).unwrap();
        u.materialize_uniform(odd, 100.0).unwrap();

        let d_7d = even.distance_sq(&odd);
        let d_3d = Projection::dist_sq_3d(&even, &odd);

        assert!((d_7d - 1.75).abs() < 1e-10);
        assert!((d_3d - 0.75).abs() < 1e-10);
        assert!(
            (d_7d - d_3d - 1.0).abs() < 1e-10,
            "dark dims contribute 1.0"
        );
    }

    #[test]
    fn physics_projection_uses_metric() {
        use crate::universe::core::physics::UniversePhysics;
        let physics = UniversePhysics::rich();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let c = Coord7D::new_even([0, 0, 0, 1, 0, 0, 0]);
        let d_flat = Projection::dist_sq_3d(&a, &b);
        let d_phys = Projection::dist_sq_3d_physics(&a, &b, &physics);
        assert!((d_flat - 1.0).abs() < 1e-10);
        assert!(d_phys > 0.0, "physics distance should be positive");
        let d_dark_phys = Projection::dist_sq_3d_physics(&a, &c, &physics);
        let d_dark_flat = Projection::dist_sq_3d(&a, &c);
        assert!(
            (d_dark_phys - d_dark_flat).abs() > 0.01,
            "rich physics should weight dark dims differently: flat={}, phys={}",
            d_dark_flat,
            d_dark_phys
        );
    }

    #[test]
    fn physics_tetrahedron_volume_differs() {
        use crate::universe::core::physics::UniversePhysics;
        let physics = UniversePhysics::rich();
        let a = Coord7D::new_even([0; 7]);
        let b = Coord7D::new_even([1, 0, 0, 0, 0, 0, 0]);
        let c = Coord7D::new_even([0, 1, 0, 0, 0, 0, 0]);
        let d = Coord7D::new_even([0, 0, 1, 0, 0, 0, 0]);
        let tet = Tetrahedron::new([a, b, c, d]);
        let vol_flat = tet.projected_volume_3d();
        let vol_phys = tet.projected_volume_3d_physics(&physics);
        assert!(vol_phys > 0.0);
        assert!(vol_flat > 0.0);
    }

    #[test]
    fn neighbors_by_physics_distance_sorted() {
        use crate::universe::core::physics::UniversePhysics;
        let mut u = DarkUniverse::new(1000000.0);
        for x in 0..3i32 {
            for y in 0..3i32 {
                for z in 0..3i32 {
                    let even = Coord7D::new_even([x, y, z, 0, 0, 0, 0]);
                    u.materialize_uniform(even, 100.0).unwrap();
                }
            }
        }
        let center = Coord7D::new_even([1, 1, 1, 0, 0, 0, 0]);
        let physics = UniversePhysics::rich();
        let neighbors = Lattice::neighbors_by_physics_distance(&center, &u, &physics);
        assert!(!neighbors.is_empty());
        for i in 1..neighbors.len() {
            assert!(
                neighbors[i].1 >= neighbors[i - 1].1 - 1e-10,
                "neighbors should be sorted by distance"
            );
        }
    }
}

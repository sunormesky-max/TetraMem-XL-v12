use crate::universe::coord::Coord7D;
use crate::universe::energy::EnergyField;
use crate::universe::lattice::{Lattice, Tetrahedron};
use crate::universe::node::DarkUniverse;
use std::fmt;

const DIM: usize = 7;
const PHYSICAL_DIM: usize = 3;
const MAX_DATA_DIM: usize = 28;
const PHYSICAL_ENCODING_BASE: f64 = 50.0;
const DATA_OFFSET: f64 = 50.0;
const MIN_DIM_VALUE: f64 = 0.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryError {
    DataTooLarge,
    EmptyData,
    NoAvailablePosition,
    InsufficientEnergy,
    NodeNotFound,
    InvalidDataRange { index: usize, value: f64 },
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryError::DataTooLarge => write!(f, "data exceeds {} dimensions", MAX_DATA_DIM),
            MemoryError::EmptyData => write!(f, "data is empty"),
            MemoryError::NoAvailablePosition => write!(f, "no available tetrahedron position"),
            MemoryError::InsufficientEnergy => write!(f, "insufficient universe energy"),
            MemoryError::NodeNotFound => write!(f, "memory node not found"),
            MemoryError::InvalidDataRange { index, value } => {
                write!(f, "data[{}] = {:.1} would make dimension negative (min base = {:.1})", index, value, DATA_OFFSET)
            }
        }
    }
}

impl std::error::Error for MemoryError {}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryAtom {
    vertices: [Coord7D; 4],
    data_dim: usize,
    physical_base: f64,
    created_at: u64,
}

impl MemoryAtom {
    pub fn vertices(&self) -> &[Coord7D; 4] {
        &self.vertices
    }

    pub fn data_dim(&self) -> usize {
        self.data_dim
    }

    pub fn physical_base_f64(&self) -> f64 {
        self.physical_base
    }

    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    pub fn from_parts(vertices: [Coord7D; 4], data_dim: usize, physical_base: f64) -> Self {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Self { vertices, data_dim, physical_base, created_at }
    }

    pub fn from_parts_with_time(vertices: [Coord7D; 4], data_dim: usize, physical_base: f64, created_at: u64) -> Self {
        Self { vertices, data_dim, physical_base, created_at }
    }

    pub fn to_tetrahedron(&self) -> Tetrahedron {
        Tetrahedron::new(self.vertices)
    }

    pub fn exists_in(&self, universe: &DarkUniverse) -> bool {
        self.vertices
            .iter()
            .all(|v| universe.get_node(v).is_some())
    }

    pub fn is_manifested(&self, universe: &DarkUniverse) -> bool {
        self.vertices.iter().all(|v| {
            universe.get_node(v).map_or(false, |n| n.is_manifested())
        })
    }

    pub fn total_energy(&self, universe: &DarkUniverse) -> f64 {
        self.vertices
            .iter()
            .filter_map(|v| universe.get_node(v).map(|n| n.energy().total()))
            .sum()
    }

    pub fn anchor(&self) -> &Coord7D {
        &self.vertices[0]
    }
}

impl fmt::Display for MemoryAtom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Mem[{} dims, anchor={}]",
            self.data_dim,
            self.vertices[0]
        )
    }
}

pub struct MemoryCodec;

impl MemoryCodec {
    pub fn max_data_dim() -> usize {
        MAX_DATA_DIM
    }

    pub fn encode(
        universe: &mut DarkUniverse,
        anchor: &Coord7D,
        data: &[f64],
    ) -> Result<MemoryAtom, MemoryError> {
        if data.is_empty() {
            return Err(MemoryError::EmptyData);
        }
        if data.len() > MAX_DATA_DIM {
            return Err(MemoryError::DataTooLarge);
        }

        let positions = Self::find_tetrahedron_positions(anchor, universe)
            .ok_or(MemoryError::NoAvailablePosition)?;

        let physical_base = PHYSICAL_ENCODING_BASE;

        for (idx, &v) in data.iter().enumerate() {
            let _node_idx = idx / DIM;
            let dim_idx = idx % DIM;
            let base = if dim_idx < PHYSICAL_DIM {
                physical_base + DATA_OFFSET
            } else {
                DATA_OFFSET
            };
            if base + v < MIN_DIM_VALUE {
                return Err(MemoryError::InvalidDataRange { index: idx, value: v });
            }
        }

        let mut total_needed = 0.0f64;
        let mut node_fields: Vec<[f64; DIM]> = Vec::with_capacity(4);
        for (node_idx, _coord) in positions.iter().enumerate() {
            let mut dims = [DATA_OFFSET; DIM];
            for d in 0..PHYSICAL_DIM {
                dims[d] = physical_base + DATA_OFFSET;
            }
            for d in 0..DIM {
                let data_idx = node_idx * DIM + d;
                if data_idx < data.len() {
                    dims[d] += data[data_idx];
                }
            }
            let node_total: f64 = dims.iter().sum();
            total_needed += node_total;
            node_fields.push(dims);
        }

        if universe.available_energy() < total_needed {
            return Err(MemoryError::InsufficientEnergy);
        }

        for (i, &coord) in positions.iter().enumerate() {
            let field = EnergyField::from_dims(node_fields[i])
                .map_err(|_| MemoryError::InvalidDataRange { index: 0, value: 0.0 })?;
            universe
                .materialize_field(coord, field)
                .map_err(|_| MemoryError::InsufficientEnergy)?;
        }

        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let atom = MemoryAtom {
            vertices: positions,
            data_dim: data.len(),
            physical_base,
            created_at,
        };

        universe.protect(&atom.vertices);

        Ok(atom)
    }

    pub fn decode(
        universe: &DarkUniverse,
        atom: &MemoryAtom,
    ) -> Result<Vec<f64>, MemoryError> {
        let mut result = Vec::with_capacity(atom.data_dim);

        for (node_idx, coord) in atom.vertices.iter().enumerate() {
            let node = universe
                .get_node(coord)
                .ok_or(MemoryError::NodeNotFound)?;
            let dims = node.energy().dims();

            for d in 0..DIM {
                let data_idx = node_idx * DIM + d;
                if data_idx < atom.data_dim {
                    let base = if d < PHYSICAL_DIM {
                        atom.physical_base + DATA_OFFSET
                    } else {
                        DATA_OFFSET
                    };
                    result.push(dims[d] - base);
                }
            }
        }

        Ok(result)
    }

    pub fn erase(universe: &mut DarkUniverse, atom: &MemoryAtom) {
        universe.unprotect(&atom.vertices);
        for coord in &atom.vertices {
            universe.dematerialize(coord);
        }
    }

    fn find_tetrahedron_positions(
        anchor: &Coord7D,
        universe: &DarkUniverse,
    ) -> Option<[Coord7D; 4]> {
        if universe.get_node(anchor).is_some() {
            return None;
        }

        let bcc = Lattice::bcc_neighbor_coords(anchor);
        let available: Vec<Coord7D> = bcc
            .into_iter()
            .filter(|c| universe.get_node(c).is_none())
            .collect();

        if available.len() < 3 {
            return None;
        }

        for i in 0..available.len() {
            for j in (i + 1)..available.len() {
                for k in (j + 1)..available.len() {
                    let tet = Tetrahedron::new([*anchor, available[i], available[j], available[k]]);
                    if tet.has_volume() {
                        return Some([*anchor, available[i], available[j], available[k]]);
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_universe() -> DarkUniverse {
        DarkUniverse::new(100000.0)
    }

    #[test]
    fn encode_decode_roundtrip_small() {
        let mut u = make_test_universe();
        let anchor = Coord7D::new_even([0; 7]);
        let data = vec![1.0, 2.0, 3.0];

        let atom = MemoryCodec::encode(&mut u, &anchor, &data).unwrap();
        let decoded = MemoryCodec::decode(&u, &atom).unwrap();

        assert_eq!(decoded.len(), 3);
        for (i, (expected, actual)) in data.iter().zip(decoded.iter()).enumerate() {
            assert!(
                (expected - actual).abs() < 1e-10,
                "dim {}: expected {}, got {}",
                i,
                expected,
                actual
            );
        }
    }

    #[test]
    fn encode_decode_roundtrip_7d() {
        let mut u = make_test_universe();
        let anchor = Coord7D::new_even([0; 7]);
        let data: Vec<f64> = (1..=7).map(|v| v as f64).collect();

        let atom = MemoryCodec::encode(&mut u, &anchor, &data).unwrap();
        let decoded = MemoryCodec::decode(&u, &atom).unwrap();

        assert_eq!(decoded.len(), 7);
        for (i, (expected, actual)) in data.iter().zip(decoded.iter()).enumerate() {
            assert!(
                (expected - actual).abs() < 1e-10,
                "dim {}: expected {}, got {}",
                i,
                expected,
                actual
            );
        }
    }

    #[test]
    fn encode_decode_roundtrip_14d() {
        let mut u = make_test_universe();
        let anchor = Coord7D::new_even([0; 7]);
        let data: Vec<f64> = (1..=14).map(|v| v as f64 * 0.5).collect();

        let atom = MemoryCodec::encode(&mut u, &anchor, &data).unwrap();
        let decoded = MemoryCodec::decode(&u, &atom).unwrap();

        assert_eq!(decoded.len(), 14);
        for (i, (expected, actual)) in data.iter().zip(decoded.iter()).enumerate() {
            assert!(
                (expected - actual).abs() < 1e-10,
                "dim {}: expected {}, got {}",
                i,
                expected,
                actual
            );
        }
    }

    #[test]
    fn encode_decode_roundtrip_28d() {
        let mut u = make_test_universe();
        let anchor = Coord7D::new_even([0; 7]);
        let data: Vec<f64> = (1..=28).map(|v| v as f64 * 0.1).collect();

        let atom = MemoryCodec::encode(&mut u, &anchor, &data).unwrap();
        let decoded = MemoryCodec::decode(&u, &atom).unwrap();

        assert_eq!(decoded.len(), 28);
        for (i, (expected, actual)) in data.iter().zip(decoded.iter()).enumerate() {
            assert!(
                (expected - actual).abs() < 1e-10,
                "dim {}: expected {}, got {}",
                i,
                expected,
                actual
            );
        }
    }

    #[test]
    fn encode_preserves_conservation() {
        let mut u = make_test_universe();
        let anchor = Coord7D::new_even([0; 7]);
        let data = vec![10.0; 7];

        let atom = MemoryCodec::encode(&mut u, &anchor, &data).unwrap();
        assert!(u.verify_conservation(), "conservation after encode");
        assert!(atom.exists_in(&u));
        assert!(atom.is_manifested(&u));
    }

    #[test]
    fn erase_frees_energy() {
        let mut u = make_test_universe();
        let anchor = Coord7D::new_even([0; 7]);
        let data = vec![5.0; 7];

        let atom = MemoryCodec::encode(&mut u, &anchor, &data).unwrap();
        let after_encode = u.available_energy();

        MemoryCodec::erase(&mut u, &atom);
        let after_erase = u.available_energy();

        assert!(after_erase > after_encode, "energy should increase after erase");
        assert!(!atom.exists_in(&u), "atom should not exist after erase");
        assert!(u.verify_conservation(), "conservation after erase");
    }

    #[test]
    fn erase_and_reuse() {
        let mut u = make_test_universe();
        let anchor = Coord7D::new_even([0; 7]);

        let data1 = vec![1.0, 2.0, 3.0];
        let atom1 = MemoryCodec::encode(&mut u, &anchor, &data1).unwrap();
        let _decoded1 = MemoryCodec::decode(&u, &atom1).unwrap();

        MemoryCodec::erase(&mut u, &atom1);

        let data2 = vec![10.0, 20.0, 30.0];
        let atom2 = MemoryCodec::encode(&mut u, &anchor, &data2).unwrap();
        let decoded2 = MemoryCodec::decode(&u, &atom2).unwrap();

        assert_eq!(decoded2.len(), 3);
        assert!((decoded2[0] - 10.0).abs() < 1e-10);
        assert!((decoded2[1] - 20.0).abs() < 1e-10);
        assert!((decoded2[2] - 30.0).abs() < 1e-10);
        assert!(u.verify_conservation());
    }

    #[test]
    fn encode_empty_data_fails() {
        let mut u = make_test_universe();
        let anchor = Coord7D::new_even([0; 7]);
        assert_eq!(
            MemoryCodec::encode(&mut u, &anchor, &[]),
            Err(MemoryError::EmptyData)
        );
    }

    #[test]
    fn encode_too_much_data_fails() {
        let mut u = make_test_universe();
        let anchor = Coord7D::new_even([0; 7]);
        let data = vec![1.0; 29];
        assert_eq!(
            MemoryCodec::encode(&mut u, &anchor, &data),
            Err(MemoryError::DataTooLarge)
        );
    }

    #[test]
    fn encode_insufficient_energy_fails() {
        let mut u = DarkUniverse::new(10.0);
        let anchor = Coord7D::new_even([0; 7]);
        let data = vec![100.0; 7];
        assert_eq!(
            MemoryCodec::encode(&mut u, &anchor, &data),
            Err(MemoryError::InsufficientEnergy)
        );
    }

    #[test]
    fn encode_anchor_occupied_fails() {
        let mut u = make_test_universe();
        let anchor = Coord7D::new_even([0; 7]);
        u.materialize_uniform(anchor, 100.0).unwrap();

        let data = vec![1.0, 2.0, 3.0];
        assert_eq!(
            MemoryCodec::encode(&mut u, &anchor, &data),
            Err(MemoryError::NoAvailablePosition)
        );
    }

    #[test]
    fn multiple_memories_dont_conflict() {
        let mut u = DarkUniverse::new(500000.0);

        let anchors = [
            Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]),
            Coord7D::new_even([10, 0, 0, 0, 0, 0, 0]),
            Coord7D::new_even([0, 10, 0, 0, 0, 0, 0]),
        ];

        let data_sets = [
            vec![1.0, 2.0, 3.0],
            vec![10.0, 20.0, 30.0],
            vec![100.0, 200.0, 300.0],
        ];

        let mut atoms = Vec::new();
        for (anchor, data) in anchors.iter().zip(data_sets.iter()) {
            let atom = MemoryCodec::encode(&mut u, anchor, data).unwrap();
            atoms.push(atom);
        }

        assert!(u.verify_conservation());

        for (atom, expected) in atoms.iter().zip(data_sets.iter()) {
            let decoded = MemoryCodec::decode(&u, atom).unwrap();
            for (i, (e, a)) in expected.iter().zip(decoded.iter()).enumerate() {
                assert!(
                    (e - a).abs() < 1e-10,
                    "mismatch at dim {}: expected {}, got {}",
                    i,
                    e,
                    a
                );
            }
        }

        assert_eq!(atoms.len(), 3);
        for atom in &atoms {
            assert!(atom.is_manifested(&u));
        }
    }

    #[test]
    fn atom_is_valid_tetrahedron() {
        let mut u = make_test_universe();
        let anchor = Coord7D::new_even([0; 7]);
        let data = vec![1.0; 7];

        let atom = MemoryCodec::encode(&mut u, &anchor, &data).unwrap();
        let tet = atom.to_tetrahedron();

        assert!(tet.has_volume(), "encoded atom should have 3D volume");
        assert!(tet.is_mixed_parity(), "encoded atom should be mixed parity");
        assert!(tet.exists_in(&u));
    }

    #[test]
    fn encode_with_negative_data_preserves_exact() {
        let mut u = make_test_universe();
        let anchor = Coord7D::new_even([0; 7]);
        let data = vec![-5.0, 3.0, -2.0];

        let atom = MemoryCodec::encode(&mut u, &anchor, &data).unwrap();
        let decoded = MemoryCodec::decode(&u, &atom).unwrap();

        assert_eq!(decoded.len(), 3);
        assert!((decoded[0] - (-5.0)).abs() < 1e-10, "negative preserved");
        assert!((decoded[1] - 3.0).abs() < 1e-10);
        assert!((decoded[2] - (-2.0)).abs() < 1e-10, "negative preserved");
        assert!(u.verify_conservation());
    }

    #[test]
    fn full_memory_stress_test() {
        let mut u = DarkUniverse::new(1000000.0);
        let mut atoms = Vec::new();

        for i in 0..10 {
            let anchor = Coord7D::new_even([i * 20, 0, 0, 0, 0, 0, 0]);
            let data: Vec<f64> = (0..7).map(|d| (i * 7 + d) as f64 * 0.5).collect();
            let atom = MemoryCodec::encode(&mut u, &anchor, &data).unwrap();
            atoms.push((atom, data));
            assert!(u.verify_conservation());
        }

        for (atom, expected) in &atoms {
            let decoded = MemoryCodec::decode(&u, atom).unwrap();
            for (i, (e, a)) in expected.iter().zip(decoded.iter()).enumerate() {
                assert!(
                    (e - a).abs() < 1e-10,
                    "stress test dim {}: expected {}, got {}",
                    i,
                    e,
                    a
                );
            }
        }

        for (atom, _) in &atoms {
            assert!(atom.is_manifested(&u));
        }

        assert!(u.verify_conservation());
    }
}

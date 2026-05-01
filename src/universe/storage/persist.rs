// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::coord::Coord7D;
use crate::universe::crystal::CrystalEngine;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct UniverseSnapshot {
    pub total_energy: f64,
    pub conservation_checksum: u64,
    pub nodes: Vec<NodeSnapshot>,
    pub hebbian_edges: Vec<EdgeSnapshot>,
    pub memories: Vec<MemorySnapshot>,
    pub crystal_channels: Vec<ChannelSnapshot>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NodeSnapshot {
    pub coord: [i32; 7],
    pub is_even: bool,
    pub dims: [f64; 7],
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EdgeSnapshot {
    pub a: [i32; 7],
    pub a_even: bool,
    pub b: [i32; 7],
    pub b_even: bool,
    pub weight: f64,
    pub traversal_count: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MemorySnapshot {
    pub vertices: [[i32; 7]; 4],
    pub vertices_even: [bool; 4],
    pub data_dim: usize,
    pub physical_base: f64,
    #[serde(default)]
    pub created_at: u64,
    #[serde(default = "default_importance")]
    pub importance: f64,
}

fn default_importance() -> f64 {
    0.5
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChannelSnapshot {
    pub a: [i32; 7],
    pub a_even: bool,
    pub b: [i32; 7],
    pub b_even: bool,
    pub strength: f64,
    pub is_super: bool,
}

#[derive(Debug)]
pub enum PersistError {
    Serialization(String),
    Deserialization(String),
    ConservationViolation(String),
}

impl std::fmt::Display for PersistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistError::Serialization(s) => write!(f, "serialization error: {}", s),
            PersistError::Deserialization(s) => write!(f, "deserialization error: {}", s),
            PersistError::ConservationViolation(s) => write!(f, "conservation violation: {}", s),
        }
    }
}

impl std::error::Error for PersistError {}

pub struct PersistReport {
    pub nodes_serialized: usize,
    pub edges_serialized: usize,
    pub memories_serialized: usize,
    pub crystals_serialized: usize,
    pub bytes_written: usize,
}

impl std::fmt::Display for PersistReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Persist[{}nodes {}edges {}mems {}crystals {}bytes]",
            self.nodes_serialized,
            self.edges_serialized,
            self.memories_serialized,
            self.crystals_serialized,
            self.bytes_written
        )
    }
}

pub struct PersistEngine;

impl PersistEngine {
    pub fn serialize(
        universe: &DarkUniverse,
        hebbian: &HebbianMemory,
        memories: &[MemoryAtom],
        crystal: &CrystalEngine,
    ) -> Result<(UniverseSnapshot, PersistReport), PersistError> {
        let stats = universe.stats();

        let nodes: Vec<NodeSnapshot> = universe
            .coords()
            .iter()
            .filter_map(|c| {
                let node = universe.get_node(c)?;
                Some(NodeSnapshot {
                    coord: c.basis(),
                    is_even: c.is_even(),
                    dims: *node.energy().dims(),
                })
            })
            .collect();

        let edges: Vec<EdgeSnapshot> = hebbian
            .edges_with_traversal()
            .iter()
            .map(|((a, b), weight, count)| EdgeSnapshot {
                a: a.basis(),
                a_even: a.is_even(),
                b: b.basis(),
                b_even: b.is_even(),
                weight: *weight,
                traversal_count: *count,
            })
            .collect();

        let mem_snapshots: Vec<MemorySnapshot> = memories
            .iter()
            .map(|m| {
                let mut verts = [[0i32; 7]; 4];
                let mut even = [false; 4];
                for (i, v) in m.vertices().iter().enumerate() {
                    verts[i] = v.basis();
                    even[i] = v.is_even();
                }
                MemorySnapshot {
                    vertices: verts,
                    vertices_even: even,
                    data_dim: m.data_dim(),
                    physical_base: m.physical_base_f64(),
                    created_at: m.created_at(),
                    importance: m.importance(),
                }
            })
            .collect();

        let channels: Vec<ChannelSnapshot> = crystal
            .all_channels()
            .iter()
            .map(|((a, b), ch)| ChannelSnapshot {
                a: a.basis(),
                a_even: a.is_even(),
                b: b.basis(),
                b_even: b.is_even(),
                strength: ch.strength(),
                is_super: ch.is_super(),
            })
            .collect();

        let mut node_dim_hash: u64 = 0;
        for ns in &nodes {
            for &d in &ns.dims {
                let s = format!("{:+.17e}", d);
                node_dim_hash = node_dim_hash.wrapping_mul(31).wrapping_add(
                    s.bytes()
                        .fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64)),
                );
            }
            node_dim_hash =
                node_dim_hash
                    .wrapping_mul(7)
                    .wrapping_add(if ns.is_even { 1 } else { 0 });
        }
        let mut edge_hash: u64 = 0;
        for es in &edges {
            edge_hash = edge_hash
                .wrapping_mul(31)
                .wrapping_add(es.traversal_count as u64)
                .wrapping_add(
                    es.a.iter()
                        .fold(0u64, |a, &v| a.wrapping_mul(31).wrapping_add(v as u64)),
                )
                .wrapping_add(
                    es.b.iter()
                        .fold(0u64, |a, &v| a.wrapping_mul(31).wrapping_add(v as u64)),
                );
        }
        let mut chan_hash: u64 = 0;
        for cs in &channels {
            chan_hash = chan_hash
                .wrapping_mul(37)
                .wrapping_add(if cs.is_super { 1u64 } else { 0u64 })
                .wrapping_add(
                    cs.a.iter()
                        .fold(0u64, |a, &v| a.wrapping_mul(31).wrapping_add(v as u64)),
                )
                .wrapping_add(
                    cs.b.iter()
                        .fold(0u64, |a, &v| a.wrapping_mul(31).wrapping_add(v as u64)),
                );
        }
        let conservation_checksum = (nodes.len() as u64).wrapping_mul(0x517cc1b727220a95)
            ^ (mem_snapshots.len() as u64).wrapping_mul(0x6c62272e07bb0142)
            ^ (edges.len() as u64).wrapping_mul(0x9e3779b97f4a7c15)
            ^ (channels.len() as u64).wrapping_mul(0x123456789abcdef0)
            ^ node_dim_hash
            ^ edge_hash
            ^ chan_hash;

        let snapshot = UniverseSnapshot {
            total_energy: stats.total_energy,
            conservation_checksum,
            nodes,
            hebbian_edges: edges,
            memories: mem_snapshots,
            crystal_channels: channels,
        };

        let report = PersistReport {
            nodes_serialized: snapshot.nodes.len(),
            edges_serialized: snapshot.hebbian_edges.len(),
            memories_serialized: snapshot.memories.len(),
            crystals_serialized: snapshot.crystal_channels.len(),
            bytes_written: serde_json::to_string(&snapshot)
                .map(|s| s.len())
                .unwrap_or(0),
        };

        Ok((snapshot, report))
    }

    pub fn deserialize(
        snapshot: &UniverseSnapshot,
    ) -> Result<(DarkUniverse, HebbianMemory, Vec<MemoryAtom>, CrystalEngine), PersistError> {
        let mut node_dim_hash: u64 = 0;
        for ns in &snapshot.nodes {
            for &d in &ns.dims {
                let s = format!("{:+.17e}", d);
                node_dim_hash = node_dim_hash.wrapping_mul(31).wrapping_add(
                    s.bytes()
                        .fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64)),
                );
            }
            node_dim_hash =
                node_dim_hash
                    .wrapping_mul(7)
                    .wrapping_add(if ns.is_even { 1 } else { 0 });
        }
        let mut edge_hash: u64 = 0;
        for es in &snapshot.hebbian_edges {
            edge_hash = edge_hash
                .wrapping_mul(31)
                .wrapping_add(es.traversal_count as u64)
                .wrapping_add(
                    es.a.iter()
                        .fold(0u64, |a, &v| a.wrapping_mul(31).wrapping_add(v as u64)),
                )
                .wrapping_add(
                    es.b.iter()
                        .fold(0u64, |a, &v| a.wrapping_mul(31).wrapping_add(v as u64)),
                );
        }
        let mut chan_hash: u64 = 0;
        for cs in &snapshot.crystal_channels {
            chan_hash = chan_hash
                .wrapping_mul(37)
                .wrapping_add(if cs.is_super { 1u64 } else { 0u64 })
                .wrapping_add(
                    cs.a.iter()
                        .fold(0u64, |a, &v| a.wrapping_mul(31).wrapping_add(v as u64)),
                )
                .wrapping_add(
                    cs.b.iter()
                        .fold(0u64, |a, &v| a.wrapping_mul(31).wrapping_add(v as u64)),
                );
        }
        let checksum = (snapshot.nodes.len() as u64).wrapping_mul(0x517cc1b727220a95)
            ^ (snapshot.memories.len() as u64).wrapping_mul(0x6c62272e07bb0142)
            ^ (snapshot.hebbian_edges.len() as u64).wrapping_mul(0x9e3779b97f4a7c15)
            ^ (snapshot.crystal_channels.len() as u64).wrapping_mul(0x123456789abcdef0)
            ^ node_dim_hash
            ^ edge_hash
            ^ chan_hash;

        if snapshot.conservation_checksum != 0 && checksum != snapshot.conservation_checksum {
            return Err(PersistError::ConservationViolation(format!(
                "snapshot checksum mismatch: stored={} computed={}",
                snapshot.conservation_checksum, checksum
            )));
        }

        let mut universe = DarkUniverse::new(snapshot.total_energy);

        for ns in &snapshot.nodes {
            let coord = if ns.is_even {
                Coord7D::new_even(ns.coord)
            } else {
                Coord7D::new_odd(ns.coord)
            };
            let field = crate::universe::energy::EnergyField::from_dims(ns.dims).map_err(|e| {
                PersistError::Deserialization(format!("invalid energy dims: {}", e))
            })?;
            if universe.materialize_field(coord, field).is_err() {
                return Err(PersistError::Deserialization(format!(
                    "failed to materialize node at {:?}",
                    ns.coord
                )));
            }
        }

        if !universe.verify_conservation() {
            return Err(PersistError::ConservationViolation(
                "conservation violated after deserialization".to_string(),
            ));
        }

        let mut hebbian = HebbianMemory::new();
        for es in &snapshot.hebbian_edges {
            let a = if es.a_even {
                Coord7D::new_even(es.a)
            } else {
                Coord7D::new_odd(es.a)
            };
            let b = if es.b_even {
                Coord7D::new_even(es.b)
            } else {
                Coord7D::new_odd(es.b)
            };
            for _ in 0..es.traversal_count.max(1) {
                hebbian.record_path(&[a, b], es.weight);
            }
        }

        let memories: Vec<MemoryAtom> = snapshot
            .memories
            .iter()
            .filter_map(|ms| {
                if ms.data_dim == 0 || ms.data_dim > 28 || ms.physical_base <= 0.0 {
                    return None;
                }
                let mut verts = [Coord7D::new_even([0; 7]); 4];
                for (i, _) in ms.vertices.iter().enumerate() {
                    verts[i] = if ms.vertices_even[i] {
                        Coord7D::new_even(ms.vertices[i])
                    } else {
                        Coord7D::new_odd(ms.vertices[i])
                    };
                }
                Some(MemoryAtom::from_parts_with_importance(
                    verts,
                    ms.data_dim,
                    ms.physical_base,
                    ms.created_at,
                    ms.importance,
                ))
            })
            .collect();

        let mut crystal = CrystalEngine::new();
        for cs in &snapshot.crystal_channels {
            let a = if cs.a_even {
                Coord7D::new_even(cs.a)
            } else {
                Coord7D::new_odd(cs.a)
            };
            let b = if cs.b_even {
                Coord7D::new_even(cs.b)
            } else {
                Coord7D::new_odd(cs.b)
            };
            crystal.restore_channel(a, b, cs.strength, cs.is_super);
        }

        Ok((universe, hebbian, memories, crystal))
    }

    pub fn to_json(
        universe: &DarkUniverse,
        hebbian: &HebbianMemory,
        memories: &[MemoryAtom],
        crystal: &CrystalEngine,
    ) -> Result<String, PersistError> {
        let (snapshot, _) = Self::serialize(universe, hebbian, memories, crystal)?;
        serde_json::to_string_pretty(&snapshot)
            .map_err(|e| PersistError::Serialization(e.to_string()))
    }

    pub fn from_json(
        json: &str,
    ) -> Result<(DarkUniverse, HebbianMemory, Vec<MemoryAtom>, CrystalEngine), PersistError> {
        let snapshot: UniverseSnapshot =
            serde_json::from_str(json).map_err(|e| PersistError::Deserialization(e.to_string()))?;
        Self::deserialize(&snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::memory::MemoryCodec;

    fn build_system() -> (DarkUniverse, HebbianMemory, Vec<MemoryAtom>, CrystalEngine) {
        let mut u = DarkUniverse::new(1_000_000.0);
        let mut h = HebbianMemory::new();

        let m1 = MemoryCodec::encode(
            &mut u,
            &Coord7D::new_even([10, 10, 10, 0, 0, 0, 0]),
            &[1.0, 2.0],
        )
        .unwrap();
        let m2 = MemoryCodec::encode(
            &mut u,
            &Coord7D::new_even([15, 15, 15, 0, 0, 0, 0]),
            &[3.0, 4.0],
        )
        .unwrap();

        for x in 0..4i32 {
            for y in 0..4i32 {
                for z in 0..4i32 {
                    let c = Coord7D::new_even([x + 10, y + 10, z + 10, 0, 0, 0, 0]);
                    u.materialize_biased(c, 50.0, 0.6).ok();
                }
            }
        }

        h.record_path(&[*m1.anchor(), *m2.anchor()], 2.0);

        let mut crystal = CrystalEngine::new();
        crystal.crystallize(&h, &u);

        (u, h, vec![m1, m2], crystal)
    }

    #[test]
    fn serialize_captures_all_state() {
        let (u, h, mems, crystal) = build_system();
        let (_, report) = PersistEngine::serialize(&u, &h, &mems, &crystal).unwrap();

        assert!(report.nodes_serialized > 0);
        assert!(report.edges_serialized > 0);
        assert_eq!(report.memories_serialized, 2);
        assert!(report.crystals_serialized > 0);
    }

    #[test]
    fn roundtrip_preserves_conservation() {
        let (u, h, mems, crystal) = build_system();
        let json = PersistEngine::to_json(&u, &h, &mems, &crystal).unwrap();

        let (u2, _h2, _mems2, _crystal2) = PersistEngine::from_json(&json).unwrap();
        assert!(
            u2.verify_conservation(),
            "conservation must survive roundtrip"
        );
    }

    #[test]
    fn roundtrip_preserves_node_count() {
        let (u, h, mems, crystal) = build_system();
        let before = u.active_node_count();
        let json = PersistEngine::to_json(&u, &h, &mems, &crystal).unwrap();

        let (u2, _, _, _) = PersistEngine::from_json(&json).unwrap();
        assert_eq!(u2.active_node_count(), before);
    }

    #[test]
    fn roundtrip_preserves_energy() {
        let (u, h, mems, crystal) = build_system();
        let before_total = u.total_energy();
        let before_alloc = u.allocated_energy();
        let json = PersistEngine::to_json(&u, &h, &mems, &crystal).unwrap();

        let (u2, _, _, _) = PersistEngine::from_json(&json).unwrap();
        assert!((u2.total_energy() - before_total).abs() < 1e-10);
        assert!((u2.allocated_energy() - before_alloc).abs() < 1e-6);
    }

    #[test]
    fn json_format_valid() {
        let (u, h, mems, crystal) = build_system();
        let json = PersistEngine::to_json(&u, &h, &mems, &crystal).unwrap();
        assert!(json.starts_with('{'));
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());
    }
}

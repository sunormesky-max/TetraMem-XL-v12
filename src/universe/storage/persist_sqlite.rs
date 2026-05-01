// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::coord::Coord7D;
use crate::universe::crystal::CrystalEngine;
use crate::universe::cognitive::functional_emotion::EmotionSource;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;
use rusqlite::{params, Connection};
use std::path::Path;

#[derive(Debug)]
pub enum SqliteError {
    Open(String),
    Query(String),
    Insert(String),
    Schema(String),
    Conservation(String),
}

impl std::fmt::Display for SqliteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SqliteError::Open(s) => write!(f, "open error: {}", s),
            SqliteError::Query(s) => write!(f, "query error: {}", s),
            SqliteError::Insert(s) => write!(f, "insert error: {}", s),
            SqliteError::Schema(s) => write!(f, "schema error: {}", s),
            SqliteError::Conservation(s) => write!(f, "conservation error: {}", s),
        }
    }
}

impl std::error::Error for SqliteError {}

pub struct PersistSqlite;

impl PersistSqlite {
    fn init_schema(conn: &Connection) -> Result<(), SqliteError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS nodes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                c0 INTEGER NOT NULL, c1 INTEGER NOT NULL, c2 INTEGER NOT NULL,
                c3 INTEGER NOT NULL, c4 INTEGER NOT NULL, c5 INTEGER NOT NULL, c6 INTEGER NOT NULL,
                is_even INTEGER NOT NULL,
                d0 REAL NOT NULL, d1 REAL NOT NULL, d2 REAL NOT NULL,
                d3 REAL NOT NULL, d4 REAL NOT NULL, d5 REAL NOT NULL, d6 REAL NOT NULL,
                UNIQUE(c0, c1, c2, c3, c4, c5, c6, is_even)
            );
            CREATE TABLE IF NOT EXISTS hebbian_edges (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                a0 INTEGER NOT NULL, a1 INTEGER NOT NULL, a2 INTEGER NOT NULL,
                a3 INTEGER NOT NULL, a4 INTEGER NOT NULL, a5 INTEGER NOT NULL, a6 INTEGER NOT NULL,
                a_even INTEGER NOT NULL,
                b0 INTEGER NOT NULL, b1 INTEGER NOT NULL, b2 INTEGER NOT NULL,
                b3 INTEGER NOT NULL, b4 INTEGER NOT NULL, b5 INTEGER NOT NULL, b6 INTEGER NOT NULL,
                b_even INTEGER NOT NULL,
                weight REAL NOT NULL,
                traversal_count INTEGER NOT NULL DEFAULT 1,
                emotion_tag TEXT DEFAULT NULL,
                emotion_weight REAL NOT NULL DEFAULT 0.0
            );
            CREATE TABLE IF NOT EXISTS memories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                v00 INTEGER NOT NULL, v01 INTEGER NOT NULL, v02 INTEGER NOT NULL,
                v03 INTEGER NOT NULL, v04 INTEGER NOT NULL, v05 INTEGER NOT NULL, v06 INTEGER NOT NULL,
                v0_even INTEGER NOT NULL,
                v10 INTEGER NOT NULL, v11 INTEGER NOT NULL, v12 INTEGER NOT NULL,
                v13 INTEGER NOT NULL, v14 INTEGER NOT NULL, v15 INTEGER NOT NULL, v16 INTEGER NOT NULL,
                v1_even INTEGER NOT NULL,
                v20 INTEGER NOT NULL, v21 INTEGER NOT NULL, v22 INTEGER NOT NULL,
                v23 INTEGER NOT NULL, v24 INTEGER NOT NULL, v25 INTEGER NOT NULL, v26 INTEGER NOT NULL,
                v2_even INTEGER NOT NULL,
                v30 INTEGER NOT NULL, v31 INTEGER NOT NULL, v32 INTEGER NOT NULL,
                v33 INTEGER NOT NULL, v34 INTEGER NOT NULL, v35 INTEGER NOT NULL, v36 INTEGER NOT NULL,
                v3_even INTEGER NOT NULL,
                data_dim INTEGER NOT NULL,
                physical_base REAL NOT NULL,
                created_at INTEGER NOT NULL DEFAULT 0,
                importance REAL NOT NULL DEFAULT 0.5
            );
            CREATE TABLE IF NOT EXISTS crystal_channels (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                a0 INTEGER NOT NULL, a1 INTEGER NOT NULL, a2 INTEGER NOT NULL,
                a3 INTEGER NOT NULL, a4 INTEGER NOT NULL, a5 INTEGER NOT NULL, a6 INTEGER NOT NULL,
                a_even INTEGER NOT NULL,
                b0 INTEGER NOT NULL, b1 INTEGER NOT NULL, b2 INTEGER NOT NULL,
                b3 INTEGER NOT NULL, b4 INTEGER NOT NULL, b5 INTEGER NOT NULL, b6 INTEGER NOT NULL,
                b_even INTEGER NOT NULL,
                strength REAL NOT NULL,
                is_super INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_nodes_coord ON nodes(c0, c1, c2, c3, c4, c5, c6);
            CREATE INDEX IF NOT EXISTS idx_memories_dim ON memories(data_dim);
            CREATE INDEX IF NOT EXISTS idx_hebbian_a ON hebbian_edges(a0,a1,a2,a3,a4,a5,a6,a_even);
            CREATE INDEX IF NOT EXISTS idx_memories_anchor ON memories(v00,v01,v02,v03,v04,v05,v06,v0_even);
            CREATE INDEX IF NOT EXISTS idx_crystal_a ON crystal_channels(a0,a1,a2,a3,a4,a5,a6,a_even);
            "
        ).map_err(|e| SqliteError::Schema(e.to_string()))?;

        conn.execute_batch(
            "ALTER TABLE hebbian_edges ADD COLUMN emotion_tag TEXT DEFAULT NULL;
             ALTER TABLE hebbian_edges ADD COLUMN emotion_weight REAL NOT NULL DEFAULT 0.0;"
        ).ok();

        Ok(())
    }

    pub fn save(
        path: &Path,
        universe: &DarkUniverse,
        hebbian: &HebbianMemory,
        memories: &[MemoryAtom],
        crystal: &CrystalEngine,
    ) -> Result<u64, SqliteError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| SqliteError::Open(e.to_string()))?;
        }

        let conn = Connection::open(path).map_err(|e| SqliteError::Open(e.to_string()))?;

        #[cfg(unix)]
        {
            if let Err(e) = std::fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(0o600)) {
                tracing::warn!("failed to set restrictive permissions on {}: {}", path.display(), e);
            }
        }

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;")
            .map_err(|e| SqliteError::Open(e.to_string()))?;

        Self::init_schema(&conn)?;

        let tx = conn
            .unchecked_transaction()
            .map_err(|e| SqliteError::Open(e.to_string()))?;

        tx.execute("DELETE FROM nodes", [])
            .map_err(|e| SqliteError::Insert(e.to_string()))?;
        tx.execute("DELETE FROM hebbian_edges", [])
            .map_err(|e| SqliteError::Insert(e.to_string()))?;
        tx.execute("DELETE FROM memories", [])
            .map_err(|e| SqliteError::Insert(e.to_string()))?;
        tx.execute("DELETE FROM crystal_channels", [])
            .map_err(|e| SqliteError::Insert(e.to_string()))?;

        let stats = universe.stats();
        tx.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES (?1, ?2)",
            params!["total_energy", stats.total_energy.to_string()],
        )
        .map_err(|e| SqliteError::Insert(e.to_string()))?;

        for c in universe.coords_iter() {
            let node = match universe.get_node(&c) {
                Some(n) => n,
                None => continue,
            };
            let b = c.basis();
            let e = *node.energy().dims();
            tx.execute(
                "INSERT OR IGNORE INTO nodes (c0,c1,c2,c3,c4,c5,c6,is_even,d0,d1,d2,d3,d4,d5,d6) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
                params![b[0], b[1], b[2], b[3], b[4], b[5], b[6], c.is_even() as i32,
                         e[0], e[1], e[2], e[3], e[4], e[5], e[6]],
            ).map_err(|e| SqliteError::Insert(e.to_string()))?;
        }

        for edge in hebbian.edges_full() {
            let ab = edge.key.0.basis();
            let bb = edge.key.1.basis();
            let et: Option<String> = edge.emotion_tag.map(|s| format!("{:?}", s));
            tx.execute(
                "INSERT INTO hebbian_edges (a0,a1,a2,a3,a4,a5,a6,a_even,b0,b1,b2,b3,b4,b5,b6,b_even,weight,traversal_count,emotion_tag,emotion_weight) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20)",
                params![ab[0], ab[1], ab[2], ab[3], ab[4], ab[5], ab[6], edge.key.0.is_even() as i32,
                         bb[0], bb[1], bb[2], bb[3], bb[4], bb[5], bb[6], edge.key.1.is_even() as i32,
                         edge.weight, edge.traversal_count as i32, et, edge.emotion_weight],
            ).map_err(|e| SqliteError::Insert(e.to_string()))?;
        }

        for m in memories {
            let verts = m.vertices();
            let mut rows = Vec::new();
            for v in verts.iter() {
                let b = v.basis();
                rows.push((b, v.is_even()));
            }
            tx.execute(
                "INSERT INTO memories (v00,v01,v02,v03,v04,v05,v06,v0_even,v10,v11,v12,v13,v14,v15,v16,v1_even,v20,v21,v22,v23,v24,v25,v26,v2_even,v30,v31,v32,v33,v34,v35,v36,v3_even,data_dim,physical_base,created_at,importance) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29,?30,?31,?32,?33,?34,?35,?36)",
                params![
                    rows[0].0[0], rows[0].0[1], rows[0].0[2], rows[0].0[3], rows[0].0[4], rows[0].0[5], rows[0].0[6], rows[0].1 as i32,
                    rows[1].0[0], rows[1].0[1], rows[1].0[2], rows[1].0[3], rows[1].0[4], rows[1].0[5], rows[1].0[6], rows[1].1 as i32,
                    rows[2].0[0], rows[2].0[1], rows[2].0[2], rows[2].0[3], rows[2].0[4], rows[2].0[5], rows[2].0[6], rows[2].1 as i32,
                    rows[3].0[0], rows[3].0[1], rows[3].0[2], rows[3].0[3], rows[3].0[4], rows[3].0[5], rows[3].0[6], rows[3].1 as i32,
                    m.data_dim(), m.physical_base_f64(), m.created_at() as i64,
                    m.importance(),
                ],
            ).map_err(|e| SqliteError::Insert(e.to_string()))?;
        }

        for ((a, b), ch) in crystal.all_channels() {
            let ab = a.basis();
            let bb = b.basis();
            tx.execute(
                "INSERT INTO crystal_channels (a0,a1,a2,a3,a4,a5,a6,a_even,b0,b1,b2,b3,b4,b5,b6,b_even,strength,is_super) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
                params![ab[0], ab[1], ab[2], ab[3], ab[4], ab[5], ab[6], a.is_even() as i32,
                         bb[0], bb[1], bb[2], bb[3], bb[4], bb[5], bb[6], b.is_even() as i32,
                         ch.strength(), ch.is_super() as i32],
            ).map_err(|e| SqliteError::Insert(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| SqliteError::Insert(e.to_string()))?;

        let node_count: u64 = stats.active_nodes as u64;
        Ok(node_count)
    }

    pub fn load(
        path: &Path,
    ) -> Result<(DarkUniverse, HebbianMemory, Vec<MemoryAtom>, CrystalEngine), SqliteError> {
        let conn = Connection::open(path).map_err(|e| SqliteError::Open(e.to_string()))?;

        Self::init_schema(&conn)?;

        let total_energy: String = conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'total_energy'",
                [],
                |row| row.get::<_, String>(0),
            )
            .map_err(|e| SqliteError::Query(e.to_string()))?;

        let total_energy: f64 = total_energy
            .parse::<f64>()
            .map_err(|e: std::num::ParseFloatError| SqliteError::Query(e.to_string()))?;

        let mut universe = DarkUniverse::new(total_energy);

        {
            let mut stmt = conn
                .prepare("SELECT c0,c1,c2,c3,c4,c5,c6,is_even,d0,d1,d2,d3,d4,d5,d6 FROM nodes")
                .map_err(|e| SqliteError::Query(e.to_string()))?;

            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        [
                            row.get::<_, i32>(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                            row.get(6)?,
                        ],
                        row.get::<_, i32>(7)?,
                        [
                            row.get::<_, f64>(8)?,
                            row.get(9)?,
                            row.get(10)?,
                            row.get(11)?,
                            row.get(12)?,
                            row.get(13)?,
                            row.get(14)?,
                        ],
                    ))
                })
                .map_err(|e| SqliteError::Query(e.to_string()))?;

            for row in rows {
                let (basis, is_even, dims): ([i32; 7], i32, [f64; 7]) =
                    row.map_err(|e| SqliteError::Query(e.to_string()))?;
                let coord = if is_even != 0 {
                    Coord7D::new_even(basis)
                } else {
                    Coord7D::new_odd(basis)
                };
                let field = crate::universe::energy::EnergyField::from_dims(dims)
                    .map_err(|e| SqliteError::Query(format!("invalid energy dims: {}", e)))?;
                universe
                    .materialize_field(coord, field)
                    .map_err(|e| SqliteError::Query(format!("materialize failed: {}", e)))?;
            }
        }

        if !universe.verify_conservation() {
            return Err(SqliteError::Conservation(
                "conservation violated after loading from SQLite".to_string(),
            ));
        }

        let mut hebbian = HebbianMemory::new();
        {
            let mut stmt = conn.prepare(
                "SELECT a0,a1,a2,a3,a4,a5,a6,a_even,b0,b1,b2,b3,b4,b5,b6,b_even,weight,traversal_count,emotion_tag,emotion_weight FROM hebbian_edges"
            ).map_err(|e| SqliteError::Query(e.to_string()))?;

            #[allow(clippy::type_complexity)]
            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        [
                            row.get::<_, i32>(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                            row.get(6)?,
                        ],
                        row.get::<_, i32>(7)?,
                        [
                            row.get::<_, i32>(8)?,
                            row.get(9)?,
                            row.get(10)?,
                            row.get(11)?,
                            row.get(12)?,
                            row.get(13)?,
                            row.get(14)?,
                        ],
                        row.get::<_, i32>(15)?,
                        row.get::<_, f64>(16)?,
                        row.get::<_, i32>(17)?,
                        row.get::<_, Option<String>>(18)?,
                        row.get::<_, f64>(19)?,
                    ))
                })
                .map_err(|e| SqliteError::Query(e.to_string()))?;

            for row in rows {
                #[allow(clippy::type_complexity)]
                let (ab, a_even, bb, b_even, weight, count, emotion_tag_str, emotion_weight): (
                    [i32; 7], i32, [i32; 7], i32, f64, i32, Option<String>, f64,
                ) = row.map_err(|e| SqliteError::Query(e.to_string()))?;
                let a = if a_even != 0 {
                    Coord7D::new_even(ab)
                } else {
                    Coord7D::new_odd(ab)
                };
                let b = if b_even != 0 {
                    Coord7D::new_even(bb)
                } else {
                    Coord7D::new_odd(bb)
                };
                let emotion_tag = emotion_tag_str.as_deref().and_then(|s| match s {
                    "Perceived" => Some(EmotionSource::Perceived),
                    "Functional" => Some(EmotionSource::Functional),
                    _ => None,
                });
                hebbian.restore_edge(a, b, weight, count.max(1) as usize, emotion_tag, emotion_weight);
            }
        }

        let mut mems = Vec::new();
        {
            let mut stmt = conn.prepare(
                "SELECT v00,v01,v02,v03,v04,v05,v06,v0_even,v10,v11,v12,v13,v14,v15,v16,v1_even,v20,v21,v22,v23,v24,v25,v26,v2_even,v30,v31,v32,v33,v34,v35,v36,v3_even,data_dim,physical_base,created_at,importance FROM memories"
            ).map_err(|e| SqliteError::Query(e.to_string()))?;

            let rows = stmt
                .query_map([], |row| {
                    let v0 = [
                        row.get::<_, i32>(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ];
                    let v0e = row.get::<_, i32>(7)?;
                    let v1 = [
                        row.get::<_, i32>(8)?,
                        row.get(9)?,
                        row.get(10)?,
                        row.get(11)?,
                        row.get(12)?,
                        row.get(13)?,
                        row.get(14)?,
                    ];
                    let v1e = row.get::<_, i32>(15)?;
                    let v2 = [
                        row.get::<_, i32>(16)?,
                        row.get(17)?,
                        row.get(18)?,
                        row.get(19)?,
                        row.get(20)?,
                        row.get(21)?,
                        row.get(22)?,
                    ];
                    let v2e = row.get::<_, i32>(23)?;
                    let v3 = [
                        row.get::<_, i32>(24)?,
                        row.get(25)?,
                        row.get(26)?,
                        row.get(27)?,
                        row.get(28)?,
                        row.get(29)?,
                        row.get(30)?,
                    ];
                    let v3e = row.get::<_, i32>(31)?;
                    let dim: i32 = row.get(32)?;
                    let pb: f64 = row.get(33)?;
                    let created_at: i64 = row.get(34)?;
                    let importance: f64 = row.get(35)?;
                    Ok((
                        v0,
                        v0e,
                        v1,
                        v1e,
                        v2,
                        v2e,
                        v3,
                        v3e,
                        dim as usize,
                        pb,
                        created_at as u64,
                        importance,
                    ))
                })
                .map_err(|e| SqliteError::Query(e.to_string()))?;

            for row in rows {
                let (v0, v0e, v1, v1e, v2, v2e, v3, v3e, dim, pb, created_at, importance) =
                    row.map_err(|e| SqliteError::Query(e.to_string()))?;
                let make = |b: [i32; 7], e: i32| -> Coord7D {
                    if e != 0 {
                        Coord7D::new_even(b)
                    } else {
                        Coord7D::new_odd(b)
                    }
                };
                let verts = [make(v0, v0e), make(v1, v1e), make(v2, v2e), make(v3, v3e)];
                mems.push(MemoryAtom::from_parts_with_importance(verts, dim, pb, created_at, importance));
            }
        }

        let mut crystal = CrystalEngine::new();
        {
            let mut stmt = conn.prepare(
                "SELECT a0,a1,a2,a3,a4,a5,a6,a_even,b0,b1,b2,b3,b4,b5,b6,b_even,strength,is_super FROM crystal_channels"
            ).map_err(|e| SqliteError::Query(e.to_string()))?;

            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        [
                            row.get::<_, i32>(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                            row.get(6)?,
                        ],
                        row.get::<_, i32>(7)?,
                        [
                            row.get::<_, i32>(8)?,
                            row.get(9)?,
                            row.get(10)?,
                            row.get(11)?,
                            row.get(12)?,
                            row.get(13)?,
                            row.get(14)?,
                        ],
                        row.get::<_, i32>(15)?,
                        row.get::<_, f64>(16)?,
                        row.get::<_, i32>(17)?,
                    ))
                })
                .map_err(|e| SqliteError::Query(e.to_string()))?;

            for row in rows {
                let (ab, ae, bb, be, str, is_s): ([i32; 7], i32, [i32; 7], i32, f64, i32) =
                    row.map_err(|e| SqliteError::Query(e.to_string()))?;
                let a = if ae != 0 {
                    Coord7D::new_even(ab)
                } else {
                    Coord7D::new_odd(ab)
                };
                let b = if be != 0 {
                    Coord7D::new_even(bb)
                } else {
                    Coord7D::new_odd(bb)
                };
                crystal.restore_channel(a, b, str, is_s != 0);
            }
        }

        Ok((universe, hebbian, mems, crystal))
    }

    pub fn exists(path: &Path) -> bool {
        path.exists()
    }

    pub fn query_stats(path: &Path) -> Result<SqliteStats, SqliteError> {
        let conn = Connection::open(path).map_err(|e| SqliteError::Open(e.to_string()))?;

        let nodes: i64 = conn
            .query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get(0))
            .map_err(|e| SqliteError::Query(e.to_string()))?;
        let edges: i64 = conn
            .query_row("SELECT COUNT(*) FROM hebbian_edges", [], |r| r.get(0))
            .map_err(|e| SqliteError::Query(e.to_string()))?;
        let memories: i64 = conn
            .query_row("SELECT COUNT(*) FROM memories", [], |r| r.get(0))
            .map_err(|e| SqliteError::Query(e.to_string()))?;
        let channels: i64 = conn
            .query_row("SELECT COUNT(*) FROM crystal_channels", [], |r| r.get(0))
            .map_err(|e| SqliteError::Query(e.to_string()))?;

        Ok(SqliteStats {
            nodes: nodes as usize,
            hebbian_edges: edges as usize,
            memories: memories as usize,
            crystal_channels: channels as usize,
        })
    }
}

pub struct SqliteStats {
    pub nodes: usize,
    pub hebbian_edges: usize,
    pub memories: usize,
    pub crystal_channels: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::memory::MemoryCodec;

    fn build_test_system() -> (DarkUniverse, HebbianMemory, Vec<MemoryAtom>, CrystalEngine) {
        let mut u = DarkUniverse::new(1_000_000.0);
        let mut h = HebbianMemory::new();

        let m1 = MemoryCodec::encode(
            &mut u,
            &Coord7D::new_even([10, 10, 10, 0, 0, 0, 0]),
            &[1.0, 2.0, 3.0],
        )
        .unwrap();
        let m2 = MemoryCodec::encode(
            &mut u,
            &Coord7D::new_even([15, 15, 15, 0, 0, 0, 0]),
            &[4.0, 5.0],
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
    fn save_and_load_roundtrip() {
        let dir = std::env::temp_dir().join("tetramem_sqlite_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test.db");

        let (u, h, mems, crystal) = build_test_system();
        let before_nodes = u.active_node_count();
        let before_energy = u.total_energy();
        let before_mems = mems.len();

        PersistSqlite::save(&path, &u, &h, &mems, &crystal).unwrap();
        assert!(path.exists());

        let (u2, _h2, mems2, _crystal2) = PersistSqlite::load(&path).unwrap();
        assert_eq!(u2.active_node_count(), before_nodes);
        assert!((u2.total_energy() - before_energy).abs() < 1e-10);
        assert_eq!(mems2.len(), before_mems);
        assert!(u2.verify_conservation());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn query_stats_works() {
        let dir = std::env::temp_dir().join("tetramem_sqlite_stats_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("stats_test.db");

        let (u, h, mems, crystal) = build_test_system();
        PersistSqlite::save(&path, &u, &h, &mems, &crystal).unwrap();

        let stats = PersistSqlite::query_stats(&path).unwrap();
        assert!(stats.nodes > 0);
        assert_eq!(stats.memories, 2);
        assert!(stats.hebbian_edges > 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn overwrite_replaces_data() {
        let dir = std::env::temp_dir().join("tetramem_sqlite_overwrite_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("overwrite_test.db");

        let (u, h, mems, crystal) = build_test_system();
        PersistSqlite::save(&path, &u, &h, &mems, &crystal).unwrap();

        let empty_u = DarkUniverse::new(1_000_000.0);
        let empty_h = HebbianMemory::new();
        PersistSqlite::save(&path, &empty_u, &empty_h, &[], &CrystalEngine::new()).unwrap();

        let stats = PersistSqlite::query_stats(&path).unwrap();
        assert_eq!(stats.nodes, 0);
        assert_eq!(stats.memories, 0);

        let _ = std::fs::remove_dir_all(&dir);
    }
}

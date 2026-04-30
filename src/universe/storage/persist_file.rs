use crate::universe::crystal::CrystalEngine;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;
use crate::universe::persist::PersistEngine;
use std::fs;
use std::path::Path;

pub struct PersistFile;

#[derive(Debug)]
pub enum FilePersistError {
    Io(String),
    Serialize(String),
    Deserialize(String),
    Conservation(String),
}

impl std::fmt::Display for FilePersistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilePersistError::Io(s) => write!(f, "IO error: {}", s),
            FilePersistError::Serialize(s) => write!(f, "serialize error: {}", s),
            FilePersistError::Deserialize(s) => write!(f, "deserialize error: {}", s),
            FilePersistError::Conservation(s) => write!(f, "conservation error: {}", s),
        }
    }
}

impl std::error::Error for FilePersistError {}

pub struct PersistInfo {
    pub path: String,
    pub bytes: usize,
    pub nodes: usize,
    pub memories: usize,
    pub edges: usize,
}

impl std::fmt::Display for PersistInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Persisted {} nodes, {} memories, {} edges ({} bytes) -> {}",
            self.nodes, self.memories, self.edges, self.bytes, self.path
        )
    }
}

impl PersistFile {
    pub fn save(
        path: &Path,
        universe: &DarkUniverse,
        hebbian: &HebbianMemory,
        memories: &[MemoryAtom],
        crystal: &CrystalEngine,
    ) -> Result<PersistInfo, FilePersistError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| FilePersistError::Io(e.to_string()))?;
        }

        let json = PersistEngine::to_json(universe, hebbian, memories, crystal)
            .map_err(|e| FilePersistError::Serialize(e.to_string()))?;

        let bytes = json.len();
        let tmp_path = path.with_extension("json.tmp");
        fs::write(&tmp_path, &json).map_err(|e| FilePersistError::Io(e.to_string()))?;

        fs::rename(&tmp_path, path).map_err(|e| FilePersistError::Io(e.to_string()))?;

        let stats = universe.stats();
        Ok(PersistInfo {
            path: path.display().to_string(),
            bytes,
            nodes: stats.active_nodes,
            memories: memories.len(),
            edges: hebbian.edge_count(),
        })
    }

    pub fn load(
        path: &Path,
    ) -> Result<(DarkUniverse, HebbianMemory, Vec<MemoryAtom>, CrystalEngine), FilePersistError>
    {
        if !path.exists() {
            return Err(FilePersistError::Io(format!(
                "persist file not found: {}",
                path.display()
            )));
        }

        let json = fs::read_to_string(path).map_err(|e| FilePersistError::Io(e.to_string()))?;

        let (universe, hebbian, memories, crystal) = PersistEngine::from_json(&json)
            .map_err(|e| FilePersistError::Deserialize(e.to_string()))?;

        if !universe.verify_conservation() {
            return Err(FilePersistError::Conservation(
                "conservation violated after loading persisted state".to_string(),
            ));
        }

        Ok((universe, hebbian, memories, crystal))
    }

    pub fn exists(path: &Path) -> bool {
        path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::coord::Coord7D;
    use crate::universe::memory::MemoryCodec;

    fn build_test_system() -> (DarkUniverse, HebbianMemory, Vec<MemoryAtom>, CrystalEngine) {
        let mut u = DarkUniverse::new(1_000_000.0);
        let h = HebbianMemory::new();

        let m1 = MemoryCodec::encode(
            &mut u,
            &Coord7D::new_even([10, 10, 10, 0, 0, 0, 0]),
            &[1.0, 2.0, 3.0],
        )
        .unwrap();

        (u, h, vec![m1], CrystalEngine::new())
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = std::env::temp_dir().join("tetramem_persist_test");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test_state.json");

        let (u, h, mems, crystal) = build_test_system();
        let before_nodes = u.active_node_count();
        let before_energy = u.total_energy();

        let info = PersistFile::save(&path, &u, &h, &mems, &crystal).unwrap();
        assert!(info.bytes > 0);
        assert!(path.exists());

        let (u2, _h2, mems2, _crystal2) = PersistFile::load(&path).unwrap();
        assert_eq!(u2.active_node_count(), before_nodes);
        assert!((u2.total_energy() - before_energy).abs() < 1e-10);
        assert_eq!(mems2.len(), 1);
        assert!(u2.verify_conservation());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_nonexistent_fails() {
        let result = PersistFile::load(Path::new("/nonexistent/state.json"));
        assert!(result.is_err());
    }

    #[test]
    fn atomic_write_no_tmp_left() {
        let dir = std::env::temp_dir().join("tetramem_atomic_test");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("atomic_state.json");

        let (u, h, mems, crystal) = build_test_system();
        PersistFile::save(&path, &u, &h, &mems, &crystal).unwrap();

        assert!(path.exists());
        assert!(!path.with_extension("json.tmp").exists());

        let _ = fs::remove_dir_all(&dir);
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
pub mod backup;
pub mod persist;
pub mod persist_file;
pub mod persist_sqlite;

pub trait PersistBackend: Send + Sync {
    fn save_backend(
        &self,
        path: &std::path::Path,
        universe: &crate::universe::node::DarkUniverse,
        hebbian: &crate::universe::hebbian::HebbianMemory,
        memories: &[crate::universe::memory::MemoryAtom],
        crystal: &crate::universe::crystal::CrystalEngine,
    ) -> Result<usize, String>;

    fn load_backend(
        &self,
        path: &std::path::Path,
    ) -> Result<
        (
            crate::universe::node::DarkUniverse,
            crate::universe::hebbian::HebbianMemory,
            Vec<crate::universe::memory::MemoryAtom>,
            crate::universe::crystal::CrystalEngine,
        ),
        String,
    >;
}

pub struct FileBackend;
pub struct SqliteBackend;

impl PersistBackend for FileBackend {
    fn save_backend(
        &self,
        path: &std::path::Path,
        universe: &crate::universe::node::DarkUniverse,
        hebbian: &crate::universe::hebbian::HebbianMemory,
        memories: &[crate::universe::memory::MemoryAtom],
        crystal: &crate::universe::crystal::CrystalEngine,
    ) -> Result<usize, String> {
        persist_file::PersistFile::save(path, universe, hebbian, memories, crystal)
            .map(|info| info.bytes)
            .map_err(|e| e.to_string())
    }

    fn load_backend(
        &self,
        path: &std::path::Path,
    ) -> Result<
        (
            crate::universe::node::DarkUniverse,
            crate::universe::hebbian::HebbianMemory,
            Vec<crate::universe::memory::MemoryAtom>,
            crate::universe::crystal::CrystalEngine,
        ),
        String,
    > {
        persist_file::PersistFile::load(path).map_err(|e| e.to_string())
    }
}

impl PersistBackend for SqliteBackend {
    fn save_backend(
        &self,
        path: &std::path::Path,
        universe: &crate::universe::node::DarkUniverse,
        hebbian: &crate::universe::hebbian::HebbianMemory,
        memories: &[crate::universe::memory::MemoryAtom],
        crystal: &crate::universe::crystal::CrystalEngine,
    ) -> Result<usize, String> {
        persist_sqlite::PersistSqlite::save(path, universe, hebbian, memories, crystal)
            .map(|n| n as usize)
            .map_err(|e| e.to_string())
    }

    fn load_backend(
        &self,
        path: &std::path::Path,
    ) -> Result<
        (
            crate::universe::node::DarkUniverse,
            crate::universe::hebbian::HebbianMemory,
            Vec<crate::universe::memory::MemoryAtom>,
            crate::universe::crystal::CrystalEngine,
        ),
        String,
    > {
        persist_sqlite::PersistSqlite::load(path).map_err(|e| e.to_string())
    }
}

pub fn create_backend(backend_type: &str) -> Box<dyn PersistBackend> {
    match backend_type {
        "sqlite" => Box::new(SqliteBackend),
        _ => Box::new(FileBackend),
    }
}

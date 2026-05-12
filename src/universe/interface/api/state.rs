// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::universe::auth::{JwtConfig, TokenBlocklist, UserStore};
use crate::universe::backup::BackupScheduler;
use crate::universe::cluster::ClusterManager;
use crate::universe::cognitive::prediction::PredictionState;
use crate::universe::config::AppConfig;
use crate::universe::constitution::Constitution;
use crate::universe::crystal::CrystalEngine;
use crate::universe::events::EventBusSender;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::memory::MemoryCodec;
use crate::universe::memory::{ClusteringEngine, InterestProfile, SemanticEngine, SurfacedMemory};
use crate::universe::node::DarkUniverse;
use crate::universe::perception::PerceptionBudget;
use crate::universe::plugins::PluginManager;
use crate::universe::safety::events::EventBus;
use crate::universe::safety::identity_guard::IdentityGuard;
use crate::universe::watchdog::Watchdog;

pub struct MemoryStore {
    pub memories: Vec<MemoryAtom>,
    pub index: HashMap<String, usize>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            memories: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.memories.len()
    }

    pub fn is_empty(&self) -> bool {
        self.memories.is_empty()
    }

    pub fn rebuild_index(&mut self) {
        self.index.clear();
        for (i, m) in self.memories.iter().enumerate() {
            self.index.insert(format!("{}", m.anchor()), i);
        }
    }

    pub fn push(&mut self, atom: MemoryAtom) {
        let idx = self.memories.len();
        self.index.insert(format!("{}", atom.anchor()), idx);
        self.memories.push(atom);
    }

    pub fn remove_at(&mut self, i: usize) -> Option<MemoryAtom> {
        if i >= self.memories.len() {
            return None;
        }
        let anchor_str = format!("{}", self.memories[i].anchor());
        self.index.remove(&anchor_str);
        let atom = self.memories.remove(i);
        for val in self.index.values_mut() {
            if *val > i {
                *val -= 1;
            }
        }
        Some(atom)
    }

    pub fn get_by_anchor(&self, anchor_str: &str) -> Option<&MemoryAtom> {
        self.index.get(anchor_str).map(|&i| &self.memories[i])
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AppState {
    pub universe: RwLock<DarkUniverse>,
    pub hebbian: RwLock<HebbianMemory>,
    pub memory_store: RwLock<MemoryStore>,
    pub crystal: RwLock<CrystalEngine>,
    pub perception: RwLock<PerceptionBudget>,
    pub semantic: RwLock<SemanticEngine>,
    pub clustering: RwLock<ClusteringEngine>,
    pub constitution: RwLock<Constitution>,
    pub events: Mutex<EventBus>,
    pub event_sender: EventBusSender,
    pub watchdog: RwLock<Watchdog>,
    pub backup: RwLock<BackupScheduler>,
    pub cluster: Mutex<ClusterManager>,
    pub interests: RwLock<HashMap<String, InterestProfile>>,
    pub memory_stream: tokio::sync::broadcast::Sender<SurfacedMemory>,
    pub surfaced_seq: std::sync::atomic::AtomicU64,
    pub config: AppConfig,
    pub jwt: JwtConfig,
    pub users: UserStore,
    pub token_blocklist: RwLock<TokenBlocklist>,
    pub identity_guard: RwLock<IdentityGuard>,
    pub plugins: RwLock<PluginManager>,
    pub prediction: RwLock<PredictionState>,
    pub shutdown: Arc<AtomicBool>,
}

pub type SharedState = Arc<AppState>;

pub fn build_shared_state(
    config: AppConfig,
    universe: DarkUniverse,
    hebbian: HebbianMemory,
    memories: Vec<MemoryAtom>,
    crystal: CrystalEngine,
    semantic: SemanticEngine,
    clustering: ClusteringEngine,
) -> SharedState {
    let total_energy = universe.total_energy();
    let (event_sender, event_rx) = EventBus::create_channel();
    let event_bus = EventBus::from_receiver(event_rx);
    let mut memory_store = MemoryStore {
        memories,
        index: HashMap::new(),
    };
    memory_store.rebuild_index();

    Arc::new(AppState {
        universe: RwLock::new(universe),
        hebbian: RwLock::new(hebbian),
        memory_store: RwLock::new(memory_store),
        crystal: RwLock::new(crystal),
        perception: RwLock::new(PerceptionBudget::new(total_energy)),
        semantic: RwLock::new(semantic),
        clustering: RwLock::new(clustering),
        constitution: RwLock::new(Constitution::tetramem_default()),
        events: Mutex::new(event_bus),
        event_sender,
        watchdog: RwLock::new(Watchdog::with_defaults(total_energy)),
        backup: RwLock::new(BackupScheduler::with_defaults()),
        cluster: Mutex::new(ClusterManager::new(1, config.server.addr.clone())),
        interests: RwLock::new(HashMap::new()),
        memory_stream: crate::universe::memory::create_broadcast_channel(),
        surfaced_seq: std::sync::atomic::AtomicU64::new(0),
        jwt: JwtConfig::new(config.auth.jwt_secret.clone(), config.auth.jwt_expiry_secs),
        users: UserStore::new(&config.auth.users, &config.auth.jwt_secret),
        token_blocklist: RwLock::new(TokenBlocklist::new(10_000)),
        identity_guard: RwLock::new(IdentityGuard::default()),
        plugins: RwLock::new(PluginManager::new(1_000_000)),
        prediction: RwLock::new(PredictionState::default()),
        shutdown: Arc::new(AtomicBool::new(false)),
        config,
    })
}

pub fn rebuild_derived_memory_indexes(
    universe: &DarkUniverse,
    memories: &[MemoryAtom],
    semantic: &mut SemanticEngine,
    clustering: &mut ClusteringEngine,
) -> usize {
    let mut rebuilt = 0usize;
    for mem in memories {
        match MemoryCodec::decode(universe, mem) {
            Ok(data) => {
                semantic.index_memory(mem, &data);
                clustering.register_memory(*mem.anchor(), &data);
                rebuilt += 1;
            }
            Err(e) => {
                tracing::warn!(
                    anchor = %mem.anchor(),
                    error = %e,
                    "failed to rebuild derived memory indexes"
                );
            }
        }
    }
    rebuilt
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::coord::Coord7D;

    #[test]
    fn rebuild_derived_indexes_restores_semantic_search() {
        let mut universe = DarkUniverse::new(1_000_000.0);
        let anchor = Coord7D::new_even([12, 0, 0, 0, 0, 0, 0]);
        let data = vec![0.25, 0.5, 0.75];
        let memory = MemoryCodec::encode(&mut universe, &anchor, &data).unwrap();
        let memories = vec![memory];
        let mut semantic = SemanticEngine::new(Default::default());
        let mut clustering = ClusteringEngine::new(Default::default());

        assert!(semantic.search_similar(&data, 1).is_empty());

        let rebuilt =
            rebuild_derived_memory_indexes(&universe, &memories, &mut semantic, &mut clustering);

        assert_eq!(rebuilt, 1);
        let hits = semantic.search_similar(&data, 1);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].atom_key.vertices_basis[0], anchor.basis());
    }
}

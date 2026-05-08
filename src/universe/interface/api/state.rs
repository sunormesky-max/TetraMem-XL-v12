// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::universe::auth::{JwtConfig, UserStore};
use crate::universe::backup::BackupScheduler;
use crate::universe::cluster::ClusterManager;
use crate::universe::config::AppConfig;
use crate::universe::constitution::Constitution;
use crate::universe::crystal::CrystalEngine;
use crate::universe::events::EventBusSender;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::memory::{ClusteringEngine, InterestProfile, SemanticEngine, SurfacedMemory};
use crate::universe::node::DarkUniverse;
use crate::universe::perception::PerceptionBudget;
use crate::universe::safety::events::EventBus;
use crate::universe::safety::identity_guard::IdentityGuard;
use crate::universe::plugins::PluginManager;
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
    pub identity_guard: RwLock<IdentityGuard>,
    pub plugins: RwLock<PluginManager>,
    pub shutdown: Arc<AtomicBool>,
}

pub type SharedState = Arc<AppState>;

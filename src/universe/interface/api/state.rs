// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use std::collections::HashMap;
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
use crate::universe::watchdog::Watchdog;

pub struct AppState {
    pub universe: RwLock<DarkUniverse>,
    pub hebbian: RwLock<HebbianMemory>,
    pub memories: RwLock<Vec<MemoryAtom>>,
    pub memory_index: RwLock<HashMap<String, usize>>,
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
}

pub type SharedState = Arc<AppState>;

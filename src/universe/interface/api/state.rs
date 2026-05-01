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
use crate::universe::crystal::CrystalEngine;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;

pub struct AppState {
    pub universe: RwLock<DarkUniverse>,
    pub hebbian: RwLock<HebbianMemory>,
    pub memories: RwLock<Vec<MemoryAtom>>,
    pub memory_index: RwLock<HashMap<String, usize>>,
    pub crystal: RwLock<CrystalEngine>,
    pub backup: RwLock<BackupScheduler>,
    pub cluster: Mutex<ClusterManager>,
    pub config: AppConfig,
    pub jwt: JwtConfig,
    pub users: UserStore,
}

pub type SharedState = Arc<AppState>;

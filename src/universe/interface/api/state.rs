use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::universe::auth::JwtConfig;
use crate::universe::backup::BackupScheduler;
use crate::universe::cluster::ClusterManager;
use crate::universe::config::AppConfig;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;

pub struct AppState {
    pub universe: RwLock<DarkUniverse>,
    pub hebbian: RwLock<HebbianMemory>,
    pub memories: RwLock<Vec<MemoryAtom>>,
    pub crystal: RwLock<crate::universe::crystal::CrystalEngine>,
    pub backup: RwLock<BackupScheduler>,
    pub cluster: Mutex<ClusterManager>,
    pub config: AppConfig,
    pub jwt: JwtConfig,
}

pub type SharedState = Arc<AppState>;

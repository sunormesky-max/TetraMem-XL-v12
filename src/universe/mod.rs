pub mod api;
pub mod auth;
pub mod autoscale;
pub mod backup;
pub mod cluster;
pub mod config;
pub mod coord;
pub mod crystal;
pub mod dream;
pub mod energy;
pub mod error;
pub mod hebbian;
pub mod lattice;
pub mod memory;
pub mod metrics;
pub mod node;
pub mod observer;
pub mod persist;
pub mod persist_file;
pub mod persist_sqlite;
pub mod pulse;
pub mod raft_node;
pub mod reasoning;
pub mod regulation;
pub mod topology;
pub mod watchdog;

pub use autoscale::{AutoScaler, AutoScaleConfig, ScaleReport, ScaleReason};
pub use auth::{Claims, JwtConfig, LoginRequest, LoginResponse};
pub use backup::{BackupConfig, BackupMetadata, BackupReport, BackupScheduler, BackupTrigger};
pub use cluster::{
    AddNodeRequest, ClusterManager, ClusterNodeInfo, ClusterStatus, InitClusterRequest,
    ProposeRequest, ProposeResponse, RemoveNodeRequest,
};
pub use config::AppConfig;
pub use coord::Coord7D;
pub use crystal::{CrystalChannel, CrystalEngine, CrystalReport};
pub use dream::{DreamConfig, DreamEngine, DreamPhase, DreamReport};
pub use energy::{EnergyField, EnergyPool};
pub use error::AppError;
pub use hebbian::HebbianMemory;
pub use lattice::{BccVerification, Lattice, NeighborShell, Projection, Tetrahedron};
pub use memory::{MemoryAtom, MemoryCodec, MemoryError};
pub use node::{DarkNode, DarkUniverse};
pub use observer::{
    HealthLevel, HealthReport, RegulatorAction, RegulatorActionType, RegulatorParams,
    SelfRegulator, UniverseObserver,
};
pub use persist::PersistEngine;
pub use pulse::{PulseEngine, PulseResult, PulseType};
pub use raft_node::{
    LogStore, LogStoreInner, NodeId, Request, Response, StateMachineInner, StateMachineStore,
    TypeName, new_log_store, new_state_machine,
};
pub use reasoning::{ReasoningEngine, ReasoningResult, ReasoningType};
pub use regulation::{RegulationEngine, RegulationReport};
pub use topology::{BettiVector, TopologyEngine, TopologyReport};
pub use watchdog::{Watchdog, WatchdogLevel, WatchdogReport, WatermarkThresholds};

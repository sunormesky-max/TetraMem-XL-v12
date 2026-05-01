// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
pub mod adaptive;
pub mod cognitive;
pub mod consensus;
pub mod core;
pub mod error;
pub mod interface;
pub mod memory;
pub mod safety;
pub mod storage;

pub use core::config;
pub use core::coord;
pub use core::energy;
pub use core::lattice;
pub use core::node;
pub use core::physics;

pub use memory::dream;
pub use memory::hebbian;
pub use memory::pulse;

pub use cognitive::agent;
pub use cognitive::crystal;
pub use cognitive::emotion;
pub use cognitive::perception;
pub use cognitive::reasoning;
pub use cognitive::topology;

pub use adaptive::autoscale;
pub use adaptive::observer;
pub use adaptive::regulation;
pub use adaptive::watchdog;

pub use storage::backup;
pub use storage::persist;
pub use storage::persist_file;
pub use storage::persist_sqlite;

pub use consensus::cluster;
pub use consensus::raft_node;

pub use interface::api;
pub use interface::auth;
pub use interface::metrics;

pub use safety::constitution;
pub use safety::events;

pub use agent::{
    AgentContext, AgentContextMut, AgentKind, AgentReport, CognitiveAgent, CrystalAgent,
    DreamAgent, EmotionAgent, ObserverAgent, PulseAgent,
};
pub use auth::{Claims, JwtConfig, LoginRequest, LoginResponse};
pub use autoscale::{AutoScaleConfig, AutoScaler, ScaleReason, ScaleReport};
pub use backup::{BackupConfig, BackupMetadata, BackupReport, BackupScheduler, BackupTrigger};
pub use cluster::{
    AddNodeRequest, ClusterManager, ClusterNodeInfo, ClusterStatus, InitClusterRequest,
    ProposeRequest, ProposeResponse, RemoveNodeRequest,
};
pub use config::AppConfig;
pub use constitution::{Constitution, ConstitutionCheck, ImmutableRule, ModifiableBound};
pub use coord::Coord7D;
pub use crystal::{CrystalChannel, CrystalEngine, CrystalReport};
pub use dream::{DreamConfig, DreamEngine, DreamPhase, DreamReport};
pub use emotion::{
    EmotionMapper, EmotionReading, EmotionalQuadrant, PadVector, PulseStrategySuggestion,
};
pub use energy::{EnergyField, EnergyPool};
pub use error::AppError;
pub use events::{EventBus, EventBusSender, UniverseEvent};
pub use hebbian::HebbianMemory;
pub use lattice::{BccVerification, Lattice, NeighborShell, Projection, Tetrahedron};
pub use memory::{MemoryAtom, MemoryCodec, MemoryError};
pub use node::{DarkNode, DarkUniverse};
pub use observer::{
    HealthLevel, HealthReport, RegulatorAction, RegulatorActionType, RegulatorParams,
    SelfRegulator, UniverseObserver,
};
pub use perception::{PerceptionBudget, PerceptionReport};
pub use persist::PersistEngine;
pub use pulse::{PulseEngine, PulseResult, PulseType};
pub use raft_node::{
    new_log_store, new_log_store_with_persistence, new_state_machine, LogStore, LogStoreInner,
    NodeId, Request, Response, StateMachineInner, StateMachineStore, TypeName,
};
pub use reasoning::{ReasoningEngine, ReasoningResult, ReasoningType};
pub use regulation::{RegulationEngine, RegulationReport};
pub use topology::{BettiVector, TopologyEngine, TopologyReport};
pub use watchdog::{Watchdog, WatchdogLevel, WatchdogReport, WatermarkThresholds};

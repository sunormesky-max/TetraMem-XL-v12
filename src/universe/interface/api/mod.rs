// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
pub mod backup_ops;
pub mod cluster_ops;
pub mod cognitive;
pub mod dark_dimension;
pub mod health;
pub mod memory_ops;
pub mod phase;
pub mod physics_ops;
pub mod raft_rpc;
pub mod router;
pub mod scale;
pub mod server;
pub mod state;
pub mod types;

pub use router::create_router;
pub use server::start_server;
pub use state::{AppState, SharedState};
pub use types::ApiResponse;

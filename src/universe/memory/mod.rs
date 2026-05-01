// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
pub mod clustering;
pub mod dream;
pub mod hebbian;
#[allow(clippy::module_inception)]
pub mod memory;
pub mod pulse;

pub use clustering::{
    dark_coords_from_data, semantic_distance, BridgeEdge, BridgeType, ClusteringConfig,
    ClusteringEngine, ClusteringReport, DarkGravityField, ResonanceTunnel, SemanticAnchorPlacer,
    TopologyBridge, TunnelEdge,
};
pub use memory::{MemoryAtom, MemoryCodec, MemoryError};

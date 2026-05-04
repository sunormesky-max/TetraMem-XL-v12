// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
pub mod clustering;
pub mod dream;
pub mod hebbian;
#[allow(clippy::module_inception)]
pub mod memory;
pub mod nlp;
pub mod pulse;
pub mod semantic;

pub use clustering::{
    dark_coords_from_data, semantic_distance, BridgeEdge, BridgeType, ClusteringConfig,
    ClusteringEngine, ClusteringReport, DarkGravityField, ResonanceTunnel, SemanticAnchorPlacer,
    TopologyBridge, TunnelEdge,
};
pub use memory::{MemoryAtom, MemoryCodec, MemoryError};
pub use nlp::{detect_contradictions, synonym_bucket, text_to_anchor, text_to_embedding};
pub use semantic::{
    AtomKey, Concept, DreamSyncReport, EmbeddingIndex, InferenceReport, KnnResult, KnowledgeGraph,
    MultihopResult, QueryFilter, QueryHit, Relation, RelationType, SemanticAnalogy, SemanticConfig,
    SemanticEmbedding, SemanticEngine, SemanticQuery, SemanticReport,
};

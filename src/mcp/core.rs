// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
// TetraMemCore: unified context holding all subsystems in a single coherent structure.

use crate::universe::adaptive::autoscale::AutoScaler;
use crate::universe::adaptive::watchdog::Watchdog;
use crate::universe::cognitive::crystal::CrystalEngine;
use crate::universe::coord::Coord7D;
use crate::universe::core::config::AppConfig;
use crate::universe::dream::DreamEngine;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::clustering::ClusteringEngine;
use crate::universe::memory::nlp;
use crate::universe::memory::semantic::{SemanticConfig, SemanticEngine};
use crate::universe::memory::{MemoryAtom, MemoryCodec};
use crate::universe::node::DarkUniverse;
use crate::universe::pulse::PulseEngine;
use crate::universe::reasoning::ReasoningEngine;
use crate::universe::regulation::RegulationEngine;
use crate::universe::topology::TopologyEngine;

use serde_json::{json, Value};

pub struct ContextEntry {
    pub role: String,
    pub content: String,
    pub token_estimate: usize,
    pub memory_id: Option<String>,
}

pub struct TetraMemCore {
    pub universe: DarkUniverse,
    pub hebbian: HebbianMemory,
    pub memories: Vec<MemoryAtom>,
    pub crystal: CrystalEngine,
    pub semantic: SemanticEngine,
    pub clustering: ClusteringEngine,
    pub context_window: Vec<ContextEntry>,
    pub context_max_tokens: usize,
}

impl TetraMemCore {
    pub fn new(total_energy: f64) -> Self {
        Self {
            universe: DarkUniverse::new(total_energy),
            hebbian: HebbianMemory::new(),
            memories: Vec::new(),
            crystal: CrystalEngine::new(),
            semantic: SemanticEngine::new(SemanticConfig::default()),
            clustering: ClusteringEngine::with_default_config(),
            context_window: Vec::new(),
            context_max_tokens: 4096,
        }
    }

    pub fn from_config(config: &AppConfig) -> Self {
        Self::new(config.universe.total_energy)
    }

    pub fn from_parts(
        universe: DarkUniverse,
        hebbian: HebbianMemory,
        memories: Vec<MemoryAtom>,
        crystal: CrystalEngine,
    ) -> Self {
        let mut core = Self {
            universe,
            hebbian,
            memories,
            crystal,
            semantic: SemanticEngine::new(SemanticConfig::default()),
            clustering: ClusteringEngine::with_default_config(),
            context_window: Vec::new(),
            context_max_tokens: 4096,
        };
        core.rebuild_derived_indexes();
        core
    }

    pub fn rebuild_derived_indexes(&mut self) -> usize {
        self.semantic = SemanticEngine::new(SemanticConfig::default());
        self.clustering = ClusteringEngine::with_default_config();

        let mut rebuilt = 0usize;
        for mem in &self.memories {
            match MemoryCodec::decode(&self.universe, mem) {
                Ok(data) => {
                    self.semantic.index_memory(mem, &data);
                    self.clustering.register_memory(*mem.anchor(), &data);
                    rebuilt += 1;
                }
                Err(e) => {
                    tracing::warn!(
                        anchor = %mem.anchor(),
                        error = %e,
                        "failed to rebuild MCP derived indexes for memory"
                    );
                }
            }
        }
        rebuilt
    }

    // -- Core memory operations --

    pub fn remember(
        &mut self,
        content: &str,
        tags: &[String],
        category: &str,
        importance: f64,
        source: &str,
    ) -> Value {
        let full_embedding = nlp::text_to_embedding(content, importance);
        let data: Vec<f64> = full_embedding.into_iter().take(28).collect();
        let anchor = self.clustering.compute_ideal_anchor(&data, &self.universe);

        let result =
            self.encode_and_link(&anchor, &data, content, tags, category, importance, source);
        match result {
            Ok(val) => val,
            Err(_) => {
                let fallback = nlp::text_to_anchor(content);
                match self.encode_and_link(
                    &fallback, &data, content, tags, category, importance, source,
                ) {
                    Ok(val) => {
                        let mut v = val;
                        v["fallback"] = json!(true);
                        v
                    }
                    Err(e) => json!({"success": false, "error": e}),
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn encode_and_link(
        &mut self,
        anchor: &Coord7D,
        data: &[f64],
        content: &str,
        tags: &[String],
        category: &str,
        importance: f64,
        source: &str,
    ) -> Result<Value, String> {
        let mut mem =
            MemoryCodec::encode(&mut self.universe, anchor, data).map_err(|e| format!("{}", e))?;

        for tag in tags {
            mem.add_tag(tag);
        }
        mem.set_category(category);
        mem.set_description(content);
        mem.set_source(source);
        mem.set_importance(importance);

        self.semantic.index_memory(&mem, data);
        let links = self.link_similar(mem.anchor(), data);
        self.clustering.register_memory(*mem.anchor(), data);

        let contradictions = nlp::detect_contradictions(
            content,
            mem.anchor(),
            &self.memories,
            &self.universe,
            &self.hebbian,
        );

        let memory_id = format!("mem_{}", self.memories.len());
        let anchor_str = format!("{}", mem.anchor());
        let is_decision = category == "decision";

        self.memories.push(mem);

        let mut result = json!({
            "success": true,
            "memory_id": memory_id,
            "anchor": anchor_str,
            "semantic_links": links,
            "conservation_ok": self.universe.verify_conservation(),
        });
        if !contradictions.is_empty() {
            result["contradiction_warnings"] = json!(contradictions);
            result["contradiction_count"] = json!(contradictions.len());
        }
        if is_decision {
            result["decision_logged"] = json!(true);
            result["decision_note"] =
                json!("this decision is tracked and can be recalled for future reference");
        }
        Ok(result)
    }

    fn link_similar(&mut self, anchor: &Coord7D, data: &[f64]) -> usize {
        let knn_results = self.semantic.search_similar(data, 6);
        let mut links = 0usize;
        for knn in &knn_results {
            if knn.atom_key.vertices_basis[0] != anchor.basis() {
                let neighbor_anchor = Coord7D::new_even(knn.atom_key.vertices_basis[0]);
                self.hebbian
                    .boost_edge(anchor, &neighbor_anchor, 0.5 * knn.similarity);
                links += 1;
            }
        }
        links
    }

    pub fn forget(&mut self, anchor: &Coord7D) -> Result<Value, String> {
        let idx = self
            .memories
            .iter()
            .position(|m| m.anchor() == anchor)
            .ok_or_else(|| format!("no memory at {:?}", anchor.basis()))?;

        let desc = self.memories[idx].description().unwrap_or("").to_string();
        MemoryCodec::erase(&mut self.universe, &self.memories[idx]);
        self.memories.remove(idx);

        Ok(json!({
            "success": true,
            "erased_anchor": format!("{}", anchor),
            "description": desc,
            "remaining_memories": self.memories.len(),
            "conservation_ok": self.universe.verify_conservation(),
        }))
    }

    pub fn find_memory(&self, anchor: &Coord7D) -> Option<usize> {
        self.memories.iter().position(|m| m.anchor() == anchor)
    }

    pub fn find_memory_by_basis(&self, basis: &[i32; 7]) -> Option<usize> {
        self.memories
            .iter()
            .position(|m| m.anchor().basis() == *basis)
    }

    // -- Subsystem accessors (stateless engines) --

    pub fn pulse(&self) -> PulseEngine {
        PulseEngine::new()
    }

    pub fn dream(&self) -> DreamEngine {
        DreamEngine::new()
    }

    pub fn topology(&self) -> &TopologyEngine {
        static ENGINE: TopologyEngine = TopologyEngine;
        &ENGINE
    }

    pub fn regulation(&self) -> RegulationEngine {
        RegulationEngine::new()
    }

    pub fn reasoning(&self) -> &ReasoningEngine {
        static ENGINE: ReasoningEngine = ReasoningEngine;
        &ENGINE
    }

    pub fn scaler(&self) -> AutoScaler {
        AutoScaler::new()
    }

    pub fn watchdog(&mut self) -> Watchdog {
        Watchdog::with_defaults(self.universe.stats().total_energy)
    }

    pub fn stats(&self) -> Value {
        let stats = self.universe.stats();
        let drift = self.universe.energy_drift();
        json!({
            "active_nodes": stats.active_nodes,
            "manifested_nodes": stats.manifested_nodes,
            "dark_nodes": stats.dark_nodes,
            "total_energy": stats.total_energy,
            "allocated_energy": stats.allocated_energy,
            "available_energy": stats.available_energy,
            "physical_energy": stats.physical_energy,
            "dark_energy": stats.dark_energy,
            "utilization": stats.utilization,
            "energy_drift": drift,
            "memory_count": self.memories.len(),
            "hebbian_edges": self.hebbian.edge_count(),
            "conservation_ok": self.universe.verify_conservation(),
        })
    }

    pub fn conservation_check(&self) -> Value {
        let ok = self.universe.verify_conservation();
        let drift = self.universe.energy_drift();
        let stats = self.universe.stats();
        json!({
            "conservation_ok": ok,
            "energy_drift": drift,
            "total_energy": stats.total_energy,
            "allocated_energy": stats.allocated_energy,
            "available_energy": stats.available_energy,
            "violation": (stats.total_energy - stats.allocated_energy - stats.available_energy).abs(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_core_has_empty_state() {
        let core = TetraMemCore::new(500.0);
        assert!(core.memories.is_empty());
        assert!(core.context_window.is_empty());
        assert_eq!(core.context_max_tokens, 4096);
        assert_eq!(core.hebbian.edge_count(), 0);
    }

    #[test]
    fn stats_reflects_initial_state() {
        let core = TetraMemCore::new(1000.0);
        let stats = core.stats();
        assert_eq!(stats["memory_count"], 0);
        assert_eq!(stats["conservation_ok"], true);
    }

    #[test]
    fn remember_creates_memory() {
        let mut core = TetraMemCore::new(1_000_000.0);
        let result = core.remember(
            "test content",
            &["tag1".to_string()],
            "test_category",
            0.7,
            "test_source",
        );
        assert_eq!(result["success"], true, "remember failed: {:?}", result);
        assert_eq!(core.memories.len(), 1);
        let mem = &core.memories[0];
        assert_eq!(mem.description(), Some("test content"));
        assert_eq!(mem.category(), Some("test_category"));
        assert!((mem.importance() - 0.7).abs() < 0.01);
    }

    #[test]
    fn remember_with_tags() {
        let mut core = TetraMemCore::new(1_000_000.0);
        let result = core.remember(
            "tagged",
            &["a".to_string(), "b".to_string()],
            "general",
            0.5,
            "agent",
        );
        assert_eq!(result["success"], true, "remember failed: {:?}", result);
        assert_eq!(core.memories[0].tags().len(), 2);
    }

    #[test]
    fn remember_decision_category() {
        let mut core = TetraMemCore::new(1_000_000.0);
        let result = core.remember("a decision", &[], "decision", 0.9, "user");
        assert_eq!(result["success"], true, "remember failed: {:?}", result);
        assert_eq!(result["decision_logged"], true);
    }

    #[test]
    fn forget_removes_memory() {
        let mut core = TetraMemCore::new(1_000_000.0);
        let r = core.remember("to be forgotten", &[], "general", 0.5, "agent");
        assert_eq!(r["success"], true, "remember failed: {:?}", r);
        assert_eq!(core.memories.len(), 1);
        let anchor = *core.memories[0].anchor();
        let result = core.forget(&anchor).unwrap();
        assert_eq!(result["success"], true);
        assert!(core.memories.is_empty());
    }

    #[test]
    fn forget_nonexistent_fails() {
        let mut core = TetraMemCore::new(1000.0);
        let anchor = Coord7D::new_even([99, 99, 99, 0, 0, 0, 0]);
        assert!(core.forget(&anchor).is_err());
    }

    #[test]
    fn find_memory_by_basis() {
        let mut core = TetraMemCore::new(1_000_000.0);
        let r = core.remember("findable", &[], "general", 0.5, "agent");
        assert_eq!(r["success"], true, "remember failed: {:?}", r);
        let basis = core.memories[0].anchor().basis();
        assert!(core.find_memory_by_basis(&basis).is_some());
        assert!(core.find_memory_by_basis(&[99; 7]).is_none());
    }

    #[test]
    fn conservation_check_initial() {
        let core = TetraMemCore::new(1_000_000.0);
        let check = core.conservation_check();
        assert_eq!(check["conservation_ok"], true);
        assert_eq!(check["total_energy"], 1_000_000.0);
    }

    #[test]
    fn multiple_remembers_preserve_conservation() {
        let mut core = TetraMemCore::new(10_000_000.0);
        for i in 0..20 {
            let r = core.remember(&format!("memory number {}", i), &[], "general", 0.5, "test");
            if r["success"] != true {
                break;
            }
        }
        assert!(!core.memories.is_empty());
        assert!(core.universe.verify_conservation());
    }

    #[test]
    fn from_parts_rebuilds_semantic_index() {
        let mut universe = DarkUniverse::new(1_000_000.0);
        let anchor = Coord7D::new_even([8, 0, 0, 0, 0, 0, 0]);
        let data = vec![0.1, 0.2, 0.3];
        let memory = MemoryCodec::encode(&mut universe, &anchor, &data).unwrap();

        let core = TetraMemCore::from_parts(
            universe,
            HebbianMemory::new(),
            vec![memory],
            CrystalEngine::new(),
        );

        let hits = core.semantic.search_similar(&data, 1);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].atom_key.vertices_basis[0], anchor.basis());
    }
}

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

    // -- Core memory operations --

    pub fn remember(
        &mut self,
        content: &str,
        tags: &[String],
        category: &str,
        importance: f64,
        source: &str,
    ) -> Value {
        let data = nlp::text_to_embedding(content, importance);
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

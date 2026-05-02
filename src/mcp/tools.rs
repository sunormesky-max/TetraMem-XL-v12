// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use serde_json::{json, Value};

use super::protocol::{ResourceDefinition, ToolDefinition};
use crate::universe::coord::Coord7D;
use crate::universe::crystal::CrystalEngine;
use crate::universe::dream::DreamEngine;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::clustering::ClusteringEngine;
use crate::universe::memory::{MemoryAtom, MemoryCodec};
use crate::universe::node::DarkUniverse;
use crate::universe::pulse::{PulseEngine, PulseType};
use crate::universe::reasoning::ReasoningEngine;
use crate::universe::regulation::RegulationEngine;
use crate::universe::topology::TopologyEngine;

pub struct ContextEntry {
    pub role: String,
    pub content: String,
    pub token_estimate: usize,
    pub memory_id: Option<String>,
}

pub struct TetraMemTools;

impl TetraMemTools {
    pub fn definitions() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "tetramem_stats".into(),
                description: "Get current universe statistics: node count, energy, utilization, conservation status".into(),
                input_schema: json!({"type": "object", "properties": {}, "required": []}),
            },
            ToolDefinition {
                name: "tetramem_health".into(),
                description: "Check universe health level (Excellent/Good/Warning/Critical) with detailed report".into(),
                input_schema: json!({"type": "object", "properties": {}, "required": []}),
            },
            ToolDefinition {
                name: "tetramem_encode".into(),
                description: "Encode data into a memory at a 3D anchor position. Returns memory ID and verification.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "anchor": {
                            "type": "array",
                            "items": {"type": "integer"},
                            "description": "3D anchor coordinate [x, y, z]",
                            "minItems": 3, "maxItems": 3
                        },
                        "data": {
                            "type": "array",
                            "items": {"type": "number"},
                            "description": "Data values to encode (1-28 dimensions)",
                            "minItems": 1, "maxItems": 28
                        }
                    },
                    "required": ["anchor", "data"]
                }),
            },
            ToolDefinition {
                name: "tetramem_decode".into(),
                description: "Decode a memory by its anchor position, returning the stored data values".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "anchor": {
                            "type": "array",
                            "items": {"type": "integer"},
                            "description": "3D anchor coordinate [x, y, z]",
                            "minItems": 3, "maxItems": 3
                        }
                    },
                    "required": ["anchor"]
                }),
            },
            ToolDefinition {
                name: "tetramem_list_memories".into(),
                description: "List all memories with their anchor positions and metadata".into(),
                input_schema: json!({"type": "object", "properties": {}, "required": []}),
            },
            ToolDefinition {
                name: "tetramem_pulse".into(),
                description: "Fire a pulse (reinforcing/exploratory/inhibitory) from a source position to explore the lattice".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "source": {
                            "type": "array",
                            "items": {"type": "integer"},
                            "description": "3D source coordinate [x, y, z]",
                            "minItems": 3, "maxItems": 3
                        },
                        "pulse_type": {
                            "type": "string",
                            "enum": ["reinforcing", "exploratory", "cascade"],
                            "description": "Type of pulse to fire"
                        }
                    },
                    "required": ["source", "pulse_type"]
                }),
            },
            ToolDefinition {
                name: "tetramem_dream".into(),
                description: "Run a dream cycle: replay (strengthen strong paths), weaken (prune weak edges), consolidate (link related memories)".into(),
                input_schema: json!({"type": "object", "properties": {}, "required": []}),
            },
            ToolDefinition {
                name: "tetramem_topology".into(),
                description: "Compute topology: Betti numbers H0-H6, Euler characteristic, connected components, bridging nodes".into(),
                input_schema: json!({"type": "object", "properties": {}, "required": []}),
            },
            ToolDefinition {
                name: "tetramem_regulate".into(),
                description: "Run dimension regulation cycle: balance energy across 7 dimensions, reduce stress".into(),
                input_schema: json!({"type": "object", "properties": {}, "required": []}),
            },
            ToolDefinition {
                name: "tetramem_trace".into(),
                description: "Trace memory associations via Hebbian and crystal channels from an anchor".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "anchor": {
                            "type": "array",
                            "items": {"type": "integer"},
                            "description": "3D anchor coordinate [x, y, z]",
                            "minItems": 3, "maxItems": 3
                        },
                        "max_hops": {
                            "type": "integer",
                            "description": "Maximum hops to trace (default 10)"
                        }
                    },
                    "required": ["anchor"]
                }),
            },
            ToolDefinition {
                name: "tetramem_phase_detect".into(),
                description: "Detect H6 phase transitions in the crystallized network".into(),
                input_schema: json!({"type": "object", "properties": {}, "required": []}),
            },
            ToolDefinition {
                name: "tetramem_materialize".into(),
                description: "Materialize a node at a 3D position with specified energy and physical/dark ratio".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "coord": {
                            "type": "array",
                            "items": {"type": "integer"},
                            "description": "3D coordinate [x, y, z]",
                            "minItems": 3, "maxItems": 3
                        },
                        "energy": {
                            "type": "number",
                            "description": "Energy amount to allocate"
                        },
                        "physical_ratio": {
                            "type": "number",
                            "description": "Physical energy ratio (0.0-1.0)"
                        }
                    },
                    "required": ["coord", "energy", "physical_ratio"]
                }),
            },
            ToolDefinition {
                name: "tetramem_conservation_check".into(),
                description: "Verify energy conservation law holds across the entire universe".into(),
                input_schema: json!({"type": "object", "properties": {}, "required": []}),
            },
            ToolDefinition {
                name: "tetramem_remember".into(),
                description: "Store a memory for an AI agent. Automatically encodes content, indexes semantically, and links to similar existing memories via Hebbian edges. Solves cross-session memory loss.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "The content to remember (natural language or structured data)"
                        },
                        "tags": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Tags for categorization"
                        },
                        "category": {
                            "type": "string",
                            "description": "Category (e.g. 'user_preference', 'technical', 'conversation')"
                        },
                        "importance": {
                            "type": "number",
                            "description": "Importance weight 0.0-1.0 (default 0.5)"
                        },
                        "source": {
                            "type": "string",
                            "description": "Source identifier (e.g. 'user', 'system', 'agent')"
                        }
                    },
                    "required": ["content"]
                }),
            },
            ToolDefinition {
                name: "tetramem_recall".into(),
                description: "Retrieve memories by natural language query. Uses semantic KNN search + Hebbian association to find relevant memories. Reconstructs context for the agent.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Natural language or keyword query"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Max results to return (default 10)"
                        },
                        "tags": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Filter by tags"
                        },
                        "category": {
                            "type": "string",
                            "description": "Filter by category"
                        },
                        "min_importance": {
                            "type": "number",
                            "description": "Minimum importance threshold (default 0.0)"
                        }
                    },
                    "required": ["query"]
                }),
            },
            ToolDefinition {
                name: "tetramem_associate".into(),
                description: "Discover associated memories for a topic. Uses Hebbian edge traversal and pulse propagation to find related concepts that the agent may not directly recall.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "topic": {
                            "type": "string",
                            "description": "Topic or query to find associations for"
                        },
                        "depth": {
                            "type": "integer",
                            "description": "Association depth / max hops (default 3)"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Max associations to return (default 10)"
                        }
                    },
                    "required": ["topic"]
                }),
            },
            ToolDefinition {
                name: "tetramem_consolidate".into(),
                description: "Run dream consolidation cycle: strengthen important memories, weaken noise, create new associations. Solves the 'cannot distinguish importance' problem.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "importance_threshold": {
                            "type": "number",
                            "description": "Memories below this importance get weakened (default 0.3)"
                        }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "tetramem_context".into(),
                description: "Manage agent context window. Add messages; when context overflows, older messages are automatically encoded to TetraMem memory and can be recalled later. Solves context window overflow.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["add", "status", "reconstruct", "clear"],
                            "description": "Action: 'add' message, 'status' check, 'reconstruct' from memory, 'clear' window"
                        },
                        "role": {
                            "type": "string",
                            "description": "Message role (for 'add': 'user', 'assistant', 'system')"
                        },
                        "content": {
                            "type": "string",
                            "description": "Message content (for 'add' or 'reconstruct' query)"
                        }
                    },
                    "required": ["action"]
                }),
            },
        ]
    }

    pub fn resources() -> Vec<ResourceDefinition> {
        vec![
            ResourceDefinition {
                uri: "tetramem://stats".into(),
                name: "Universe Statistics".into(),
                description: "Live universe statistics snapshot".into(),
                mime_type: Some("application/json".into()),
            },
            ResourceDefinition {
                uri: "tetramem://health".into(),
                name: "Health Report".into(),
                description: "Current universe health assessment".into(),
                mime_type: Some("application/json".into()),
            },
        ]
    }

    #[allow(clippy::too_many_arguments)]
    pub fn handle_tool(
        name: &str,
        args: &Value,
        universe: &mut DarkUniverse,
        hebbian: &mut HebbianMemory,
        memories: &mut Vec<MemoryAtom>,
        crystal: &mut CrystalEngine,
        semantic: &mut crate::universe::memory::semantic::SemanticEngine,
        _clustering: &mut ClusteringEngine,
        context_window: &mut Vec<ContextEntry>,
        context_max_tokens: usize,
    ) -> super::protocol::ToolCallResult {
        match name {
            "tetramem_stats" => {
                let stats = universe.stats();
                let drift = universe.energy_drift();
                super::protocol::ToolCallResult::ok(
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
                        "memory_count": memories.len(),
                        "hebbian_edges": hebbian.edge_count(),
                        "conservation_ok": universe.verify_conservation(),
                    })
                    .to_string(),
                )
            }
            "tetramem_health" => {
                let report = crate::universe::observer::UniverseObserver::inspect(
                    universe, hebbian, memories,
                );
                super::protocol::ToolCallResult::ok(
                    json!({
                        "health_level": report.health_level().as_str(),
                        "node_count": report.node_count,
                        "energy_utilization": report.energy_utilization,
                        "conservation_ok": report.conservation_ok,
                        "hebbian_edge_count": report.hebbian_edge_count,
                        "memory_count": report.memory_count,
                    })
                    .to_string(),
                )
            }
            "tetramem_encode" => {
                let anchor = match args.get("anchor").and_then(|v| v.as_array()) {
                    Some(a) if a.len() == 3 => {
                        let coords: Result<Vec<i32>, _> = a
                            .iter()
                            .map(|v| {
                                v.as_i64()
                                    .filter(|&n| n >= i32::MIN as i64 && n <= i32::MAX as i64)
                                    .map(|n| n as i32)
                                    .ok_or(())
                            })
                            .collect();
                        match coords {
                            Ok(c) => Coord7D::new_even([c[0], c[1], c[2], 0, 0, 0, 0]),
                            Err(_) => {
                                return super::protocol::ToolCallResult::err(
                                    "anchor must be 3 integers",
                                )
                            }
                        }
                    }
                    _ => {
                        return super::protocol::ToolCallResult::err(
                            "anchor must be array of 3 integers",
                        )
                    }
                };
                let data: Vec<f64> = match args.get("data").and_then(|v| v.as_array()) {
                    Some(a) => a.iter().filter_map(|v| v.as_f64()).collect(),
                    None => {
                        return super::protocol::ToolCallResult::err(
                            "data must be array of numbers",
                        )
                    }
                };
                if data.is_empty() || data.len() > 28 {
                    return super::protocol::ToolCallResult::err("data must have 1-28 values");
                }
                match MemoryCodec::encode(universe, &anchor, &data) {
                    Ok(mem) => {
                        let anchor_str = format!("{}", mem.anchor());
                        let dim = mem.data_dim();
                        memories.push(mem);
                        super::protocol::ToolCallResult::ok(
                            json!({
                                "success": true,
                                "anchor": anchor_str,
                                "dimensions": dim,
                                "conservation_ok": universe.verify_conservation(),
                            })
                            .to_string(),
                        )
                    }
                    Err(e) => super::protocol::ToolCallResult::err(format!("encode failed: {}", e)),
                }
            }
            "tetramem_decode" => {
                let anchor = match parse_3d_coord(args, "anchor") {
                    Ok(c) => c,
                    Err(e) => return super::protocol::ToolCallResult::err(e),
                };
                match memories.iter().find(|m| m.anchor() == &anchor) {
                    Some(mem) => match MemoryCodec::decode(universe, mem) {
                        Ok(data) => super::protocol::ToolCallResult::ok(
                            json!({
                                "anchor": format!("{}", mem.anchor()),
                                "data": data,
                                "dimensions": mem.data_dim(),
                            })
                            .to_string(),
                        ),
                        Err(e) => {
                            super::protocol::ToolCallResult::err(format!("decode failed: {}", e))
                        }
                    },
                    None => super::protocol::ToolCallResult::err(format!(
                        "no memory at {:?}",
                        anchor.basis()
                    )),
                }
            }
            "tetramem_list_memories" => {
                let list: Vec<Value> = memories
                    .iter()
                    .map(|m| {
                        json!({
                            "anchor": format!("{}", m.anchor()),
                            "dimensions": m.data_dim(),
                            "created_at": m.created_at(),
                        })
                    })
                    .collect();
                super::protocol::ToolCallResult::ok(
                    json!({
                        "count": list.len(),
                        "memories": list,
                    })
                    .to_string(),
                )
            }
            "tetramem_pulse" => {
                let source = match parse_3d_coord(args, "source") {
                    Ok(c) => c,
                    Err(e) => return super::protocol::ToolCallResult::err(e),
                };
                let pulse_type = match args.get("pulse_type").and_then(|v| v.as_str()) {
                    Some("reinforcing") => PulseType::Reinforcing,
                    Some("exploratory") => PulseType::Exploratory,
                    Some("cascade") => PulseType::Cascade,
                    Some("inhibitory") => PulseType::Cascade,
                    _ => {
                        return super::protocol::ToolCallResult::err(
                            "pulse_type must be reinforcing/exploratory/cascade",
                        )
                    }
                };
                let engine = PulseEngine::new();
                let result = engine.propagate(&source, pulse_type, universe, hebbian);
                super::protocol::ToolCallResult::ok(
                    json!({
                        "visited_nodes": result.visited_nodes,
                        "total_activation": result.total_activation,
                        "paths_recorded": result.paths_recorded,
                        "final_strength": result.final_strength,
                    })
                    .to_string(),
                )
            }
            "tetramem_dream" => {
                let engine = DreamEngine::new();
                let report = engine.dream(universe, hebbian, memories);
                super::protocol::ToolCallResult::ok(
                    json!({
                        "phases": format!("{}", report),
                        "edges_before": report.hebbian_edges_before,
                        "edges_after": report.hebbian_edges_after,
                        "weight_before": report.weight_before,
                        "weight_after": report.weight_after,
                        "conservation_ok": universe.verify_conservation(),
                    })
                    .to_string(),
                )
            }
            "tetramem_topology" => {
                let report = TopologyEngine::analyze(universe);
                super::protocol::ToolCallResult::ok(
                    json!({
                        "betti": format!("{}", report.betti),
                        "connected_components": report.connected_components,
                        "cycles_detected": report.cycles_detected,
                        "tetrahedra_count": report.tetrahedra_count,
                        "bridging_nodes": report.bridging_nodes,
                        "isolated_nodes": report.isolated_nodes,
                        "average_coordination": report.average_coordination,
                        "euler_characteristic": report.betti.euler_characteristic(),
                    })
                    .to_string(),
                )
            }
            "tetramem_regulate" => {
                let engine = RegulationEngine::new();
                let report = engine.regulate(universe, hebbian, crystal, memories);
                super::protocol::ToolCallResult::ok(
                    json!({
                        "stress_level": report.stress_level,
                        "entropy": report.entropy,
                        "imbalance": report.dimension_pressure.imbalance,
                        "actions_taken": report.actions.len(),
                        "conservation_ok": universe.verify_conservation(),
                    })
                    .to_string(),
                )
            }
            "tetramem_trace" => {
                let anchor = match parse_3d_coord(args, "anchor") {
                    Ok(c) => c,
                    Err(e) => return super::protocol::ToolCallResult::err(e),
                };
                let max_hops = args.get("max_hops").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                let associations = ReasoningEngine::find_associations(
                    universe, hebbian, crystal, &anchor, max_hops,
                );
                let results: Vec<Value> = associations
                    .iter()
                    .map(|a| {
                        json!({
                            "source": a.source,
                            "targets": a.targets,
                            "confidence": a.confidence,
                            "hops": a.hops,
                        })
                    })
                    .collect();
                super::protocol::ToolCallResult::ok(
                    json!({
                        "associations": results,
                        "total": results.len(),
                    })
                    .to_string(),
                )
            }
            "tetramem_phase_detect" => {
                let report = crystal.detect_phase_transition(hebbian, universe);
                super::protocol::ToolCallResult::ok(
                    json!({
                        "super_channel_candidates": report.super_channel_candidates,
                        "existing_super_channels": report.existing_super_channels,
                        "avg_edge_weight": report.avg_edge_weight,
                        "phase_coherent": report.phase_coherent,
                        "requires_consensus": report.requires_consensus,
                    })
                    .to_string(),
                )
            }
            "tetramem_materialize" => {
                let coord = match parse_3d_coord(args, "coord") {
                    Ok(c) => c,
                    Err(e) => return super::protocol::ToolCallResult::err(e),
                };
                let energy = match args.get("energy").and_then(|v| v.as_f64()) {
                    Some(e) if e > 0.0 => e,
                    _ => {
                        return super::protocol::ToolCallResult::err(
                            "energy must be a positive number",
                        )
                    }
                };
                let ratio = match args.get("physical_ratio").and_then(|v| v.as_f64()) {
                    Some(r) if (0.0..=1.0).contains(&r) => r,
                    _ => {
                        return super::protocol::ToolCallResult::err(
                            "physical_ratio must be between 0.0 and 1.0",
                        )
                    }
                };
                match universe.materialize_biased(coord, energy, ratio) {
                    Ok(_) => super::protocol::ToolCallResult::ok(
                        json!({
                            "success": true,
                            "coord": format!("{}", coord),
                            "energy": energy,
                            "conservation_ok": universe.verify_conservation(),
                        })
                        .to_string(),
                    ),
                    Err(e) => {
                        super::protocol::ToolCallResult::err(format!("materialize failed: {}", e))
                    }
                }
            }
            "tetramem_conservation_check" => {
                let ok = universe.verify_conservation();
                let drift = universe.energy_drift();
                let stats = universe.stats();
                super::protocol::ToolCallResult::ok(json!({
                    "conservation_ok": ok,
                    "energy_drift": drift,
                    "total_energy": stats.total_energy,
                    "allocated_energy": stats.allocated_energy,
                    "available_energy": stats.available_energy,
                    "violation": (stats.total_energy - stats.allocated_energy - stats.available_energy).abs(),
                }).to_string())
            }
            "tetramem_remember" => {
                let content = match args.get("content").and_then(|v| v.as_str()) {
                    Some(c) => c.to_string(),
                    None => return super::protocol::ToolCallResult::err("content is required"),
                };
                let tags: Vec<String> = args
                    .get("tags")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                let category = args
                    .get("category")
                    .and_then(|v| v.as_str())
                    .unwrap_or("general")
                    .to_string();
                let importance = args
                    .get("importance")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.5);
                let source = args
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("agent")
                    .to_string();

                let data = text_to_embedding(&content, importance);
                let anchor = text_to_anchor(&content);

                match MemoryCodec::encode(universe, &anchor, &data) {
                    Ok(mut mem) => {
                        let anchor_str = format!("{}", mem.anchor());

                        for tag in &tags {
                            mem.add_tag(tag);
                        }
                        mem.set_category(&category);
                        mem.set_description(&content);
                        mem.set_source(&source);
                        mem.set_importance(importance);

                        semantic.index_memory(&mem, &data);
                        let knn_results = semantic.search_similar(&data, 6);
                        let mut links = 0usize;
                        for knn in &knn_results {
                            if knn.atom_key.vertices_basis[0] != mem.anchor().basis() {
                                let neighbor_anchor =
                                    Coord7D::new_even(knn.atom_key.vertices_basis[0]);
                                hebbian.boost_edge(
                                    mem.anchor(),
                                    &neighbor_anchor,
                                    0.5 * knn.similarity,
                                );
                                links += 1;
                            }
                        }

                        let memory_id = format!("mem_{}", memories.len());
                        memories.push(mem);
                        super::protocol::ToolCallResult::ok(
                            json!({
                                "success": true,
                                "memory_id": memory_id,
                                "anchor": anchor_str,
                                "semantic_links": links,
                                "conservation_ok": universe.verify_conservation(),
                            })
                            .to_string(),
                        )
                    }
                    Err(e) => {
                        super::protocol::ToolCallResult::err(format!("remember failed: {}", e))
                    }
                }
            }
            "tetramem_recall" => {
                let query = match args.get("query").and_then(|v| v.as_str()) {
                    Some(q) => q.to_string(),
                    None => return super::protocol::ToolCallResult::err("query is required"),
                };
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

                let query_data = text_to_embedding(&query, 0.5);
                let knn_results = semantic.search_similar(&query_data, limit * 2);

                let mut hits = Vec::new();
                for knn in &knn_results {
                    if hits.len() >= limit {
                        break;
                    }
                    let anchor_basis = knn.atom_key.vertices_basis[0];
                    if let Some(mem) = memories.iter().find(|m| m.anchor().basis() == anchor_basis)
                    {
                        let anchor_coord = *mem.anchor();
                        let nb = hebbian.get_neighbors(&anchor_coord);
                        let associated: Vec<String> = nb
                            .iter()
                            .filter_map(|(coord, _)| {
                                memories
                                    .iter()
                                    .find(|m| m.anchor() == coord)
                                    .map(|m| format!("{}", m.anchor()))
                            })
                            .take(5)
                            .collect();
                        hits.push(json!({
                            "anchor": format!("{}", mem.anchor()),
                            "similarity": knn.similarity,
                            "dimensions": mem.data_dim(),
                            "hebbian_neighbors": nb.len(),
                            "associated_memories": associated,
                            "description": mem.description().unwrap_or(""),
                            "tags": mem.tags(),
                            "category": mem.category().unwrap_or(""),
                            "importance": mem.importance(),
                        }));
                    }
                }

                super::protocol::ToolCallResult::ok(
                    json!({
                        "query": query,
                        "results": hits,
                        "total_found": knn_results.len(),
                        "returned": hits.len(),
                    })
                    .to_string(),
                )
            }
            "tetramem_associate" => {
                let topic = match args.get("topic").and_then(|v| v.as_str()) {
                    Some(t) => t.to_string(),
                    None => return super::protocol::ToolCallResult::err("topic is required"),
                };
                let depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

                let topic_data = text_to_embedding(&topic, 0.5);
                let knn = semantic.search_similar(&topic_data, 1);
                let seed_anchor = match knn.first() {
                    Some(k) => Coord7D::new_even(k.atom_key.vertices_basis[0]),
                    None => {
                        return super::protocol::ToolCallResult::ok(
                            json!({"topic": topic, "associations": [], "message": "no matching memories found"})
                                .to_string(),
                        )
                    }
                };

                let associations = ReasoningEngine::find_associations(
                    universe,
                    hebbian,
                    crystal,
                    &seed_anchor,
                    depth,
                );

                let mut results = Vec::new();
                for assoc in associations.iter().take(limit) {
                    let targets: Vec<String> = assoc
                        .targets
                        .iter()
                        .take(5)
                        .map(|t| t.to_string())
                        .collect();
                    results.push(json!({
                        "source": assoc.source,
                        "targets": targets,
                        "confidence": assoc.confidence,
                        "hops": assoc.hops,
                    }));
                }

                let engine = PulseEngine::new();
                let pulse_result =
                    engine.propagate(&seed_anchor, PulseType::Exploratory, universe, hebbian);

                super::protocol::ToolCallResult::ok(
                    json!({
                        "topic": topic,
                        "seed_anchor": format!("{}", seed_anchor),
                        "associations": results,
                        "pulse_spread": {
                            "visited_nodes": pulse_result.visited_nodes,
                            "activation": pulse_result.total_activation,
                        },
                        "total": results.len(),
                    })
                    .to_string(),
                )
            }
            "tetramem_consolidate" => {
                let importance_threshold = args
                    .get("importance_threshold")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.3);

                let engine = DreamEngine::new();
                let report = engine.dream(universe, hebbian, memories);

                let mut weakened = 0usize;
                let mut strengthened = 0usize;
                for mem in memories.iter() {
                    let neighbors = hebbian.get_neighbors(mem.anchor());
                    for (_, weight) in &neighbors {
                        if *weight < importance_threshold {
                            weakened += 1;
                        } else {
                            strengthened += 1;
                        }
                    }
                }

                semantic.auto_link_similar(memories);

                super::protocol::ToolCallResult::ok(
                    json!({
                        "consolidation": format!("{}", report),
                        "edges_before": report.hebbian_edges_before,
                        "edges_after": report.hebbian_edges_after,
                        "strengthened_paths": strengthened,
                        "weakened_paths": weakened,
                        "conservation_ok": universe.verify_conservation(),
                    })
                    .to_string(),
                )
            }
            "tetramem_context" => {
                let action = match args.get("action").and_then(|v| v.as_str()) {
                    Some(a) => a.to_string(),
                    None => return super::protocol::ToolCallResult::err("action is required"),
                };

                match action.as_str() {
                    "add" => {
                        let role = args
                            .get("role")
                            .and_then(|v| v.as_str())
                            .unwrap_or("user")
                            .to_string();
                        let content = args
                            .get("content")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let token_estimate = content.split_whitespace().count() * 2;

                        let memory_id = if context_window
                            .iter()
                            .map(|e| e.token_estimate)
                            .sum::<usize>()
                            + token_estimate
                            > context_max_tokens
                        {
                            let overflow_entries: Vec<ContextEntry> =
                                context_window.drain(..context_window.len() / 2).collect();
                            let mut archived_ids = Vec::new();
                            for entry in overflow_entries {
                                let data = text_to_embedding(&entry.content, 0.3);

                                let anchor = text_to_anchor(&entry.content);

                                if let Ok(mem) = MemoryCodec::encode(universe, &anchor, &data) {
                                    semantic.index_memory(&mem, &data);
                                    let knn = semantic.search_similar(&data, 3);
                                    for k in &knn {
                                        let k_anchor =
                                            Coord7D::new_even(k.atom_key.vertices_basis[0]);
                                        if k_anchor != *mem.anchor() {
                                            hebbian.boost_edge(
                                                mem.anchor(),
                                                &k_anchor,
                                                0.3 * k.similarity,
                                            );
                                        }
                                    }
                                    archived_ids.push(format!("{}", mem.anchor()));
                                    memories.push(mem);
                                }
                            }
                            Some(archived_ids)
                        } else {
                            None
                        };

                        context_window.push(ContextEntry {
                            role: role.clone(),
                            content: content.clone(),
                            token_estimate,
                            memory_id: None,
                        });

                        let current_tokens: usize =
                            context_window.iter().map(|e| e.token_estimate).sum();
                        super::protocol::ToolCallResult::ok(
                            json!({
                                "action": "add",
                                "context_entries": context_window.len(),
                                "current_tokens": current_tokens,
                                "max_tokens": context_max_tokens,
                                "overflow_archived": memory_id.as_ref().map(|ids| ids.len()).unwrap_or(0),
                            })
                            .to_string(),
                        )
                    }
                    "status" => {
                        let current_tokens: usize =
                            context_window.iter().map(|e| e.token_estimate).sum();
                        let entries: Vec<Value> = context_window
                            .iter()
                            .map(|e| json!({"role": e.role, "tokens": e.token_estimate}))
                            .collect();
                        super::protocol::ToolCallResult::ok(json!({
                            "entries": entries,
                            "total_tokens": current_tokens,
                            "max_tokens": context_max_tokens,
                            "utilization": if context_max_tokens > 0 { current_tokens as f64 / context_max_tokens as f64 } else { 0.0 },
                            "total_memories": memories.len(),
                        }).to_string())
                    }
                    "reconstruct" => {
                        let query = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
                        let query_data = text_to_embedding(query, 0.5);
                        let knn = semantic.search_similar(&query_data, 5);
                        let mut reconstructed = Vec::new();
                        for k in &knn {
                            let anchor_basis = k.atom_key.vertices_basis[0];
                            if let Some(mem) =
                                memories.iter().find(|m| m.anchor().basis() == anchor_basis)
                            {
                                reconstructed.push(json!({
                                    "anchor": format!("{}", mem.anchor()),
                                    "similarity": k.similarity,
                                    "description": mem.description().unwrap_or(""),
                                }));
                            }
                        }

                        super::protocol::ToolCallResult::ok(
                            json!({
                                "reconstructed_context": reconstructed,
                                "current_window_entries": context_window.len(),
                            })
                            .to_string(),
                        )
                    }
                    "clear" => {
                        let count = context_window.len();
                        context_window.clear();
                        super::protocol::ToolCallResult::ok(
                            json!({
                                "action": "clear",
                                "cleared_entries": count,
                                "memories_preserved": memories.len(),
                            })
                            .to_string(),
                        )
                    }
                    _ => super::protocol::ToolCallResult::err(format!(
                        "unknown context action: {}. Use add/status/reconstruct/clear",
                        action
                    )),
                }
            }
            _ => super::protocol::ToolCallResult::err(format!("unknown tool: {}", name)),
        }
    }

    pub fn read_resource(
        uri: &str,
        universe: &DarkUniverse,
        hebbian: &HebbianMemory,
        memories: &[MemoryAtom],
    ) -> Option<super::protocol::ResourceContent> {
        match uri {
            "tetramem://stats" => {
                let stats = universe.stats();
                Some(super::protocol::ResourceContent {
                    uri: uri.into(),
                    mime_type: Some("application/json".into()),
                    text: json!({
                        "active_nodes": stats.active_nodes,
                        "total_energy": stats.total_energy,
                        "utilization": stats.utilization,
                    })
                    .to_string(),
                })
            }
            "tetramem://health" => {
                let report = crate::universe::observer::UniverseObserver::inspect(
                    universe, hebbian, memories,
                );
                Some(super::protocol::ResourceContent {
                    uri: uri.into(),
                    mime_type: Some("application/json".into()),
                    text: json!({
                        "health_level": report.health_level().as_str(),
                        "conservation_ok": report.conservation_ok,
                    })
                    .to_string(),
                })
            }
            _ => None,
        }
    }
}

fn text_to_embedding(text: &str, importance: f64) -> Vec<f64> {
    let dim = 28usize;
    let mut vec = vec![0.0f64; dim];

    let lower = text.to_lowercase();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();

    for word in &words {
        let mut h: u64 = 0;
        for b in word.as_bytes() {
            h = h.wrapping_mul(31).wrapping_add(*b as u64);
        }
        let slot1 = (h as usize) % dim;
        let slot2 = (h.wrapping_mul(37) as usize) % dim;
        let slot3 = (h.wrapping_mul(53) as usize) % dim;
        vec[slot1] += 1.0;
        vec[slot2] += 0.7;
        vec[slot3] += 0.3;
    }

    for i in 0..lower.len().saturating_sub(2) {
        let trigram = &lower[i..i + 3];
        let mut h: u64 = 0;
        for b in trigram.as_bytes() {
            h = h.wrapping_mul(37).wrapping_add(*b as u64);
        }
        let slot = (h as usize) % dim;
        vec[slot] += 0.5;
    }

    for i in 0..lower.len().saturating_sub(1) {
        let bigram = &lower[i..i + 2];
        let mut h: u64 = 0;
        for b in bigram.as_bytes() {
            h = h.wrapping_mul(31).wrapping_add(*b as u64);
        }
        let slot = (h as usize) % dim;
        vec[slot] += 0.3;
    }

    vec[0] = words.len() as f64 * 0.1;
    vec[1] = lower.len() as f64 * 0.01;
    vec[2] = importance;

    let norm: f64 = vec.iter().map(|v| v * v).sum::<f64>().sqrt();
    if norm > 1e-10 {
        for v in &mut vec {
            *v /= norm;
        }
    }

    vec
}

fn text_to_anchor(text: &str) -> Coord7D {
    let lower = text.to_lowercase();
    let mut h1: u64 = 5381;
    let mut h2: u64 = 5271;
    let mut h3: u64 = 65537;
    for b in lower.as_bytes() {
        h1 = h1.wrapping_mul(33).wrapping_add(*b as u64);
        h2 = h2.wrapping_mul(37).wrapping_add(*b as u64);
        h3 = h3.wrapping_mul(41).wrapping_add(*b as u64);
    }
    let ax = (h1 as i32).abs() % 10000;
    let ay = (h2 as i32).abs() % 10000;
    let az = (h3 as i32).abs() % 10000;
    Coord7D::new_even([ax, ay, az, 0, 0, 0, 0])
}

fn parse_3d_coord(args: &Value, key: &str) -> Result<Coord7D, String> {
    match args.get(key).and_then(|v| v.as_array()) {
        Some(a) if a.len() == 3 => {
            let coords: Result<Vec<i32>, _> = a
                .iter()
                .map(|v| {
                    v.as_i64()
                        .filter(|&n| n >= i32::MIN as i64 && n <= i32::MAX as i64)
                        .map(|n| n as i32)
                        .ok_or(())
                })
                .collect();
            match coords {
                Ok(c) => Ok(Coord7D::new_even([c[0], c[1], c[2], 0, 0, 0, 0])),
                Err(_) => Err(format!("{} must be 3 integers", key)),
            }
        }
        _ => Err(format!("{} must be array of 3 integers", key)),
    }
}

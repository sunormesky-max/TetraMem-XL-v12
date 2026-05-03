// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use serde_json::{json, Value};

use super::protocol::{ResourceDefinition, ToolDefinition};
use crate::universe::adaptive::autoscale::AutoScaler;
use crate::universe::adaptive::watchdog::Watchdog;
use crate::universe::cognitive::emotion::EmotionMapper;
use crate::universe::cognitive::functional_emotion::FunctionalEmotion;
use crate::universe::cognitive::perception::PerceptionBudget;
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
                description: "Store a memory for an AI agent. Automatically encodes content, indexes semantically, links to similar memories, and detects contradictions with existing beliefs. Use category='decision' for decision logging. Returns contradiction warnings if conflicting memories found.".into(),
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
                description: "Run dream consolidation cycle: strengthen important memories, weaken noise, detect emergent knowledge clusters, and track self-model evolution via dark dimension analysis. Returns fermentation report with emergence insights.".into(),
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
                description: "Manage agent context window. Add messages; when context overflows, older messages are automatically encoded to TetraMem memory and can be recalled later. Use 'pre_work' to auto-activate relevant memories before starting a task.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["add", "status", "reconstruct", "clear", "pre_work"],
                            "description": "Action: 'add' message, 'status' check, 'reconstruct' from memory, 'clear' window, 'pre_work' activate relevant memories for upcoming task"
                        },
                        "role": {
                            "type": "string",
                            "description": "Message role (for 'add': 'user', 'assistant', 'system')"
                        },
                        "content": {
                            "type": "string",
                            "description": "Message content (for 'add' or 'reconstruct' or 'pre_work' query)"
                        }
                    },
                    "required": ["action"]
                }),
            },
            ToolDefinition {
                name: "tetramem_reason".into(),
                description: "Advanced reasoning: find analogies between memories, infer chains between two anchors, or discover new knowledge via exploratory pulse. Returns confidence scores and hop counts.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "method": {
                            "type": "string",
                            "enum": ["analogies", "infer_chain", "discover"],
                            "description": "Reasoning method: 'analogies' finds similar memories, 'infer_chain' finds path between two anchors, 'discover' explores via pulse"
                        },
                        "threshold": {
                            "type": "number",
                            "description": "Similarity threshold for analogies (default 0.7)"
                        },
                        "from_anchor": {
                            "type": "array",
                            "items": {"type": "integer"},
                            "description": "Start anchor for infer_chain [x, y, z]",
                            "minItems": 3, "maxItems": 3
                        },
                        "to_anchor": {
                            "type": "array",
                            "items": {"type": "integer"},
                            "description": "End anchor for infer_chain [x, y, z]",
                            "minItems": 3, "maxItems": 3
                        },
                        "seed_anchor": {
                            "type": "array",
                            "items": {"type": "integer"},
                            "description": "Seed anchor for discover [x, y, z]",
                            "minItems": 3, "maxItems": 3
                        },
                        "max_hops": {
                            "type": "integer",
                            "description": "Maximum hops for infer_chain (default 15)"
                        }
                    },
                    "required": ["method"]
                }),
            },
            ToolDefinition {
                name: "tetramem_emotion".into(),
                description: "Read the emotional state of the universe from dark energy dimensions. Returns PAD vector (Pleasure-Arousal-Dominance), emotional quadrant, functional emotion cluster, and pulse strategy recommendation.".into(),
                input_schema: json!({"type": "object", "properties": {}, "required": []}),
            },
            ToolDefinition {
                name: "tetramem_scale".into(),
                description: "Auto-scale the universe: expand energy when utilization is high, shrink when low, or grow the lattice frontier. Ensures energy conservation throughout.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["auto", "frontier"],
                            "description": "'auto' evaluates and scales based on utilization, 'frontier' expands lattice boundary"
                        },
                        "max_new_nodes": {
                            "type": "integer",
                            "description": "Max nodes to add for frontier expansion (default 200)"
                        }
                    },
                    "required": ["action"]
                }),
            },
            ToolDefinition {
                name: "tetramem_watchdog".into(),
                description: "Run a watchdog checkup: comprehensive health monitoring with watermark thresholds, conservation tracking, and auto-backup triggering. Returns multi-level alert status.".into(),
                input_schema: json!({"type": "object", "properties": {}, "required": []}),
            },
            ToolDefinition {
                name: "tetramem_forget".into(),
                description: "Erase a memory by its anchor position, freeing its energy back to the universe pool. Requires exact anchor coordinates.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "anchor": {
                            "type": "array",
                            "items": {"type": "integer"},
                            "description": "3D anchor coordinate of the memory to erase [x, y, z]",
                            "minItems": 3, "maxItems": 3
                        }
                    },
                    "required": ["anchor"]
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
        clustering: &mut ClusteringEngine,
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

                let anchor = clustering.compute_ideal_anchor(&data, universe);

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

                        clustering.register_memory(*mem.anchor(), &data);

                        let memory_id = format!("mem_{}", memories.len());

                        let is_decision = category == "decision";

                        let contradictions = detect_contradictions(
                            &content,
                            mem.anchor(),
                            memories,
                            universe,
                            hebbian,
                        );

                        memories.push(mem);
                        let mut result = json!({
                            "success": true,
                            "memory_id": memory_id,
                            "anchor": anchor_str,
                            "semantic_links": links,
                            "conservation_ok": universe.verify_conservation(),
                        });
                        if !contradictions.is_empty() {
                            result["contradiction_warnings"] = json!(contradictions);
                            result["contradiction_count"] = json!(contradictions.len());
                        }
                        if is_decision {
                            result["decision_logged"] = json!(true);
                            result["decision_note"] = json!(
                                "this decision is tracked and can be recalled for future reference"
                            );
                        }
                        super::protocol::ToolCallResult::ok(result.to_string())
                    }
                    Err(e) => {
                        let fallback = text_to_anchor(&content);
                        match MemoryCodec::encode(universe, &fallback, &data) {
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
                                clustering.register_memory(*mem.anchor(), &data);
                                let memory_id = format!("mem_{}", memories.len());
                                memories.push(mem);
                                super::protocol::ToolCallResult::ok(
                                    json!({
                                        "success": true,
                                        "memory_id": memory_id,
                                        "anchor": anchor_str,
                                        "semantic_links": links,
                                        "conservation_ok": universe.verify_conservation(),
                                        "fallback": true,
                                    })
                                    .to_string(),
                                )
                            }
                            Err(e2) => super::protocol::ToolCallResult::err(format!(
                                "remember failed: {} (fallback: {})",
                                e, e2
                            )),
                        }
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

                let ideal_anchor = clustering.compute_ideal_anchor(&query_data, universe);
                let ideal_phys = ideal_anchor.physical();
                let mut spatial_hits: Vec<(usize, f64)> = Vec::new();
                for (i, mem) in memories.iter().enumerate() {
                    let mem_phys = mem.anchor().physical();
                    let dx = (ideal_phys[0] - mem_phys[0]).abs();
                    let dy = (ideal_phys[1] - mem_phys[1]).abs();
                    let dz = (ideal_phys[2] - mem_phys[2]).abs();
                    if dx + dy + dz < 100 {
                        let dist_sq = dx * dx + dy * dy + dz * dz;
                        let score = 1.0 / (1.0 + (dist_sq as f64).sqrt());
                        spatial_hits.push((i, score));
                    }
                }
                spatial_hits
                    .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                let mut seen_anchors = std::collections::HashSet::new();
                let mut hits = Vec::new();

                for &(idx, spatial_score) in &spatial_hits {
                    if hits.len() >= limit {
                        break;
                    }
                    let mem = &memories[idx];
                    let anchor_key = mem.anchor().basis();
                    if seen_anchors.contains(&anchor_key) {
                        continue;
                    }
                    seen_anchors.insert(anchor_key);

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
                        "similarity": spatial_score,
                        "method": "spatial",
                        "dimensions": mem.data_dim(),
                        "hebbian_neighbors": nb.len(),
                        "associated_memories": associated,
                        "description": mem.description().unwrap_or(""),
                        "tags": mem.tags(),
                        "category": mem.category().unwrap_or(""),
                        "importance": mem.importance(),
                    }));
                }

                if hits.len() < limit {
                    let knn_results = semantic.search_similar(&query_data, limit * 2);
                    for knn in &knn_results {
                        if hits.len() >= limit {
                            break;
                        }
                        let anchor_basis = knn.atom_key.vertices_basis[0];
                        if seen_anchors.contains(&anchor_basis) {
                            continue;
                        }
                        seen_anchors.insert(anchor_basis);
                        if let Some(mem) =
                            memories.iter().find(|m| m.anchor().basis() == anchor_basis)
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
                                "method": "knn",
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
                }

                super::protocol::ToolCallResult::ok(
                    json!({
                        "query": query,
                        "results": hits,
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
                let ideal_anchor = clustering.compute_ideal_anchor(&topic_data, universe);

                let seed_anchor = memories
                    .iter()
                    .min_by(|a, b| {
                        let da = a.anchor().distance_sq(&ideal_anchor);
                        let db = b.anchor().distance_sq(&ideal_anchor);
                        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|m| *m.anchor())
                    .unwrap_or_else(|| {
                        let knn = semantic.search_similar(&topic_data, 1);
                        match knn.first() {
                            Some(k) => Coord7D::new_even(k.atom_key.vertices_basis[0]),
                            None => Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]),
                        }
                    });

                let associations = ReasoningEngine::find_associations(
                    universe,
                    hebbian,
                    crystal,
                    &seed_anchor,
                    depth,
                );

                let mut results = Vec::new();
                for assoc in associations.iter().take(limit) {
                    let targets: Vec<serde_json::Value> = assoc
                        .targets
                        .iter()
                        .take(5)
                        .map(|t| {
                            let desc = memories
                                .iter()
                                .find(|m| format!("{}", m.anchor()) == *t)
                                .and_then(|m| m.description().map(String::from))
                                .unwrap_or_default();
                            json!({"anchor": t, "description": desc})
                        })
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

                let mut cluster_neighbors = Vec::new();
                let seed_phys = seed_anchor.physical();
                for mem in memories.iter() {
                    let mp = mem.anchor().physical();
                    let d = (seed_phys[0] - mp[0]).abs()
                        + (seed_phys[1] - mp[1]).abs()
                        + (seed_phys[2] - mp[2]).abs();
                    if d > 0 && d < 50 {
                        cluster_neighbors.push(json!({
                            "anchor": format!("{}", mem.anchor()),
                            "distance": d,
                            "description": mem.description().unwrap_or(""),
                        }));
                    }
                }

                super::protocol::ToolCallResult::ok(
                    json!({
                        "topic": topic,
                        "seed_anchor": format!("{}", seed_anchor),
                        "associations": results,
                        "cluster_neighbors": cluster_neighbors,
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

                let cluster_report = clustering.run_maintenance_cycle(memories, hebbian, universe);

                let mut weakened = 0usize;
                let mut strengthened = 0usize;
                let mut intra_cluster = 0usize;
                let mut inter_cluster = 0usize;

                let mem_anchors: Vec<Coord7D> = memories.iter().map(|m| *m.anchor()).collect();
                for mem in memories.iter() {
                    let neighbors = hebbian.get_neighbors(mem.anchor());
                    for (coord, weight) in &neighbors {
                        if *weight < importance_threshold {
                            weakened += 1;
                        } else {
                            strengthened += 1;
                        }
                        let is_near = mem_anchors.iter().any(|a| a.distance_sq(coord) < 2500.0);
                        if is_near {
                            intra_cluster += 1;
                        } else {
                            inter_cluster += 1;
                        }
                    }
                }

                if intra_cluster > 0 && inter_cluster > 0 {
                    let ratio = intra_cluster as f64 / inter_cluster as f64;
                    if ratio < 1.0 {
                        for mem in memories.iter() {
                            let mem_phys = mem.anchor().physical();
                            for other in memories.iter() {
                                if mem.anchor() == other.anchor() {
                                    continue;
                                }
                                let other_phys = other.anchor().physical();
                                let d = (mem_phys[0] - other_phys[0]).abs()
                                    + (mem_phys[1] - other_phys[1]).abs()
                                    + (mem_phys[2] - other_phys[2]).abs();
                                if d < 30 {
                                    hebbian.boost_edge(mem.anchor(), other.anchor(), 0.1);
                                }
                            }
                        }
                    }
                }

                semantic.auto_link_similar(memories);

                let mut clusters: Vec<serde_json::Value> = Vec::new();
                let mut visited = std::collections::HashSet::new();
                for (i, _mem_a) in memories.iter().enumerate() {
                    if visited.contains(&i) {
                        continue;
                    }
                    let mut cluster_members = Vec::new();
                    let mut stack = vec![i];
                    while let Some(ci) = stack.pop() {
                        if visited.contains(&ci) {
                            continue;
                        }
                        visited.insert(ci);
                        let cm = &memories[ci];
                        cluster_members.push(json!({
                            "anchor": format!("{}", cm.anchor()),
                            "description": cm.description().unwrap_or(""),
                            "category": cm.category().unwrap_or(""),
                            "importance": cm.importance(),
                        }));
                        for (cj, mem_b) in memories.iter().enumerate() {
                            if visited.contains(&cj) {
                                continue;
                            }
                            let d = cm.anchor().distance_sq(mem_b.anchor());
                            if d < 2500.0 {
                                stack.push(cj);
                            }
                        }
                    }
                    if cluster_members.len() >= 2 {
                        let cats: Vec<&str> = cluster_members
                            .iter()
                            .filter_map(|m| m.get("category").and_then(|c| c.as_str()))
                            .filter(|c| !c.is_empty())
                            .collect();
                        let mut cat_set = std::collections::HashSet::new();
                        for c in &cats {
                            cat_set.insert(*c);
                        }
                        clusters.push(json!({
                            "size": cluster_members.len(),
                            "categories": cat_set.into_iter().collect::<Vec<&str>>(),
                            "members": cluster_members,
                        }));
                    }
                }

                let mut emergent_links: Vec<serde_json::Value> = Vec::new();
                for mem_a in memories.iter() {
                    let nb = hebbian.get_neighbors(mem_a.anchor());
                    for (coord, weight) in &nb {
                        if *weight > 0.8 {
                            if let Some(mem_b) = memories.iter().find(|m| m.anchor() == coord) {
                                let cat_a = mem_a.category().unwrap_or("");
                                let cat_b = mem_b.category().unwrap_or("");
                                if !cat_a.is_empty() && !cat_b.is_empty() && cat_a != cat_b {
                                    emergent_links.push(json!({
                                        "from": mem_a.description().unwrap_or(""),
                                        "to": mem_b.description().unwrap_or(""),
                                        "categories": [cat_a, cat_b],
                                        "strength": weight,
                                    }));
                                }
                            }
                        }
                    }
                }
                emergent_links.sort_by(|a, b| {
                    b.get("strength")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0)
                        .partial_cmp(&a.get("strength").and_then(|v| v.as_f64()).unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                emergent_links.truncate(5);

                let mut dark_dims = [0.0f64; 4];
                let mut phys_center = [0.0f64; 3];
                for mem in memories.iter() {
                    let p = mem.anchor().physical();
                    let d = mem.anchor().dark();
                    phys_center[0] += p[0] as f64;
                    phys_center[1] += p[1] as f64;
                    phys_center[2] += p[2] as f64;
                    dark_dims[0] += d[0] as f64;
                    dark_dims[1] += d[1] as f64;
                    dark_dims[2] += d[2] as f64;
                    dark_dims[3] += d[3] as f64;
                }
                let n = memories.len().max(1) as f64;
                for v in phys_center.iter_mut() {
                    *v /= n;
                }
                for v in dark_dims.iter_mut() {
                    *v /= n;
                }

                let mut knowledge_spread = 0.0f64;
                for mem in memories.iter() {
                    let p = mem.anchor().physical();
                    knowledge_spread += (p[0] as f64 - phys_center[0]).powi(2)
                        + (p[1] as f64 - phys_center[1]).powi(2)
                        + (p[2] as f64 - phys_center[2]).powi(2);
                }
                knowledge_spread = (knowledge_spread / n).sqrt();

                let mut belief_stability = 0.0f64;
                if memories.len() > 1 {
                    let mut times: Vec<u64> = memories.iter().map(|m| m.created_at()).collect();
                    times.sort();
                    if let (Some(first), Some(last)) = (times.first(), times.last()) {
                        let span = (*last - *first).max(1);
                        belief_stability = (memories.len() as f64).ln()
                            / (span as f64 / 86400000.0).ln().abs().max(1.0);
                    }
                }

                let uncertainty = if knowledge_spread > 0.0 {
                    1.0 / (1.0 + knowledge_spread * 0.01)
                } else {
                    1.0
                };

                super::protocol::ToolCallResult::ok(
                    json!({
                        "consolidation": format!("{}", report),
                        "edges_before": report.hebbian_edges_before,
                        "edges_after": report.hebbian_edges_after,
                        "strengthened_paths": strengthened,
                        "weakened_paths": weakened,
                        "intra_cluster_edges": intra_cluster,
                        "inter_cluster_edges": inter_cluster,
                        "conservation_ok": universe.verify_conservation(),
                        "fermentation_report": {
                            "clusters_found": clusters.len(),
                            "clusters": clusters,
                            "emergent_cross_category_links": emergent_links,
                            "cluster_maintenance": {
                                "attractors": cluster_report.attractors,
                                "tunnels_applied": cluster_report.tunnels_applied,
                                "bridges_bridged": cluster_report.bridges_created,
                            },
                        },
                        "self_model": {
                            "attention_density": dark_dims[0],
                            "knowledge_breadth": dark_dims[1],
                            "belief_stability": belief_stability,
                            "uncertainty": uncertainty,
                            "knowledge_spread": knowledge_spread,
                            "total_beliefs": memories.len(),
                            "physical_center": phys_center,
                        },
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
                    "pre_work" => {
                        let query = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
                        let query_data = text_to_embedding(query, 0.6);

                        let ideal_anchor = clustering.compute_ideal_anchor(&query_data, universe);
                        let ideal_phys = ideal_anchor.physical();

                        let mut recent: Vec<serde_json::Value> = Vec::new();
                        let mut all_anchors = Vec::new();
                        for mem in memories.iter() {
                            all_anchors.push((
                                mem.anchor().physical(),
                                *mem.anchor(),
                                mem.created_at(),
                            ));
                        }
                        all_anchors.sort_by_key(|b| std::cmp::Reverse(b.2));

                        let mut used_basis = std::collections::HashSet::<[i32; 7]>::new();
                        for (phys, anchor, _ts) in &all_anchors {
                            if recent.len() >= 5 {
                                break;
                            }
                            let dx = (ideal_phys[0] - phys[0]).abs();
                            let dy = (ideal_phys[1] - phys[1]).abs();
                            let dz = (ideal_phys[2] - phys[2]).abs();
                            if dx + dy + dz < 150 {
                                if used_basis.contains(&anchor.basis()) {
                                    continue;
                                }
                                used_basis.insert(anchor.basis());
                                if let Some(mem) = memories.iter().find(|m| m.anchor() == anchor) {
                                    let desc = mem.description().unwrap_or("").to_string();
                                    if !desc.is_empty() {
                                        recent.push(json!({
                                            "description": desc,
                                            "tags": mem.tags(),
                                            "category": mem.category().unwrap_or(""),
                                            "importance": mem.importance(),
                                            "method": "spatial",
                                        }));
                                    }
                                }
                            }
                        }

                        if recent.len() < 5 {
                            let knn = semantic.search_similar(&query_data, 10);
                            for k in &knn {
                                if recent.len() >= 5 {
                                    break;
                                }
                                let anchor = Coord7D::new_even(k.atom_key.vertices_basis[0]);
                                if used_basis.contains(&anchor.basis()) {
                                    continue;
                                }
                                used_basis.insert(anchor.basis());
                                if let Some(mem) = memories
                                    .iter()
                                    .find(|m| m.anchor().basis() == k.atom_key.vertices_basis[0])
                                {
                                    let desc = mem.description().unwrap_or("").to_string();
                                    if !desc.is_empty() {
                                        recent.push(json!({
                                            "description": desc,
                                            "tags": mem.tags(),
                                            "category": mem.category().unwrap_or(""),
                                            "importance": mem.importance(),
                                            "method": "knn",
                                            "similarity": k.similarity,
                                        }));
                                    }
                                }
                            }
                        }

                        let knn_seed = semantic.search_similar(&query_data, 3);
                        if let Some(seed_k) = knn_seed.first() {
                            let seed_anchor = Coord7D::new_even(seed_k.atom_key.vertices_basis[0]);
                            let h_neighbors = hebbian.get_neighbors(&seed_anchor);
                            for (coord, weight) in h_neighbors.iter().take(3) {
                                if recent.len() >= 8 {
                                    break;
                                }
                                if let Some(mem) = memories.iter().find(|m| m.anchor() == coord) {
                                    let desc = mem.description().unwrap_or("").to_string();
                                    if !desc.is_empty()
                                        && !used_basis.contains(&mem.anchor().basis())
                                    {
                                        used_basis.insert(mem.anchor().basis());
                                        recent.push(json!({
                                            "description": desc,
                                            "tags": mem.tags(),
                                            "method": "hebbian",
                                            "edge_weight": weight,
                                        }));
                                    }
                                }
                            }
                        }

                        for entry in &recent {
                            if let Some(desc) = entry.get("description").and_then(|d| d.as_str()) {
                                context_window.push(ContextEntry {
                                    role: "system".to_string(),
                                    content: format!("[activated memory] {}", desc),
                                    token_estimate: desc.split_whitespace().count() * 2,
                                    memory_id: None,
                                });
                            }
                        }

                        let current_tokens: usize =
                            context_window.iter().map(|e| e.token_estimate).sum();
                        super::protocol::ToolCallResult::ok(
                            json!({
                                "action": "pre_work",
                                "query": query,
                                "activated_memories": recent.len(),
                                "memories": recent,
                                "context_entries": context_window.len(),
                                "current_tokens": current_tokens,
                                "max_tokens": context_max_tokens,
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
            "tetramem_forget" => {
                let anchor = match parse_3d_coord(args, "anchor") {
                    Ok(c) => c,
                    Err(e) => return super::protocol::ToolCallResult::err(e),
                };
                let idx = match memories.iter().position(|m| m.anchor() == &anchor) {
                    Some(i) => i,
                    None => {
                        return super::protocol::ToolCallResult::err(format!(
                            "no memory at {:?}",
                            anchor.basis()
                        ))
                    }
                };
                let mem = &memories[idx];
                let desc = mem.description().unwrap_or("").to_string();
                MemoryCodec::erase(universe, mem);
                memories.remove(idx);
                super::protocol::ToolCallResult::ok(
                    json!({
                        "success": true,
                        "erased_anchor": format!("{}", anchor),
                        "description": desc,
                        "remaining_memories": memories.len(),
                        "conservation_ok": universe.verify_conservation(),
                    })
                    .to_string(),
                )
            }
            "tetramem_reason" => {
                let method = match args.get("method").and_then(|v| v.as_str()) {
                    Some(m) => m,
                    None => {
                        return super::protocol::ToolCallResult::err(
                            "method is required: analogies/infer_chain/discover",
                        )
                    }
                };
                match method {
                    "analogies" => {
                        let threshold = args
                            .get("threshold")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.7);
                        let analogies =
                            ReasoningEngine::find_analogies(universe, memories, threshold);
                        let results: Vec<Value> = analogies
                            .iter()
                            .map(|r| {
                                let desc_source = memories
                                    .iter()
                                    .find(|m| format!("{}", m.anchor()) == r.source)
                                    .and_then(|m| m.description().map(String::from))
                                    .unwrap_or_default();
                                let desc_target = r
                                    .targets
                                    .first()
                                    .and_then(|t| {
                                        memories
                                            .iter()
                                            .find(|m| format!("{}", m.anchor()) == *t)
                                            .and_then(|m| m.description().map(String::from))
                                    })
                                    .unwrap_or_default();
                                json!({
                                    "source": r.source,
                                    "source_description": desc_source,
                                    "target": r.targets.first().unwrap_or(&"".into()),
                                    "target_description": desc_target,
                                    "confidence": r.confidence,
                                })
                            })
                            .collect();
                        super::protocol::ToolCallResult::ok(
                            json!({
                                "method": "analogies",
                                "threshold": threshold,
                                "analogies_found": results.len(),
                                "results": results,
                            })
                            .to_string(),
                        )
                    }
                    "infer_chain" => {
                        let from = match parse_3d_coord(args, "from_anchor") {
                            Ok(c) => c,
                            Err(e) => {
                                return super::protocol::ToolCallResult::err(format!(
                                    "from_anchor required: {}",
                                    e
                                ))
                            }
                        };
                        let to = match parse_3d_coord(args, "to_anchor") {
                            Ok(c) => c,
                            Err(e) => {
                                return super::protocol::ToolCallResult::err(format!(
                                    "to_anchor required: {}",
                                    e
                                ))
                            }
                        };
                        let max_hops =
                            args.get("max_hops").and_then(|v| v.as_u64()).unwrap_or(15) as usize;
                        let chain =
                            ReasoningEngine::infer_chain(universe, hebbian, &from, &to, max_hops);
                        let hops: Vec<Value> = chain
                            .iter()
                            .map(|r| {
                                json!({
                                    "from": r.source,
                                    "to": r.targets.first().unwrap_or(&"".into()),
                                    "confidence": r.confidence,
                                    "hop": r.hops,
                                })
                            })
                            .collect();
                        super::protocol::ToolCallResult::ok(
                            json!({
                                "method": "infer_chain",
                                "from": format!("{}", from),
                                "to": format!("{}", to),
                                "chain_length": hops.len(),
                                "found": !hops.is_empty(),
                                "hops": hops,
                            })
                            .to_string(),
                        )
                    }
                    "discover" => {
                        let seed = match parse_3d_coord(args, "seed_anchor") {
                            Ok(c) => c,
                            Err(e) => {
                                return super::protocol::ToolCallResult::err(format!(
                                    "seed_anchor required: {}",
                                    e
                                ))
                            }
                        };
                        let discoveries = ReasoningEngine::discover(universe, hebbian, &seed, 0.5);
                        let results: Vec<Value> = discoveries
                            .iter()
                            .map(|r| {
                                let desc = r
                                    .targets
                                    .first()
                                    .and_then(|t| {
                                        memories
                                            .iter()
                                            .find(|m| format!("{}", m.anchor()) == *t)
                                            .and_then(|m| m.description().map(String::from))
                                    })
                                    .unwrap_or_default();
                                json!({
                                    "from": r.source,
                                    "discovered": r.targets.first().unwrap_or(&"".into()),
                                    "description": desc,
                                    "confidence": r.confidence,
                                })
                            })
                            .collect();
                        super::protocol::ToolCallResult::ok(
                            json!({
                                "method": "discover",
                                "seed": format!("{}", seed),
                                "discoveries": results.len(),
                                "results": results,
                            })
                            .to_string(),
                        )
                    }
                    _ => super::protocol::ToolCallResult::err(
                        "method must be analogies/infer_chain/discover",
                    ),
                }
            }
            "tetramem_emotion" => {
                let reading = EmotionMapper::read(universe);
                let func_emotion = FunctionalEmotion::from_pad(
                    reading.pad,
                    crate::universe::cognitive::functional_emotion::EmotionSource::Functional,
                );
                let budget = PerceptionBudget::new(universe.stats().total_energy);
                let perception_report = budget.report();
                super::protocol::ToolCallResult::ok(
                    json!({
                        "pad": {
                            "pleasure": reading.pad.pleasure,
                            "arousal": reading.pad.arousal,
                            "dominance": reading.pad.dominance,
                            "magnitude": reading.pad.magnitude(),
                            "quadrant": format!("{:?}", reading.quadrant),
                            "dominance_label": reading.pad.dominance_label(),
                        },
                        "functional_emotion": {
                            "cluster": func_emotion.cluster.name(),
                            "valence": format!("{:?}", func_emotion.valence),
                            "arousal_level": format!("{:?}", func_emotion.arousal),
                            "is_positive": func_emotion.is_positive(),
                            "is_high_arousal": func_emotion.is_high_arousal(),
                        },
                        "recommendations": {
                            "pulse_strategy": format!("{:?}", reading.pulse_suggestion),
                            "dream_frequency_multiplier": reading.dream_frequency_multiplier,
                            "crystal_threshold_modifier": reading.crystal_threshold_modifier,
                        },
                        "perception": {
                            "total_budget": perception_report.total_budget,
                            "allocated": perception_report.allocated,
                            "spent": perception_report.spent,
                            "returned": perception_report.returned,
                            "utilization": perception_report.utilization,
                            "active_perceptions": perception_report.active_perceptions,
                        },
                        "universe_state": {
                            "energy_utilization": reading.energy_utilization,
                            "manifested_ratio": reading.manifested_ratio,
                        },
                    })
                    .to_string(),
                )
            }
            "tetramem_scale" => {
                let action = match args.get("action").and_then(|v| v.as_str()) {
                    Some(a) => a,
                    None => {
                        return super::protocol::ToolCallResult::err(
                            "action is required: auto/frontier",
                        )
                    }
                };
                let scaler = AutoScaler::new();
                match action {
                    "auto" => {
                        let report = scaler.auto_scale(universe, hebbian, memories);
                        super::protocol::ToolCallResult::ok(
                            json!({
                                "action": "auto_scale",
                                "energy_expanded_by": report.energy_expanded_by,
                                "nodes_added": report.nodes_added,
                                "nodes_removed": report.nodes_removed,
                                "rebalanced": report.rebalanced,
                                "reason": format!("{:?}", report.reason),
                                "conservation_ok": universe.verify_conservation(),
                            })
                            .to_string(),
                        )
                    }
                    "frontier" => {
                        let max_new = args
                            .get("max_new_nodes")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(200) as usize;
                        let report = scaler.frontier_expansion(universe, max_new);
                        super::protocol::ToolCallResult::ok(
                            json!({
                                "action": "frontier_expansion",
                                "energy_expanded_by": report.energy_expanded_by,
                                "nodes_added": report.nodes_added,
                                "nodes_removed": report.nodes_removed,
                                "rebalanced": report.rebalanced,
                                "conservation_ok": universe.verify_conservation(),
                            })
                            .to_string(),
                        )
                    }
                    _ => super::protocol::ToolCallResult::err("action must be auto or frontier"),
                }
            }
            "tetramem_watchdog" => {
                let stats = universe.stats();
                let mut watchdog = Watchdog::with_defaults(stats.total_energy);
                let report = watchdog.checkup(universe, hebbian, crystal, memories);
                super::protocol::ToolCallResult::ok(
                    json!({
                        "level": report.level.as_str(),
                        "health": report.health.as_str(),
                        "utilization": report.utilization,
                        "node_count": report.node_count,
                        "memory_count": report.memory_count,
                        "conservation_ok": report.conservation_ok,
                        "consecutive_conservation_failures": report.consecutive_conservation_failures,
                        "total_checkups": report.total_checkups,
                        "backup_count": report.backup_count,
                        "energy": {
                            "initial": report.initial_energy,
                            "current": report.current_energy,
                            "total": stats.total_energy,
                            "available": stats.available_energy,
                        },
                        "elapsed_ms": report.elapsed_ms,
                        "actions_available": report.actions.iter().map(|a| a.action.clone()).collect::<Vec<_>>(),
                    })
                    .to_string(),
                )
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
        .filter(|w| !w.is_empty() && w.len() > 1)
        .collect();

    let stop_words: &[&str] = &[
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "shall", "can",
        "need", "must", "ought", "to", "of", "in", "for", "on", "with", "at", "by", "from", "as",
        "into", "through", "during", "before", "after", "above", "below", "between", "out", "off",
        "over", "under", "again", "further", "then", "once", "and", "but", "or", "nor", "not",
        "so", "yet", "both", "either", "neither", "each", "every", "all", "any", "few", "more",
        "most", "other", "some", "such", "no", "only", "own", "same", "than", "too", "very",
        "just", "because", "if", "when", "where", "how", "what", "which", "who", "whom", "this",
        "that", "these", "those", "it", "its", "he", "she", "they", "them", "we", "you", "me",
        "my", "your", "his", "her", "our", "their",
    ];

    for word in &words {
        if stop_words.contains(word) {
            continue;
        }

        let subwords = extract_subwords(word);

        let synonym_hash = synonym_bucket(word);

        for sw in &subwords {
            let mut h: u64 = 5381;
            for b in sw.as_bytes() {
                h = h.wrapping_mul(33).wrapping_add(*b as u64);
            }
            let s1 = (h as usize) % dim;
            let s2 = (h.wrapping_mul(37) as usize) % dim;
            let s3 = (h.wrapping_mul(53) as usize) % dim;
            let s4 = (h.wrapping_mul(59) as usize) % dim;
            vec[s1] += 1.0;
            vec[s2] += 0.8;
            vec[s3] += 0.5;
            vec[s4] += 0.3;
        }

        if let Some(bucket) = synonym_hash {
            let b = bucket as usize;
            vec[b % dim] += 2.0;
            vec[(b * 7 + 3) % dim] += 1.5;
            vec[(b * 13 + 7) % dim] += 1.0;
        }
    }

    for i in 0..lower.len().saturating_sub(2) {
        let trigram = &lower[i..i + 3];
        let mut h: u64 = 5381;
        for b in trigram.as_bytes() {
            h = h.wrapping_mul(37).wrapping_add(*b as u64);
        }
        vec[(h as usize) % dim] += 0.4;
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

fn extract_subwords(word: &str) -> Vec<String> {
    let mut subs = Vec::new();
    let bytes = word.as_bytes();
    let n = bytes.len();

    if n >= 3 {
        for start in 0..=n.saturating_sub(3) {
            let end = (start + 5).min(n);
            if end - start >= 3 {
                subs.push(String::from_utf8_lossy(&bytes[start..end]).to_string());
            }
        }
        subs.push(format!("<{}>", word));
        subs.push(format!("<{}", &word[word.len().min(3)..]));
        subs.push(format!("{}>", &word[..word.len().saturating_sub(3).max(1)]));
    }

    subs.push(word.to_string());

    subs
}

fn synonym_bucket(word: &str) -> Option<u64> {
    let buckets: &[&[&str]] = &[
        &[
            "prefer",
            "like",
            "love",
            "enjoy",
            "favor",
            "fancy",
            "adore",
            "appreciate",
        ],
        &["dislike", "hate", "detest", "loathe", "abhor", "despise"],
        &[
            "good",
            "great",
            "excellent",
            "fine",
            "nice",
            "wonderful",
            "amazing",
            "awesome",
            "fantastic",
        ],
        &[
            "bad", "poor", "terrible", "awful", "horrible", "dreadful", "worst",
        ],
        &[
            "big", "large", "huge", "vast", "enormous", "massive", "giant", "immense",
        ],
        &[
            "small", "tiny", "little", "mini", "micro", "compact", "minor",
        ],
        &[
            "fast", "quick", "rapid", "swift", "speedy", "prompt", "hasty",
        ],
        &["slow", "sluggish", "gradual", "steady", "leisurely"],
        &[
            "happy",
            "glad",
            "joyful",
            "cheerful",
            "pleased",
            "delighted",
            "content",
        ],
        &[
            "sad",
            "unhappy",
            "depressed",
            "miserable",
            "gloomy",
            "sorrowful",
        ],
        &[
            "important",
            "significant",
            "crucial",
            "vital",
            "essential",
            "critical",
            "key",
        ],
        &[
            "think", "believe", "consider", "suppose", "assume", "guess", "reckon",
        ],
        &[
            "know",
            "understand",
            "comprehend",
            "grasp",
            "realize",
            "recognize",
        ],
        &["use", "utilize", "employ", "apply", "operate", "leverage"],
        &[
            "make",
            "create",
            "build",
            "construct",
            "produce",
            "generate",
            "develop",
        ],
        &["help", "assist", "support", "aid", "facilitate", "enable"],
        &["need", "require", "demand", "want", "desire", "wish"],
        &[
            "change",
            "modify",
            "alter",
            "adjust",
            "transform",
            "update",
            "convert",
        ],
        &["start", "begin", "launch", "initiate", "commence", "open"],
        &[
            "stop",
            "end",
            "finish",
            "complete",
            "conclude",
            "terminate",
            "halt",
        ],
        &["work", "function", "operate", "perform", "run", "execute"],
        &[
            "system",
            "platform",
            "framework",
            "engine",
            "architecture",
            "infrastructure",
        ],
        &[
            "data",
            "information",
            "knowledge",
            "facts",
            "details",
            "records",
        ],
        &["user", "client", "customer", "person", "human", "people"],
        &["dark", "night", "shadow", "dim", "black", "obscure"],
        &["light", "bright", "luminous", "clear", "vivid", "radiant"],
        &[
            "mode",
            "setting",
            "option",
            "preference",
            "configuration",
            "theme",
        ],
        &["memory", "recall", "remember", "store", "retain", "record"],
        &["learn", "study", "acquire", "absorb", "train", "educate"],
        &[
            "search", "find", "look", "seek", "discover", "explore", "query",
        ],
        &[
            "show",
            "display",
            "present",
            "reveal",
            "exhibit",
            "demonstrate",
        ],
        &["hide", "conceal", "mask", "cover", "obscure", "cloak"],
        &[
            "connect",
            "link",
            "join",
            "associate",
            "bind",
            "attach",
            "relate",
        ],
        &[
            "error", "bug", "fault", "defect", "issue", "problem", "mistake",
        ],
        &[
            "fix", "repair", "correct", "resolve", "patch", "solve", "debug",
        ],
        &[
            "new", "fresh", "recent", "latest", "modern", "current", "novel",
        ],
        &[
            "old", "ancient", "outdated", "legacy", "obsolete", "vintage",
        ],
        &[
            "simple",
            "easy",
            "basic",
            "straightforward",
            "plain",
            "elementary",
        ],
        &[
            "complex",
            "complicated",
            "intricate",
            "elaborate",
            "sophisticated",
        ],
        &[
            "safe",
            "secure",
            "protected",
            "guarded",
            "reliable",
            "stable",
        ],
        &[
            "danger",
            "risk",
            "threat",
            "hazard",
            "peril",
            "vulnerability",
        ],
    ];

    for (i, bucket) in buckets.iter().enumerate() {
        if bucket.contains(&word) {
            return Some(i as u64 * 17 + 3);
        }
        if bucket.iter().any(|&w| {
            word.len() >= 4 && w.len() >= 4 && (word.starts_with(w) || w.starts_with(word))
        }) {
            return Some(i as u64 * 17 + 3);
        }
    }

    None
}

fn detect_contradictions(
    new_content: &str,
    new_anchor: &Coord7D,
    memories: &[MemoryAtom],
    _universe: &DarkUniverse,
    hebbian: &HebbianMemory,
) -> Vec<serde_json::Value> {
    let mut contradictions = Vec::new();
    let _new_data = text_to_embedding(new_content, 0.5);
    let new_phys = new_anchor.physical();

    let contradict_pairs: &[&[&str]] = &[
        &["should", "should not", "must not", "never"],
        &["always", "never", "sometimes", "rarely"],
        &["good", "bad", "terrible", "awful"],
        &["like", "hate", "dislike", "loathe"],
        &["agree", "disagree", "oppose", "reject"],
        &["true", "false", "wrong", "incorrect"],
        &["yes", "no"],
        &["possible", "impossible"],
        &["easy", "hard", "difficult", "complex"],
        &["safe", "dangerous", "risky", "unsafe"],
    ];

    let new_lower = new_content.to_lowercase();
    let mut new_sentiment_group: Option<usize> = None;
    'outer: for (gi, group) in contradict_pairs.iter().enumerate() {
        for word in *group {
            if new_lower.contains(word) {
                new_sentiment_group = Some(gi);
                break 'outer;
            }
        }
    }

    if new_sentiment_group.is_none() {
        return contradictions;
    }

    let sg = new_sentiment_group.unwrap();
    let group = contradict_pairs[sg];

    let neighbors = hebbian.get_neighbors(new_anchor);
    let mut candidates: Vec<&MemoryAtom> = Vec::new();

    for mem in memories.iter() {
        let mp = mem.anchor().physical();
        let d =
            (new_phys[0] - mp[0]).abs() + (new_phys[1] - mp[1]).abs() + (new_phys[2] - mp[2]).abs();
        if d < 200 && d > 0 {
            candidates.push(mem);
        }
    }

    for (coord, _weight) in &neighbors {
        if let Some(mem) = memories.iter().find(|m| m.anchor() == coord) {
            if !candidates.iter().any(|c| c.anchor() == mem.anchor()) {
                candidates.push(mem);
            }
        }
    }

    for mem in candidates.iter().take(10) {
        let desc = match mem.description() {
            Some(d) => d.to_lowercase(),
            None => continue,
        };

        let mut has_same = false;
        let mut has_opposite = false;
        for (wi, word) in group.iter().enumerate() {
            if new_lower.contains(word) {
                for (wj, other_word) in group.iter().enumerate() {
                    if desc.contains(other_word) {
                        if wi == wj {
                            has_same = true;
                        } else if wi / 2 != wj / 2 {
                            has_opposite = true;
                        }
                    }
                }
            }
        }

        if has_opposite && !has_same {
            let edge_w = hebbian.get_bias(new_anchor, mem.anchor());
            contradictions.push(json!({
                "conflict_with": desc,
                "anchor": format!("{}", mem.anchor()),
                "edge_weight": edge_w,
                "confidence": if edge_w > 0.5 { "high" } else { "medium" },
            }));
        }
    }

    contradictions
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

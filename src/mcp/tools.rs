// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use serde_json::{json, Value};

use super::core::TetraMemCore;
use super::protocol::{ResourceDefinition, ToolDefinition};
use crate::universe::coord::Coord7D;
use crate::universe::memory::nlp;

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
                description: "Check universe health level (Healthy/Good/Warning/Critical) with detailed report".into(),
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
                description: "Fire a pulse (reinforcing/exploratory/cascade) from a source position to explore the lattice".into(),
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
                description: "Materialize a node at a 3D position with specified energy and physical/dark ratio.".into(),
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
                description: "Verify energy conservation law holds across the entire universe.".into(),
                input_schema: json!({"type": "object", "properties": {}, "required": []}),
            },
            ToolDefinition {
                name: "tetramem_remember".into(),
                description: "Store a memory for an AI agent. Automatically encodes content, indexes semantically, links to similar memories, and detects contradictions with existing beliefs. Use category='decision' for decision logging.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "The content to remember"
                        },
                        "tags": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Tags for categorization"
                        },
                        "category": {
                            "type": "string",
                            "description": "Category (e.g. 'user_preference', 'technical', 'decision')"
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
                description: "Retrieve memories by natural language query. Uses spatial proximity + KNN + Hebbian association.".into(),
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
                        }
                    },
                    "required": ["query"]
                }),
            },
            ToolDefinition {
                name: "tetramem_associate".into(),
                description: "Discover associated memories for a topic. Uses Hebbian edge traversal and pulse propagation.".into(),
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
                description: "Run dream consolidation cycle: strengthen important memories, weaken noise, detect emergent knowledge clusters, and track self-model evolution.".into(),
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
                description: "Manage agent context window. Add messages; when context overflows, older messages are automatically encoded to TetraMem memory. Use 'pre_work' to auto-activate relevant memories before starting a task.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["add", "status", "reconstruct", "clear", "pre_work"],
                            "description": "Action: 'add' message, 'status' check, 'reconstruct' from memory, 'clear' window, 'pre_work' activate relevant memories"
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
                description: "Advanced reasoning: find analogies between memories, infer chains between two anchors, or discover new knowledge via exploratory pulse.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "method": {
                            "type": "string",
                            "enum": ["analogies", "infer_chain", "discover"],
                            "description": "Reasoning method"
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
                description: "Read the emotional state of the universe from dark energy dimensions. Returns PAD vector, emotional quadrant, functional emotion cluster, and pulse strategy recommendation.".into(),
                input_schema: json!({"type": "object", "properties": {}, "required": []}),
            },
            ToolDefinition {
                name: "tetramem_scale".into(),
                description: "Auto-scale the universe: expand energy when utilization is high, shrink when low, or grow the lattice frontier.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["auto", "frontier"],
                            "description": "'auto' evaluates and scales, 'frontier' expands lattice boundary"
                        },
                        "max_new_nodes": {
                            "type": "integer",
                            "description": "Max nodes for frontier expansion (default 200)"
                        }
                    },
                    "required": ["action"]
                }),
            },
            ToolDefinition {
                name: "tetramem_watchdog".into(),
                description: "Run a watchdog checkup: comprehensive health monitoring with conservation tracking and multi-level alert status.".into(),
                input_schema: json!({"type": "object", "properties": {}, "required": []}),
            },
            ToolDefinition {
                name: "tetramem_forget".into(),
                description: "Erase a memory by its anchor position, freeing its energy back to the universe pool.".into(),
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
        ]
    }

    pub fn handle_tool(
        name: &str,
        args: &Value,
        core: &mut TetraMemCore,
    ) -> super::protocol::ToolCallResult {
        match name {
            "tetramem_stats" => super::protocol::ToolCallResult::ok(core.stats().to_string()),
            "tetramem_health" => handle_health(core),
            "tetramem_encode" => handle_encode(args, core),
            "tetramem_decode" => handle_decode(args, core),
            "tetramem_list_memories" => handle_list_memories(core),
            "tetramem_pulse" => handle_pulse(args, core),
            "tetramem_dream" => handle_dream(core),
            "tetramem_topology" => handle_topology(core),
            "tetramem_regulate" => handle_regulate(core),
            "tetramem_trace" => handle_trace(args, core),
            "tetramem_phase_detect" => handle_phase_detect(core),
            "tetramem_materialize" => handle_materialize(args, core),
            "tetramem_conservation_check" => {
                super::protocol::ToolCallResult::ok(core.conservation_check().to_string())
            }
            "tetramem_remember" => handle_remember(args, core),
            "tetramem_recall" => handle_recall(args, core),
            "tetramem_associate" => handle_associate(args, core),
            "tetramem_consolidate" => handle_consolidate(args, core),
            "tetramem_context" => handle_context(args, core),
            "tetramem_reason" => handle_reason(args, core),
            "tetramem_emotion" => handle_emotion(core),
            "tetramem_scale" => handle_scale(args, core),
            "tetramem_watchdog" => handle_watchdog(core),
            "tetramem_forget" => handle_forget(args, core),
            _ => super::protocol::ToolCallResult::err(format!("unknown tool: {}", name)),
        }
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

    pub fn read_resource(
        uri: &str,
        core: &TetraMemCore,
    ) -> Option<super::protocol::ResourceContent> {
        match uri {
            "tetramem://stats" => Some(super::protocol::ResourceContent {
                uri: uri.into(),
                mime_type: Some("application/json".into()),
                text: core.stats().to_string(),
            }),
            "tetramem://health" => {
                let report = crate::universe::observer::UniverseObserver::inspect(
                    &core.universe,
                    &core.hebbian,
                    &core.memories,
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

// -- Individual tool handlers --

fn handle_health(core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
    let report = crate::universe::observer::UniverseObserver::inspect(
        &core.universe,
        &core.hebbian,
        &core.memories,
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

fn handle_encode(args: &Value, core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
    let anchor = match parse_3d_coord(args, "anchor") {
        Ok(c) => c,
        Err(e) => return super::protocol::ToolCallResult::err(e),
    };
    let data: Vec<f64> = match args.get("data").and_then(|v| v.as_array()) {
        Some(a) => a.iter().filter_map(|v| v.as_f64()).collect(),
        None => return super::protocol::ToolCallResult::err("data must be array of numbers"),
    };
    if data.is_empty() || data.len() > 28 {
        return super::protocol::ToolCallResult::err("data must have 1-28 values");
    }
    match crate::universe::memory::MemoryCodec::encode(&mut core.universe, &anchor, &data) {
        Ok(mem) => {
            let anchor_str = format!("{}", mem.anchor());
            let dim = mem.data_dim();
            core.memories.push(mem);
            super::protocol::ToolCallResult::ok(
                json!({
                    "success": true,
                    "anchor": anchor_str,
                    "dimensions": dim,
                    "conservation_ok": core.universe.verify_conservation(),
                })
                .to_string(),
            )
        }
        Err(e) => super::protocol::ToolCallResult::err(format!("encode failed: {}", e)),
    }
}

fn handle_decode(args: &Value, core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
    let anchor = match parse_3d_coord(args, "anchor") {
        Ok(c) => c,
        Err(e) => return super::protocol::ToolCallResult::err(e),
    };
    match core.memories.iter().find(|m| m.anchor() == &anchor) {
        Some(mem) => match crate::universe::memory::MemoryCodec::decode(&core.universe, mem) {
            Ok(data) => super::protocol::ToolCallResult::ok(
                json!({
                    "anchor": format!("{}", mem.anchor()),
                    "data": data,
                    "dimensions": mem.data_dim(),
                })
                .to_string(),
            ),
            Err(e) => super::protocol::ToolCallResult::err(format!("decode failed: {}", e)),
        },
        None => super::protocol::ToolCallResult::err(format!("no memory at {:?}", anchor.basis())),
    }
}

fn handle_list_memories(core: &TetraMemCore) -> super::protocol::ToolCallResult {
    let list: Vec<Value> = core
        .memories
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
        json!({ "count": list.len(), "memories": list }).to_string(),
    )
}

fn handle_pulse(args: &Value, core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
    let source = match parse_3d_coord(args, "source") {
        Ok(c) => c,
        Err(e) => return super::protocol::ToolCallResult::err(e),
    };
    let pulse_type = match args.get("pulse_type").and_then(|v| v.as_str()) {
        Some("reinforcing") => crate::universe::pulse::PulseType::Reinforcing,
        Some("exploratory") => crate::universe::pulse::PulseType::Exploratory,
        Some("cascade") => crate::universe::pulse::PulseType::Cascade,
        _ => {
            return super::protocol::ToolCallResult::err(
                "pulse_type must be reinforcing/exploratory/cascade",
            )
        }
    };
    let engine = crate::universe::pulse::PulseEngine::new();
    let result = engine.propagate(&source, pulse_type, &core.universe, &mut core.hebbian);
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

fn handle_dream(core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
    let engine = crate::universe::dream::DreamEngine::new();
    let report = engine.dream(&core.universe, &mut core.hebbian, &core.memories);
    super::protocol::ToolCallResult::ok(
        json!({
            "phases": format!("{}", report),
            "edges_before": report.hebbian_edges_before,
            "edges_after": report.hebbian_edges_after,
            "weight_before": report.weight_before,
            "weight_after": report.weight_after,
            "conservation_ok": core.universe.verify_conservation(),
        })
        .to_string(),
    )
}

fn handle_topology(core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
    let report = crate::universe::topology::TopologyEngine::analyze(&core.universe);
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

fn handle_regulate(core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
    let engine = crate::universe::regulation::RegulationEngine::new();
    let report = engine.regulate(
        &mut core.universe,
        &mut core.hebbian,
        &mut core.crystal,
        &core.memories,
    );
    super::protocol::ToolCallResult::ok(
        json!({
            "stress_level": report.stress_level,
            "entropy": report.entropy,
            "imbalance": report.dimension_pressure.imbalance,
            "actions_taken": report.actions.len(),
            "conservation_ok": core.universe.verify_conservation(),
        })
        .to_string(),
    )
}

fn handle_trace(args: &Value, core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
    let anchor = match parse_3d_coord(args, "anchor") {
        Ok(c) => c,
        Err(e) => return super::protocol::ToolCallResult::err(e),
    };
    let max_hops = args.get("max_hops").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
    let associations = crate::universe::reasoning::ReasoningEngine::find_associations(
        &core.universe,
        &core.hebbian,
        &core.crystal,
        &anchor,
        max_hops,
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
        json!({ "associations": results, "total": results.len() }).to_string(),
    )
}

fn handle_phase_detect(core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
    let report = core
        .crystal
        .detect_phase_transition(&core.hebbian, &core.universe);
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

fn handle_materialize(args: &Value, core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
    let coord = match parse_3d_coord(args, "coord") {
        Ok(c) => c,
        Err(e) => return super::protocol::ToolCallResult::err(e),
    };
    let energy = match args.get("energy").and_then(|v| v.as_f64()) {
        Some(e) if e > 0.0 => e,
        _ => return super::protocol::ToolCallResult::err("energy must be a positive number"),
    };
    let ratio = match args.get("physical_ratio").and_then(|v| v.as_f64()) {
        Some(r) if (0.0..=1.0).contains(&r) => r,
        _ => {
            return super::protocol::ToolCallResult::err(
                "physical_ratio must be between 0.0 and 1.0",
            )
        }
    };
    match core.universe.materialize_biased(coord, energy, ratio) {
        Ok(_) => super::protocol::ToolCallResult::ok(
            json!({
                "success": true,
                "coord": format!("{}", coord),
                "energy": energy,
                "conservation_ok": core.universe.verify_conservation(),
            })
            .to_string(),
        ),
        Err(e) => super::protocol::ToolCallResult::err(format!("materialize failed: {}", e)),
    }
}

fn handle_remember(args: &Value, core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
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

    let result = core.remember(&content, &tags, &category, importance, &source);
    super::protocol::ToolCallResult::ok(result.to_string())
}

fn handle_recall(args: &Value, core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
    let query = match args.get("query").and_then(|v| v.as_str()) {
        Some(q) => q.to_string(),
        None => return super::protocol::ToolCallResult::err("query is required"),
    };
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

    let query_data = nlp::text_to_embedding(&query, 0.5);
    let ideal_anchor = core
        .clustering
        .compute_ideal_anchor(&query_data, &core.universe);
    let ideal_phys = ideal_anchor.physical();

    let mut spatial_hits: Vec<(usize, f64)> = Vec::new();
    for (i, mem) in core.memories.iter().enumerate() {
        let mp = mem.anchor().physical();
        let dx = (ideal_phys[0] - mp[0]).abs();
        let dy = (ideal_phys[1] - mp[1]).abs();
        let dz = (ideal_phys[2] - mp[2]).abs();
        if dx + dy + dz < 100 {
            let dist_sq = dx * dx + dy * dy + dz * dz;
            let score = 1.0 / (1.0 + (dist_sq as f64).sqrt());
            spatial_hits.push((i, score));
        }
    }
    spatial_hits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut seen_anchors = std::collections::HashSet::new();
    let mut hits = Vec::new();

    for &(idx, spatial_score) in &spatial_hits {
        if hits.len() >= limit {
            break;
        }
        let mem = &core.memories[idx];
        let anchor_key = mem.anchor().basis();
        if seen_anchors.contains(&anchor_key) {
            continue;
        }
        seen_anchors.insert(anchor_key);

        let nb = core.hebbian.get_neighbors(mem.anchor());
        let associated: Vec<String> = nb
            .iter()
            .filter_map(|(coord, _)| {
                core.memories
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
        let knn_results = core.semantic.search_similar(&query_data, limit * 2);
        for knn in &knn_results {
            if hits.len() >= limit {
                break;
            }
            let anchor_basis = knn.atom_key.vertices_basis[0];
            if seen_anchors.contains(&anchor_basis) {
                continue;
            }
            seen_anchors.insert(anchor_basis);
            if let Some(mem) = core
                .memories
                .iter()
                .find(|m| m.anchor().basis() == anchor_basis)
            {
                let nb = core.hebbian.get_neighbors(mem.anchor());
                let associated: Vec<String> = nb
                    .iter()
                    .filter_map(|(coord, _)| {
                        core.memories
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
        json!({ "query": query, "results": hits, "returned": hits.len() }).to_string(),
    )
}

fn handle_associate(args: &Value, core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
    let topic = match args.get("topic").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return super::protocol::ToolCallResult::err("topic is required"),
    };
    let depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

    let topic_data = nlp::text_to_embedding(&topic, 0.5);
    let ideal_anchor = core
        .clustering
        .compute_ideal_anchor(&topic_data, &core.universe);

    let seed_anchor = core
        .memories
        .iter()
        .min_by(|a, b| {
            let da = a.anchor().distance_sq(&ideal_anchor);
            let db = b.anchor().distance_sq(&ideal_anchor);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|m| *m.anchor())
        .unwrap_or_else(|| {
            let knn = core.semantic.search_similar(&topic_data, 1);
            match knn.first() {
                Some(k) => Coord7D::new_even(k.atom_key.vertices_basis[0]),
                None => Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]),
            }
        });

    let associations = crate::universe::reasoning::ReasoningEngine::find_associations(
        &core.universe,
        &core.hebbian,
        &core.crystal,
        &seed_anchor,
        depth,
    );

    let mut results = Vec::new();
    for assoc in associations.iter().take(limit) {
        let targets: Vec<Value> = assoc
            .targets
            .iter()
            .take(5)
            .map(|t| {
                let desc = core
                    .memories
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

    let pulse_result = crate::universe::pulse::PulseEngine::new().propagate(
        &seed_anchor,
        crate::universe::pulse::PulseType::Exploratory,
        &core.universe,
        &mut core.hebbian,
    );

    let mut cluster_neighbors = Vec::new();
    let seed_phys = seed_anchor.physical();
    for mem in core.memories.iter() {
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

fn handle_consolidate(args: &Value, core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
    let importance_threshold = args
        .get("importance_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.3);

    let report = crate::universe::dream::DreamEngine::new().dream(
        &core.universe,
        &mut core.hebbian,
        &core.memories,
    );

    let cluster_report =
        core.clustering
            .run_maintenance_cycle(&core.memories, &mut core.hebbian, &core.universe);

    let mut weakened = 0usize;
    let mut strengthened = 0usize;
    let mut intra_cluster = 0usize;
    let mut inter_cluster = 0usize;

    let mem_anchors: Vec<Coord7D> = core.memories.iter().map(|m| *m.anchor()).collect();
    for mem in core.memories.iter() {
        let neighbors = core.hebbian.get_neighbors(mem.anchor());
        for (coord, weight) in &neighbors {
            if *weight < importance_threshold {
                weakened += 1;
            } else {
                strengthened += 1;
            }
            if mem_anchors.iter().any(|a| a.distance_sq(coord) < 2500.0) {
                intra_cluster += 1;
            } else {
                inter_cluster += 1;
            }
        }
    }

    if intra_cluster > 0 && inter_cluster > 0 {
        let ratio = intra_cluster as f64 / inter_cluster as f64;
        if ratio < 1.0 {
            for mem in core.memories.iter() {
                let mem_phys = mem.anchor().physical();
                for other in core.memories.iter() {
                    if mem.anchor() == other.anchor() {
                        continue;
                    }
                    let other_phys = other.anchor().physical();
                    let d = (mem_phys[0] - other_phys[0]).abs()
                        + (mem_phys[1] - other_phys[1]).abs()
                        + (mem_phys[2] - other_phys[2]).abs();
                    if d < 30 {
                        core.hebbian.boost_edge(mem.anchor(), other.anchor(), 0.1);
                    }
                }
            }
        }
    }

    core.semantic.auto_link_similar(&core.memories);

    let mut clusters: Vec<Value> = Vec::new();
    let mut visited = std::collections::HashSet::new();
    for (i, _mem_a) in core.memories.iter().enumerate() {
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
            let cm = &core.memories[ci];
            cluster_members.push(json!({
                "anchor": format!("{}", cm.anchor()),
                "description": cm.description().unwrap_or(""),
                "category": cm.category().unwrap_or(""),
                "importance": cm.importance(),
            }));
            for (cj, mem_b) in core.memories.iter().enumerate() {
                if visited.contains(&cj) {
                    continue;
                }
                if cm.anchor().distance_sq(mem_b.anchor()) < 2500.0 {
                    stack.push(cj);
                }
            }
        }
        if cluster_members.len() >= 2 {
            let cat_set: std::collections::HashSet<&str> = cluster_members
                .iter()
                .filter_map(|m| m.get("category").and_then(|c| c.as_str()))
                .filter(|c| !c.is_empty())
                .collect();
            clusters.push(json!({
                "size": cluster_members.len(),
                "categories": cat_set.into_iter().collect::<Vec<&str>>(),
                "members": cluster_members,
            }));
        }
    }

    let mut emergent_links: Vec<Value> = Vec::new();
    for mem_a in core.memories.iter() {
        let nb = core.hebbian.get_neighbors(mem_a.anchor());
        for (coord, weight) in &nb {
            if *weight > 0.8 {
                if let Some(mem_b) = core.memories.iter().find(|m| m.anchor() == coord) {
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
    for mem in core.memories.iter() {
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
    let n = core.memories.len().max(1) as f64;
    for v in phys_center.iter_mut() {
        *v /= n;
    }
    for v in dark_dims.iter_mut() {
        *v /= n;
    }

    let mut knowledge_spread = 0.0f64;
    for mem in core.memories.iter() {
        let p = mem.anchor().physical();
        knowledge_spread += (p[0] as f64 - phys_center[0]).powi(2)
            + (p[1] as f64 - phys_center[1]).powi(2)
            + (p[2] as f64 - phys_center[2]).powi(2);
    }
    knowledge_spread = (knowledge_spread / n).sqrt();

    let mut belief_stability = 0.0f64;
    if core.memories.len() > 1 {
        let mut times: Vec<u64> = core.memories.iter().map(|m| m.created_at()).collect();
        times.sort();
        if let (Some(first), Some(last)) = (times.first(), times.last()) {
            let span = (*last - *first).max(1);
            belief_stability =
                (core.memories.len() as f64).ln() / (span as f64 / 86400000.0).ln().abs().max(1.0);
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
            "conservation_ok": core.universe.verify_conservation(),
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
                "total_beliefs": core.memories.len(),
                "physical_center": phys_center,
            },
        })
        .to_string(),
    )
}

fn handle_context(args: &Value, core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
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

            let memory_id = if core
                .context_window
                .iter()
                .map(|e| e.token_estimate)
                .sum::<usize>()
                + token_estimate
                > core.context_max_tokens
            {
                let overflow: Vec<super::core::ContextEntry> = core
                    .context_window
                    .drain(..core.context_window.len() / 2)
                    .collect();
                let mut archived = Vec::new();
                for entry in overflow {
                    let data = nlp::text_to_embedding(&entry.content, 0.3);
                    let anchor = nlp::text_to_anchor(&entry.content);
                    if let Ok(mem) = crate::universe::memory::MemoryCodec::encode(
                        &mut core.universe,
                        &anchor,
                        &data,
                    ) {
                        core.semantic.index_memory(&mem, &data);
                        for k in core.semantic.search_similar(&data, 3) {
                            let k_anchor = Coord7D::new_even(k.atom_key.vertices_basis[0]);
                            if k_anchor != *mem.anchor() {
                                core.hebbian.boost_edge(
                                    mem.anchor(),
                                    &k_anchor,
                                    0.3 * k.similarity,
                                );
                            }
                        }
                        archived.push(format!("{}", mem.anchor()));
                        core.memories.push(mem);
                    }
                }
                Some(archived)
            } else {
                None
            };

            core.context_window.push(super::core::ContextEntry {
                role: role.clone(),
                content: content.clone(),
                token_estimate,
                memory_id: None,
            });

            let current_tokens: usize = core.context_window.iter().map(|e| e.token_estimate).sum();
            super::protocol::ToolCallResult::ok(
                json!({
                    "action": "add",
                    "context_entries": core.context_window.len(),
                    "current_tokens": current_tokens,
                    "max_tokens": core.context_max_tokens,
                    "overflow_archived": memory_id.as_ref().map(|ids| ids.len()).unwrap_or(0),
                })
                .to_string(),
            )
        }
        "status" => {
            let current_tokens: usize = core.context_window.iter().map(|e| e.token_estimate).sum();
            let entries: Vec<Value> = core
                .context_window
                .iter()
                .map(|e| json!({"role": e.role, "tokens": e.token_estimate}))
                .collect();
            super::protocol::ToolCallResult::ok(json!({
                "entries": entries,
                "total_tokens": current_tokens,
                "max_tokens": core.context_max_tokens,
                "utilization": if core.context_max_tokens > 0 { current_tokens as f64 / core.context_max_tokens as f64 } else { 0.0 },
                "total_memories": core.memories.len(),
            }).to_string())
        }
        "reconstruct" => {
            let query = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let query_data = nlp::text_to_embedding(query, 0.5);
            let knn = core.semantic.search_similar(&query_data, 5);
            let mut reconstructed = Vec::new();
            for k in &knn {
                if let Some(mem) = core
                    .memories
                    .iter()
                    .find(|m| m.anchor().basis() == k.atom_key.vertices_basis[0])
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
                    "current_window_entries": core.context_window.len(),
                })
                .to_string(),
            )
        }
        "pre_work" => {
            let query = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let query_data = nlp::text_to_embedding(query, 0.6);
            let ideal_anchor = core
                .clustering
                .compute_ideal_anchor(&query_data, &core.universe);
            let ideal_phys = ideal_anchor.physical();

            let mut recent: Vec<Value> = Vec::new();
            let mut all_anchors: Vec<_> = core
                .memories
                .iter()
                .map(|m| (m.anchor().physical(), *m.anchor(), m.created_at()))
                .collect();
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
                    if let Some(mem) = core.memories.iter().find(|m| m.anchor() == anchor) {
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
                let knn = core.semantic.search_similar(&query_data, 10);
                for k in &knn {
                    if recent.len() >= 5 {
                        break;
                    }
                    let anchor = Coord7D::new_even(k.atom_key.vertices_basis[0]);
                    if used_basis.contains(&anchor.basis()) {
                        continue;
                    }
                    used_basis.insert(anchor.basis());
                    if let Some(mem) = core
                        .memories
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

            let knn_seed = core.semantic.search_similar(&query_data, 3);
            if let Some(seed_k) = knn_seed.first() {
                let seed_anchor = Coord7D::new_even(seed_k.atom_key.vertices_basis[0]);
                let h_neighbors = core.hebbian.get_neighbors(&seed_anchor);
                for (coord, weight) in h_neighbors.iter().take(3) {
                    if recent.len() >= 8 {
                        break;
                    }
                    if let Some(mem) = core.memories.iter().find(|m| m.anchor() == coord) {
                        let desc = mem.description().unwrap_or("").to_string();
                        if !desc.is_empty() && !used_basis.contains(&mem.anchor().basis()) {
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
                    core.context_window.push(super::core::ContextEntry {
                        role: "system".to_string(),
                        content: format!("[activated memory] {}", desc),
                        token_estimate: desc.split_whitespace().count() * 2,
                        memory_id: None,
                    });
                }
            }

            let current_tokens: usize = core.context_window.iter().map(|e| e.token_estimate).sum();
            super::protocol::ToolCallResult::ok(
                json!({
                    "action": "pre_work",
                    "query": query,
                    "activated_memories": recent.len(),
                    "memories": recent,
                    "context_entries": core.context_window.len(),
                    "current_tokens": current_tokens,
                    "max_tokens": core.context_max_tokens,
                })
                .to_string(),
            )
        }
        "clear" => {
            let count = core.context_window.len();
            core.context_window.clear();
            super::protocol::ToolCallResult::ok(
                json!({ "action": "clear", "cleared_entries": count, "memories_preserved": core.memories.len() }).to_string(),
            )
        }
        _ => super::protocol::ToolCallResult::err(format!(
            "unknown context action: {}. Use add/status/reconstruct/clear/pre_work",
            action
        )),
    }
}

fn handle_reason(args: &Value, core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
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
            let analogies = crate::universe::reasoning::ReasoningEngine::find_analogies(
                &core.universe,
                &core.memories,
                threshold,
            );
            let results: Vec<Value> = analogies
                .iter()
                .map(|r| {
                    let desc_source = core
                        .memories
                        .iter()
                        .find(|m| format!("{}", m.anchor()) == r.source)
                        .and_then(|m| m.description().map(String::from))
                        .unwrap_or_default();
                    let desc_target = r
                        .targets
                        .first()
                        .and_then(|t| {
                            core.memories
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
                json!({ "method": "analogies", "threshold": threshold, "analogies_found": results.len(), "results": results }).to_string(),
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
            let max_hops = args.get("max_hops").and_then(|v| v.as_u64()).unwrap_or(15) as usize;
            let chain = crate::universe::reasoning::ReasoningEngine::infer_chain(
                &core.universe,
                &core.hebbian,
                &from,
                &to,
                max_hops,
            );
            let hops: Vec<Value> = chain.iter().map(|r| {
                json!({ "from": r.source, "to": r.targets.first().unwrap_or(&"".into()), "confidence": r.confidence, "hop": r.hops })
            }).collect();
            super::protocol::ToolCallResult::ok(
                json!({ "method": "infer_chain", "from": format!("{}", from), "to": format!("{}", to), "chain_length": hops.len(), "found": !hops.is_empty(), "hops": hops }).to_string(),
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
            let discoveries = crate::universe::reasoning::ReasoningEngine::discover(
                &core.universe,
                &mut core.hebbian,
                &seed,
                0.5,
            );
            let results: Vec<Value> = discoveries.iter().map(|r| {
                let desc = r.targets.first().and_then(|t| {
                    core.memories.iter().find(|m| format!("{}", m.anchor()) == *t)
                        .and_then(|m| m.description().map(String::from))
                }).unwrap_or_default();
                json!({ "from": r.source, "discovered": r.targets.first().unwrap_or(&"".into()), "description": desc, "confidence": r.confidence })
            }).collect();
            super::protocol::ToolCallResult::ok(
                json!({ "method": "discover", "seed": format!("{}", seed), "discoveries": results.len(), "results": results }).to_string(),
            )
        }
        _ => super::protocol::ToolCallResult::err("method must be analogies/infer_chain/discover"),
    }
}

fn handle_emotion(core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
    let report = crate::universe::cognitive::emotion::EmotionReport::analyze(&core.universe);
    let budget = crate::universe::cognitive::perception::PerceptionBudget::new(
        core.universe.stats().total_energy,
    );
    let perception_report = budget.report();
    super::protocol::ToolCallResult::ok(
        json!({
            "pad": {
                "pleasure": report.pad.pleasure,
                "arousal": report.pad.arousal,
                "dominance": report.pad.dominance,
                "magnitude": report.pad.magnitude(),
                "quadrant": format!("{:?}", report.quadrant),
                "dominance_label": report.pad.dominance_label(),
            },
            "functional_emotion": {
                "cluster": report.functional_cluster,
                "valence": report.functional_valence,
                "arousal_level": report.functional_arousal,
                "is_positive": report.is_positive,
                "is_high_arousal": report.is_high_arousal,
            },
            "recommendations": {
                "pulse_strategy": format!("{:?}", report.pulse_suggestion),
                "dream_frequency_multiplier": report.dream_frequency_multiplier,
                "crystal_threshold_modifier": report.crystal_threshold_modifier,
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
                "energy_utilization": report.energy_utilization,
                "manifested_ratio": report.manifested_ratio,
            },
        })
        .to_string(),
    )
}

fn handle_scale(args: &Value, core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
    let action = match args.get("action").and_then(|v| v.as_str()) {
        Some(a) => a,
        None => return super::protocol::ToolCallResult::err("action is required: auto/frontier"),
    };
    let scaler = crate::universe::adaptive::autoscale::AutoScaler::new();
    match action {
        "auto" => {
            let report = scaler.auto_scale(&mut core.universe, &core.hebbian, &core.memories);
            super::protocol::ToolCallResult::ok(
                json!({
                    "action": "auto_scale",
                    "energy_expanded_by": report.energy_expanded_by,
                    "nodes_added": report.nodes_added,
                    "nodes_removed": report.nodes_removed,
                    "rebalanced": report.rebalanced,
                    "reason": format!("{:?}", report.reason),
                    "conservation_ok": core.universe.verify_conservation(),
                })
                .to_string(),
            )
        }
        "frontier" => {
            let max_new = args
                .get("max_new_nodes")
                .and_then(|v| v.as_u64())
                .unwrap_or(200) as usize;
            let report = scaler.frontier_expansion(&mut core.universe, max_new);
            super::protocol::ToolCallResult::ok(
                json!({
                    "action": "frontier_expansion",
                    "energy_expanded_by": report.energy_expanded_by,
                    "nodes_added": report.nodes_added,
                    "nodes_removed": report.nodes_removed,
                    "rebalanced": report.rebalanced,
                    "conservation_ok": core.universe.verify_conservation(),
                })
                .to_string(),
            )
        }
        _ => super::protocol::ToolCallResult::err("action must be auto or frontier"),
    }
}

fn handle_watchdog(core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
    let mut watchdog = crate::universe::adaptive::watchdog::Watchdog::with_defaults(
        core.universe.stats().total_energy,
    );
    let report = watchdog.checkup(
        &mut core.universe,
        &mut core.hebbian,
        &mut core.crystal,
        &core.memories,
    );
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
            },
            "elapsed_ms": report.elapsed_ms,
            "actions_available": report.actions.iter().map(|a| a.action.clone()).collect::<Vec<_>>(),
        })
        .to_string(),
    )
}

fn handle_forget(args: &Value, core: &mut TetraMemCore) -> super::protocol::ToolCallResult {
    let anchor = match parse_3d_coord(args, "anchor") {
        Ok(c) => c,
        Err(e) => return super::protocol::ToolCallResult::err(e),
    };
    match core.forget(&anchor) {
        Ok(result) => super::protocol::ToolCallResult::ok(result.to_string()),
        Err(e) => super::protocol::ToolCallResult::err(e),
    }
}

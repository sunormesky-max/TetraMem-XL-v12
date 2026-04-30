use serde_json::{json, Value};

use super::protocol::{ResourceDefinition, ToolDefinition};
use crate::universe::coord::Coord7D;
use crate::universe::crystal::CrystalEngine;
use crate::universe::dream::DreamEngine;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::{MemoryAtom, MemoryCodec};
use crate::universe::node::DarkUniverse;
use crate::universe::pulse::{PulseEngine, PulseType};
use crate::universe::reasoning::ReasoningEngine;
use crate::universe::regulation::RegulationEngine;
use crate::universe::topology::TopologyEngine;

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

    pub fn handle_tool(
        name: &str,
        args: &Value,
        universe: &mut DarkUniverse,
        hebbian: &mut HebbianMemory,
        memories: &mut Vec<MemoryAtom>,
        crystal: &mut CrystalEngine,
    ) -> super::protocol::ToolCallResult {
        match name {
            "tetramem_stats" => {
                let stats = universe.stats();
                let drift = universe.energy_drift();
                super::protocol::ToolCallResult::ok(json!({
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
                }).to_string())
            }
            "tetramem_health" => {
                let report = crate::universe::observer::UniverseObserver::inspect(universe, hebbian, memories);
                super::protocol::ToolCallResult::ok(json!({
                    "health_level": report.health_level().as_str(),
                    "node_count": report.node_count,
                    "energy_utilization": report.energy_utilization,
                    "conservation_ok": report.conservation_ok,
                    "hebbian_edge_count": report.hebbian_edge_count,
                    "memory_count": report.memory_count,
                }).to_string())
            }
            "tetramem_encode" => {
                let anchor = match args.get("anchor").and_then(|v| v.as_array()) {
                    Some(a) if a.len() == 3 => {
                        let coords: Result<Vec<i32>, _> = a.iter().map(|v| v.as_i64().map(|n| n as i32).ok_or(())).collect();
                        match coords {
                            Ok(c) => Coord7D::new_even([c[0], c[1], c[2], 0, 0, 0, 0]),
                            Err(_) => return super::protocol::ToolCallResult::err("anchor must be 3 integers"),
                        }
                    }
                    _ => return super::protocol::ToolCallResult::err("anchor must be array of 3 integers"),
                };
                let data: Vec<f64> = match args.get("data").and_then(|v| v.as_array()) {
                    Some(a) => a.iter().filter_map(|v| v.as_f64()).collect(),
                    None => return super::protocol::ToolCallResult::err("data must be array of numbers"),
                };
                if data.is_empty() || data.len() > 28 {
                    return super::protocol::ToolCallResult::err("data must have 1-28 values");
                }
                match MemoryCodec::encode(universe, &anchor, &data) {
                    Ok(mem) => {
                        let anchor_str = format!("{}", mem.anchor());
                        let dim = mem.data_dim();
                        memories.push(mem);
                        super::protocol::ToolCallResult::ok(json!({
                            "success": true,
                            "anchor": anchor_str,
                            "dimensions": dim,
                            "conservation_ok": universe.verify_conservation(),
                        }).to_string())
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
                    Some(mem) => {
                        match MemoryCodec::decode(universe, mem) {
                            Ok(data) => super::protocol::ToolCallResult::ok(json!({
                                "anchor": format!("{}", mem.anchor()),
                                "data": data,
                                "dimensions": mem.data_dim(),
                            }).to_string()),
                            Err(e) => super::protocol::ToolCallResult::err(format!("decode failed: {}", e)),
                        }
                    }
                    None => super::protocol::ToolCallResult::err(format!("no memory at {:?}", anchor.basis())),
                }
            }
            "tetramem_list_memories" => {
                let list: Vec<Value> = memories.iter().map(|m| {
                    json!({
                        "anchor": format!("{}", m.anchor()),
                        "dimensions": m.data_dim(),
                        "created_at": m.created_at(),
                    })
                }).collect();
                super::protocol::ToolCallResult::ok(json!({
                    "count": list.len(),
                    "memories": list,
                }).to_string())
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
                    _ => return super::protocol::ToolCallResult::err("pulse_type must be reinforcing/exploratory/cascade"),
                };
                let engine = PulseEngine::new();
                let result = engine.propagate(&source, pulse_type, universe, hebbian);
                super::protocol::ToolCallResult::ok(json!({
                    "visited_nodes": result.visited_nodes,
                    "total_activation": result.total_activation,
                    "paths_recorded": result.paths_recorded,
                    "final_strength": result.final_strength,
                }).to_string())
            }
            "tetramem_dream" => {
                let engine = DreamEngine::new();
                let report = engine.dream(universe, hebbian, memories);
                super::protocol::ToolCallResult::ok(json!({
                    "phases": format!("{}", report),
                    "edges_before": report.hebbian_edges_before,
                    "edges_after": report.hebbian_edges_after,
                    "weight_before": report.weight_before,
                    "weight_after": report.weight_after,
                    "conservation_ok": universe.verify_conservation(),
                }).to_string())
            }
            "tetramem_topology" => {
                let report = TopologyEngine::analyze(universe);
                super::protocol::ToolCallResult::ok(json!({
                    "betti": format!("{}", report.betti),
                    "connected_components": report.connected_components,
                    "cycles_detected": report.cycles_detected,
                    "tetrahedra_count": report.tetrahedra_count,
                    "bridging_nodes": report.bridging_nodes,
                    "isolated_nodes": report.isolated_nodes,
                    "average_coordination": report.average_coordination,
                    "euler_characteristic": report.betti.euler_characteristic(),
                }).to_string())
            }
            "tetramem_regulate" => {
                let engine = RegulationEngine::new();
                let report = engine.regulate(universe, hebbian, crystal, memories);
                super::protocol::ToolCallResult::ok(json!({
                    "stress_level": report.stress_level,
                    "entropy": report.entropy,
                    "imbalance": report.dimension_pressure.imbalance,
                    "actions_taken": report.actions.len(),
                    "conservation_ok": universe.verify_conservation(),
                }).to_string())
            }
            "tetramem_trace" => {
                let anchor = match parse_3d_coord(args, "anchor") {
                    Ok(c) => c,
                    Err(e) => return super::protocol::ToolCallResult::err(e),
                };
                let max_hops = args.get("max_hops").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                let associations = ReasoningEngine::find_associations(universe, hebbian, crystal, &anchor, max_hops);
                let results: Vec<Value> = associations.iter().map(|a| {
                    json!({
                        "source": a.source,
                        "targets": a.targets,
                        "confidence": a.confidence,
                        "hops": a.hops,
                    })
                }).collect();
                super::protocol::ToolCallResult::ok(json!({
                    "associations": results,
                    "total": results.len(),
                }).to_string())
            }
            "tetramem_phase_detect" => {
                let report = crystal.detect_phase_transition(hebbian, universe);
                super::protocol::ToolCallResult::ok(json!({
                    "super_channel_candidates": report.super_channel_candidates,
                    "existing_super_channels": report.existing_super_channels,
                    "avg_edge_weight": report.avg_edge_weight,
                    "phase_coherent": report.phase_coherent,
                    "requires_consensus": report.requires_consensus,
                }).to_string())
            }
            "tetramem_materialize" => {
                let coord = match parse_3d_coord(args, "coord") {
                    Ok(c) => c,
                    Err(e) => return super::protocol::ToolCallResult::err(e),
                };
                let energy = match args.get("energy").and_then(|v| v.as_f64()) {
                    Some(e) if e > 0.0 => e,
                    _ => return super::protocol::ToolCallResult::err("energy must be a positive number"),
                };
                let ratio = match args.get("physical_ratio").and_then(|v| v.as_f64()) {
                    Some(r) if r >= 0.0 && r <= 1.0 => r,
                    _ => return super::protocol::ToolCallResult::err("physical_ratio must be between 0.0 and 1.0"),
                };
                match universe.materialize_biased(coord, energy, ratio) {
                    Ok(_) => super::protocol::ToolCallResult::ok(json!({
                        "success": true,
                        "coord": format!("{}", coord),
                        "energy": energy,
                        "conservation_ok": universe.verify_conservation(),
                    }).to_string()),
                    Err(e) => super::protocol::ToolCallResult::err(format!("materialize failed: {}", e)),
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
                    }).to_string(),
                })
            }
            "tetramem://health" => {
                let report = crate::universe::observer::UniverseObserver::inspect(universe, hebbian, memories);
                Some(super::protocol::ResourceContent {
                    uri: uri.into(),
                    mime_type: Some("application/json".into()),
                    text: json!({
                        "health_level": report.health_level().as_str(),
                        "conservation_ok": report.conservation_ok,
                    }).to_string(),
                })
            }
            _ => None,
        }
    }
}

fn parse_3d_coord(args: &Value, key: &str) -> Result<Coord7D, String> {
    match args.get(key).and_then(|v| v.as_array()) {
        Some(a) if a.len() == 3 => {
            let coords: Result<Vec<i32>, _> = a.iter()
                .map(|v| v.as_i64().map(|n| n as i32).ok_or(()))
                .collect();
            match coords {
                Ok(c) => Ok(Coord7D::new_even([c[0], c[1], c[2], 0, 0, 0, 0])),
                Err(_) => Err(format!("{} must be 3 integers", key)),
            }
        }
        _ => Err(format!("{} must be array of 3 integers", key)),
    }
}

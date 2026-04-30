use serde_json::{json, Value};

use super::types::*;
use crate::universe::coord::Coord7D;
use crate::universe::dream::DreamEngine;
use crate::universe::memory::MemoryCodec;
use crate::universe::pulse::{PulseEngine, PulseType};
use crate::universe::reasoning::ReasoningEngine;
use crate::universe::regulation::RegulationEngine;
use crate::universe::topology::TopologyEngine;

macro_rules! skill_impl {
    ($name:expr, $version:expr, $desc:expr, $input:expr, $output:expr, $exec:expr) => {
        pub struct Skill;
        impl crate::skills::types::Skill for Skill {
            fn signature(&self) -> crate::skills::types::SkillSignature {
                crate::skills::types::SkillSignature {
                    name: $name.into(),
                    version: $version.into(),
                    description: $desc.into(),
                    input_schema: $input,
                    output_schema: $output,
                }
            }
            fn execute(
                &self,
                ctx: &mut crate::skills::types::SkillContext,
                args: &Value,
            ) -> Result<Value, crate::skills::types::SkillError> {
                $exec(ctx, args)
            }
        }
    };
}

pub mod encode_memory {
    use super::*;
    skill_impl!(
        "encode_memory",
        "1.0.0",
        "Encode data into a memory at a 3D anchor position",
        json!({"anchor": "[i32;3]", "data": "[f64;1-28]"}),
        json!({"anchor": "String", "dimensions": "usize", "conservation_ok": "bool"}),
        |ctx: &mut SkillContext, args: &Value| {
            let anchor = parse_anchor(args)?;
            let data = parse_data(args)?;
            match MemoryCodec::encode(ctx.universe, &anchor, &data) {
                Ok(mem) => {
                    let s = format!("{}", mem.anchor());
                    let dim = mem.data_dim();
                    ctx.memories.push(mem);
                    Ok(
                        json!({"anchor": s, "dimensions": dim, "conservation_ok": ctx.universe.verify_conservation()}),
                    )
                }
                Err(e) => Err(SkillError::new("encode_memory", e.to_string())),
            }
        }
    );
}

pub mod decode_memory {
    use super::*;
    skill_impl!(
        "decode_memory",
        "1.0.0",
        "Decode a memory by its anchor position",
        json!({"anchor": "[i32;3]"}),
        json!({"data": "[f64]", "dimensions": "usize"}),
        |ctx: &mut SkillContext, args: &Value| {
            let anchor = parse_anchor(args)?;
            match ctx.memories.iter().find(|m| m.anchor() == &anchor) {
                Some(mem) => match MemoryCodec::decode(ctx.universe, mem) {
                    Ok(data) => Ok(json!({"data": data, "dimensions": mem.data_dim()})),
                    Err(e) => Err(SkillError::new("decode_memory", e.to_string())),
                },
                None => Err(SkillError::new("decode_memory", "memory not found")),
            }
        }
    );
}

pub mod fire_pulse {
    use super::*;
    skill_impl!(
        "fire_pulse",
        "1.0.0",
        "Fire a pulse through the universe lattice",
        json!({"source": "[i32;3]", "pulse_type": "reinforcing|exploratory|cascade"}),
        json!({"visited": "usize", "activation": "f64", "paths_recorded": "usize"}),
        |ctx: &mut SkillContext, args: &Value| {
            let source = parse_coord(args, "source")?;
            let pt = parse_pulse_type(args)?;
            let engine = PulseEngine::new();
            let r = engine.propagate(&source, pt, ctx.universe, ctx.hebbian);
            Ok(
                json!({"visited": r.visited_nodes, "activation": r.total_activation, "paths_recorded": r.paths_recorded}),
            )
        }
    );
}

pub mod run_dream {
    use super::*;
    skill_impl!(
        "run_dream",
        "1.0.0",
        "Run a dream cycle: replay, weaken, consolidate",
        json!({}),
        json!({"edges_before": "usize", "edges_after": "usize"}),
        |ctx: &mut SkillContext, _args: &Value| {
            let engine = DreamEngine::new();
            let r = engine.dream(ctx.universe, ctx.hebbian, ctx.memories);
            Ok(
                json!({"edges_before": r.hebbian_edges_before, "edges_after": r.hebbian_edges_after, "weight_delta": r.weight_after - r.weight_before}),
            )
        }
    );
}

pub mod analyze_topology {
    use super::*;
    skill_impl!(
        "analyze_topology",
        "1.0.0",
        "Compute Betti numbers and topology",
        json!({}),
        json!({"betti": "String", "components": "usize", "euler": "i64"}),
        |ctx: &mut SkillContext, _args: &Value| {
            let r = TopologyEngine::analyze(ctx.universe);
            Ok(
                json!({"betti": format!("{}", r.betti), "components": r.connected_components, "euler": r.betti.euler_characteristic()}),
            )
        }
    );
}

pub mod regulate_dimensions {
    use super::*;
    skill_impl!(
        "regulate_dimensions",
        "1.0.0",
        "Balance energy across 7 dimensions",
        json!({}),
        json!({"stress": "f64", "entropy": "f64"}),
        |ctx: &mut SkillContext, _args: &Value| {
            let engine = RegulationEngine::new();
            let r = engine.regulate(ctx.universe, ctx.hebbian, ctx.crystal, ctx.memories);
            Ok(json!({"stress": r.stress_level, "entropy": r.entropy, "actions": r.actions.len()}))
        }
    );
}

pub mod trace_associations {
    use super::*;
    skill_impl!(
        "trace_associations",
        "1.0.0",
        "Trace memory associations via Hebbian and crystal channels",
        json!({"anchor": "[i32;3]", "max_hops": "usize"}),
        json!({"associations": "Vec", "total": "usize"}),
        |ctx: &mut SkillContext, args: &Value| {
            let anchor = parse_anchor(args)?;
            let max_hops = args.get("max_hops").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            let results = ReasoningEngine::find_associations(
                ctx.universe,
                ctx.hebbian,
                ctx.crystal,
                &anchor,
                max_hops,
            );
            let out: Vec<Value> = results
                .iter()
                .map(|a| json!({"targets": a.targets, "confidence": a.confidence, "hops": a.hops}))
                .collect();
            Ok(json!({"associations": out, "total": out.len()}))
        }
    );
}

pub mod check_conservation {
    use super::*;
    skill_impl!(
        "check_conservation",
        "1.0.0",
        "Verify energy conservation across the entire universe",
        json!({}),
        json!({"conserved": "bool", "drift": "f64"}),
        |ctx: &mut SkillContext, _args: &Value| {
            let ok = ctx.universe.verify_conservation();
            let drift = ctx.universe.energy_drift();
            Ok(json!({"conserved": ok, "drift": drift}))
        }
    );
}

fn parse_anchor(args: &Value) -> Result<Coord7D, SkillError> {
    parse_coord(args, "anchor")
}

fn parse_coord(args: &Value, key: &str) -> Result<Coord7D, SkillError> {
    match args.get(key).and_then(|v| v.as_array()) {
        Some(a) if a.len() == 3 => {
            let c: Result<Vec<i32>, _> = a
                .iter()
                .map(|v| v.as_i64().map(|n| n as i32).ok_or(()))
                .collect();
            match c {
                Ok(v) => Ok(Coord7D::new_even([v[0], v[1], v[2], 0, 0, 0, 0])),
                Err(_) => Err(SkillError::new(
                    "parse",
                    format!("{} must be 3 integers", key),
                )),
            }
        }
        _ => Err(SkillError::new(
            "parse",
            format!("{} must be array of 3 integers", key),
        )),
    }
}

fn parse_data(args: &Value) -> Result<Vec<f64>, SkillError> {
    match args.get("data").and_then(|v| v.as_array()) {
        Some(a) => {
            let data: Vec<f64> = a.iter().filter_map(|v| v.as_f64()).collect();
            if data.is_empty() || data.len() > 28 {
                Err(SkillError::new("parse", "data must have 1-28 values"))
            } else {
                Ok(data)
            }
        }
        None => Err(SkillError::new("parse", "data must be array of numbers")),
    }
}

fn parse_pulse_type(args: &Value) -> Result<PulseType, SkillError> {
    match args.get("pulse_type").and_then(|v| v.as_str()) {
        Some("reinforcing") => Ok(PulseType::Reinforcing),
        Some("exploratory") => Ok(PulseType::Exploratory),
        Some("cascade") | Some("inhibitory") => Ok(PulseType::Cascade),
        _ => Err(SkillError::new(
            "parse",
            "pulse_type must be reinforcing/exploratory/cascade",
        )),
    }
}

pub fn register_all(registry: &mut super::registry::SkillRegistry) {
    registry.register(encode_memory::Skill);
    registry.register(decode_memory::Skill);
    registry.register(fire_pulse::Skill);
    registry.register(run_dream::Skill);
    registry.register(analyze_topology::Skill);
    registry.register(regulate_dimensions::Skill);
    registry.register(trace_associations::Skill);
    registry.register(check_conservation::Skill);
}

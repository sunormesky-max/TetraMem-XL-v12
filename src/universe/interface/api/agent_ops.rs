// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use axum::http::StatusCode;
use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::universe::coord::Coord7D;
use crate::universe::error::AppError;
use crate::universe::memory::nlp;

use super::state::SharedState;
use super::types::*;

pub async fn remember(
    State(state): State<SharedState>,
    Json(req): Json<RememberRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    let content = req.content;
    if content.trim().is_empty() {
        return Err(AppError::BadRequest("content is required".to_string()));
    }

    let tags = req.tags;
    let category = req.category.unwrap_or_else(|| "general".to_string());
    let importance = req.importance;
    let source = req.source.unwrap_or_else(|| "api".to_string());

    let data = nlp::text_to_embedding(&content, importance);

    let anchor = {
        let u = state.universe.read().await;
        let cl = state.clustering.read().await;
        cl.compute_ideal_anchor(&data, &u)
    };

    let mut u = state.universe.write().await;
    let mut mems = state.memories.write().await;
    let mut idx = state.memory_index.write().await;
    let mut sem = state.semantic.write().await;
    let mut h = state.hebbian.write().await;
    let mut cl = state.clustering.write().await;

    let encode_result = crate::universe::memory::MemoryCodec::encode(&mut u, &anchor, &data);

    let (_final_anchor, mut atom) = match encode_result {
        Ok(atom) => (anchor, atom),
        Err(_) => {
            let fallback = nlp::text_to_anchor(&content);
            match crate::universe::memory::MemoryCodec::encode(&mut u, &fallback, &data) {
                Ok(atom) => (fallback, atom),
                Err(e) => {
                    return Ok((
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::err(format!("encode failed: {}", e))),
                    ))
                }
            }
        }
    };

    for tag in &tags {
        atom.add_tag(tag);
    }
    atom.set_category(&category);
    atom.set_description(&content);
    atom.set_source(&source);
    atom.set_importance(importance);

    let anchor_str = format!("{}", atom.anchor());
    let created_at = atom.created_at();
    let manifested = atom.is_manifested(&u);

    sem.index_memory(&atom, &data);
    let similar = sem.search_similar(&data, 5);
    for hit in &similar {
        if let Some(other) = mems.iter().find(|m| {
            let mk = crate::universe::memory::AtomKey::from_atom(m);
            mk == hit.atom_key
        }) {
            let semantic_strength = hit.similarity * 1.5;
            h.boost_edge(atom.anchor(), other.anchor(), semantic_strength);
        }
    }

    cl.register_memory(*atom.anchor(), &data);

    let i = mems.len();
    mems.push(atom);
    idx.insert(anchor_str.clone(), i);

    let conservation_ok = u.verify_conservation();

    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok(json!({
            "success": true,
            "anchor": anchor_str,
            "manifested": manifested,
            "created_at": created_at,
            "conservation_ok": conservation_ok,
        }))),
    ))
}

pub async fn recall(
    State(state): State<SharedState>,
    Json(req): Json<RecallRequest>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let limit = req.limit.clamp(1, 100);
    let query_data = nlp::text_to_embedding(&req.query, 0.5);

    let ideal_anchor = {
        let u = state.universe.read().await;
        let cl = state.clustering.read().await;
        cl.compute_ideal_anchor(&query_data, &u)
    };
    let ideal_phys = ideal_anchor.physical();

    let u = state.universe.read().await;
    let mems = state.memories.read().await;
    let h = state.hebbian.read().await;

    let mut spatial_hits: Vec<(usize, f64)> = Vec::new();
    for (i, mem) in mems.iter().enumerate() {
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

    let mut hits = Vec::new();
    for &(idx, spatial_score) in &spatial_hits {
        if hits.len() >= limit {
            break;
        }
        let mem = &mems[idx];
        let nb = h.get_neighbors(mem.anchor());
        let associated: Vec<String> = nb
            .iter()
            .filter_map(|(coord, _)| {
                mems.iter()
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
        let sem = state.semantic.read().await;
        let knn = sem.search_similar(&query_data, limit - hits.len());
        for k in &knn {
            let k_anchor = Coord7D::new_even(k.atom_key.vertices_basis[0]);
            let k_str = format!("{}", k_anchor);
            if hits.iter().any(|h| h["anchor"] == k_str) {
                continue;
            }
            if let Some(mem) = mems
                .iter()
                .find(|m| m.anchor().basis() == k.atom_key.vertices_basis[0])
            {
                hits.push(json!({
                    "anchor": format!("{}", mem.anchor()),
                    "similarity": k.similarity,
                    "distance": k.distance,
                    "method": "knn",
                    "description": mem.description().unwrap_or(""),
                    "tags": mem.tags(),
                    "category": mem.category().unwrap_or(""),
                }));
            }
        }
    }

    let _ = &u;
    Ok(Json(ApiResponse::ok(json!({
        "query": req.query,
        "results": hits,
        "total": hits.len(),
    }))))
}

pub async fn associate(
    State(state): State<SharedState>,
    Json(req): Json<AssociateRequest>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let depth = req.depth.clamp(1, 20);
    let limit = req.limit.clamp(1, 50);

    let topic_data = nlp::text_to_embedding(&req.topic, 0.5);
    let ideal_anchor = {
        let u = state.universe.read().await;
        let cl = state.clustering.read().await;
        cl.compute_ideal_anchor(&topic_data, &u)
    };

    let u = state.universe.read().await;
    let mems = state.memories.read().await;
    let h = state.hebbian.read().await;
    let crystal = state.crystal.read().await;

    let seed_anchor = mems
        .iter()
        .min_by(|a, b| {
            let da = a.anchor().distance_sq(&ideal_anchor);
            let db = b.anchor().distance_sq(&ideal_anchor);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|m| *m.anchor())
        .unwrap_or_else(|| Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]));

    let associations = crate::universe::reasoning::ReasoningEngine::find_associations(
        &u,
        &h,
        &crystal,
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
                let desc = mems
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

    Ok(Json(ApiResponse::ok(json!({
        "topic": req.topic,
        "seed_anchor": format!("{}", seed_anchor),
        "associations": results,
        "total": results.len(),
    }))))
}

pub async fn consolidate(
    State(state): State<SharedState>,
    Json(req): Json<ConsolidateRequest>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let importance_threshold = req.importance_threshold;

    let report = {
        let u = state.universe.read().await;
        let mut h = state.hebbian.write().await;
        let mems = state.memories.read().await;
        crate::universe::dream::DreamEngine::new().dream(&u, &mut h, &mems)
    };

    let u = state.universe.read().await;
    let h = state.hebbian.read().await;
    let mems = state.memories.read().await;
    let mut cl = state.clustering.write().await;
    let mut h_mut = state.hebbian.write().await;

    let _cluster_report = cl.run_maintenance_cycle(&mems, &mut h_mut, &u);

    let mut weakened = 0usize;
    let mut strengthened = 0usize;
    for mem in mems.iter() {
        let neighbors = h.get_neighbors(mem.anchor());
        for (_, weight) in &neighbors {
            if *weight < importance_threshold {
                weakened += 1;
            } else {
                strengthened += 1;
            }
        }
    }

    let conservation_ok = u.verify_conservation();

    Ok(Json(ApiResponse::ok(json!({
        "dream_report": {
            "paths_replayed": report.paths_replayed,
            "paths_weakened": report.paths_weakened,
            "memories_consolidated": report.memories_consolidated,
            "hebbian_edges_before": report.hebbian_edges_before,
            "hebbian_edges_after": report.hebbian_edges_after,
            "weight_before": report.weight_before,
            "weight_after": report.weight_after,
        },
        "maintenance": {
            "weakened_edges": weakened,
            "strengthened_edges": strengthened,
        },
        "conservation_ok": conservation_ok,
    }))))
}

pub async fn context(
    State(state): State<SharedState>,
    Json(req): Json<ContextRequest>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    match req.action.as_str() {
        "status" => {
            let mems = state.memories.read().await;
            Ok(Json(ApiResponse::ok(json!({
                "total_memories": mems.len(),
            }))))
        }
        "reconstruct" => {
            let query = req.content.unwrap_or_default();
            let query_data = nlp::text_to_embedding(&query, 0.5);
            let mems = state.memories.read().await;
            let sem = state.semantic.read().await;

            let knn = sem.search_similar(&query_data, 5);
            let mut reconstructed = Vec::new();
            for k in &knn {
                if let Some(mem) = mems
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

            Ok(Json(ApiResponse::ok(json!({
                "reconstructed_context": reconstructed,
            }))))
        }
        "pre_work" => {
            let query = req.content.unwrap_or_default();
            let query_data = nlp::text_to_embedding(&query, 0.6);
            let ideal_anchor = {
                let u = state.universe.read().await;
                let cl = state.clustering.read().await;
                cl.compute_ideal_anchor(&query_data, &u)
            };
            let ideal_phys = ideal_anchor.physical();

            let mems = state.memories.read().await;
            let h = state.hebbian.read().await;
            let sem = state.semantic.read().await;

            let mut recent = Vec::new();
            let mut used_basis = std::collections::HashSet::<[i32; 7]>::new();

            let mut all_anchors: Vec<_> = mems
                .iter()
                .map(|m| (m.anchor().physical(), *m.anchor(), m.created_at()))
                .collect();
            all_anchors.sort_by_key(|b| std::cmp::Reverse(b.2));

            for (phys, anchor, _) in &all_anchors {
                if recent.len() >= 5 {
                    break;
                }
                let dx = (ideal_phys[0] - phys[0]).abs();
                let dy = (ideal_phys[1] - phys[1]).abs();
                let dz = (ideal_phys[2] - phys[2]).abs();
                if dx + dy + dz < 150 && !used_basis.contains(&anchor.basis()) {
                    used_basis.insert(anchor.basis());
                    if let Some(mem) = mems.iter().find(|m| m.anchor() == anchor) {
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
                let knn = sem.search_similar(&query_data, 10);
                for k in &knn {
                    if recent.len() >= 5 {
                        break;
                    }
                    let anchor = Coord7D::new_even(k.atom_key.vertices_basis[0]);
                    if used_basis.contains(&anchor.basis()) {
                        continue;
                    }
                    used_basis.insert(anchor.basis());
                    if let Some(mem) = mems
                        .iter()
                        .find(|m| m.anchor().basis() == k.atom_key.vertices_basis[0])
                    {
                        let desc = mem.description().unwrap_or("").to_string();
                        if !desc.is_empty() {
                            recent.push(json!({
                                "description": desc,
                                "tags": mem.tags(),
                                "method": "knn",
                                "similarity": k.similarity,
                            }));
                        }
                    }
                }
            }

            let knn_seed = sem.search_similar(&query_data, 3);
            if let Some(seed_k) = knn_seed.first() {
                let seed_anchor = Coord7D::new_even(seed_k.atom_key.vertices_basis[0]);
                let h_neighbors = h.get_neighbors(&seed_anchor);
                for (coord, weight) in h_neighbors.iter().take(3) {
                    if recent.len() >= 8 {
                        break;
                    }
                    if let Some(mem) = mems.iter().find(|m| m.anchor() == coord) {
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

            Ok(Json(ApiResponse::ok(json!({
                "pre_work_results": recent,
                "total": recent.len(),
            }))))
        }
        _ => Err(AppError::BadRequest(format!(
            "unknown context action: '{}'. Use: status, reconstruct, pre_work",
            req.action
        ))),
    }
}

pub async fn forget(
    State(state): State<SharedState>,
    Json(req): Json<ForgetRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    if req.anchor.iter().any(|&v| !(-10000..=10000).contains(&v)) {
        return Err(AppError::BadRequest(
            "coordinate values must be in [-10000, 10000]".to_string(),
        ));
    }

    let anchor = Coord7D::new_even([req.anchor[0], req.anchor[1], req.anchor[2], 0, 0, 0, 0]);
    let anchor_str = format!("{}", &anchor);

    let mut u = state.universe.write().await;
    let mut mems = state.memories.write().await;
    let mut idx = state.memory_index.write().await;

    let pos = idx.remove(&anchor_str);
    match pos {
        Some(i) => {
            if i < mems.len() {
                let desc = mems[i].description().unwrap_or("").to_string();
                crate::universe::memory::MemoryCodec::erase(&mut u, &mems[i]);
                mems.remove(i);
                for (_key, val) in idx.iter_mut() {
                    if *val > i {
                        *val -= 1;
                    }
                }
                let conservation_ok = u.verify_conservation();
                Ok((
                    StatusCode::OK,
                    Json(ApiResponse::ok(json!({
                        "success": true,
                        "erased_anchor": anchor_str,
                        "description": desc,
                        "remaining_memories": mems.len(),
                        "conservation_ok": conservation_ok,
                    }))),
                ))
            } else {
                Ok((
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::err("memory index out of bounds")),
                ))
            }
        }
        None => Ok((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::err("memory not found")),
        )),
    }
}

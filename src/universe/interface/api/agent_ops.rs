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

    super::types::validate_field_len("content", &content, super::types::MAX_STRING_FIELD_LEN)
        .map_err(AppError::BadRequest)?;

    for tag in &req.tags {
        super::types::validate_field_len("tag", tag, super::types::MAX_TAG_LEN)
            .map_err(AppError::BadRequest)?;
    }
    if req.tags.len() > super::types::MAX_TAGS_COUNT {
        return Err(AppError::BadRequest(format!(
            "too many tags (max {})",
            super::types::MAX_TAGS_COUNT
        )));
    }

    let tags = req.tags;
    let category = req.category.unwrap_or_else(|| "general".to_string());
    let importance = req.importance;
    let source = req.source.unwrap_or_else(|| "api".to_string());

    let full_embedding = nlp::text_to_embedding(&content, importance);
    let data: Vec<f64> = full_embedding.into_iter().take(28).collect();

    let anchor = {
        let u = state.universe.read().await;
        let cl = state.clustering.read().await;
        cl.compute_ideal_anchor(&data, &u)
    };

    let novelty_report = {
        let sem = state.semantic.read().await;
        let store = state.memory_store.read().await;
        let h = state.hebbian.read().await;
        let knn = sem.search_similar(&data, 5);
        let knn_distances: Vec<(f64, usize)> = knn
            .iter()
            .filter_map(|r| {
                store
                    .memories
                    .iter()
                    .position(|m| {
                        let mk = crate::universe::memory::AtomKey::from_atom(m);
                        mk == r.atom_key
                    })
                    .map(|idx| (r.distance, idx))
            })
            .collect();
        let detector = crate::universe::memory::NoveltyDetector::default();
        detector.assess(&data, &knn_distances, &anchor, &h, &store.memories)
    };

    let importance = req.importance;
    if !importance.is_finite() || !(0.0..=1.0).contains(&importance) {
        return Err(AppError::BadRequest(
            "importance must be a finite number between 0.0 and 1.0".to_string(),
        ));
    }

    {
        let store = state.memory_store.read().await;
        if store.memories.len() >= state.config.maintenance.max_memories {
            return Err(AppError::BadRequest(format!(
                "memory limit reached ({} memories)",
                state.config.maintenance.max_memories
            )));
        }
    }

    let adjusted_importance = if importance < 0.01 {
        novelty_report.suggested_importance
    } else {
        importance * 0.7 + novelty_report.suggested_importance * 0.3
    };

    let mut u = state.universe.write().await;
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
    atom.set_importance(adjusted_importance);

    let anchor_str = format!("{}", atom.anchor());
    let created_at = atom.created_at();
    let manifested = atom.is_manifested(&u);
    let conservation_ok = u.verify_conservation();
    drop(u);

    let deferred = state.config.maintenance.deferred_binding;

    let mut sem = state.semantic.write().await;
    let mut store = state.memory_store.write().await;
    let mut h = state.hebbian.write().await;
    let mut cl = state.clustering.write().await;

    sem.index_memory(&atom, &data);
    let similar = sem.search_similar(&data, 5);
    for hit in &similar {
        if let Some(other) = store.memories.iter().find(|m| {
            let mk = crate::universe::memory::AtomKey::from_atom(m);
            mk == hit.atom_key
        }) {
            let semantic_strength = hit.similarity * 1.5;
            if deferred {
                h.defer_edge(atom.anchor(), other.anchor(), semantic_strength);
            } else {
                h.boost_edge(atom.anchor(), other.anchor(), semantic_strength);
            }
        }
    }

    cl.register_memory(*atom.anchor(), &data);

    store.push(atom);

    {
        let interests = state.interests.read().await;
        let surfacer = crate::universe::memory::MemorySurfacer::default();
        let surfaced = surfacer.surface(
            &anchor,
            &h,
            &store.memories,
            &interests,
            novelty_report.score,
        );
        drop(interests);
        for mut sm in surfaced {
            sm.seq = state
                .surfaced_seq
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let _ = state.memory_stream.send(sm);
        }
    }

    drop(cl);
    drop(h);
    drop(store);
    drop(sem);

    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok(json!({
            "success": true,
            "anchor": anchor_str,
            "manifested": manifested,
            "created_at": created_at,
            "conservation_ok": conservation_ok,
            "novelty": {
                "score": novelty_report.score,
                "level": format!("{}", novelty_report.level),
                "suggested_importance": novelty_report.suggested_importance,
                "adjusted_importance": adjusted_importance,
                "wavelet_energy": novelty_report.wavelet_energy,
                "detail_energy": novelty_report.detail_energy,
                "anomaly_score": novelty_report.anomaly_score,
            },
        }))),
    ))
}

pub async fn recall(
    State(state): State<SharedState>,
    Json(req): Json<RecallRequest>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    if req.query.trim().is_empty() {
        return Err(AppError::BadRequest("query is required".to_string()));
    }
    super::types::validate_field_len("query", &req.query, super::types::MAX_STRING_FIELD_LEN)
        .map_err(AppError::BadRequest)?;

    let limit = req.limit.clamp(1, 100);
    let query_data = nlp::text_to_embedding(&req.query, 0.5);
    let tag_filter = if req.tags.is_empty() {
        None
    } else {
        match req.tag_mode.as_deref().unwrap_or("any") {
            "all" => Some(("all", req.tags.clone())),
            _ => Some(("any", req.tags.clone())),
        }
    };

    let ideal_anchor = {
        let u = state.universe.read().await;
        let cl = state.clustering.read().await;
        cl.compute_ideal_anchor(&query_data, &u)
    };
    let ideal_phys = ideal_anchor.physical();

    let u = state.universe.read().await;
    let store = state.memory_store.read().await;
    let h = state.hebbian.read().await;

    let mut spatial_hits: Vec<(usize, f64)> = Vec::new();
    for (i, mem) in store.memories.iter().enumerate() {
        if let Some(ref src) = req.source {
            if mem.source().map(|s| s != src.as_str()).unwrap_or(true) {
                continue;
            }
        }
        if let Some((mode, ref tags)) = tag_filter {
            let matches = match mode {
                "all" => tags.iter().all(|t| mem.has_tag(t)),
                _ => tags.iter().any(|t| mem.has_tag(t)),
            };
            if !matches {
                continue;
            }
        }
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
        let mem = &store.memories[idx];
        let nb = h.get_neighbors(mem.anchor());
        let associated: Vec<String> = nb
            .iter()
            .filter_map(|(coord, _)| {
                store
                    .memories
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
        let sem = state.semantic.read().await;
        let knn = sem.search_similar(&query_data, limit - hits.len());
        for k in &knn {
            let k_anchor = Coord7D::new_even(k.atom_key.vertices_basis[0]);
            let k_str = format!("{}", k_anchor);
            if hits.iter().any(|h| h["anchor"] == k_str) {
                continue;
            }
            if let Some(mem) = store
                .memories
                .iter()
                .find(|m| m.anchor().basis() == k.atom_key.vertices_basis[0])
            {
                if let Some(ref src) = req.source {
                    if mem.source().map(|s| s != src.as_str()).unwrap_or(true) {
                        continue;
                    }
                }
                if let Some((mode, ref tags)) = tag_filter {
                    let matches = match mode {
                        "all" => tags.iter().all(|t| mem.has_tag(t)),
                        _ => tags.iter().any(|t| mem.has_tag(t)),
                    };
                    if !matches {
                        continue;
                    }
                }
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
    if req.topic.trim().is_empty() {
        return Err(AppError::BadRequest("topic is required".to_string()));
    }
    super::types::validate_field_len("topic", &req.topic, super::types::MAX_STRING_FIELD_LEN)
        .map_err(AppError::BadRequest)?;

    let depth = req.depth.clamp(1, 20);
    let limit = req.limit.clamp(1, 50);

    let topic_data = nlp::text_to_embedding(&req.topic, 0.5);
    let ideal_anchor = {
        let u = state.universe.read().await;
        let cl = state.clustering.read().await;
        cl.compute_ideal_anchor(&topic_data, &u)
    };

    let u = state.universe.read().await;
    let store = state.memory_store.read().await;
    let h = state.hebbian.read().await;
    let crystal = state.crystal.read().await;

    let seed_anchor = store
        .memories
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
                let desc = store
                    .memories
                    .iter()
                    .find(|m| *m.anchor() == *t)
                    .and_then(|m| m.description().map(String::from))
                    .unwrap_or_default();
                json!({"anchor": t.to_string(), "description": desc})
            })
            .collect();
        results.push(json!({
            "source": assoc.source.to_string(),
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
        let store = state.memory_store.read().await;
        crate::universe::dream::DreamEngine::new().dream(&u, &mut h, &store.memories)
    };

    let maintenance_report = {
        let u = state.universe.read().await;
        let store = state.memory_store.read().await;
        let mut h = state.hebbian.write().await;
        let mut cl = state.clustering.write().await;
        cl.run_maintenance_cycle(&store.memories, &mut h, &u);
        (cl, h)
    };
    let (_cl, h) = maintenance_report;

    let u = state.universe.read().await;
    let store = state.memory_store.read().await;

    let mut weakened = 0usize;
    let mut strengthened = 0usize;
    for mem in store.memories.iter() {
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
    super::types::validate_field_len("action", &req.action, 64).map_err(AppError::BadRequest)?;
    if let Some(ref content) = req.content {
        super::types::validate_field_len("content", content, super::types::MAX_STRING_FIELD_LEN)
            .map_err(AppError::BadRequest)?;
    }

    match req.action.as_str() {
        "status" => {
            let store = state.memory_store.read().await;
            Ok(Json(ApiResponse::ok(json!({
                "total_memories": store.memories.len(),
            }))))
        }
        "reconstruct" => {
            let query = req.content.unwrap_or_default();
            let query_data = nlp::text_to_embedding(&query, 0.5);
            let store = state.memory_store.read().await;
            let sem = state.semantic.read().await;

            let knn = sem.search_similar(&query_data, 5);
            let mut reconstructed = Vec::new();
            for k in &knn {
                if let Some(mem) = store
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

            let mems = state.memory_store.read().await;
            let h = state.hebbian.read().await;
            let sem = state.semantic.read().await;

            let mut recent = Vec::new();
            let mut used_basis = std::collections::HashSet::<[i32; 7]>::new();

            let mut all_anchors: Vec<_> = mems
                .memories
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
                    if let Some(mem) = mems.memories.iter().find(|m| m.anchor() == anchor) {
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
                        .memories
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
                    if let Some(mem) = mems.memories.iter().find(|m| m.anchor() == coord) {
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
        "add" => {
            let content = req.content.unwrap_or_default();
            let role = req.role.unwrap_or_else(|| "user".to_string());
            if content.trim().is_empty() {
                return Err(AppError::BadRequest(
                    "content is required for add action".to_string(),
                ));
            }
            let tags = vec!["context".to_string(), format!("role:{}", role)];
            let query_data = nlp::text_to_embedding(&content, 0.5);
            let data: Vec<f64> = query_data.into_iter().take(28).collect();
            let anchor = {
                let u = state.universe.read().await;
                let cl = state.clustering.read().await;
                cl.compute_ideal_anchor(&data, &u)
            };
            let mut u = state.universe.write().await;
            let mut store = state.memory_store.write().await;
            let encode_result =
                crate::universe::memory::MemoryCodec::encode(&mut u, &anchor, &data);
            if let Ok(mut atom) = encode_result {
                for tag in &tags {
                    atom.add_tag(tag);
                }
                atom.set_description(&content);
                atom.set_importance(0.3);
                store.push(atom);
            }
            drop(store);
            drop(u);
            Ok(Json(ApiResponse::ok(json!({
                "added": true,
                "role": role,
                "content_length": content.len(),
            }))))
        }
        _ => Err(AppError::BadRequest(format!(
            "unknown context action: '{}'. Use: status, reconstruct, pre_work, add",
            req.action
        ))),
    }
}

pub async fn forget(
    State(state): State<SharedState>,
    Json(req): Json<ForgetRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    let anchor = req.anchor;
    let anchor_str = format!("{}", &anchor);

    let mut u = state.universe.write().await;
    let mut store = state.memory_store.write().await;

    let pos = store.index.get(&anchor_str).copied();
    match pos {
        Some(i) => {
            if i < store.memories.len() {
                let desc = store.memories[i].description().unwrap_or("").to_string();
                crate::universe::memory::MemoryCodec::erase(&mut u, &store.memories[i]);
                store.remove_at(i);
                let conservation_ok = u.verify_conservation();
                Ok((
                    StatusCode::OK,
                    Json(ApiResponse::ok(json!({
                        "success": true,
                        "erased_anchor": anchor_str,
                        "description": desc,
                        "remaining_memories": store.memories.len(),
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

pub async fn adjust_weight(
    State(state): State<SharedState>,
    Json(req): Json<AdjustWeightRequest>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    let boost = req.boost.clamp(-5.0, 5.0);
    if boost == 0.0 {
        return Err(AppError::BadRequest("boost must be non-zero".to_string()));
    }

    let mut h = state.hebbian.write().await;
    let old_weight = h.get_bias(&req.from, &req.to);
    let new_weight = h.adjust_edge_weight(&req.from, &req.to, boost);
    drop(h);

    Ok(Json(ApiResponse::ok(json!({
        "success": true,
        "from": format!("{}", req.from),
        "to": format!("{}", req.to),
        "old_weight": old_weight,
        "new_weight": new_weight,
        "adjustment": boost,
    }))))
}

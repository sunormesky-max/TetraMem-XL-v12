// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use axum::extract::Query;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use serde::Deserialize;

use crate::universe::coord::Coord7D;
use crate::universe::error::AppError;
use crate::universe::events::UniverseEvent;
use crate::universe::memory::{MemoryAtom, MemoryCodec};
use crate::universe::metrics;

use super::state::SharedState;
use super::types::*;

const MAX_DATA_VALUE: f64 = 1e15;

#[derive(Debug, Deserialize)]
pub struct ListMemoriesParams {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

fn validate_coord_3(c: &[i32; 3]) -> Result<(), AppError> {
    for &v in c {
        if !(-10000..=10000).contains(&v) {
            return Err(AppError::BadRequest(format!(
                "coordinate value {} out of range [-10000, 10000]",
                v
            )));
        }
    }
    Ok(())
}

pub async fn encode_memory(
    State(state): State<SharedState>,
    Json(req): Json<EncodeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<EncodeResponse>>), AppError> {
    if req.data.is_empty() || req.data.len() > 28 {
        return Err(AppError::BadRequest(format!(
            "data length must be between 1 and 28, got {}",
            req.data.len()
        )));
    }
    validate_coord_3(&req.anchor)?;
    validate_tags(&req.tags).map_err(AppError::BadRequest)?;
    if let Some(ref desc) = req.description {
        validate_field_len("description", desc, MAX_STRING_FIELD_LEN)
            .map_err(AppError::BadRequest)?;
    }
    if let Some(ref cat) = req.category {
        validate_field_len("category", cat, MAX_TAG_LEN).map_err(AppError::BadRequest)?;
    }
    for &v in &req.data {
        if !v.is_finite() {
            return Err(AppError::BadRequest(
                "data values must be finite".to_string(),
            ));
        }
        if v.abs() > MAX_DATA_VALUE {
            return Err(AppError::BadRequest(format!(
                "data value {} exceeds maximum allowed magnitude",
                v
            )));
        }
    }
    {
        let con = state.constitution.read().await;
        let check = con.validate_operation("materialize");
        if !check.allowed {
            return Err(AppError::Forbidden(format!(
                "constitution blocks encode: {}",
                check.violations.join("; ")
            )));
        }
    }

    let anchor = Coord7D::new_even([req.anchor[0], req.anchor[1], req.anchor[2], 0, 0, 0, 0]);

    tracing::info!(anchor = %anchor, dims = req.data.len(), "encoding memory");
    if let Some(c) = metrics::API_ENCODE_TOTAL.get() {
        c.inc();
    }

    let (novelty_report, knn_cache) = {
        let sem = state.semantic.read().await;
        let store = state.memory_store.read().await;
        let h = state.hebbian.read().await;
        let knn = sem.search_similar(&req.data, 5);
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
        let knn_cache: Vec<(crate::universe::memory::AtomKey, f64)> = knn
            .into_iter()
            .map(|r| (r.atom_key, r.similarity))
            .collect();
        let detector = crate::universe::memory::NoveltyDetector::default();
        let report = detector.assess(&req.data, &knn_distances, &anchor, &h, &store.memories);
        (report, knn_cache)
    };

    let adjusted_importance = if req.importance < 0.01 {
        novelty_report.suggested_importance
    } else {
        req.importance * 0.7 + novelty_report.suggested_importance * 0.3
    };

    let mut u = state.universe.write().await;
    let mut store = state.memory_store.write().await;

    let encode_result = match MemoryCodec::encode(&mut u, &anchor, &req.data) {
        Ok(mut atom) => {
            for tag in &req.tags {
                atom.add_tag(tag);
            }
            if let Some(ref cat) = req.category {
                atom.set_category(cat);
            }
            if let Some(ref desc) = req.description {
                atom.set_description(desc);
            }
            if let Some(ref src) = req.source {
                atom.set_source(src);
            }
            atom.set_importance(adjusted_importance);

            let manifested = atom.is_manifested(&u);
            let anchor_str = format!("{}", atom.anchor());
            let created_at = atom.created_at();
            let data_dim = req.data.len();
            let anchor_7 = atom.anchor().basis();
            let importance = atom.importance();

            drop(u);

            {
                let mut sem = state.semantic.write().await;
                sem.index_memory(&atom, &req.data);
                drop(sem);

                if !knn_cache.is_empty() {
                    let mut h = state.hebbian.write().await;
                    for (hit_key, hit_sim) in &knn_cache {
                        if let Some(other) = store.memories.iter().find(|m| {
                            let mk = crate::universe::memory::AtomKey::from_atom(m);
                            mk == *hit_key
                        }) {
                            let semantic_strength = hit_sim * 1.5;
                            h.boost_edge(atom.anchor(), other.anchor(), semantic_strength);
                        }
                    }
                }
            }

            store.push(atom);
            state.event_sender.publish(UniverseEvent::MemoryEncoded {
                anchor: anchor_7,
                data_dim,
                importance,
            });

            {
                let interests = state.interests.read().await;
                let h_surf = state.hebbian.read().await;
                let surfacer = crate::universe::memory::MemorySurfacer::default();
                let surfaced = surfacer.surface(
                    &anchor,
                    &h_surf,
                    &store.memories,
                    &interests,
                    novelty_report.score,
                );
                drop(interests);
                drop(h_surf);
                for mut sm in surfaced {
                    sm.seq = state
                        .surfaced_seq
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let _ = state.memory_stream.send(sm);
                }
            }

            Ok((
                StatusCode::OK,
                Json(ApiResponse::ok(EncodeResponse {
                    anchor: anchor_str,
                    data_dim: req.data.len(),
                    manifested,
                    created_at,
                    novelty: Some(NoveltyInfo {
                        score: novelty_report.score,
                        level: format!("{}", novelty_report.level),
                        suggested_importance: novelty_report.suggested_importance,
                        adjusted_importance,
                        wavelet_energy: novelty_report.wavelet_energy,
                        detail_energy: novelty_report.detail_energy,
                        anomaly_score: novelty_report.anomaly_score,
                    }),
                })),
            ))
        }
        Err(e) => {
            drop(u);
            tracing::warn!(error = %e, "memory encode failed");
            Ok((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::err(format!("encode failed: {}", e))),
            ))
        }
    };

    encode_result
}

pub async fn decode_memory(
    State(state): State<SharedState>,
    Json(req): Json<DecodeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<DecodeResponse>>), AppError> {
    validate_coord_3(&req.anchor)?;
    let u = state.universe.read().await;
    let store = state.memory_store.read().await;

    if let Some(c) = metrics::API_DECODE_TOTAL.get() {
        c.inc();
    }
    let anchor = Coord7D::new_even([req.anchor[0], req.anchor[1], req.anchor[2], 0, 0, 0, 0]);
    let anchor_str = format!("{}", &anchor);

    if let Some(&i) = store.index.get(&anchor_str) {
        if let Some(mem) = store.memories.get(i) {
            if mem.data_dim() == req.data_dim {
                match MemoryCodec::decode(&u, mem) {
                    Ok(data) => {
                        tracing::debug!(anchor = %anchor, dims = data.len(), "memory decoded");
                        return Ok((
                            StatusCode::OK,
                            Json(ApiResponse::ok(DecodeResponse { data })),
                        ));
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "memory decode failed");
                        return Ok((
                            StatusCode::BAD_REQUEST,
                            Json(ApiResponse::err(format!("decode failed: {}", e))),
                        ));
                    }
                }
            }
        }
    }

    Ok((
        StatusCode::NOT_FOUND,
        Json(ApiResponse::err("memory not found")),
    ))
}

pub async fn list_memories(
    State(state): State<SharedState>,
    Query(params): Query<ListMemoriesParams>,
) -> Json<ApiResponseWithMeta<Vec<MemoryListItem>>> {
    let store = state.memory_store.read().await;
    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(100).min(1000);
    let total = store.memories.len();
    let list: Vec<MemoryListItem> = store
        .memories
        .iter()
        .skip(offset)
        .take(limit)
        .map(|m| MemoryListItem {
            anchor: format!("{}", m.anchor()),
            data_dim: m.data_dim(),
            created_at: m.created_at(),
            tags: m.tags().to_vec(),
            category: m.category().map(String::from),
            description: m.description().map(String::from),
            importance: m.importance(),
        })
        .collect();
    Json(ApiResponseWithMeta::ok(
        list,
        serde_json::json!({"offset": offset, "limit": limit, "total": total}),
    ))
}

pub async fn memory_timeline(
    State(state): State<SharedState>,
) -> Json<ApiResponse<Vec<TimelineDay>>> {
    let store = state.memory_store.read().await;
    let mut day_map: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for m in store.memories.iter() {
        let ts = if m.created_at() > 0 {
            m.created_at()
        } else {
            0
        };
        let date = if ts > 0 {
            chrono::DateTime::from_timestamp_millis(ts as i64)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            "unknown".to_string()
        };
        day_map
            .entry(date)
            .or_default()
            .push(format!("{}", m.anchor()));
    }
    let max_days = state.config.universe.max_timeline_days;
    let timeline: Vec<TimelineDay> = day_map
        .into_iter()
        .rev()
        .take(max_days)
        .map(|(date, anchors)| TimelineDay {
            count: anchors.len(),
            date,
            anchors,
        })
        .collect();
    Json(ApiResponse::ok(timeline))
}

pub async fn memory_trace(
    State(state): State<SharedState>,
    Json(req): Json<TraceRequest>,
) -> Result<Json<ApiResponse<Vec<TraceHop>>>, AppError> {
    validate_coord_3(&req.anchor)?;
    let u = state.universe.read().await;
    let h = state.hebbian.read().await;
    let crystal = state.crystal.read().await;
    let store = state.memory_store.read().await;

    let source = Coord7D::new_even([req.anchor[0], req.anchor[1], req.anchor[2], 0, 0, 0, 0]);
    let max_hops = req.max_hops.unwrap_or(10).min(100);

    let associations = crate::universe::reasoning::ReasoningEngine::find_associations(
        &u, &h, &crystal, &source, max_hops,
    );

    let mut hops: Vec<TraceHop> = Vec::new();

    let mem_index: std::collections::HashMap<String, &MemoryAtom> = store
        .memories
        .iter()
        .map(|m| (format!("{}", m.anchor()), m))
        .collect();

    let source_str = format!("{}", source);
    if let Some(m) = mem_index.get(&source_str) {
        hops.push(TraceHop {
            anchor: source_str,
            created_at: m.created_at(),
            data_dim: m.data_dim(),
            confidence: 1.0,
            hop: 0,
        });
    }

    for r in &associations {
        for target_str in &r.targets {
            if let Some(m) = mem_index.get(target_str) {
                hops.push(TraceHop {
                    anchor: target_str.clone(),
                    created_at: m.created_at(),
                    data_dim: m.data_dim(),
                    confidence: r.confidence,
                    hop: r.hops,
                });
            }
        }
    }

    Ok(Json(ApiResponse::ok(hops)))
}

pub async fn annotate_memory(
    State(state): State<SharedState>,
    Json(req): Json<AnnotateRequest>,
) -> Result<(StatusCode, Json<ApiResponse<AnnotateResponse>>), AppError> {
    validate_coord_3(&req.anchor)?;
    validate_tags(&req.tags).map_err(AppError::BadRequest)?;
    if let Some(ref desc) = req.description {
        validate_field_len("description", desc, MAX_STRING_FIELD_LEN)
            .map_err(AppError::BadRequest)?;
    }
    if let Some(ref cat) = req.category {
        validate_field_len("category", cat, MAX_TAG_LEN).map_err(AppError::BadRequest)?;
    }
    let mut store = state.memory_store.write().await;
    let anchor = Coord7D::new_even([req.anchor[0], req.anchor[1], req.anchor[2], 0, 0, 0, 0]);
    let anchor_str = format!("{}", &anchor);

    if let Some(&i) = store.index.get(&anchor_str) {
        if let Some(mem) = store.memories.get_mut(i) {
            for tag in &req.tags {
                mem.add_tag(tag);
            }
            if let Some(ref cat) = req.category {
                mem.set_category(cat);
            }
            if let Some(ref desc) = req.description {
                mem.set_description(desc);
            }
            if let Some(ref src) = req.source {
                mem.set_source(src);
            }
            {
                let mut guard = state.identity_guard.write().await;
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let report = guard.check_importance_change(mem, req.importance, now_ms);
                mem.set_importance(report.allowed_importance);
                if report.protected {
                    tracing::warn!(
                        anchor = %anchor_str,
                        requested = req.importance,
                        allowed = report.allowed_importance,
                        reason = %report.reason,
                        "identity guard: importance change protected"
                    );
                }
            }

            let resp = AnnotateResponse {
                anchor: anchor_str,
                tags: mem.tags().to_vec(),
                category: mem.category().map(String::from),
                description: mem.description().map(String::from),
                source: mem.source().map(String::from),
                importance: mem.importance(),
            };
            Ok((StatusCode::OK, Json(ApiResponse::ok(resp))))
        } else {
            Ok((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::err("memory index out of bounds")),
            ))
        }
    } else {
        Ok((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::err("memory not found")),
        ))
    }
}

pub async fn semantic_search(
    State(state): State<SharedState>,
    Json(req): Json<SemanticSearchRequest>,
) -> Json<ApiResponse<SemanticSearchResponse>> {
    let sem = state.semantic.read().await;
    let store = state.memory_store.read().await;
    let k = req.k.clamp(1, 100);

    let results = sem.search_similar(&req.data, k);
    let hits: Vec<SemanticHit> = results
        .into_iter()
        .filter_map(|r| {
            let mem = store.memories.iter().find(|m| {
                let mk = crate::universe::memory::AtomKey::from_atom(m);
                mk == r.atom_key
            })?;
            if let Some(ref src) = req.source {
                if mem.source().map(|s| s != src.as_str()).unwrap_or(true) {
                    return None;
                }
            }
            let anchor_str = format!("{}", mem.anchor());
            Some(SemanticHit {
                anchor: anchor_str,
                similarity: r.similarity,
                distance: r.distance,
                tags: mem.tags().to_vec(),
                category: mem.category().map(String::from),
                description: mem.description().map(String::from),
                importance: mem.importance(),
            })
        })
        .collect();

    Json(ApiResponse::ok(SemanticSearchResponse { results: hits }))
}

pub async fn semantic_text_query(
    State(state): State<SharedState>,
    Json(req): Json<SemanticTextQueryRequest>,
) -> Json<ApiResponse<SemanticSearchResponse>> {
    let sem = state.semantic.read().await;
    let store = state.memory_store.read().await;
    let k = req.k.clamp(1, 100);
    let text = if req.text.len() > MAX_STRING_FIELD_LEN {
        &req.text[..MAX_STRING_FIELD_LEN]
    } else {
        &req.text
    };

    let knn_results = sem.search_by_text(text, k);

    let hits: Vec<SemanticHit> = knn_results
        .into_iter()
        .filter_map(|r| {
            let mem = store.memories.iter().find(|m| {
                let mk = crate::universe::memory::AtomKey::from_atom(m);
                mk == r.atom_key
            })?;
            let anchor_str = format!("{}", mem.anchor());
            Some(SemanticHit {
                anchor: anchor_str,
                similarity: r.similarity,
                distance: r.distance,
                tags: mem.tags().to_vec(),
                category: mem.category().map(String::from),
                description: mem.description().map(String::from),
                importance: mem.importance(),
            })
        })
        .collect();

    Json(ApiResponse::ok(SemanticSearchResponse { results: hits }))
}

pub async fn semantic_relations(
    State(state): State<SharedState>,
    Json(req): Json<SemanticRelationRequest>,
) -> Result<Json<ApiResponse<SemanticRelationResponse>>, AppError> {
    validate_coord_3(&req.anchor)?;
    let sem = state.semantic.read().await;
    let store = state.memory_store.read().await;
    let anchor = Coord7D::new_even([req.anchor[0], req.anchor[1], req.anchor[2], 0, 0, 0, 0]);
    let anchor_str = format!("{}", &anchor);

    if let Some(&i) = store.index.get(&anchor_str) {
        if let Some(mem) = store.memories.get(i) {
            let relations = sem.find_relations(mem);
            let rel_infos: Vec<RelationInfo> = relations
                .iter()
                .map(|r| {
                    let from_str = store
                        .memories
                        .iter()
                        .find(|m| {
                            let mk = crate::universe::memory::AtomKey::from_atom(m);
                            mk == r.from
                        })
                        .map(|m| format!("{}", m.anchor()))
                        .unwrap_or_else(|| "?".to_string());
                    let to_str = store
                        .memories
                        .iter()
                        .find(|m| {
                            let mk = crate::universe::memory::AtomKey::from_atom(m);
                            mk == r.to
                        })
                        .map(|m| format!("{}", m.anchor()))
                        .unwrap_or_else(|| "?".to_string());
                    RelationInfo {
                        from_anchor: from_str,
                        to_anchor: to_str,
                        relation_type: format!("{:?}", r.rel_type),
                        weight: r.weight,
                    }
                })
                .collect();
            return Ok(Json(ApiResponse::ok(SemanticRelationResponse {
                anchor: anchor_str,
                relations: rel_infos,
            })));
        }
    }
    Ok(Json(ApiResponse::ok(SemanticRelationResponse {
        anchor: anchor_str,
        relations: vec![],
    })))
}

pub async fn temporal_predict(
    State(state): State<SharedState>,
    Json(req): Json<PredictRequest>,
) -> Result<Json<ApiResponse<PredictResponse>>, AppError> {
    validate_coord_3(&req.anchor)?;
    let max_steps = req.max_steps.unwrap_or(5).min(20);
    let anchor = Coord7D::new_even([req.anchor[0], req.anchor[1], req.anchor[2], 0, 0, 0, 0]);

    let h = state.hebbian.read().await;
    let store = state.memory_store.read().await;
    let sequence = h.get_temporal_sequence(&anchor, max_steps);

    let predictions: Vec<PredictedMemory> = sequence
        .into_iter()
        .enumerate()
        .filter_map(|(step, (coord, strength))| {
            let mem = store.memories.iter().find(|m| m.anchor() == &coord)?;
            Some(PredictedMemory {
                anchor: format!("{}", mem.anchor()),
                step: step + 1,
                temporal_strength: strength,
                avg_delay_ms: 0.0,
                description: mem.description().map(String::from),
            })
        })
        .collect();

    Ok(Json(ApiResponse::ok(PredictResponse { predictions })))
}

pub async fn reconstruct(
    State(state): State<SharedState>,
    Json(req): Json<ReconstructRequest>,
) -> Result<Json<ApiResponse<ReconstructResponse>>, AppError> {
    validate_coord_3(&req.anchor)?;
    let max_hops = req.max_hops.unwrap_or(5).min(20);
    let anchor = Coord7D::new_even([req.anchor[0], req.anchor[1], req.anchor[2], 0, 0, 0, 0]);
    let anchor_str = format!("{}", &anchor);

    let h = state.hebbian.read().await;
    let store = state.memory_store.read().await;

    let reconstructed = crate::universe::HebbianMemory::reconstruct_from_cue(
        &h,
        &anchor,
        &store.memories,
        &store.index,
        max_hops,
    );

    let results: Vec<ReconstructedMemory> = match reconstructed {
        Some(items) => items
            .into_iter()
            .enumerate()
            .map(|(hop, (coord, weight, _data))| {
                let mem = store.memories.iter().find(|m| m.anchor() == &coord);
                ReconstructedMemory {
                    anchor: format!("{}", coord),
                    hop: hop + 1,
                    edge_weight: weight,
                    description: mem.and_then(|m| m.description().map(String::from)),
                    tags: mem.map(|m| m.tags().to_vec()).unwrap_or_default(),
                    importance: mem.map(|m| m.importance()).unwrap_or(0.0),
                }
            })
            .collect(),
        None => vec![],
    };

    Ok(Json(ApiResponse::ok(ReconstructResponse {
        seed_anchor: anchor_str,
        reconstructed: results,
    })))
}

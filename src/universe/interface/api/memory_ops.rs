use axum::http::StatusCode;
use axum::{extract::State, Json};

use crate::universe::coord::Coord7D;
use crate::universe::error::AppError;
use crate::universe::memory::{MemoryAtom, MemoryCodec};
use crate::universe::metrics;

use super::state::SharedState;
use super::types::*;

pub async fn encode_memory(
    State(state): State<SharedState>,
    Json(req): Json<EncodeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<EncodeResponse>>), AppError> {
    let mut u = state.universe.write().await;
    let mut mems = state.memories.write().await;

    let anchor = Coord7D::new_even([req.anchor[0], req.anchor[1], req.anchor[2], 0, 0, 0, 0]);

    tracing::info!(anchor = %anchor, dims = req.data.len(), "encoding memory");
    metrics::API_ENCODE_TOTAL.inc();

    match MemoryCodec::encode(&mut u, &anchor, &req.data) {
        Ok(atom) => {
            let manifested = atom.is_manifested(&u);
            let anchor_str = format!("{}", atom.anchor());
            let created_at = atom.created_at();
            tracing::info!(anchor = %anchor_str, manifested, "memory encoded successfully");
            mems.push(atom);
            Ok((
                StatusCode::OK,
                Json(ApiResponse::ok(EncodeResponse {
                    anchor: anchor_str,
                    data_dim: req.data.len(),
                    manifested,
                    created_at,
                })),
            ))
        }
        Err(e) => {
            tracing::warn!(error = %e, "memory encode failed");
            Ok((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::err(format!("encode failed: {}", e))),
            ))
        }
    }
}

pub async fn decode_memory(
    State(state): State<SharedState>,
    Json(req): Json<DecodeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<DecodeResponse>>), AppError> {
    let u = state.universe.read().await;
    let mems = state.memories.read().await;

    metrics::API_DECODE_TOTAL.inc();
    let anchor = Coord7D::new_even([req.anchor[0], req.anchor[1], req.anchor[2], 0, 0, 0, 0]);

    for mem in mems.iter() {
        if mem.anchor() == &anchor && mem.data_dim() == req.data_dim {
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

    Ok((
        StatusCode::NOT_FOUND,
        Json(ApiResponse::err("memory not found")),
    ))
}

pub async fn list_memories(State(state): State<SharedState>) -> Json<ApiResponse<Vec<String>>> {
    let mems = state.memories.read().await;
    let list: Vec<String> = mems.iter().map(|m| format!("{}", m)).collect();
    Json(ApiResponse::ok(list))
}

pub async fn memory_timeline(
    State(state): State<SharedState>,
) -> Json<ApiResponse<Vec<TimelineDay>>> {
    let mems = state.memories.read().await;
    let mut day_map: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for m in mems.iter() {
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
    let u = state.universe.read().await;
    let h = state.hebbian.read().await;
    let mems = state.memories.read().await;
    let c = state.crystal.read().await;

    let source = Coord7D::new_even([req.anchor[0], req.anchor[1], req.anchor[2], 0, 0, 0, 0]);
    let max_hops = req.max_hops.unwrap_or(10);

    let associations = crate::universe::reasoning::ReasoningEngine::find_associations(
        &u, &h, &c, &source, max_hops,
    );

    let mut hops: Vec<TraceHop> = Vec::new();

    let mem_index: std::collections::HashMap<String, &MemoryAtom> = mems
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

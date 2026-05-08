// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use futures::stream::Stream;
use serde_json::Value;
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::universe::error::AppError;
use crate::universe::memory::InterestProfile;

use super::state::SharedState;
use super::types::*;

pub async fn register_interest(
    State(state): State<SharedState>,
    Json(mut profile): Json<InterestProfile>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    if profile.agent_id.trim().is_empty() {
        return Err(AppError::BadRequest("agent_id is required".to_string()));
    }
    validate_tags(&profile.tags).map_err(AppError::BadRequest)?;
    if let Some(ref cat) = profile.categories.iter().find(|c| c.len() > MAX_TAG_LEN) {
        return Err(AppError::BadRequest(format!(
            "category '{}' too long (max {})",
            cat, MAX_TAG_LEN
        )));
    }

    if profile.ttl_secs == 0 {
        profile.ttl_secs = state.config.maintenance.interest_default_ttl_secs;
    }
    profile.registered_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let agent_id = profile.agent_id.clone();
    let mut interests = state.interests.write().await;

    if interests.len() >= state.config.maintenance.max_interests
        && !interests.contains_key(&agent_id)
    {
        return Err(AppError::BadRequest(format!(
            "maximum interests ({}) reached",
            state.config.maintenance.max_interests
        )));
    }

    interests.insert(agent_id.clone(), profile);
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "registered": true,
        "agent_id": agent_id,
        "total_interests": interests.len(),
    }))))
}

pub async fn unregister_interest(
    State(state): State<SharedState>,
    Json(req): Json<UnregisterInterestRequest>,
) -> Result<Json<ApiResponse<Value>>, AppError> {
    if req.agent_id.trim().is_empty() {
        return Err(AppError::BadRequest("agent_id is required".to_string()));
    }
    let mut interests = state.interests.write().await;
    let removed = interests.remove(&req.agent_id).is_some();
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "unregistered": removed,
        "agent_id": req.agent_id,
        "remaining_interests": interests.len(),
    }))))
}

pub async fn list_interests(
    State(state): State<SharedState>,
) -> Json<ApiResponse<Vec<InterestProfile>>> {
    let interests = state.interests.read().await;
    let list: Vec<InterestProfile> = interests.values().cloned().collect();
    Json(ApiResponse::ok(list))
}

pub async fn memory_stream(
    State(state): State<SharedState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.memory_stream.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(surfaced) => {
            let data = serde_json::to_string(&surfaced).unwrap_or_default();
            let id = surfaced.seq.to_string();
            Some(Ok(Event::default()
                .data(data)
                .event("surfaced_memory")
                .id(id)))
        }
        Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(n)) => {
            tracing::warn!("SSE client lagged, skipped {} messages", n);
            None
        }
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

pub async fn surface_status(State(state): State<SharedState>) -> Json<ApiResponse<Value>> {
    let interests = state.interests.read().await;
    let receiver_count = state.memory_stream.receiver_count();
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let agent_details: Vec<serde_json::Value> = interests
        .iter()
        .map(|(id, p)| {
            let remaining = if p.ttl_secs == 0 {
                None
            } else {
                let elapsed = now_secs.saturating_sub(p.registered_at);
                Some(p.ttl_secs.saturating_sub(elapsed))
            };
            serde_json::json!({
                "agent_id": id,
                "ttl_remaining_secs": remaining,
                "tags_count": p.tags.len(),
            })
        })
        .collect();
    Json(ApiResponse::ok(serde_json::json!({
        "registered_agents": interests.len(),
        "active_streams": receiver_count,
        "max_interests": state.config.maintenance.max_interests,
        "default_ttl_secs": state.config.maintenance.interest_default_ttl_secs,
        "ttl_cleanup_enabled": state.config.maintenance.interest_ttl_enabled,
        "agents": agent_details,
    })))
}

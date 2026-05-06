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
    Json(profile): Json<InterestProfile>,
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
    let agent_id = profile.agent_id.clone();
    let mut interests = state.interests.write().await;
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
            Some(Ok(Event::default().data(data).event("surfaced_memory")))
        }
        Err(_) => None,
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

pub async fn surface_status(State(state): State<SharedState>) -> Json<ApiResponse<Value>> {
    let interests = state.interests.read().await;
    let receiver_count = state.memory_stream.receiver_count();
    Json(ApiResponse::ok(serde_json::json!({
        "registered_agents": interests.len(),
        "active_streams": receiver_count,
        "agents": interests.keys().collect::<Vec<_>>(),
    })))
}

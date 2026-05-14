// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use axum::http::StatusCode;
use axum::{extract::State, Json};

use crate::universe::backup::BackupTrigger;
use crate::universe::error::AppError;
use crate::universe::events::UniverseEvent;

use super::state::SharedState;
use super::types::*;

pub async fn create_backup(
    State(state): State<SharedState>,
) -> Result<(StatusCode, Json<ApiResponse<CreateBackupResponse>>), AppError> {
    let report = {
        let mut bs = state.backup.write().await;
        let c = state.crystal.read().await;
        let h = state.hebbian.read().await;
        let store = state.memory_store.read().await;
        let u = state.universe.read().await;
        bs.create_backup(BackupTrigger::Manual, &u, &h, &store.memories, &c)
            .map_err(|e| AppError::Internal(e.to_string()))?
    };

    state.event_sender.publish(UniverseEvent::BackupCreated {
        backup_id: report.metadata.id,
        bytes: report.metadata.bytes,
        conservation_ok: report.metadata.conservation_ok,
    });

    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok(CreateBackupResponse {
            backup_id: report.metadata.id,
            generation: report.metadata.generation,
            node_count: report.metadata.node_count,
            memory_count: report.metadata.memory_count,
            bytes: report.metadata.bytes,
            elapsed_ms: report.elapsed_ms,
        })),
    ))
}

pub async fn list_backups(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<Vec<BackupInfo>>>, AppError> {
    let bs = state.backup.read().await;
    let list: Vec<BackupInfo> = bs
        .list_backups()
        .into_iter()
        .map(|m| {
            let trigger = match m.trigger {
                BackupTrigger::Manual => "MANUAL",
                BackupTrigger::Timer => "TIMER",
                BackupTrigger::PreOperation => "PRE-OP",
                BackupTrigger::ConservationCheckpoint => "CONSERV",
            };
            BackupInfo {
                id: m.id,
                timestamp_ms: m.timestamp_ms,
                trigger: trigger.to_string(),
                node_count: m.node_count,
                memory_count: m.memory_count,
                total_energy: m.total_energy,
                conservation_ok: m.conservation_ok,
                bytes: m.bytes,
                generation: m.generation,
            }
        })
        .collect();
    Ok(Json(ApiResponse::ok(list)))
}

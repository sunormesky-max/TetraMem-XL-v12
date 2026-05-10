// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::error::AppError;
use crate::universe::interface::api::state::SharedState;
use crate::universe::interface::api::types::ApiResponse;
use crate::universe::plugins::manifest::{PluginExecutionRequest, PluginManifest};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct InstallRequest {
    pub manifest: PluginManifest,
    pub wasm_base64: String,
}

pub async fn plugin_install(
    State(state): State<SharedState>,
    Json(req): Json<InstallRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    const MAX_BASE64_SIZE: usize = 4 * 1024 * 1024;

    if req.wasm_base64.len() > MAX_BASE64_SIZE {
        return Err(AppError::BadRequest(format!(
            "WASM base64 payload exceeds {} bytes",
            MAX_BASE64_SIZE
        )));
    }

    use base64::Engine;
    let wasm_bytes = base64::engine::general_purpose::STANDARD
        .decode(&req.wasm_base64)
        .map_err(|e| AppError::BadRequest(format!("invalid base64: {}", e)))?;

    if wasm_bytes.is_empty() {
        return Err(AppError::BadRequest("no WASM bytes provided".into()));
    }

    let name = req.manifest.name.clone();
    let mut plugins = state.plugins.write().await;
    plugins.install(req.manifest, wasm_bytes)?;
    drop(plugins);

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "installed": name,
        "status": "installed"
    }))))
}

pub async fn plugin_uninstall(
    State(state): State<SharedState>,
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let mut plugins = state.plugins.write().await;
    let mf = plugins.uninstall(&name)?;
    drop(plugins);

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "uninstalled": mf.name,
        "version": mf.version
    }))))
}

pub async fn plugin_list(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, AppError> {
    let plugins = state.plugins.read().await;
    let list: Vec<serde_json::Value> = plugins
        .list()
        .into_iter()
        .map(|info| serde_json::to_value(info).unwrap_or_default())
        .collect();
    drop(plugins);

    Ok(Json(ApiResponse::ok(list)))
}

pub async fn plugin_status(
    State(state): State<SharedState>,
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let plugins = state.plugins.read().await;
    let info = plugins
        .get(&name)
        .ok_or_else(|| AppError::NotFound(format!("plugin '{}' not found", name)))?;
    let val = serde_json::to_value(info).unwrap_or_default();
    drop(plugins);

    Ok(Json(ApiResponse::ok(val)))
}

pub async fn plugin_enable(
    State(state): State<SharedState>,
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let mut plugins = state.plugins.write().await;
    plugins.enable(&name)?;
    drop(plugins);

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "name": name,
        "status": "enabled"
    }))))
}

pub async fn plugin_disable(
    State(state): State<SharedState>,
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let mut plugins = state.plugins.write().await;
    plugins.disable(&name)?;
    drop(plugins);

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "name": name,
        "status": "disabled"
    }))))
}

pub async fn plugin_execute(
    State(state): State<SharedState>,
    Path(name): Path<String>,
    Json(req): Json<PluginExecutionRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let (wasm_bytes, permissions, energy_limit, energy_budget, current_consumed) = {
        let plugins = state.plugins.read().await;
        let entry = plugins
            .get_entry(&name)
            .ok_or_else(|| AppError::NotFound(format!("plugin '{}' not found", name)))?;
        if !entry.is_enabled() {
            return Err(AppError::BadRequest(format!(
                "plugin '{}' is not enabled",
                name
            )));
        }
        let energy_limit = req.energy_limit.unwrap_or(entry.manifest().energy_budget);
        (
            entry.wasm_bytes().to_vec(),
            entry.manifest().permissions.clone(),
            energy_limit,
            entry.manifest().energy_budget,
            entry.energy_consumed(),
        )
    };

    let sandbox = crate::universe::plugins::sandbox::WasmSandbox::new();
    let req_clone = req.clone();
    let result = tokio::task::spawn_blocking(move || {
        sandbox.execute(&wasm_bytes, &req_clone, &permissions, energy_limit)
    })
    .await
    .map_err(|e| AppError::Internal(format!("plugin execution task failed: {}", e)))?;

    {
        let mut plugins = state.plugins.write().await;
        plugins.record_execution(
            &name,
            result.energy_consumed,
            energy_budget,
            current_consumed,
        )?;
    }

    let val = serde_json::to_value(&result).unwrap_or_default();
    Ok(Json(ApiResponse::ok(val)))
}

pub async fn plugin_reset_energy(
    State(state): State<SharedState>,
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let mut plugins = state.plugins.write().await;
    plugins.reset_energy(&name)?;
    drop(plugins);

    Ok(Json(ApiResponse::ok(serde_json::json!({
        "name": name,
        "energy_reset": true
    }))))
}

pub async fn plugin_manager_stats(
    State(state): State<SharedState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let plugins = state.plugins.read().await;
    let stats = plugins.stats();
    let val = serde_json::to_value(stats).unwrap_or_default();
    drop(plugins);

    Ok(Json(ApiResponse::ok(val)))
}

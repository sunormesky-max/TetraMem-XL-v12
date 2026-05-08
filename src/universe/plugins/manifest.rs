// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub api_version: u32,
    pub energy_budget: u64,
    #[serde(default)]
    pub permissions: PluginPermissions,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginPermissions {
    #[serde(default)]
    pub memory_read: bool,
    #[serde(default)]
    pub memory_write: bool,
    #[serde(default)]
    pub hebbian_read: bool,
    #[serde(default)]
    pub hebbian_write: bool,
    #[serde(default)]
    pub pulse_fire: bool,
    #[serde(default)]
    pub universe_read: bool,
    #[serde(default)]
    pub event_publish: bool,
    #[serde(default)]
    pub event_subscribe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginStatus {
    Installed,
    Enabled,
    Running,
    Disabled,
    Error(String),
    SuspendedEnergyBudgetExceeded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub manifest: PluginManifest,
    pub status: PluginStatus,
    pub energy_consumed: u64,
    pub executions: u64,
    pub last_execution: Option<String>,
    pub installed_at: String,
}

impl PluginInfo {
    pub fn new(manifest: PluginManifest) -> Self {
        Self {
            status: PluginStatus::Installed,
            energy_consumed: 0,
            executions: 0,
            last_execution: None,
            installed_at: chrono::Utc::now().to_rfc3339(),
            manifest,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExecutionRequest {
    pub function: String,
    pub input: Vec<u8>,
    pub energy_limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExecutionResult {
    pub output: Vec<u8>,
    pub energy_consumed: u64,
    pub execution_time_us: u64,
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMarketplaceEntry {
    pub manifest: PluginManifest,
    pub downloads: u64,
    pub rating: f64,
    pub reviews: u64,
    pub wasm_url: String,
}

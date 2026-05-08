// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::error::AppError;
use crate::universe::plugins::manifest::{
    PluginExecutionRequest, PluginExecutionResult, PluginInfo, PluginManifest, PluginStatus,
};
use crate::universe::plugins::sandbox::WasmSandbox;
use std::collections::HashMap;
use std::time::Instant;

pub struct PluginManager {
    sandbox: WasmSandbox,
    plugins: HashMap<String, PluginEntry>,
    global_energy_budget: u64,
}

struct PluginEntry {
    info: PluginInfo,
    wasm_bytes: Vec<u8>,
    #[allow(dead_code)]
    installed_at: Instant,
}

impl PluginManager {
    pub fn new(global_energy_budget: u64) -> Self {
        Self {
            sandbox: WasmSandbox::new(),
            plugins: HashMap::new(),
            global_energy_budget,
        }
    }

    pub fn install(
        &mut self,
        manifest: PluginManifest,
        wasm_bytes: Vec<u8>,
    ) -> Result<(), AppError> {
        if manifest.name.is_empty() {
            return Err(AppError::BadRequest("plugin name cannot be empty".into()));
        }
        if manifest.api_version != 1 {
            return Err(AppError::BadRequest(format!(
                "unsupported API version: {}",
                manifest.api_version
            )));
        }
        if self.plugins.contains_key(&manifest.name) {
            return Err(AppError::BadRequest(format!(
                "plugin '{}' already installed",
                manifest.name
            )));
        }
        self.sandbox.validate(&wasm_bytes)?;
        let info = PluginInfo::new(manifest.clone());
        self.plugins.insert(
            manifest.name.clone(),
            PluginEntry {
                info,
                wasm_bytes,
                installed_at: Instant::now(),
            },
        );
        tracing::info!(name = %manifest.name, version = %manifest.version, "plugin installed");
        Ok(())
    }

    pub fn uninstall(&mut self, name: &str) -> Result<PluginManifest, AppError> {
        let entry = self
            .plugins
            .remove(name)
            .ok_or_else(|| AppError::NotFound(format!("plugin '{}' not found", name)))?;
        tracing::info!(name = %name, "plugin uninstalled");
        Ok(entry.info.manifest)
    }

    pub fn enable(&mut self, name: &str) -> Result<(), AppError> {
        let entry = self.plugins.get_mut(name).ok_or_else(|| {
            AppError::NotFound(format!("plugin '{}' not found", name))
        })?;
        match &entry.info.status {
            PluginStatus::Installed | PluginStatus::Disabled => {
                entry.info.status = PluginStatus::Enabled;
                tracing::info!(name = %name, "plugin enabled");
                Ok(())
            }
            PluginStatus::Enabled => Err(AppError::BadRequest("already enabled".into())),
            PluginStatus::Running => Err(AppError::BadRequest("currently running".into())),
            PluginStatus::Error(e) => Err(AppError::BadRequest(format!("plugin in error: {}", e))),
            PluginStatus::SuspendedEnergyBudgetExceeded => {
                entry.info.energy_consumed = 0;
                entry.info.status = PluginStatus::Enabled;
                tracing::info!(name = %name, "plugin re-enabled after energy reset");
                Ok(())
            }
        }
    }

    pub fn disable(&mut self, name: &str) -> Result<(), AppError> {
        let entry = self.plugins.get_mut(name).ok_or_else(|| {
            AppError::NotFound(format!("plugin '{}' not found", name))
        })?;
        entry.info.status = PluginStatus::Disabled;
        tracing::info!(name = %name, "plugin disabled");
        Ok(())
    }

    pub fn execute(
        &mut self,
        name: &str,
        request: PluginExecutionRequest,
    ) -> Result<PluginExecutionResult, AppError> {
        let entry = self.plugins.get_mut(name).ok_or_else(|| {
            AppError::NotFound(format!("plugin '{}' not found", name))
        })?;

        if !matches!(entry.info.status, PluginStatus::Enabled) {
            return Err(AppError::BadRequest(format!(
                "plugin '{}' is not enabled (status: {:?})",
                name, entry.info.status
            )));
        }

        let energy_limit = request
            .energy_limit
            .unwrap_or(entry.info.manifest.energy_budget);

        entry.info.status = PluginStatus::Running;
        let result = self.sandbox.execute(
            &entry.wasm_bytes,
            &request,
            &entry.info.manifest.permissions,
            energy_limit,
        );

        entry.info.executions += 1;
        entry.info.energy_consumed += result.energy_consumed;
        entry.info.last_execution = Some(chrono::Utc::now().to_rfc3339());

        if entry.info.energy_consumed >= entry.info.manifest.energy_budget {
            entry.info.status = PluginStatus::SuspendedEnergyBudgetExceeded;
            tracing::warn!(
                name = %name,
                consumed = entry.info.energy_consumed,
                budget = entry.info.manifest.energy_budget,
                "plugin suspended: energy budget exceeded"
            );
        } else {
            entry.info.status = PluginStatus::Enabled;
        }

        Ok(result)
    }

    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins.values().map(|e| e.info.clone()).collect()
    }

    pub fn get(&self, name: &str) -> Option<PluginInfo> {
        self.plugins.get(name).map(|e| e.info.clone())
    }

    pub fn reset_energy(&mut self, name: &str) -> Result<(), AppError> {
        let entry = self.plugins.get_mut(name).ok_or_else(|| {
            AppError::NotFound(format!("plugin '{}' not found", name))
        })?;
        entry.info.energy_consumed = 0;
        if matches!(entry.info.status, PluginStatus::SuspendedEnergyBudgetExceeded) {
            entry.info.status = PluginStatus::Enabled;
        }
        tracing::info!(name = %name, "plugin energy budget reset");
        Ok(())
    }

    pub fn stats(&self) -> PluginManagerStats {
        let total = self.plugins.len();
        let enabled = self
            .plugins
            .values()
            .filter(|e| matches!(e.info.status, PluginStatus::Enabled))
            .count();
        let running = self
            .plugins
            .values()
            .filter(|e| matches!(e.info.status, PluginStatus::Running))
            .count();
        let suspended = self
            .plugins
            .values()
            .filter(|e| matches!(e.info.status, PluginStatus::SuspendedEnergyBudgetExceeded))
            .count();
        let total_energy: u64 = self.plugins.values().map(|e| e.info.energy_consumed).sum();
        let total_executions: u64 = self.plugins.values().map(|e| e.info.executions).sum();
        PluginManagerStats {
            total,
            enabled,
            running,
            suspended,
            total_energy_consumed: total_energy,
            total_executions,
            global_energy_budget: self.global_energy_budget,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManagerStats {
    pub total: usize,
    pub enabled: usize,
    pub running: usize,
    pub suspended: usize,
    pub total_energy_consumed: u64,
    pub total_executions: u64,
    pub global_energy_budget: u64,
}

use serde::{Deserialize, Serialize};

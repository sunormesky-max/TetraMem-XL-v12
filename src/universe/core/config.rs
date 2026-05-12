// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_server")]
    pub server: ServerConfig,
    #[serde(default = "default_universe")]
    pub universe: UniverseConfig,
    #[serde(default = "default_auth")]
    pub auth: AuthConfig,
    #[serde(default = "default_logging")]
    pub logging: LoggingConfig,
    #[serde(default = "default_backup")]
    pub backup: BackupFileConfig,
    #[serde(default = "default_rate_limit")]
    pub rate_limit: RateLimitConfig,
    #[serde(default = "default_maintenance")]
    pub maintenance: MaintenanceConfig,
    #[serde(default = "default_spontaneous")]
    pub spontaneous: SpontaneousConfig,
    #[serde(default = "default_neural_embed")]
    pub neural_embed: NeuralEmbedConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_addr")]
    pub addr: String,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "default_body_limit")]
    pub body_limit_bytes: usize,
    #[serde(default)]
    pub tls: Option<TlsConfig>,
    #[serde(default = "default_cors_origins")]
    pub cors_origins: Vec<String>,
    #[serde(default)]
    pub static_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseConfig {
    #[serde(default = "default_total_energy")]
    pub total_energy: f64,
    #[serde(default = "default_manifestation_threshold")]
    pub manifestation_threshold: f64,
    #[serde(default = "default_energy_drift_tolerance")]
    pub energy_drift_tolerance: f64,
    #[serde(default = "default_max_timeline_days")]
    pub max_timeline_days: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(default = "default_auth_enabled")]
    pub enabled: bool,
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
    #[serde(default = "default_jwt_expiry_secs")]
    pub jwt_expiry_secs: u64,
    #[serde(default)]
    pub users: Vec<crate::universe::auth::UserConfig>,
    #[serde(default = "default_raft_secret")]
    pub raft_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub json_format: bool,
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(default = "default_tracing_enabled")]
    pub tracing_enabled: bool,
    #[serde(default = "default_conservation_check_interval_secs")]
    pub conservation_check_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFileConfig {
    #[serde(default = "default_backup_dir")]
    pub dir: String,
    #[serde(default = "default_backup_interval_secs")]
    pub interval_secs: u64,
    #[serde(default = "default_max_generations")]
    pub max_generations: usize,
    #[serde(default = "default_auto_persist")]
    pub auto_persist: bool,
    #[serde(default = "default_persist_path")]
    pub persist_path: String,
    #[serde(default = "default_persist_backend")]
    pub persist_backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    #[serde(default = "default_rpm")]
    pub requests_per_minute: u64,
    #[serde(default = "default_burst")]
    pub burst: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceConfig {
    #[serde(default = "default_maintenance_enabled")]
    pub enabled: bool,
    #[serde(default = "default_maintenance_interval_secs")]
    pub interval_secs: u64,
    #[serde(default = "default_dream_min_urgency")]
    pub dream_min_urgency: f64,
    #[serde(default = "default_aging_enabled")]
    pub aging_enabled: bool,
    #[serde(default = "default_clustering_enabled")]
    pub clustering_enabled: bool,
    #[serde(default = "default_watchdog_enabled")]
    pub watchdog_enabled: bool,
    #[serde(default = "default_crystal_decay_enabled")]
    pub crystal_decay_enabled: bool,
    #[serde(default = "default_regulation_enabled")]
    pub regulation_enabled: bool,
    #[serde(default = "default_event_drain_enabled")]
    pub event_drain_enabled: bool,
    #[serde(default = "default_auto_forget_enabled")]
    pub auto_forget_enabled: bool,
    #[serde(default = "default_auto_forget_grace_cycles")]
    pub auto_forget_grace_cycles: u32,
    #[serde(default = "default_max_memories")]
    pub max_memories: usize,
    #[serde(default = "default_interest_ttl_enabled")]
    pub interest_ttl_enabled: bool,
    #[serde(default = "default_interest_default_ttl_secs")]
    pub interest_default_ttl_secs: u64,
    #[serde(default = "default_max_interests")]
    pub max_interests: usize,
    #[serde(default)]
    pub deferred_binding: bool,
    #[serde(default = "default_hebbian_target_avg")]
    pub hebbian_target_avg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpontaneousConfig {
    #[serde(default = "default_spontaneous_enabled")]
    pub enabled: bool,
    #[serde(default = "default_pulse_enabled")]
    pub pulse_enabled: bool,
    #[serde(default = "default_recall_enabled")]
    pub recall_enabled: bool,
    #[serde(default = "default_curiosity_enabled")]
    pub curiosity_enabled: bool,
    #[serde(default = "default_event_reaction_enabled")]
    pub event_reaction_enabled: bool,
    #[serde(default = "default_base_curiosity")]
    pub base_curiosity: f64,
    #[serde(default = "default_base_reflection")]
    pub base_reflection: f64,
    #[serde(default = "default_base_exploration")]
    pub base_exploration: f64,
}

impl Default for SpontaneousConfig {
    fn default() -> Self {
        default_spontaneous()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralEmbedConfig {
    #[serde(default = "default_neural_enabled")]
    pub enabled: bool,
    #[serde(default = "default_neural_model_dir")]
    pub model_dir: String,
}

fn default_neural_enabled() -> bool {
    false
}

fn default_neural_model_dir() -> String {
    "models/granite-embedding-small".to_string()
}

fn default_neural_embed() -> NeuralEmbedConfig {
    NeuralEmbedConfig {
        enabled: default_neural_enabled(),
        model_dir: default_neural_model_dir(),
    }
}

impl Default for NeuralEmbedConfig {
    fn default() -> Self {
        default_neural_embed()
    }
}

fn default_server() -> ServerConfig {
    ServerConfig {
        addr: default_addr(),
        timeout_secs: default_timeout_secs(),
        body_limit_bytes: default_body_limit(),
        tls: None,
        cors_origins: default_cors_origins(),
        static_dir: None,
    }
}

fn default_cors_origins() -> Vec<String> {
    vec!["http://localhost:5173".to_string()]
}

fn default_universe() -> UniverseConfig {
    UniverseConfig {
        total_energy: default_total_energy(),
        manifestation_threshold: default_manifestation_threshold(),
        energy_drift_tolerance: default_energy_drift_tolerance(),
        max_timeline_days: default_max_timeline_days(),
    }
}

fn default_auth() -> AuthConfig {
    AuthConfig {
        enabled: default_auth_enabled(),
        jwt_secret: default_jwt_secret(),
        jwt_expiry_secs: default_jwt_expiry_secs(),
        users: Vec::new(),
        raft_secret: default_raft_secret(),
    }
}

fn default_logging() -> LoggingConfig {
    LoggingConfig {
        level: default_log_level(),
        json_format: false,
        file_path: None,
        tracing_enabled: default_tracing_enabled(),
        conservation_check_interval_secs: default_conservation_check_interval_secs(),
    }
}

fn default_backup() -> BackupFileConfig {
    BackupFileConfig {
        dir: default_backup_dir(),
        interval_secs: default_backup_interval_secs(),
        max_generations: default_max_generations(),
        auto_persist: default_auto_persist(),
        persist_path: default_persist_path(),
        persist_backend: default_persist_backend(),
    }
}

fn default_rate_limit() -> RateLimitConfig {
    RateLimitConfig {
        requests_per_minute: default_rpm(),
        burst: default_burst(),
    }
}

fn default_maintenance() -> MaintenanceConfig {
    MaintenanceConfig {
        enabled: default_maintenance_enabled(),
        interval_secs: default_maintenance_interval_secs(),
        dream_min_urgency: default_dream_min_urgency(),
        aging_enabled: default_aging_enabled(),
        clustering_enabled: default_clustering_enabled(),
        watchdog_enabled: default_watchdog_enabled(),
        crystal_decay_enabled: default_crystal_decay_enabled(),
        regulation_enabled: default_regulation_enabled(),
        event_drain_enabled: default_event_drain_enabled(),
        auto_forget_enabled: default_auto_forget_enabled(),
        auto_forget_grace_cycles: default_auto_forget_grace_cycles(),
        max_memories: default_max_memories(),
        interest_ttl_enabled: default_interest_ttl_enabled(),
        interest_default_ttl_secs: default_interest_default_ttl_secs(),
        max_interests: default_max_interests(),
        deferred_binding: false,
        hebbian_target_avg: default_hebbian_target_avg(),
    }
}

fn default_addr() -> String {
    "127.0.0.1:3456".to_string()
}
fn default_auth_enabled() -> bool {
    true
}
fn default_timeout_secs() -> u64 {
    30
}
fn default_body_limit() -> usize {
    10 * 1024 * 1024
}
fn default_total_energy() -> f64 {
    10_000_000.0
}
fn default_manifestation_threshold() -> f64 {
    0.5
}
fn default_energy_drift_tolerance() -> f64 {
    1e-8
}
fn default_max_timeline_days() -> usize {
    365
}
fn default_jwt_secret() -> String {
    "change-me-in-production".to_string()
}
fn default_raft_secret() -> String {
    "change-raft-secret".to_string()
}
fn default_jwt_expiry_secs() -> u64 {
    86400
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_tracing_enabled() -> bool {
    true
}
fn default_conservation_check_interval_secs() -> u64 {
    60
}
fn default_backup_dir() -> String {
    "./backups".to_string()
}
fn default_backup_interval_secs() -> u64 {
    300
}
fn default_max_generations() -> usize {
    10
}
fn default_auto_persist() -> bool {
    true
}
fn default_persist_path() -> String {
    "./data/tetramem_state.json".to_string()
}
fn default_persist_backend() -> String {
    "json".to_string()
}
fn default_rpm() -> u64 {
    1000
}
fn default_burst() -> u64 {
    50
}
fn default_maintenance_enabled() -> bool {
    true
}
fn default_maintenance_interval_secs() -> u64 {
    120
}
fn default_dream_min_urgency() -> f64 {
    0.4
}
fn default_aging_enabled() -> bool {
    true
}
fn default_clustering_enabled() -> bool {
    true
}
fn default_watchdog_enabled() -> bool {
    true
}
fn default_crystal_decay_enabled() -> bool {
    true
}
fn default_regulation_enabled() -> bool {
    true
}
fn default_event_drain_enabled() -> bool {
    true
}
fn default_auto_forget_enabled() -> bool {
    true
}
fn default_auto_forget_grace_cycles() -> u32 {
    3
}
fn default_max_memories() -> usize {
    100_000
}
fn default_interest_ttl_enabled() -> bool {
    true
}
fn default_interest_default_ttl_secs() -> u64 {
    3600
}
fn default_max_interests() -> usize {
    1000
}
fn default_hebbian_target_avg() -> f64 {
    2.0
}
fn default_spontaneous_enabled() -> bool {
    true
}
fn default_pulse_enabled() -> bool {
    true
}
fn default_recall_enabled() -> bool {
    true
}
fn default_curiosity_enabled() -> bool {
    true
}
fn default_event_reaction_enabled() -> bool {
    true
}
fn default_base_curiosity() -> f64 {
    0.5
}
fn default_base_reflection() -> f64 {
    0.3
}
fn default_base_exploration() -> f64 {
    0.4
}

fn default_spontaneous() -> SpontaneousConfig {
    SpontaneousConfig {
        enabled: default_spontaneous_enabled(),
        pulse_enabled: default_pulse_enabled(),
        recall_enabled: default_recall_enabled(),
        curiosity_enabled: default_curiosity_enabled(),
        event_reaction_enabled: default_event_reaction_enabled(),
        base_curiosity: default_base_curiosity(),
        base_reflection: default_base_reflection(),
        base_exploration: default_base_exploration(),
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: default_server(),
            universe: default_universe(),
            auth: default_auth(),
            logging: default_logging(),
            backup: default_backup(),
            rate_limit: default_rate_limit(),
            maintenance: default_maintenance(),
            spontaneous: default_spontaneous(),
            neural_embed: default_neural_embed(),
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            let dev_mode = std::env::var("TETRAMEM_DEV_MODE")
                .map(|v| v == "1")
                .unwrap_or(false);
            if !dev_mode {
                return Err(ConfigError::Io(format!(
                    "config file not found: {} — set TETRAMEM_DEV_MODE=1 to allow development mode",
                    path.display()
                )));
            }
            tracing::warn!("TETRAMEM_DEV_MODE=1 — running without config file, auth disabled, NOT for production");
            let mut config = Self::default();
            config.auth.enabled = false;
            config.resolve_env_overrides();
            config.validate()?;
            return Ok(config);
        }
        let content = fs::read_to_string(path).map_err(|e| ConfigError::Io(e.to_string()))?;
        let mut config: Self =
            toml::from_str(&content).map_err(|e| ConfigError::Parse(e.to_string()))?;
        tracing::info!("loaded config from {}", path.display());
        config.resolve_env_overrides();
        config.validate()?;
        Ok(config)
    }

    pub fn load_without_validation(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::Io(format!(
                "config file not found: {}",
                path.display()
            )));
        }
        let content = fs::read_to_string(path).map_err(|e| ConfigError::Io(e.to_string()))?;
        let mut config: Self =
            toml::from_str(&content).map_err(|e| ConfigError::Parse(e.to_string()))?;
        config.resolve_env_overrides();
        Ok(config)
    }

    fn resolve_env_overrides(&mut self) {
        if let Ok(secret) = std::env::var("TETRAMEM_JWT_SECRET") {
            if !secret.is_empty() {
                tracing::info!("JWT secret overridden from TETRAMEM_JWT_SECRET env var");
                self.auth.jwt_secret = secret;
            }
        }
        if let Ok(secret) = std::env::var("TETRAMEM_RAFT_SECRET") {
            if !secret.is_empty() {
                tracing::info!("Raft secret overridden from TETRAMEM_RAFT_SECRET env var");
                self.auth.raft_secret = secret;
            }
        }
        if let Ok(origins) = std::env::var("TETRAMEM_CORS_ORIGINS") {
            if !origins.is_empty() {
                self.server.cors_origins =
                    origins.split(',').map(|s| s.trim().to_string()).collect();
            }
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.universe.total_energy <= 0.0 || self.universe.total_energy.is_nan() {
            return Err(ConfigError::Parse(
                "universe.total_energy must be > 0 and not NaN".to_string(),
            ));
        }
        if self.universe.manifestation_threshold < 0.0
            || self.universe.manifestation_threshold > 1.0
            || self.universe.manifestation_threshold.is_nan()
        {
            return Err(ConfigError::Parse(
                "universe.manifestation_threshold must be in [0, 1]".to_string(),
            ));
        }
        if self.universe.energy_drift_tolerance <= 0.0
            || self.universe.energy_drift_tolerance.is_nan()
        {
            return Err(ConfigError::Parse(
                "universe.energy_drift_tolerance must be > 0".to_string(),
            ));
        }
        if self.server.body_limit_bytes == 0 {
            return Err(ConfigError::Parse(
                "server.body_limit_bytes must be > 0".to_string(),
            ));
        }
        if self.server.timeout_secs == 0 {
            return Err(ConfigError::Parse(
                "server.timeout_secs must be > 0".to_string(),
            ));
        }
        if self.auth.enabled && self.auth.jwt_secret == "change-me-in-production" {
            return Err(ConfigError::Parse(
                "auth.enabled=true with default JWT secret is insecure; \
                 set TETRAMEM_JWT_SECRET env var"
                    .to_string(),
            ));
        }
        if self.auth.enabled && self.auth.raft_secret == "change-raft-secret" {
            return Err(ConfigError::Parse(
                "default raft_secret is insecure when auth is enabled; \
                 set auth.raft_secret or TETRAMEM_RAFT_SECRET env var"
                    .to_string(),
            ));
        }
        if self.auth.enabled && !self.auth.users.is_empty() {
            let default_hash = "$argon2id$v=19$m=19456,t=2,p=1$l7+kFgPk+WRQHfRzEvZgGA$IrIHnE+KcLW7CRfv02DDMj/53fjTmUqsDVOHeibmAGs";
            for user in &self.auth.users {
                if user.password.is_empty() && user.password_hash.is_empty() {
                    return Err(ConfigError::Parse(format!(
                        "auth user '{}' has no password or password_hash set",
                        user.username
                    )));
                }
                if user.password_hash == default_hash {
                    return Err(ConfigError::Parse(format!(
                        "auth user '{}' uses the default/example password hash; \
                          generate a unique hash with: cargo test password_hashing_and_verify",
                        user.username
                    )));
                }
                if user.password == "changeme" {
                    return Err(ConfigError::Parse(format!(
                        "auth user '{}' uses the default password 'changeme'; \
                          change it before deploying to production",
                        user.username
                    )));
                }
            }
        }
        if self.auth.enabled && self.auth.users.is_empty() {
            return Err(ConfigError::Parse(
                "auth.enabled=true but no users configured; add [[auth.users]] entries".to_string(),
            ));
        }
        if self.auth.enabled && self.auth.jwt_expiry_secs == 0 {
            return Err(ConfigError::Parse(
                "auth.jwt_expiry_secs must be > 0".to_string(),
            ));
        }
        if self.backup.interval_secs == 0 {
            return Err(ConfigError::Parse(
                "backup.interval_secs must be > 0".to_string(),
            ));
        }
        if self.backup.max_generations == 0 {
            return Err(ConfigError::Parse(
                "backup.max_generations must be > 0".to_string(),
            ));
        }
        if self.rate_limit.requests_per_minute == 0 {
            return Err(ConfigError::Parse(
                "rate_limit.requests_per_minute must be > 0".to_string(),
            ));
        }
        if self.rate_limit.burst == 0 {
            return Err(ConfigError::Parse(
                "rate_limit.burst must be > 0".to_string(),
            ));
        }
        if self.maintenance.interval_secs == 0 {
            return Err(ConfigError::Parse(
                "maintenance.interval_secs must be > 0".to_string(),
            ));
        }
        if self.maintenance.max_memories == 0 {
            return Err(ConfigError::Parse(
                "maintenance.max_memories must be > 0".to_string(),
            ));
        }
        if self.maintenance.interest_default_ttl_secs == 0 {
            return Err(ConfigError::Parse(
                "maintenance.interest_default_ttl_secs must be > 0".to_string(),
            ));
        }
        if self.maintenance.max_interests == 0 {
            return Err(ConfigError::Parse(
                "maintenance.max_interests must be > 0".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.spontaneous.base_curiosity) {
            return Err(ConfigError::Parse(
                "spontaneous.base_curiosity must be in [0, 1]".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.spontaneous.base_reflection) {
            return Err(ConfigError::Parse(
                "spontaneous.base_reflection must be in [0, 1]".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.spontaneous.base_exploration) {
            return Err(ConfigError::Parse(
                "spontaneous.base_exploration must be in [0, 1]".to_string(),
            ));
        }
        Ok(())
    }

    pub fn save_default(path: &Path) -> Result<(), ConfigError> {
        let config = Self::default();
        let content =
            toml::to_string_pretty(&config).map_err(|e| ConfigError::Serialize(e.to_string()))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| ConfigError::Io(e.to_string()))?;
        }
        fs::write(path, content).map_err(|e| ConfigError::Io(e.to_string()))?;
        Ok(())
    }

    pub fn generate_example() -> String {
        let config = Self::default();
        toml::to_string_pretty(&config).unwrap_or_default()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("serialize error: {0}")]
    Serialize(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = AppConfig::default();
        assert_eq!(config.server.addr, "127.0.0.1:3456");
        assert_eq!(config.universe.total_energy, 10_000_000.0);
        assert!(config.auth.enabled);
    }

    #[test]
    fn roundtrip_toml() {
        let config = AppConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: AppConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.server.addr, config.server.addr);
        assert_eq!(parsed.universe.total_energy, config.universe.total_energy);
    }

    #[test]
    fn load_nonexistent_returns_default() {
        std::env::set_var("TETRAMEM_DEV_MODE", "1");
        let config = AppConfig::load(Path::new("/nonexistent/config.toml")).unwrap();
        assert_eq!(config.server.addr, "127.0.0.1:3456");
        std::env::remove_var("TETRAMEM_DEV_MODE");
    }

    #[test]
    fn default_with_no_users_disables_auth() {
        let mut config = AppConfig::default();
        config.auth.enabled = false;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn generate_example_not_empty() {
        let example = AppConfig::generate_example();
        assert!(!example.is_empty());
        assert!(example.contains("[server]"));
        assert!(example.contains("[universe]"));
    }
}

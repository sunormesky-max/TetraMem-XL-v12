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
    #[serde(default)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    #[serde(default = "default_rpm")]
    pub requests_per_minute: u64,
    #[serde(default = "default_burst")]
    pub burst: u64,
}

fn default_server() -> ServerConfig {
    ServerConfig {
        addr: default_addr(),
        timeout_secs: default_timeout_secs(),
        body_limit_bytes: default_body_limit(),
        tls: None,
        cors_origins: default_cors_origins(),
    }
}

fn default_cors_origins() -> Vec<String> {
    vec!["*".to_string()]
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
        enabled: false,
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
    }
}

fn default_rate_limit() -> RateLimitConfig {
    RateLimitConfig {
        requests_per_minute: default_rpm(),
        burst: default_burst(),
    }
}

fn default_addr() -> String {
    "127.0.0.1:3456".to_string()
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
fn default_rpm() -> u64 {
    1000
}
fn default_burst() -> u64 {
    50
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
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            tracing::info!("config file not found, using defaults: {}", path.display());
            let mut config = Self::default();
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
            for user in &self.auth.users {
                if user.password.is_empty() && user.password_hash.is_empty() {
                    return Err(ConfigError::Parse(format!(
                        "auth user '{}' has no password or password_hash set",
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
        assert!(!config.auth.enabled);
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
        let config = AppConfig::load(Path::new("/nonexistent/config.toml")).unwrap();
        assert_eq!(config.server.addr, "127.0.0.1:3456");
    }

    #[test]
    fn generate_example_not_empty() {
        let example = AppConfig::generate_example();
        assert!(!example.is_empty());
        assert!(example.contains("[server]"));
        assert!(example.contains("[universe]"));
    }
}

//! Configuration module for Anchor Engine.
//!
//! Handles loading and saving user settings from user_settings.json
//! Single Source of Truth for all configurable parameters.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use thiserror::Error;

/// Configuration errors.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Root configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_version")]
    pub version: String,
    
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,
    
    /// GitHub configuration
    #[serde(default)]
    pub github: GitHubConfig,
    
    /// Path configuration
    #[serde(default)]
    pub paths: PathConfig,
    
    /// Search configuration
    #[serde(default)]
    pub search: SearchConfig,
    
    /// Watcher configuration
    #[serde(default)]
    pub watcher: WatcherConfig,
    
    /// Limits configuration
    #[serde(default)]
    pub limits: LimitsConfig,
    
    /// Ingestion configuration
    #[serde(default)]
    pub ingestion: IngestionConfig,

    /// Search budget configuration
    #[serde(default)]
    pub budget: BudgetConfig,

    /// Transient filter configuration
    #[serde(default)]
    pub transient_filter: TransientFilterConfig,

    /// Legacy flat structure (for backwards compatibility)
    #[serde(flatten)]
    pub legacy: Option<serde_json::Value>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// Server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    
    #[serde(default = "default_port")]
    pub port: u16,
    
    #[serde(default = "default_api_key")]
    pub api_key: String,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3160
}

fn default_api_key() -> String {
    "anchor-engine-default-key".to_string()
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            api_key: default_api_key(),
        }
    }
}

/// GitHub configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub token: Option<String>,
    
    #[serde(default = "default_branch")]
    pub default_branch: String,
    
    #[serde(default = "github_api_url")]
    pub api_base_url: String,
}

fn default_branch() -> String {
    "main".to_string()
}

fn github_api_url() -> String {
    "https://api.github.com".to_string()
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            token: None,
            default_branch: default_branch(),
            api_base_url: github_api_url(),
        }
    }
}

/// Path configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    #[serde(default = "default_notebook")]
    pub notebook: String,
    
    #[serde(default = "default_inbox")]
    pub inbox: String,
    
    #[serde(default = "default_external_inbox")]
    pub external_inbox: String,
    
    #[serde(default = "default_mirrored_brain")]
    pub mirrored_brain: String,
    
    #[serde(default = "default_database")]
    pub database: String,
    
    #[serde(default = "default_logs")]
    pub logs: String,
}

fn default_notebook() -> String {
    "notebook".to_string()
}

fn default_inbox() -> String {
    "notebook/inbox".to_string()
}

fn default_external_inbox() -> String {
    "notebook/external-inbox".to_string()
}

fn default_mirrored_brain() -> String {
    ".anchor/mirrored_brain".to_string()
}

fn default_database() -> String {
    "anchor.db".to_string()
}

fn default_logs() -> String {
    "logs".to_string()
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            notebook: default_notebook(),
            inbox: default_inbox(),
            external_inbox: default_external_inbox(),
            mirrored_brain: default_mirrored_brain(),
            database: default_database(),
            logs: default_logs(),
        }
    }
}

/// Search configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    #[serde(default = "default_strategy")]
    pub strategy: String,
    
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    
    #[serde(default = "default_fts_window")]
    pub fts_window_size: usize,
    
    #[serde(default = "default_fts_padding")]
    pub fts_padding: usize,
}

fn default_strategy() -> String {
    "hybrid".to_string()
}

fn default_max_results() -> usize {
    50
}

fn default_fts_window() -> usize {
    1500
}

fn default_fts_padding() -> usize {
    750
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            strategy: default_strategy(),
            max_results: default_max_results(),
            fts_window_size: default_fts_window(),
            fts_padding: default_fts_padding(),
        }
    }
}

/// Watcher configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    #[serde(default = "default_debounce")]
    pub debounce_ms: u64,
    
    #[serde(default = "default_stability")]
    pub stability_threshold_ms: u64,
    
    #[serde(default)]
    pub extra_paths: Vec<String>,
    
    #[serde(default)]
    pub auto_start: bool,
}

fn default_debounce() -> u64 {
    2000
}

fn default_stability() -> u64 {
    2000
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            debounce_ms: default_debounce(),
            stability_threshold_ms: default_stability(),
            extra_paths: vec![],
            auto_start: false,
        }
    }
}

/// Limits configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    #[serde(default = "default_max_file_size")]
    pub max_file_size_bytes: u64,
    
    #[serde(default = "default_max_content")]
    pub max_content_length_chars: usize,
    
    #[serde(default = "default_max_chunk")]
    pub max_chunk_size_chars: usize,
}

fn default_max_file_size() -> u64 {
    104857600 // 100MB
}

fn default_max_content() -> usize {
    5000
}

fn default_max_chunk() -> usize {
    3000
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            max_file_size_bytes: default_max_file_size(),
            max_content_length_chars: default_max_content(),
            max_chunk_size_chars: default_max_chunk(),
        }
    }
}

/// Ingestion configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionConfig {
    #[serde(default = "default_concept_density")]
    pub concept_density: String,

    #[serde(default = "default_tag_threshold")]
    pub tag_threshold: f64,

    #[serde(default = "default_dedup")]
    pub dedup_strength: String,

    #[serde(default = "default_token_budget")]
    pub token_budget_default: usize,

    #[serde(default = "default_max_keywords")]
    pub max_keywords: usize,

    #[serde(default = "default_min_keyword_score")]
    pub min_keyword_score: f64,

    #[serde(default = "default_true")]
    pub sanitize: bool,
}

fn default_concept_density() -> String {
    "high".to_string()
}

fn default_tag_threshold() -> f64 {
    0.7
}

fn default_dedup() -> String {
    "aggressive".to_string()
}

fn default_ingestion_token_budget() -> usize {
    2000
}

fn default_max_keywords() -> usize {
    10
}

fn default_min_keyword_score() -> f64 {
    0.3
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            concept_density: default_concept_density(),
            tag_threshold: default_tag_threshold(),
            dedup_strength: default_dedup(),
            token_budget_default: default_ingestion_token_budget(),
            max_keywords: default_max_keywords(),
            min_keyword_score: default_min_keyword_score(),
            sanitize: true,
        }
    }
}

/// Budget configuration for search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Planet budget fraction (default: 0.70)
    #[serde(default = "default_planet_budget")]
    pub planet_budget: f32,

    /// Moon budget fraction (default: 0.30)
    #[serde(default = "default_moon_budget")]
    pub moon_budget: f32,

    /// Total token budget
    #[serde(default = "default_token_budget")]
    pub total_tokens: usize,

    /// Enable max-recall mode (for 16K+ token queries)
    #[serde(default)]
    pub max_recall: bool,
}

fn default_planet_budget() -> f32 {
    0.70
}

fn default_moon_budget() -> f32 {
    0.30
}

fn default_token_budget() -> usize {
    8192
}

fn default_budget_tokens() -> usize {
    8192
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            planet_budget: default_planet_budget(),
            moon_budget: default_moon_budget(),
            total_tokens: default_budget_tokens(),
            max_recall: false,
        }
    }
}

/// Configuration for transient filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransientFilterConfig {
    /// Minimum number of lines to consider for filtering
    #[serde(default = "default_min_lines")]
    pub min_lines: usize,

    /// Threshold for transient content (0.0-1.0)
    /// If >50% lines match transient patterns, skip entire document
    #[serde(default = "default_threshold")]
    pub threshold: f64,
}

fn default_min_lines() -> usize {
    5
}

fn default_threshold() -> f64 {
    0.5
}

impl Default for TransientFilterConfig {
    fn default() -> Self {
        Self {
            min_lines: default_min_lines(),
            threshold: default_threshold(),
        }
    }
}

/// Main UserSettings structure (backwards compatible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    /// Directories to watch for ingestion
    #[serde(default)]
    pub watch_paths: Vec<String>,

    /// Path to the inbox directory
    pub inbox_path: Option<String>,

    /// Path to the external inbox directory
    pub external_inbox_path: Option<String>,

    /// Path to the mirrored brain directory
    pub mirrored_brain_path: Option<String>,

    /// Path to the database file (ephemeral)
    pub database_path: Option<String>,

    /// GitHub OAuth token (optional, for private repos)
    pub github_token: Option<String>,

    /// Watcher stability threshold in milliseconds
    #[serde(default = "default_stability_threshold")]
    pub watcher_stability_threshold_ms: u64,

    /// Enable auto-ingest on file change
    #[serde(default = "default_true")]
    pub auto_ingest: bool,

    /// Batch size for ingestion
    #[serde(default = "default_batch_size")]
    pub ingestion_batch_size: usize,
}

fn default_stability_threshold() -> u64 {
    500
}

fn default_true() -> bool {
    true
}

fn default_batch_size() -> usize {
    50
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            watch_paths: vec![],
            inbox_path: None,
            external_inbox_path: None,
            mirrored_brain_path: None,
            database_path: None,
            github_token: None,
            watcher_stability_threshold_ms: 500,
            auto_ingest: true,
            ingestion_batch_size: 50,
        }
    }
}

impl Config {
    /// Load configuration from user_settings.json.
    pub fn load() -> Result<Self, ConfigError> {
        // Try to load from current directory first, then parent
        let paths = [
            PathBuf::from("user_settings.json"),
            PathBuf::from("../user_settings.json"),
            PathBuf::from("../../user_settings.json"),
        ];

        for path in &paths {
            if path.exists() {
                let content = fs::read_to_string(path)?;
                let config: Config = serde_json::from_str(&content)?;
                return Ok(config);
            }
        }

        // Return default config if no file found
        Ok(Self::default())
    }

    /// Load configuration with environment variable overrides.
    ///
    /// Environment variables:
    /// - ANCHOR_SERVER_PORT: Override server port
    /// - ANCHOR_SERVER_HOST: Override server host
    /// - ANCHOR_API_KEY: Override API key
    /// - ANCHOR_DB_PATH: Override database path
    /// - ANCHOR_MIRROR_PATH: Override mirrored brain path
    pub fn load_with_env_overrides() -> Result<Self, ConfigError> {
        let mut config = Self::load()?;

        // Override with environment variables
        if let Ok(port) = std::env::var("ANCHOR_SERVER_PORT") {
            if let Ok(parsed) = port.parse() {
                config.server.port = parsed;
            }
        }

        if let Ok(host) = std::env::var("ANCHOR_SERVER_HOST") {
            config.server.host = host;
        }

        if let Ok(api_key) = std::env::var("ANCHOR_API_KEY") {
            config.server.api_key = api_key;
        }

        if let Ok(db_path) = std::env::var("ANCHOR_DB_PATH") {
            config.paths.database = db_path;
        }

        if let Ok(mirror_path) = std::env::var("ANCHOR_MIRROR_PATH") {
            config.paths.mirrored_brain = mirror_path;
        }

        Ok(config)
    }

    /// Validate configuration.
    ///
    /// Checks for:
    /// - Valid port range (1-65535)
    /// - Valid budget fractions (must sum to ~1.0)
    /// - Valid threshold values (0.0-1.0)
    /// - Path validity
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate port
        if self.server.port == 0 {
            return Err(ConfigError::ValidationError(
                "Server port must be between 1 and 65535".to_string(),
            ));
        }

        // Validate budget fractions
        let budget_sum = self.budget.planet_budget + self.budget.moon_budget;
        if (budget_sum - 1.0).abs() > 0.01 {
            return Err(ConfigError::ValidationError(
                format!(
                    "Budget fractions must sum to 1.0, got {}",
                    budget_sum
                ),
            ));
        }

        // Validate transient filter threshold
        if self.transient_filter.threshold < 0.0 || self.transient_filter.threshold > 1.0 {
            return Err(ConfigError::ValidationError(
                format!(
                    "Transient filter threshold must be between 0.0 and 1.0, got {}",
                    self.transient_filter.threshold
                ),
            ));
        }

        // Validate ingestion settings
        if self.ingestion.max_keywords == 0 {
            return Err(ConfigError::ValidationError(
                "max_keywords must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
    
    /// Get the inbox path.
    pub fn inbox_path(&self) -> PathBuf {
        PathBuf::from(&self.paths.inbox)
    }
    
    /// Get the external inbox path.
    pub fn external_inbox_path(&self) -> PathBuf {
        PathBuf::from(&self.paths.external_inbox)
    }
    
    /// Get the mirrored brain path.
    pub fn mirrored_brain_path(&self) -> PathBuf {
        PathBuf::from(&self.paths.mirrored_brain)
    }
    
    /// Get the database path.
    pub fn database_path(&self) -> PathBuf {
        PathBuf::from(&self.paths.database)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: default_version(),
            server: ServerConfig::default(),
            github: GitHubConfig::default(),
            paths: PathConfig::default(),
            search: SearchConfig::default(),
            watcher: WatcherConfig::default(),
            limits: LimitsConfig::default(),
            ingestion: IngestionConfig::default(),
            budget: BudgetConfig::default(),
            transient_filter: TransientFilterConfig::default(),
            legacy: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 3160);
        assert_eq!(config.github.default_branch, "main");
        assert_eq!(config.paths.inbox, "notebook/inbox");
        assert_eq!(config.ingestion.max_keywords, 10);
    }
}

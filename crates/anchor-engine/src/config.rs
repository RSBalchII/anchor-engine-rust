//! Configuration module for Anchor Engine.
//!
//! Handles loading and saving user settings from user_settings.json

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use thiserror::Error;

/// Configuration errors.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] serde_json::Error),
}

/// User settings configuration.
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

impl UserSettings {
    /// Load settings from a JSON file.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            // Return default settings if file doesn't exist
            return Ok(Self::default());
        }
        
        let content = fs::read_to_string(path)?;
        let settings: UserSettings = serde_json::from_str(&content)?;
        Ok(settings)
    }
    
    /// Save settings to a JSON file.
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    /// Get the inbox directory path.
    pub fn inbox_path(&self) -> PathBuf {
        self.inbox_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("inbox"))
    }
    
    /// Get the external inbox directory path.
    pub fn external_inbox_path(&self) -> PathBuf {
        self.external_inbox_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("external-inbox"))
    }
    
    /// Get the mirrored brain directory path.
    pub fn mirrored_brain_path(&self) -> PathBuf {
        self.mirrored_brain_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("mirrored_brain"))
    }
    
    /// Get the database file path.
    pub fn database_path(&self) -> PathBuf {
        self.database_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("anchor.db"))
    }
    
    /// Get all watch paths (inbox, external-inbox, and extra paths).
    pub fn all_watch_paths(&self) -> Vec<PathBuf> {
        let mut paths = vec![
            self.inbox_path(),
            self.external_inbox_path(),
        ];
        
        // Add extra watch paths
        for path in &self.watch_paths {
            paths.push(PathBuf::from(path));
        }
        
        paths
    }
}

/// Global configuration.
#[derive(Debug, Clone)]
pub struct Config {
    pub settings: UserSettings,
    pub config_path: PathBuf,
}

impl Config {
    /// Load configuration from the default path.
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = PathBuf::from("user_settings.json");
        let settings = UserSettings::load(&config_path)?;
        
        Ok(Self {
            settings,
            config_path,
        })
    }
    
    /// Load configuration from a specific path.
    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        let settings = UserSettings::load(path)?;
        
        Ok(Self {
            settings,
            config_path: path.to_path_buf(),
        })
    }
    
    /// Save current configuration.
    pub fn save(&self) -> Result<(), ConfigError> {
        self.settings.save(&self.config_path)
    }
    
    /// Add a watch path and save.
    pub fn add_watch_path(&mut self, path: &str) -> Result<(), ConfigError> {
        if !self.settings.watch_paths.contains(&path.to_string()) {
            self.settings.watch_paths.push(path.to_string());
            self.save()?;
        }
        Ok(())
    }
    
    /// Remove a watch path and save.
    pub fn remove_watch_path(&mut self, path: &str) -> Result<(), ConfigError> {
        self.settings.watch_paths.retain(|p| p != path);
        self.save()?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            settings: UserSettings::default(),
            config_path: PathBuf::from("user_settings.json"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_default_settings() {
        let settings = UserSettings::default();
        assert!(settings.watch_paths.is_empty());
        assert_eq!(settings.watcher_stability_threshold_ms, 500);
        assert!(settings.auto_ingest);
        assert_eq!(settings.ingestion_batch_size, 50);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("nonexistent.json");
        
        let settings = UserSettings::load(&config_path).unwrap();
        assert_eq!(settings.watch_paths.len(), 0);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("settings.json");
        
        let mut settings = UserSettings::default();
        settings.inbox_path = Some("/test/inbox".to_string());
        settings.watch_paths = vec!["/extra/path".to_string()];
        
        settings.save(&config_path).unwrap();
        
        let loaded = UserSettings::load(&config_path).unwrap();
        assert_eq!(loaded.inbox_path, Some("/test/inbox".to_string()));
        assert_eq!(loaded.watch_paths.len(), 1);
    }

    #[test]
    fn test_config_paths() {
        let settings = UserSettings::default();
        
        assert_eq!(settings.inbox_path(), PathBuf::from("inbox"));
        assert_eq!(settings.external_inbox_path(), PathBuf::from("external-inbox"));
        assert_eq!(settings.mirrored_brain_path(), PathBuf::from("mirrored_brain"));
        assert_eq!(settings.database_path(), PathBuf::from("anchor.db"));
    }

    #[test]
    fn test_all_watch_paths() {
        let mut settings = UserSettings::default();
        settings.watch_paths = vec!["/extra/path".to_string()];
        
        let paths = settings.all_watch_paths();
        assert_eq!(paths.len(), 3); // inbox, external-inbox, extra
        assert!(paths.contains(&PathBuf::from("inbox")));
        assert!(paths.contains(&PathBuf::from("external-inbox")));
        assert!(paths.contains(&PathBuf::from("/extra/path")));
    }

    #[test]
    fn test_add_remove_watch_path() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("settings.json");
        
        let mut config = Config::load_from(&config_path).unwrap();
        
        config.add_watch_path("/path/one").unwrap();
        assert_eq!(config.settings.watch_paths.len(), 1);
        
        config.add_watch_path("/path/two").unwrap();
        assert_eq!(config.settings.watch_paths.len(), 2);
        
        config.remove_watch_path("/path/one").unwrap();
        assert_eq!(config.settings.watch_paths.len(), 1);
        assert_eq!(config.settings.watch_paths[0], "/path/two");
    }
}

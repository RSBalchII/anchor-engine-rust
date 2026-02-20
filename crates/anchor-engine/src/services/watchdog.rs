//! Watchdog Service - Monitors directories for file changes and triggers ingestion.
//!
//! This service watches configured directories (inbox, external-inbox, and extra paths)
//! for new files and changes, then triggers the ingestion pipeline.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{Duration, sleep};
use tracing::{info, warn, error};
use ignore::WalkBuilder;

use crate::config::UserSettings;
use crate::services::ingestion::IngestionService;

/// Watchdog service configuration.
#[derive(Debug, Clone)]
pub struct WatchdogConfig {
    /// Directories to watch
    pub watch_paths: Vec<PathBuf>,
    /// Stability threshold in milliseconds
    pub stability_threshold_ms: u64,
    /// Enable auto-ingest
    pub auto_ingest: bool,
    /// Ignore patterns (dotfiles, etc.)
    pub ignore_patterns: Vec<String>,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            watch_paths: vec![],
            stability_threshold_ms: 500,
            auto_ingest: true,
            ignore_patterns: vec![
                ".git".to_string(),        // Git directories (not all dotfiles)
                "node_modules".to_string(), // Node modules
                "target".to_string(),      // Rust build artifacts
                "*.swp".to_string(),       // Vim swap files
                "*.bak".to_string(),       // Backup files
                "*.log".to_string(),       // Log files
            ],
        }
    }
}

impl From<&UserSettings> for WatchdogConfig {
    fn from(settings: &UserSettings) -> Self {
        Self {
            watch_paths: settings.all_watch_paths(),
            stability_threshold_ms: settings.watcher_stability_threshold_ms,
            auto_ingest: settings.auto_ingest,
            ..Default::default()
        }
    }
}

/// Watchdog service state.
#[derive(Debug, Clone)]
pub struct WatchdogState {
    /// Is the watchdog running?
    pub is_running: bool,
    /// Number of files processed
    pub files_processed: usize,
    /// Number of errors
    pub errors: usize,
    /// Currently watched paths
    pub watched_paths: Vec<PathBuf>,
}

/// Watchdog service for monitoring file system changes.
pub struct WatchdogService {
    /// Service configuration
    config: WatchdogConfig,
    /// Ingestion service
    ingestion: Arc<RwLock<IngestionService>>,
    /// Service state
    state: Arc<Mutex<WatchdogState>>,
    /// Set of processed files (to avoid duplicates)
    processed_files: Arc<Mutex<HashSet<PathBuf>>>,
}

impl WatchdogService {
    /// Create a new Watchdog service.
    pub fn new(config: WatchdogConfig, ingestion: Arc<RwLock<IngestionService>>) -> Self {
        Self {
            config,
            ingestion,
            state: Arc::new(Mutex::new(WatchdogState {
                is_running: false,
                files_processed: 0,
                errors: 0,
                watched_paths: vec![],
            })),
            processed_files: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Create from user settings.
    pub fn from_settings(
        settings: &UserSettings,
        ingestion: Arc<RwLock<IngestionService>>,
    ) -> Self {
        let config = WatchdogConfig::from(settings);
        Self::new(config, ingestion)
    }

    /// Start the watchdog service.
    pub async fn start(&self) {
        let mut state = self.state.lock().await;
        if state.is_running {
            warn!("Watchdog service is already running");
            return;
        }

        info!("📥 Watchdog Service: ACTIVE");
        info!("   Watching {} directories:", self.config.watch_paths.len());
        for path in &self.config.watch_paths {
            info!("     - {:?}", path);
        }

        state.is_running = true;
        state.watched_paths = self.config.watch_paths.clone();
        drop(state);

        // Start the watch loop
        let self_arc = Arc::new(self.clone());
        tokio::spawn(async move {
            self_arc.watch_loop().await;
        });
    }

    /// Stop the watchdog service.
    pub async fn stop(&self) {
        let mut state = self.state.lock().await;
        if !state.is_running {
            warn!("Watchdog service is not running");
            return;
        }

        info!("Stopping Watchdog service...");
        state.is_running = false;
        info!("Watchdog service stopped");
    }

    /// Main watch loop.
    async fn watch_loop(&self) {
        let stability_duration = Duration::from_millis(self.config.stability_threshold_ms);

        loop {
            // Check if we should stop
            {
                let state = self.state.lock().await;
                if !state.is_running {
                    break;
                }
            }

            // Scan all watched directories
            for watch_path in &self.config.watch_paths {
                if let Err(e) = self.scan_directory(watch_path).await {
                    error!("Error scanning directory {:?}: {}", watch_path, e);
                    let mut state = self.state.lock().await;
                    state.errors += 1;
                }
            }

            // Wait before next scan
            sleep(stability_duration).await;
        }
    }

    /// Scan a directory for files to ingest (recursively, respecting .gitignore).
    async fn scan_directory(&self, dir_path: &Path) -> Result<(), std::io::Error> {
        if !dir_path.exists() {
            return Ok(());
        }

        // Use ignore crate for .gitignore-aware walking
        let mut found_files = Vec::new();
        
        let walker = WalkBuilder::new(dir_path)
            .hidden(false)  // Don't skip hidden files (we want .gitignore etc)
            .git_global(true)  // Use global gitignore
            .git_ignore(true)  // Use .gitignore files
            .ignore(false)  // Don't use .ignore files
            .build();

        for result in walker {
            match result {
                Ok(entry) => {
                    if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                        found_files.push(entry.path().to_path_buf());
                    }
                }
                Err(e) => {
                    warn!("Error walking directory: {}", e);
                }
            }
        }

        // Process found files
        for path in found_files {
            self.process_file(&path).await;
        }

        Ok(())
    }

    /// Process a single file for ingestion.
    async fn process_file(&self, path: &Path) {
        // Check if file should be ignored
        if self.should_ignore(path) {
            return;
        }

        // Check if already processed
        {
            let processed = self.processed_files.lock().await;
            if processed.contains(path) {
                return;
            }
        }

        // Ingest the file
        if self.config.auto_ingest {
            let ingestion = self.ingestion.read().await;
            match ingestion.ingest_file(path).await {
                Ok(result) => {
                    info!("✅ INGESTED: {:?} → {} atoms, {} tags in {:.1}ms", 
                          path.file_name().unwrap_or_default(), 
                          result.atoms_created, 
                          result.tags.len(),
                          result.processing_time_ms);
                    
                    // Mark as processed
                    let mut processed = self.processed_files.lock().await;
                    processed.insert(path.to_path_buf());
                    
                    // Update stats
                    let mut state = self.state.lock().await;
                    state.files_processed += 1;
                }
                Err(e) => {
                    error!("❌ FAILED: {:?} - {}", path.file_name().unwrap_or_default(), e);
                    let mut state = self.state.lock().await;
                    state.errors += 1;
                }
            }
        }
    }

    /// Check if a path should be ignored.
    fn should_ignore(&self, path: &Path) -> bool {
        // Check filename
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            for pattern in &self.config.ignore_patterns {
                if pattern.starts_with('*') {
                    // Extension pattern (e.g., *.swp)
                    if file_name.ends_with(&pattern[1..]) {
                        return true;
                    }
                } else if file_name.contains(pattern) {
                    return true;
                }
            }
        }

        // Check if any parent directory should be ignored
        if let Some(parent) = path.parent() {
            for component in parent.components() {
                if let Some(name) = component.as_os_str().to_str() {
                    for pattern in &self.config.ignore_patterns {
                        if !pattern.starts_with('*') && name.contains(pattern) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Add a watch path dynamically.
    pub async fn add_watch_path(&self, path: &Path) {
        if !path.exists() {
            warn!("Watch path does not exist: {:?}", path);
            return;
        }

        let mut state = self.state.lock().await;
        let path_buf = path.to_path_buf();
        if !state.watched_paths.contains(&path_buf) {
            info!("Added watch path: {:?}", path);
            state.watched_paths.push(path_buf);
        }
    }

    /// Remove a watch path dynamically.
    pub async fn remove_watch_path(&self, path: &Path) {
        let mut state = self.state.lock().await;
        if let Some(pos) = state.watched_paths.iter().position(|p| p == path) {
            info!("Removed watch path: {:?}", path);
            state.watched_paths.remove(pos);
        }
    }

    /// Get current service state.
    pub async fn get_state(&self) -> WatchdogState {
        self.state.lock().await.clone()
    }

    /// Reset processed files set (allows re-ingestion).
    pub async fn reset_processed_files(&self) {
        let mut processed = self.processed_files.lock().await;
        processed.clear();
        info!("Reset processed files cache");
    }
}

impl Clone for WatchdogService {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            ingestion: Arc::clone(&self.ingestion),
            state: Arc::clone(&self.state),
            processed_files: Arc::clone(&self.processed_files),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[tokio::test]
    async fn test_watchdog_creation() {
        let config = WatchdogConfig::default();
        let ingestion = Arc::new(RwLock::new(IngestionService::in_memory().unwrap()));
        let watchdog = WatchdogService::new(config, ingestion);
        
        let state = watchdog.get_state().await;
        assert!(!state.is_running);
        assert_eq!(state.files_processed, 0);
        assert_eq!(state.errors, 0);
    }

    #[tokio::test]
    async fn test_should_ignore_patterns() {
        let config = WatchdogConfig::default();
        let ingestion = Arc::new(RwLock::new(IngestionService::in_memory().unwrap()));
        let watchdog = WatchdogService::new(config, ingestion);
        
        // Should ignore dotfiles (files starting with .)
        assert!(watchdog.should_ignore(Path::new(".gitignore")));
        assert!(watchdog.should_ignore(Path::new(".bashrc")));
        
        // Should ignore node_modules directory
        assert!(watchdog.should_ignore(Path::new("node_modules")));
        
        // Should ignore *.swp files (vim swap files)
        assert!(watchdog.should_ignore(Path::new("file.txt.swp")));
        assert!(watchdog.should_ignore(Path::new("test.swp")));
        
        // Should ignore target directory (Rust build artifacts)
        assert!(watchdog.should_ignore(Path::new("target")));
        
        // Should NOT ignore normal files without extensions that might match patterns
        assert!(!watchdog.should_ignore(Path::new("document")));
        assert!(!watchdog.should_ignore(Path::new("code")));
        assert!(!watchdog.should_ignore(Path::new("README")));
    }

    #[tokio::test]
    async fn test_add_remove_watch_path() {
        let temp_dir = tempdir().unwrap();
        let watch_path = temp_dir.path().to_path_buf();
        
        let config = WatchdogConfig::default();
        let ingestion = Arc::new(RwLock::new(IngestionService::in_memory().unwrap()));
        let watchdog = WatchdogService::new(config, ingestion);
        
        // Add watch path
        watchdog.add_watch_path(&watch_path).await;
        let state = watchdog.get_state().await;
        assert!(state.watched_paths.contains(&watch_path));
        
        // Remove watch path
        watchdog.remove_watch_path(&watch_path).await;
        let state = watchdog.get_state().await;
        assert!(!state.watched_paths.contains(&watch_path));
    }

    #[tokio::test]
    async fn test_reset_processed_files() {
        let config = WatchdogConfig::default();
        let ingestion = Arc::new(RwLock::new(IngestionService::in_memory().unwrap()));
        let watchdog = WatchdogService::new(config, ingestion);
        
        // Add some processed files
        {
            let mut processed = watchdog.processed_files.lock().await;
            processed.insert(PathBuf::from("/test/file1.txt"));
            processed.insert(PathBuf::from("/test/file2.txt"));
        }
        
        // Reset
        watchdog.reset_processed_files().await;
        
        // Should be empty
        let processed = watchdog.processed_files.lock().await;
        assert!(processed.is_empty());
    }
}

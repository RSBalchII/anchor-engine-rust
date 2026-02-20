//! GitHub Service - Fetch and extract repository tarballs.
//!
//! This service handles:
//! 1. Fetching tarballs from GitHub (public or private repos)
//! 2. Extracting tarballs to external-inbox directory
//! 3. Tracking repository metadata

use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Read, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use flate2::read::GzDecoder;
use tar::Archive;
use thiserror::Error;
use tracing::{info, warn, error, debug};

/// GitHub service errors.
#[derive(Error, Debug)]
pub enum GitHubError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("Tar extraction error: {0}")]
    TarError(String),
    
    #[error("Invalid GitHub URL: {0}")]
    InvalidUrl(String),
    
    #[error("Repository not found: {0}")]
    NotFound(String),
}

/// Result type for GitHub operations.
pub type Result<T> = std::result::Result<T, GitHubError>;

/// GitHub repository information.
#[derive(Debug, Clone)]
pub struct GitHubRepo {
    /// Owner username
    pub owner: String,
    /// Repository name
    pub repo: String,
    /// Branch, tag, or commit SHA (default: "main" or "master")
    pub ref_name: Option<String>,
    /// GitHub OAuth token (optional, for private repos)
    pub token: Option<String>,
}

impl GitHubRepo {
    /// Parse a GitHub URL into repo info.
    ///
    /// Supports formats:
    /// - `https://github.com/owner/repo`
    /// - `https://github.com/owner/repo/tree/branch`
    /// - `owner/repo`
    pub fn from_url(url: &str) -> Result<Self> {
        let url = url.trim();
        
        // Handle short format: owner/repo
        if !url.starts_with("http") {
            let parts: Vec<&str> = url.split('/').collect();
            if parts.len() != 2 {
                return Err(GitHubError::InvalidUrl(
                    "Expected format: owner/repo or https://github.com/owner/repo".to_string()
                ));
            }
            return Ok(Self {
                owner: parts[0].to_string(),
                repo: parts[1].to_string(),
                ref_name: None,
                token: None,
            });
        }
        
        // Parse full URL
        let url = url.trim_end_matches('/');
        
        // Extract owner and repo from URL
        // Format: https://github.com/owner/repo[/tree/ref]
        let parts: Vec<&str> = url.split('/').collect();
        
        if parts.len() < 5 || parts[2] != "github.com" {
            return Err(GitHubError::InvalidUrl(
                "Expected format: https://github.com/owner/repo".to_string()
            ));
        }
        
        let owner = parts[3].to_string();
        let repo_part = parts[4];
        
        // Handle /tree/branch format
        let (repo, ref_name) = if repo_part.contains("?") || url.contains("/tree/") {
            let repo = repo_part.split('?').next().unwrap_or(repo_part).to_string();
            let ref_name = url.split("/tree/").nth(1)
                .and_then(|s| s.split('?').next())
                .map(|s| s.to_string());
            (repo, ref_name)
        } else {
            (repo_part.to_string(), None)
        };
        
        Ok(Self {
            owner,
            repo,
            ref_name,
            token: None,
        })
    }
    
    /// Set the OAuth token.
    pub fn with_token(mut self, token: &str) -> Self {
        self.token = Some(token.to_string());
        self
    }
    
    /// Set the ref (branch/tag/commit).
    pub fn with_ref(mut self, ref_name: &str) -> Self {
        self.ref_name = Some(ref_name.to_string());
        self
    }
    
    /// Get the tarball download URL.
    pub fn tarball_url(&self) -> String {
        let ref_part = match &self.ref_name {
            Some(ref_name) => format!("/{}", ref_name),
            None => String::new(),
        };
        
        format!(
            "https://api.github.com/repos/{}/{}/tarball{}",
            self.owner, self.repo, ref_part
        )
    }
    
    /// Get a name for the extracted directory.
    pub fn dir_name(&self) -> String {
        format!("{}-{}", self.owner, self.repo)
    }
}

/// GitHub service for fetching and extracting repositories.
pub struct GitHubService {
    /// HTTP client
    client: reqwest::Client,
    /// Base directory for extraction (external-inbox)
    extract_base: PathBuf,
    /// Tracked repositories (for periodic sync)
    tracked_repos: Arc<Mutex<Vec<TrackedRepo>>>,
}

/// A tracked repository for periodic sync.
#[derive(Debug, Clone)]
pub struct TrackedRepo {
    /// Repository information
    pub repo: GitHubRepo,
    /// Last sync timestamp
    pub last_sync: Option<std::time::SystemTime>,
    /// Sync interval in seconds
    pub sync_interval_secs: u64,
    /// Whether sync is enabled
    pub enabled: bool,
}

impl TrackedRepo {
    /// Create a new tracked repo.
    pub fn new(repo: GitHubRepo, sync_interval_secs: u64) -> Self {
        Self {
            repo,
            last_sync: None,
            sync_interval_secs,
            enabled: true,
        }
    }
    
    /// Check if sync is due.
    pub fn should_sync(&self) -> bool {
        if !self.enabled {
            return false;
        }
        
        match self.last_sync {
            None => true,  // Never synced, should sync now
            Some(last) => {
                let elapsed = last.elapsed().unwrap_or(std::time::Duration::from_secs(0));
                elapsed.as_secs() >= self.sync_interval_secs
            }
        }
    }
}

impl GitHubService {
    /// Create a new GitHub service.
    pub fn new(extract_base: PathBuf) -> Self {
        Self {
            client: reqwest::Client::new(),
            extract_base,
            tracked_repos: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Create with custom sync interval.
    pub fn with_sync_interval(extract_base: PathBuf, _interval_secs: u64) -> Self {
        Self::new(extract_base)
    }
    
    /// Fetch and extract a GitHub repository.
    ///
    /// # Arguments
    ///
    /// * `repo` - GitHub repository information
    ///
    /// # Returns
    ///
    /// Path to the extracted directory
    pub async fn fetch_and_extract(&self, repo: &GitHubRepo) -> Result<PathBuf> {
        info!("📥 Fetching GitHub repo: {}/{}", repo.owner, repo.repo);
        
        // Build request
        let url = repo.tarball_url();
        let mut request = self.client.get(&url);
        
        // Add authentication if token provided
        if let Some(token) = &repo.token {
            request = request.header("Authorization", format!("token {}", token));
        }
        
        // Add required GitHub API headers
        request = request.header("Accept", "application/vnd.github.v3+tar");
        request = request.header("User-Agent", "anchor-engine-rust");
        
        // Fetch tarball
        debug!("Downloading tarball from: {}", url);
        let response = request.send().await?;
        
        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(GitHubError::NotFound(repo.to_string()));
            }
            return Err(GitHubError::HttpError(
                reqwest::Error::from(response.error_for_status().unwrap_err())
            ));
        }
        
        // Get tarball bytes
        let tarball_bytes = response.bytes().await?;
        info!("✓ Downloaded tarball: {} bytes", tarball_bytes.len());
        
        // Extract tarball
        let extract_path = self.extract_base.join(&repo.dir_name());
        self.extract_tarball(&tarball_bytes, &extract_path)?;
        
        info!("✅ Extracted to: {:?}", extract_path);
        
        Ok(extract_path)
    }
    
    /// Extract a tarball to a directory.
    fn extract_tarball(&self, tarball_bytes: &[u8], extract_path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = extract_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Remove existing directory if it exists
        if extract_path.exists() {
            fs::remove_dir_all(extract_path)?;
        }
        
        // Create extraction directory
        fs::create_dir_all(extract_path)?;
        
        // Decompress gzip
        let tar = GzDecoder::new(tarball_bytes);
        
        // Extract tar archive
        let mut archive = Archive::new(tar);
        
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            
            // Skip the top-level directory (GitHub adds owner-repo-commit/)
            let relative_path = path.components().skip(1).collect::<PathBuf>();
            
            if relative_path.as_os_str().is_empty() {
                continue;
            }
            
            let dest_path = extract_path.join(&relative_path);
            
            // Ensure parent directory exists
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Extract file or directory
            if entry.header().entry_type().is_file() {
                let mut contents = Vec::new();
                entry.read_to_end(&mut contents)?;
                fs::write(&dest_path, &contents)?;
                debug!("✓ Extracted: {:?}", relative_path);
            }
        }
        
        Ok(())
    }
    
    /// Fetch repository metadata (optional enhancement).
    pub async fn get_repo_info(&self, repo: &GitHubRepo) -> Result<serde_json::Value> {
        let url = format!("https://api.github.com/repos/{}/{}", repo.owner, repo.repo);
        let mut request = self.client.get(&url);
        
        if let Some(token) = &repo.token {
            request = request.header("Authorization", format!("token {}", token));
        }
        
        request = request.header("User-Agent", "anchor-engine-rust");
        
        let response = request.send().await?;
        
        if !response.status().is_success() {
            return Err(GitHubError::HttpError(
                reqwest::Error::from(response.error_for_status().unwrap_err())
            ));
        }
        
        let json: serde_json::Value = response.json().await?;
        Ok(json)
    }
    
    /// Add a repository to the watch list.
    pub async fn track_repo(&self, repo: GitHubRepo, sync_interval_secs: u64) {
        let tracked = TrackedRepo::new(repo, sync_interval_secs);
        let mut repos = self.tracked_repos.lock().await;
        repos.push(tracked);
        info!("📍 Tracking GitHub repo: {}/{} (sync every {}s)", 
              repos.last().unwrap().repo.owner, 
              repos.last().unwrap().repo.repo,
              sync_interval_secs);
    }
    
    /// Sync all tracked repositories.
    pub async fn sync_tracked_repos(&self) -> Result<Vec<PathBuf>> {
        let mut synced_paths = Vec::new();
        
        // Clone the list to avoid holding the lock
        let repos_to_sync = {
            let repos = self.tracked_repos.lock().await;
            repos.iter()
                .filter(|r| r.should_sync())
                .map(|r| r.repo.clone())
                .collect::<Vec<_>>()
        };
        
        for repo in repos_to_sync {
            info!("🔄 Syncing GitHub repo: {}/{}", repo.owner, repo.repo);
            
            match self.fetch_and_extract(&repo).await {
                Ok(path) => {
                    synced_paths.push(path.clone());
                    
                    // Update last sync time
                    let mut repos = self.tracked_repos.lock().await;
                    if let Some(tracked) = repos.iter_mut().find(|t| t.repo.owner == repo.owner && t.repo.repo == repo.repo) {
                        tracked.last_sync = Some(std::time::SystemTime::now());
                    }
                    
                    info!("✅ Synced: {}/{}", repo.owner, repo.repo);
                }
                Err(e) => {
                    error!("❌ Failed to sync {}/{}: {}", repo.owner, repo.repo, e);
                }
            }
        }
        
        Ok(synced_paths)
    }
    
    /// Start the background sync loop.
    pub async fn start_sync_loop(self: Arc<Self>) {
        tokio::spawn(async move {
            info!("🔄 GitHub sync loop started");
            
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                
                match self.sync_tracked_repos().await {
                    Ok(paths) => {
                        if !paths.is_empty() {
                            info!("📊 Synced {} repos", paths.len());
                        }
                    }
                    Err(e) => {
                        error!("❌ Sync loop error: {}", e);
                    }
                }
            }
        });
    }
    
    /// Get list of tracked repos.
    pub async fn get_tracked_repos(&self) -> Vec<TrackedRepo> {
        self.tracked_repos.lock().await.clone()
    }
}

impl std::fmt::Display for GitHubRepo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.owner, self.repo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url_short() {
        let repo = GitHubRepo::from_url("owner/repo").unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.repo, "repo");
        assert!(repo.ref_name.is_none());
    }

    #[test]
    fn test_parse_github_url_full() {
        let repo = GitHubRepo::from_url("https://github.com/owner/repo").unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.repo, "repo");
        assert!(repo.ref_name.is_none());
    }

    #[test]
    fn test_parse_github_url_with_branch() {
        let repo = GitHubRepo::from_url("https://github.com/owner/repo/tree/main").unwrap();
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.repo, "repo");
        assert_eq!(repo.ref_name, Some("main".to_string()));
    }

    #[test]
    fn test_parse_github_url_invalid() {
        assert!(GitHubRepo::from_url("invalid").is_err());
        assert!(GitHubRepo::from_url("https://gitlab.com/owner/repo").is_err());
    }

    #[test]
    fn test_tarball_url() {
        let repo = GitHubRepo::from_url("owner/repo").unwrap();
        assert_eq!(
            repo.tarball_url(),
            "https://api.github.com/repos/owner/repo/tarball"
        );
        
        let repo_with_ref = repo.with_ref("develop");
        assert_eq!(
            repo_with_ref.tarball_url(),
            "https://api.github.com/repos/owner/repo/tarball/develop"
        );
    }

    #[test]
    fn test_dir_name() {
        let repo = GitHubRepo::from_url("RSBalchII/anchor-engine").unwrap();
        assert_eq!(repo.dir_name(), "RSBalchII-anchor-engine");
    }
}

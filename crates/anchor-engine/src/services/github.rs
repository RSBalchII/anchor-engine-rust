//! GitHub Service - Fetch and extract repository tarballs with full metadata.
//!
//! This service provides:
//! 1. Secure credential storage (Windows Credential Manager / macOS Keychain / Linux libsecret)
//! 2. Fetching tarballs from GitHub (public or private repos)
//! 3. Extracting tarballs to external-inbox directory
//! 4. Fetching rich metadata (issues, PRs, contributors, releases)
//! 5. Generating YAML context files
//! 6. Commit history with authors
//! 7. Incremental update support

use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Read};
use std::sync::Arc;
use tokio::sync::Mutex;
use flate2::read::GzDecoder;
use tar::Archive;
use thiserror::Error;
use tracing::{info, warn, error, debug};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use chrono::Utc;

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
    /// Secure credential storage (Windows Credential Manager / macOS Keychain / Linux libsecret)
    credential_entry: Entry,
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

/// GitHub credential status (for API/UI responses).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CredentialStatus {
    pub has_credentials: bool,
    pub credential_source: Option<String>,
    pub message: String,
}

// ============================================================================
// GitHub Metadata Structures (like Node.js Octokit responses)
// ============================================================================

/// GitHub Issue metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub labels: Vec<String>,
    pub author: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub comments: i64,
}

/// GitHub Pull Request metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPullRequest {
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub author: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub merged: bool,
    pub merged_at: Option<String>,
    pub additions: i64,
    pub deletions: i64,
    pub changed_files: i64,
}

/// GitHub Contributor metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubContributor {
    pub login: String,
    pub contributions: i64,
    pub avatar_url: String,
}

/// GitHub Release metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub author: Option<String>,
    pub published_at: String,
    pub is_prerelease: bool,
}

/// Commit information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub author_email: Option<String>,
    pub date: String,
    pub committer: String,
}

/// Full GitHub repository metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubMetadata {
    pub issues: Vec<GitHubIssue>,
    pub pull_requests: Vec<GitHubPullRequest>,
    pub contributors: Vec<GitHubContributor>,
    pub releases: Vec<GitHubRelease>,
    pub commits: Vec<CommitInfo>,
    pub last_fetched: String,
}

/// Ingestion summary (saved to JSON for incremental updates).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionSummary {
    pub repo: String,
    pub branch: String,
    pub tarball: String,
    pub last_ingestion: String,
    pub commit_hash: String,
    pub metadata: IngestionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionMetadata {
    pub issues: i64,
    pub pull_requests: i64,
    pub contributors: i64,
    pub releases: i64,
    pub commits: i64,
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
    pub fn new(extract_base: PathBuf) -> Result<Self> {
        // Create credential entry (service="AnchorEngine", user="GitHub")
        let credential_entry = Entry::new("AnchorEngine", "GitHub")
            .map_err(|e| GitHubError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to create credential entry: {}", e)
            )))?;

        Ok(Self {
            client: reqwest::Client::new(),
            extract_base,
            tracked_repos: Arc::new(Mutex::new(Vec::new())),
            credential_entry,
        })
    }

    /// Create with custom sync interval.
    pub fn with_sync_interval(extract_base: PathBuf, _interval_secs: u64) -> Result<Self> {
        Self::new(extract_base)
    }

    // ========================================================================
    // Credential Management (Secure Storage)
    // ========================================================================

    /// Store GitHub PAT securely in OS credential manager.
    /// 
    /// # Arguments
    /// * `token` - GitHub Personal Access Token (format: ghp_...)
    pub fn store_credentials(&self, token: &str) -> Result<()> {
        self.credential_entry.set_password(token)
            .map_err(|e| GitHubError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to store credentials: {}", e)
            )))?;
        info!("✅ GitHub credentials stored securely");
        Ok(())
    }

    /// Retrieve GitHub PAT from secure storage.
    /// 
    /// # Returns
    /// * `Some(String)` - Token if found
    /// * `None` - No token stored (falls back to GITHUB_TOKEN env var)
    pub fn get_credentials(&self) -> Option<String> {
        // Try secure storage first
        if let Ok(token) = self.credential_entry.get_password() {
            return Some(token);
        }
        
        // Fallback to environment variable
        std::env::var("GITHUB_TOKEN").ok()
    }

    /// Delete stored credentials.
    pub fn delete_credentials(&self) -> Result<()> {
        self.credential_entry.delete_password()
            .map_err(|e| GitHubError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to delete credentials: {}", e)
            )))?;
        info!("🗑️ GitHub credentials deleted");
        Ok(())
    }

    /// Check if credentials are available (secure storage or env var).
    pub fn has_credentials(&self) -> bool {
        self.get_credentials().is_some()
    }

    /// Get credential status with details (for UI/API).
    pub fn get_credential_status(&self) -> CredentialStatus {
        let has_creds = self.has_credentials();
        let source = if self.credential_entry.get_password().is_ok() {
            Some("secure_storage".to_string())
        } else if std::env::var("GITHUB_TOKEN").is_ok() {
            Some("environment_variable".to_string())
        } else {
            None
        };
        
        CredentialStatus {
            has_credentials: has_creds,
            credential_source: source,
            message: if has_creds {
                "GitHub credentials configured".to_string()
            } else {
                "No GitHub credentials found. Set token via 'set' action or GITHUB_TOKEN env var.".to_string()
            },
        }
    }
    
    // ========================================================================
    // Metadata Fetching (Like Node.js Octokit)
    // ========================================================================

    /// Fetch full repository metadata including issues, PRs, contributors, releases, and commits.
    pub async fn fetch_metadata(&self, owner: &str, repo: &str, token: Option<&str>, since: Option<&str>) -> Result<GitHubMetadata> {
        info!("📊 Fetching GitHub metadata for {}/{}", owner, repo);
        
        let issues = self.fetch_issues(owner, repo, token, since).await?;
        let pull_requests = self.fetch_pull_requests(owner, repo, token).await?;
        let contributors = self.fetch_contributors(owner, repo, token).await?;
        let releases = self.fetch_releases(owner, repo, token).await?;
        let commits = self.fetch_commits(owner, repo, token, since).await?;
        
        Ok(GitHubMetadata {
            issues,
            pull_requests,
            contributors,
            releases,
            commits,
            last_fetched: Utc::now().to_rfc3339(),
        })
    }

    /// Fetch issues from GitHub API.
    async fn fetch_issues(&self, owner: &str, repo: &str, token: Option<&str>, since: Option<&str>) -> Result<Vec<GitHubIssue>> {
        let mut url = format!("https://api.github.com/repos/{}/{}/issues?state=all&per_page=100", owner, repo);
        if let Some(since_date) = since {
            url.push_str(&format!("&since={}", since_date));
        }
        
        let mut request = self.client.get(&url);
        if let Some(t) = token {
            request = request.header("Authorization", format!("token {}", t));
        }
        request = request.header("User-Agent", "anchor-engine-rust");
        
        let response = request.send().await?;
        if !response.status().is_success() {
            warn!("⚠️  Issues fetch failed: {}", response.status());
            return Ok(Vec::new());
        }
        
        let issues: Vec<serde_json::Value> = response.json().await?;
        Ok(issues.into_iter().map(|i| GitHubIssue {
            number: i["number"].as_i64().unwrap_or(0),
            title: i["title"].as_str().unwrap_or("").to_string(),
            body: i["body"].as_str().map(|s| s.to_string()),
            state: i["state"].as_str().unwrap_or("").to_string(),
            labels: i["labels"].as_array()
                .map(|arr| arr.iter().filter_map(|l| l["name"].as_str()).map(|s| s.to_string()).collect())
                .unwrap_or_default(),
            author: i["user"]["login"].as_str().map(|s| s.to_string()),
            created_at: i["created_at"].as_str().unwrap_or("").to_string(),
            updated_at: i["updated_at"].as_str().unwrap_or("").to_string(),
            comments: i["comments"].as_i64().unwrap_or(0),
        }).collect())
    }

    /// Fetch pull requests from GitHub API.
    async fn fetch_pull_requests(&self, owner: &str, repo: &str, token: Option<&str>) -> Result<Vec<GitHubPullRequest>> {
        let url = format!("https://api.github.com/repos/{}/{}/pulls?state=all&per_page=100&sort=updated&direction=desc", owner, repo);
        let mut request = self.client.get(&url);
        if let Some(t) = token {
            request = request.header("Authorization", format!("token {}", t));
        }
        request = request.header("User-Agent", "anchor-engine-rust");
        
        let response = request.send().await?;
        if !response.status().is_success() {
            warn!("⚠️  PRs fetch failed: {}", response.status());
            return Ok(Vec::new());
        }
        
        let prs: Vec<serde_json::Value> = response.json().await?;
        Ok(prs.into_iter().map(|p| GitHubPullRequest {
            number: p["number"].as_i64().unwrap_or(0),
            title: p["title"].as_str().unwrap_or("").to_string(),
            body: p["body"].as_str().map(|s| s.to_string()),
            state: p["state"].as_str().unwrap_or("").to_string(),
            author: p["user"]["login"].as_str().map(|s| s.to_string()),
            created_at: p["created_at"].as_str().unwrap_or("").to_string(),
            updated_at: p["updated_at"].as_str().unwrap_or("").to_string(),
            merged: p["merged_at"].as_str().is_some(),
            merged_at: p["merged_at"].as_str().map(|s| s.to_string()),
            additions: p["additions"].as_i64().unwrap_or(0),
            deletions: p["deletions"].as_i64().unwrap_or(0),
            changed_files: p["changed_files"].as_i64().unwrap_or(0),
        }).collect())
    }

    /// Fetch contributors from GitHub API.
    async fn fetch_contributors(&self, owner: &str, repo: &str, token: Option<&str>) -> Result<Vec<GitHubContributor>> {
        let url = format!("https://api.github.com/repos/{}/{}/contributors?per_page=100", owner, repo);
        let mut request = self.client.get(&url);
        if let Some(t) = token {
            request = request.header("Authorization", format!("token {}", t));
        }
        request = request.header("User-Agent", "anchor-engine-rust");
        
        let response = request.send().await?;
        if !response.status().is_success() {
            warn!("⚠️  Contributors fetch failed: {}", response.status());
            return Ok(Vec::new());
        }
        
        let contributors: Vec<serde_json::Value> = response.json().await?;
        Ok(contributors.into_iter().map(|c| GitHubContributor {
            login: c["login"].as_str().unwrap_or("").to_string(),
            contributions: c["contributions"].as_i64().unwrap_or(0),
            avatar_url: c["avatar_url"].as_str().unwrap_or("").to_string(),
        }).collect())
    }

    /// Fetch releases from GitHub API.
    async fn fetch_releases(&self, owner: &str, repo: &str, token: Option<&str>) -> Result<Vec<GitHubRelease>> {
        let url = format!("https://api.github.com/repos/{}/{}/releases?per_page=50", owner, repo);
        let mut request = self.client.get(&url);
        if let Some(t) = token {
            request = request.header("Authorization", format!("token {}", t));
        }
        request = request.header("User-Agent", "anchor-engine-rust");
        
        let response = request.send().await?;
        if !response.status().is_success() {
            warn!("⚠️  Releases fetch failed: {}", response.status());
            return Ok(Vec::new());
        }
        
        let releases: Vec<serde_json::Value> = response.json().await?;
        Ok(releases.into_iter().map(|r| GitHubRelease {
            tag_name: r["tag_name"].as_str().unwrap_or("").to_string(),
            name: r["name"].as_str().map(|s| s.to_string()),
            body: r["body"].as_str().map(|s| s.to_string()),
            author: r["author"]["login"].as_str().map(|s| s.to_string()),
            published_at: r["published_at"].as_str().unwrap_or("").to_string(),
            is_prerelease: r["prerelease"].as_bool().unwrap_or(false),
        }).collect())
    }

    /// Fetch commits from GitHub API.
    async fn fetch_commits(&self, owner: &str, repo: &str, token: Option<&str>, since: Option<&str>) -> Result<Vec<CommitInfo>> {
        let mut url = format!("https://api.github.com/repos/{}/{}/commits?per_page=100", owner, repo);
        if let Some(since_date) = since {
            url.push_str(&format!("&since={}", since_date));
        }
        
        let mut request = self.client.get(&url);
        if let Some(t) = token {
            request = request.header("Authorization", format!("token {}", t));
        }
        request = request.header("User-Agent", "anchor-engine-rust");
        
        let response = request.send().await?;
        if !response.status().is_success() {
            warn!("⚠️  Commits fetch failed: {}", response.status());
            return Ok(Vec::new());
        }
        
        let commits: Vec<serde_json::Value> = response.json().await?;
        Ok(commits.into_iter().map(|c| CommitInfo {
            hash: c["sha"].as_str().unwrap_or("").to_string(),
            message: c["commit"]["message"].as_str().unwrap_or("").to_string(),
            author: c["commit"]["author"]["name"].as_str().unwrap_or("").to_string(),
            author_email: c["commit"]["author"]["email"].as_str().map(|s| s.to_string()),
            date: c["commit"]["author"]["date"].as_str().unwrap_or("").to_string(),
            committer: c["commit"]["committer"]["name"].as_str().unwrap_or("").to_string(),
        }).collect())
    }

    /// Generate YAML context file from metadata.
    pub fn generate_yaml_context(
        &self,
        repo: &str,
        branch: &str,
        commit_info: &CommitInfo,
        metadata: &GitHubMetadata,
        output_path: &Path,
    ) -> Result<PathBuf> {
        info!("📝 Generating YAML context...");
        
        let parts: Vec<&str> = repo.split('/').collect();
        let owner = parts.get(0).unwrap_or(&"unknown");
        let repo_name = parts.get(1).unwrap_or(&"unknown");
        
        let yaml_content = format!(
r#"# GitHub Repository: {}
# Branch: {}
# Ingested: {}
# Commit: {} by {} on {}

project: {}
owner: {}
repository: {}
branch: {}
commit: {}
commit_date: {}
commit_author: {}
ingested_at: {}

# Contributors ({})
contributors:
{}

# Recent Issues ({})
issues:
{}

# Recent Pull Requests ({})
pull_requests:
{}

# Releases ({})
releases:
{}

# Repository Statistics
stats:
  total_issues: {}
  total_pull_requests: {}
  total_contributors: {}
  total_releases: {}
  total_commits: {}
  open_issues: {}
  merged_prs: {}

# Metadata (full JSON)
metadata_json: |
  {}
"#,
            repo, branch, Utc::now().to_rfc3339(),
            commit_info.hash, commit_info.author, commit_info.date,
            repo_name, owner, repo, branch,
            commit_info.hash, commit_info.date, commit_info.author,
            Utc::now().to_rfc3339(),
            metadata.contributors.len(),
            metadata.contributors.iter().take(20).map(|c| format!("  - {}: {} contributions", c.login, c.contributions)).collect::<Vec<_>>().join("\n"),
            metadata.issues.len(),
            metadata.issues.iter().take(50).map(|i| format!("  - #{}: {} [{}] by {}", i.number, i.title, i.state, i.author.as_deref().unwrap_or("unknown"))).collect::<Vec<_>>().join("\n"),
            metadata.pull_requests.len(),
            metadata.pull_requests.iter().take(50).map(|p| format!("  - #{}: {} [{}]{} by {}", p.number, p.title, p.state, if p.merged { " (merged)" } else { "" }, p.author.as_deref().unwrap_or("unknown"))).collect::<Vec<_>>().join("\n"),
            metadata.releases.len(),
            metadata.releases.iter().take(20).map(|r| format!("  - {}: {} ({})", r.tag_name, r.name.as_deref().unwrap_or("Untitled"), r.published_at)).collect::<Vec<_>>().join("\n"),
            metadata.issues.len(),
            metadata.pull_requests.len(),
            metadata.contributors.len(),
            metadata.releases.len(),
            metadata.commits.len(),
            metadata.issues.iter().filter(|i| i.state == "open").count(),
            metadata.pull_requests.iter().filter(|p| p.merged).count(),
            serde_json::to_string_pretty(metadata).unwrap_or_default()
        );
        
        fs::write(output_path, &yaml_content)?;
        info!("✅ Generated YAML context: {:?}", output_path);
        Ok(output_path.to_path_buf())
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

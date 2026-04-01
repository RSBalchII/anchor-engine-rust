//! Data Transfer Objects (DTOs) for external data ingestion.
//!
//! This module defines simplified, validated representations of external payloads
//! (e.g., GitHub webhooks) that are reformatted before reaching internal logic.
//!
//! # Architecture
//!
//! External data (complex, nested, unpredictable) → DTO Extractor → Clean DTO → Handler → Internal Service
//!
//! This pattern:
//! - Isolates internal logic from external schema changes
//! - Provides clear validation errors at the boundary
//! - Simplifies handler signatures
//! - Enables custom error handling

use serde::{Deserialize, Serialize};

/// GitHub action types from webhooks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GithubAction {
    Push,
    Create,
    Delete,
    Ping,
    Unknown,
}

impl Default for GithubAction {
    fn default() -> Self {
        GithubAction::Unknown
    }
}

/// Data Transfer Object for GitHub repository ingestion.
///
/// Simplified, validated representation of GitHub webhook/API payloads.
/// Contains only the fields Anchor Engine needs to execute a repository sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubIngestDto {
    /// Full repository URL (e.g., "https://github.com/owner/repo")
    pub repo_url: String,
    /// Branch name (default: "main" or "master")
    pub branch: String,
    /// Repository owner username
    pub owner: String,
    /// Repository name
    pub repo_name: String,
    /// Webhook sender (user or bot login)
    pub sender: String,
    /// Action that triggered the webhook
    pub action: GithubAction,
    /// Optional token (extracted from Authorization header or query param)
    pub token: Option<String>,
}

impl GithubIngestDto {
    /// Create a new DTO with minimal required fields.
    pub fn new(repo_url: String, owner: String, repo_name: String) -> Self {
        Self {
            repo_url,
            branch: "main".to_string(),
            owner,
            repo_name,
            sender: "unknown".to_string(),
            action: GithubAction::Unknown,
            token: None,
        }
    }

    /// Set the branch.
    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = branch.into();
        self
    }

    /// Set the sender.
    pub fn with_sender(mut self, sender: impl Into<String>) -> Self {
        self.sender = sender.into();
        self
    }

    /// Set the action.
    pub fn with_action(mut self, action: GithubAction) -> Self {
        self.action = action;
        self
    }

    /// Set the token.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }
}

/// Parameters for manual GitHub sync via MCP/API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubSyncParams {
    /// Repository URL to sync
    pub url: String,
    /// Optional bucket for categorization
    pub bucket: Option<String>,
    /// Optional token (overrides stored credentials)
    pub token: Option<String>,
    /// Include commit history in mirroring
    pub include_history: Option<bool>,
}

/// Parameters for credential management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubCredentialsParams {
    /// Action: "check", "set", or "delete"
    pub action: String,
    /// Token (required for "set" action)
    pub token: Option<String>,
}

/// Parameters for rate limit check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubRateLimitParams {
    /// Optional token to check authenticated rate limit
    pub token: Option<String>,
}

/// Rate limit information response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub limit: i64,
    pub remaining: i64,
    pub reset_at: String,
    pub authenticated: bool,
}

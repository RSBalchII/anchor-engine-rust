//! Custom Axum extractor for GitHub webhook payloads.
//!
//! This extractor intercepts raw JSON bodies, parses them flexibly,
//! and extracts only the fields Anchor Engine needs.

use axum::{
    async_trait,
    body::Bytes,
    extract::{FromRequest, Request},
    http::StatusCode,
};
use serde_json::Value;
use crate::dto::{GithubIngestDto, GithubAction};

/// Extract GitHub webhook payload into a simplified DTO.
///
/// This extractor:
/// 1. Reads raw body bytes
/// 2. Parses to generic JSON Value (flexible, won't fail on unexpected fields)
/// 3. Extracts only required fields with safe defaults
/// 4. Returns validated DTO to handler
///
/// # Important
///
/// This extractor consumes the request body, so it must be the **last** argument
/// in your handler function (after State, Headers, etc.).
///
/// # Example
///
/// ```rust
/// #[axum::debug_handler]
/// async fn handle_webhook(
///     State(state): State<SharedState>,
///     dto: GithubIngestDto,  // ← Must be last!
/// ) -> Result<Json<Response>, (StatusCode, String)> {
///     // dto is guaranteed valid here
/// }
/// ```
#[async_trait]
impl<S> FromRequest<S> for GithubIngestDto
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Extract body bytes
        let bytes = Bytes::from_request(req, _state)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read request body: {}", e)))?;

        // 2. Parse to generic JSON Value (flexible parsing)
        let json: Value = serde_json::from_slice(&bytes)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)))?;

        // 3. Extract required fields with safe defaults
        // GitHub webhook payload structure:
        // {
        //   "action": "push",
        //   "repository": {
        //     "html_url": "https://github.com/owner/repo",
        //     "name": "repo",
        //     "owner": {
        //       "login": "owner"
        //     },
        //     "default_branch": "main"
        //   },
        //   "sender": {
        //     "login": "username"
        //   },
        //   "ref": "refs/heads/main"
        // }

        let repo_url = json["repository"]["html_url"]
            .as_str()
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing repository.html_url".into()))?
            .to_string();

        let owner = json["repository"]["owner"]["login"]
            .as_str()
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing repository.owner.login".into()))?
            .to_string();

        let repo_name = json["repository"]["name"]
            .as_str()
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing repository.name".into()))?
            .to_string();

        let branch = json["repository"]["default_branch"]
            .as_str()
            .unwrap_or("main")
            .to_string();

        let sender = json["sender"]["login"]
            .as_str()
            .unwrap_or("webhook")
            .to_string();

        let action = json["action"]
            .as_str()
            .map(|a| match a.to_lowercase().as_str() {
                "push" => GithubAction::Push,
                "created" => GithubAction::Create,
                "deleted" => GithubAction::Delete,
                "ping" => GithubAction::Ping,
                _ => GithubAction::Unknown,
            })
            .unwrap_or(GithubAction::Unknown);

        // 4. Validate required fields
        if repo_url.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "Repository URL cannot be empty".into()));
        }

        if owner.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "Repository owner cannot be empty".into()));
        }

        if repo_name.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "Repository name cannot be empty".into()));
        }

        // 5. Return validated DTO
        Ok(GithubIngestDto {
            repo_url,
            branch,
            owner,
            repo_name,
            sender,
            action,
            token: None,  // Token extracted separately via Authorization header if needed
        })
    }
}

/// Extract GitHub token from Authorization header.
///
/// Usage:
/// ```rust
/// async fn handler(
///     token: GithubToken,  // Extracts "Bearer ghp_..." or "token ghp_..."
/// ) -> Result<(), (StatusCode, String)> {
///     // token.0 contains the raw token string
/// }
/// ```
#[derive(Debug, Clone)]
pub struct GithubToken(pub String);

#[async_trait]
impl<S> FromRequest<S> for GithubToken
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| {
                // Support both "Bearer ghp_..." and "token ghp_..." formats
                v.strip_prefix("Bearer ")
                    .or_else(|| v.strip_prefix("token "))
            })
            .map(|s| s.to_string());

        match token {
            Some(t) => Ok(GithubToken(t)),
            None => Err((StatusCode::UNAUTHORIZED, "Missing or invalid Authorization header. Use: Bearer ghp_...".into())),
        }
    }
}

/// Optional GitHub token from Authorization header.
///
/// Unlike GithubToken, this doesn't fail if the header is missing.
/// Useful for endpoints that support both authenticated and unauthenticated requests.
#[derive(Debug, Clone)]
pub struct OptionalGithubToken(pub Option<String>);

#[async_trait]
impl<S> FromRequest<S> for OptionalGithubToken
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| {
                v.strip_prefix("Bearer ")
                    .or_else(|| v.strip_prefix("token "))
            })
            .map(|s| s.to_string());

        Ok(OptionalGithubToken(token))
    }
}

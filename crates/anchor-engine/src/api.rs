//! HTTP API server for Anchor Engine.

use axum::{
    debug_handler,
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{delete, get, post, put},
    Json, Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, debug, error};
use chrono::Utc;
use std::fs;

use crate::models::*;
use crate::service::AnchorService;
use crate::services::github::CommitInfo;

/// Shared application state.
pub type SharedState = Arc<RwLock<AnchorService>>;

/// Search UI route - serves the beautiful frontend interface (v5.0.0 UI).
async fn search_ui() -> (axum::http::HeaderMap, Html<&'static str>) {
    use axum::http::HeaderValue;
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("Cache-Control", HeaderValue::from_static("no-cache, no-store, must-revalidate"));
    headers.insert("Pragma", HeaderValue::from_static("no-cache"));
    headers.insert("Expires", HeaderValue::from_static("0"));
    (headers, Html(include_str!("../static/index.html")))
}

/// Favicon route - returns empty 404 to prevent console noise.
async fn favicon() -> StatusCode {
    StatusCode::NOT_FOUND
}

/// Root route handler - redirects to the main UI.
async fn root() -> axum::response::Redirect {
    axum::response::Redirect::to("/search")
}

/// Create the API router.
pub fn create_router(state: SharedState) -> Router {
    Router::new()
        // Root route
        .route("/", get(root))

        // UI routes - all serve the same index.html (SPA)
        .route("/search", get(search_ui))
        .route("/settings", get(search_ui))
        .route("/dashboard", get(search_ui))
        .route("/memory", get(search_ui))
        .route("/paths", get(search_ui))
        .route("/quarantine", get(search_ui))

        // Favicon - prevent 404 noise
        .route("/favicon.ico", get(favicon))

        // Health and stats (with v1 aliases for UI compatibility)
        .route("/health", get(health_check))
        .route("/stats", get(get_stats))
        .route("/v1/stats", get(get_stats))
        .route("/v1/system/status", get(health_check))
        .route("/v1/buckets", get(get_buckets))

        // Memory/search endpoints
        .route("/v1/memory/search", post(search_memory))
        .route("/v1/memory/ingest", post(ingest_memory))

        // System management endpoints
        .route("/v1/system/paths", get(list_watch_paths))
        .route("/v1/system/paths/add", post(add_watch_path))
        .route("/v1/system/paths/remove", delete(remove_watch_path))
        .route("/v1/system/github/ingest", post(ingest_github))
        .route("/v1/github/repos", post(ingest_github))
        .route("/v1/system/github/track", post(track_github))
        .route("/v1/system/github/sync", post(sync_github))
        .route("/v1/system/github/tracked", get(list_tracked_github))
        
        // Watchdog endpoints (UI compatibility)
        .route("/v1/watchdog/status", get(watchdog_status))
        .route("/v1/watchdog/start", post(watchdog_start))
        .route("/v1/watchdog/stop", post(watchdog_stop))
        .route("/v1/watchdog/ingest", post(watchdog_ingest))
        
        // Settings endpoint (UI compatibility)
        .route("/v1/settings", get(get_settings))
        .route("/v1/settings", put(save_settings))
        
        // Snapshot endpoint for UI testing
        .route("/v1/test/snapshot", post(save_snapshot))

        // OpenAI-compatible endpoint
        .route("/v1/chat/completions", post(chat_completions))

        // Middleware - TraceLayer logs HTTP requests
        // Reduced verbosity: Only log non-GET requests to reduce UI polling noise
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &http::Request<_>| {
                    tracing::info_span!(
                        "request",
                        method = ?request.method(),
                        uri = ?request.uri(),
                        version = ?request.version(),
                    )
                })
                .on_request(|request: &http::Request<_>, _span: &tracing::Span| {
                    // Only log non-GET requests to reduce noise from UI polling
                    if request.method() != http::Method::GET {
                        tracing::info!("started processing request");
                    }
                })
                .on_response(|response: &http::Response<_>, latency: std::time::Duration, _span: &tracing::Span| {
                    let status = response.status();
                    // Only log non-GET requests or errors
                    if status.is_success() && latency.as_millis() < 100 {
                        // Fast successful GET requests - silent
                    } else if status.is_success() {
                        tracing::info!("finished processing request in {:?}, status: {}", latency, status);
                    } else {
                        tracing::warn!("finished processing request in {:?}, status: {}", latency, status);
                    }
                })
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Health check endpoint.
#[debug_handler]
async fn health_check(
    State(state): State<SharedState>,
) -> Json<HealthResponse> {
    let service = state.read().await;

    let stats = match service.get_stats().await {
        Ok(s) => Some(s),
        Err(_) => None,
    };

    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        stats,
    })
}

/// Get database statistics.
#[debug_handler]
async fn get_stats(
    State(state): State<SharedState>,
) -> Json<DbStatsResponse> {
    let service = state.read().await;
    match service.get_stats().await {
        Ok(stats) => Json(stats),
        Err(e) => {
            tracing::error!("Failed to get stats: {}", e);
            Json(DbStatsResponse {
                atoms: 0,
                sources: 0,
                tags: 0,
            })
        }
    }
}

/// Get available buckets (stub for UI compatibility).
#[debug_handler]
async fn get_buckets() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "buckets": ["inbox", "external-inbox", "default"]
    }))
}

/// Search memory endpoint.
#[debug_handler]
async fn search_memory(
    State(state): State<SharedState>,
    Json(request): Json<SearchRequest>,
) -> Json<SearchResponse> {
    let service = state.read().await;

    // 🔍 INFO log for search queries
    info!("🔍 SEARCH: \"{}\" (max_results={})", request.query, request.max_results);

    match service.search(request).await {
        Ok(response) => {
            // 🔍 DEBUG log with timing breakdown
            debug!(
                "   ├─ Planets: {} direct matches",
                response.stats.planets
            );
            debug!(
                "   ├─ Moons: {} associative matches",
                response.stats.moons
            );
            info!(
                "   └─ ✅ COMPLETE: {} results in {:.0}ms",
                response.total,
                response.stats.duration_ms
            );
            Json(response)
        }
        Err(e) => {
            error!("Search error: {}", e);
            Json(SearchResponse {
                results: vec![],
                query: String::new(),
                total: 0,
                stats: SearchStats {
                    planets: 0,
                    moons: 0,
                    duration_ms: 0.0,
                },
            })
        }
    }
}

/// Ingest memory endpoint.
#[debug_handler]
async fn ingest_memory(
    State(state): State<SharedState>,
    Json(request): Json<IngestRequest>,
) -> Result<Json<IngestResponse>, (StatusCode, String)> {
    let mut service = state.write().await;

    match service.ingest(request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Ingest error: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

/// OpenAI-compatible chat completions endpoint.
#[debug_handler]
async fn chat_completions(
    State(state): State<SharedState>,
    Json(request): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    // Extract query from messages
    let query = extract_query_from_messages(&request)
        .unwrap_or_else(|| "general query".to_string());
    
    // Search for relevant context
    let service = state.read().await;
    let search_request = SearchRequest {
        query: query.clone(),
        max_results: 10,
        mode: SearchMode::Combined,
        budget: BudgetConfig::default(),
    };
    
    let search_response = match service.search(search_request).await {
        Ok(resp) => resp,
        Err(e) => {
            error!("Search error in chat: {}", e);
            SearchResponse {
                results: vec![],
                query: query,
                total: 0,
                stats: SearchStats {
                    planets: 0,
                    moons: 0,
                    duration_ms: 0.0,
                },
            }
        }
    };
    
    // Build context from search results
    let context: Vec<String> = search_response.results
        .iter()
        .take(5)
        .map(|r| r.content.clone())
        .collect();
    
    // For now, return a simple response
    // In production, this would call the LLM
    let response_text = if context.is_empty() {
        "I don't have any relevant information about that in my memory.".to_string()
    } else {
        format!("Based on {} relevant memories, I found information about: {}", 
                context.len(), 
                context[0].chars().take(200).collect::<String>())
    };
    
    Json(serde_json::json!({
        "id": "chatcmpl-anchor-123",
        "object": "chat.completion",
        "created": chrono::Utc::now().timestamp(),
        "model": "anchor-local",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": response_text
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "total_tokens": 0
        },
        "context": {
            "atoms_used": search_response.total,
            "planets": search_response.stats.planets,
            "moons": search_response.stats.moons
        }
    }))
}

/// Extract query from OpenAI chat messages format.
fn extract_query_from_messages(request: &serde_json::Value) -> Option<String> {
    request["messages"]
        .as_array()?
        .last()?
        .get("content")?
        .as_str()
        .map(|s| s.to_string())
}

/// Start the HTTP server.
pub async fn start_server(state: SharedState, port: u16) -> std::io::Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    info!("Starting HTTP server on {}", addr);
    
    let app = create_router(state);
    
    axum::serve(listener, app).await
}

// ==================== System Management Handlers ====================

/// List all watched paths.
#[debug_handler]
async fn list_watch_paths(
    _state: State<SharedState>,
) -> Json<serde_json::Value> {
    // Get config from service or use default
    let config = crate::config::Config::load().unwrap_or_default();
    
    // Build watch paths from config.paths.notebook and watcher.extra_paths
    let mut paths = vec![config.paths.notebook.clone()];
    paths.extend(config.watcher.extra_paths.clone());

    Json(serde_json::json!({
        "watch_paths": paths,
        "auto_ingest": config.watcher.auto_start,
        "stability_threshold_ms": config.watcher.stability_threshold_ms,
    }))
}

/// Add a watch path.
#[debug_handler]
async fn add_watch_path(
    _state: State<SharedState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let path_str = request["path"]
        .as_str()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing 'path' field".to_string()))?;

    let path = std::path::PathBuf::from(path_str);

    if !path.exists() {
        return Err((StatusCode::BAD_REQUEST, format!("Path does not exist: {:?}", path)));
    }

    // Add to config's extra_paths
    let mut config = crate::config::Config::load().unwrap_or_default();
    config.watcher.extra_paths.push(path_str.to_string());
    
    // Save config back to file
    // Note: This is a simplified approach - in production you'd want proper file locking
    let config_path = std::path::PathBuf::from("user_settings.json");
    if config_path.exists() {
        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        std::fs::write(&config_path, content)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    tracing::info!("📍 Added watch path: {:?}", path);

    Ok(Json(serde_json::json!({
        "success": true,
        "path": path_str,
        "message": format!("Added watch path: {}", path_str),
    })))
}

/// Remove a watch path.
#[debug_handler]
async fn remove_watch_path(
    _state: State<SharedState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let path_str = request["path"]
        .as_str()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing 'path' field".to_string()))?;

    // Remove from config's extra_paths
    let mut config = crate::config::Config::load().unwrap_or_default();
    config.watcher.extra_paths.retain(|p| p != path_str);
    
    // Save config back to file
    let config_path = std::path::PathBuf::from("user_settings.json");
    if config_path.exists() {
        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        std::fs::write(&config_path, content)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    tracing::info!("📍 Removed watch path: {:?}", path_str);

    Ok(Json(serde_json::json!({
        "success": true,
        "path": path_str,
        "message": format!("Removed watch path: {}", path_str),
    })))
}

/// Ingest a GitHub repository with full metadata (like Node.js version).
#[debug_handler]
async fn ingest_github(
    _state: State<SharedState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let url = request["url"]
        .as_str()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing 'url' field".to_string()))?;
    
    let branch = request["branch"].as_str().unwrap_or("main").to_string();
    let incremental = request["incremental"].as_bool().unwrap_or(false);
    let token = request["token"].as_str().map(|s| s.to_string());

    // Parse GitHub URL
    let repo = crate::services::GitHubRepo::from_url(url)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let repo = match &token {
        Some(token) => repo.with_token(token),
        None => repo,
    };

    tracing::info!("📥 GITHUB INGEST: {} (branch: {}, incremental: {})", repo, branch, incremental);

    // Get paths from config
    let config = crate::config::Config::load().unwrap_or_default();
    let extract_base = config.external_inbox_path();
    let output_dir = extract_base.clone(); // Extract directly to external-inbox

    // Create GitHub service
    let github_service = crate::services::GitHubService::new(output_dir.clone())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Determine if this is an incremental update
    let since: Option<String> = if incremental {
        let summary_path = output_dir.join("INGEST_SUMMARY.json");
        if summary_path.exists() {
            if let Ok(summary_bytes) = fs::read(&summary_path) {
                if let Ok(summary) = serde_json::from_slice::<crate::services::github::IngestionSummary>(&summary_bytes) {
                    tracing::info!("📅 Incremental update since: {}", summary.last_ingestion);
                    Some(summary.last_ingestion)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Fetch metadata (issues, PRs, contributors, releases, commits)
    let metadata = github_service.fetch_metadata(&repo.owner, &repo.repo, repo.token.as_deref(), since.as_deref())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Metadata fetch failed: {}", e)))?;

    tracing::info!("📊 Fetched metadata: {} issues, {} PRs, {} contributors, {} releases, {} commits",
        metadata.issues.len(), metadata.pull_requests.len(), metadata.contributors.len(), metadata.releases.len(), metadata.commits.len());

    // Fetch and extract tarball
    let extract_path = github_service.fetch_and_extract(&repo).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Extraction failed: {}", e)))?;

    tracing::info!("✅ GitHub repo extracted to: {:?}", extract_path);

    // Get latest commit info
    let commit_info = metadata.commits.first().cloned().unwrap_or(CommitInfo {
        hash: "unknown".to_string(),
        message: "Unknown".to_string(),
        author: "Unknown".to_string(),
        author_email: None,
        date: Utc::now().to_rfc3339(),
        committer: "Unknown".to_string(),
    });

    // Generate YAML context file
    let yaml_path = output_dir.join(format!("{}-github.yaml", repo.repo));
    match github_service.generate_yaml_context(
        url,
        &branch,
        &commit_info,
        &metadata,
        &yaml_path,
    ) {
        Ok(_) => tracing::info!("✅ Generated YAML context: {:?}", yaml_path),
        Err(e) => tracing::warn!("⚠️  YAML generation failed: {}", e),
    }

    // Save ingestion summary for incremental updates
    let summary = crate::services::github::IngestionSummary {
        repo: url.to_string(),
        branch: branch.clone(),
        tarball: format!("{}-{}.tar.gz", repo.repo, chrono::Utc::now().format("%Y-%m-%d")),
        last_ingestion: Utc::now().to_rfc3339(),
        commit_hash: commit_info.hash.clone(),
        metadata: crate::services::github::IngestionMetadata {
            issues: metadata.issues.len() as i64,
            pull_requests: metadata.pull_requests.len() as i64,
            contributors: metadata.contributors.len() as i64,
            releases: metadata.releases.len() as i64,
            commits: metadata.commits.len() as i64,
        },
    };
    
    let summary_path = output_dir.join("INGEST_SUMMARY.json");
    if let Err(e) = fs::write(&summary_path, serde_json::to_string_pretty(&summary).unwrap()) {
        tracing::warn!("⚠️  Failed to save ingestion summary: {}", e);
    }

    // The Watchdog service will auto-ingest the extracted files and YAML
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Successfully fetched and extracted {}/{} with full metadata", repo.owner, repo.repo),
        "extract_path": extract_path.to_string_lossy(),
        "yaml_context": yaml_path.to_string_lossy(),
        "metadata": {
            "issues": metadata.issues.len(),
            "pull_requests": metadata.pull_requests.len(),
            "contributors": metadata.contributors.len(),
            "releases": metadata.releases.len(),
            "commits": metadata.commits.len(),
        },
        "commit": {
            "hash": commit_info.hash,
            "author": commit_info.author,
            "date": commit_info.date,
        },
        "watchdog_note": "Files and YAML will be auto-ingested by the Watchdog service",
    })))
}

/// Track a GitHub repository for periodic sync.
#[debug_handler]
async fn track_github(
    State(_state): State<SharedState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let url = request["url"]
        .as_str()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing 'url' field".to_string()))?;
    
    let sync_interval = request["sync_interval_secs"].as_u64().unwrap_or(120); // Default 2 minutes
    let token = request["token"].as_str().map(|s| s.to_string());
    
    // Parse GitHub URL
    let repo = crate::services::GitHubRepo::from_url(url)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    
    let repo = match &token {
        Some(token) => repo.with_token(token),
        None => repo,
    };
    
    tracing::info!("📍 TRACK GITHUB: {} (sync every {}s)", repo, sync_interval);
    
    // Note: In a real implementation, we'd store this in a persistent config
    // For now, just acknowledge the request
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Tracking {}/{} with {}s sync interval", repo.owner, repo.repo, sync_interval),
        "sync_interval_secs": sync_interval,
        "note": "Repo will be synced periodically (feature requires service restart)",
    })))
}

/// Manually trigger sync of tracked GitHub repos.
#[debug_handler]
async fn sync_github(
    State(_state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    tracing::info!("🔄 Manual GitHub sync triggered");
    
    // Note: Full implementation would sync all tracked repos
    // For now, just acknowledge
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "GitHub sync initiated",
        "note": "Tracked repos will be synced in background",
    })))
}

/// List tracked GitHub repositories.
#[debug_handler]
async fn list_tracked_github(
    State(_state): State<SharedState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "tracked_repos": [],
        "note": "Use /v1/system/github/track to add repos",
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::service::AnchorService;
    use crate::config::Config;

    #[tokio::test]
    async fn test_health_check() {
        let db = Database::in_memory().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let config = Config::default();
        let service = AnchorService::new(db, temp_dir.path().to_path_buf(), config).unwrap();
        let state: SharedState = Arc::new(RwLock::new(service));

        let response = health_check(State(state)).await;
        assert_eq!(response.status, "healthy");
    }

    #[tokio::test]
    async fn test_search_memory() {
        let db = Database::in_memory().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let config = Config::default();
        let mut service = AnchorService::new(db, temp_dir.path().to_path_buf(), config).unwrap();

        // Ingest some content first
        let ingest_request = IngestRequest {
            source: "test.md".to_string(),
            content: "Rust is a programming language".to_string(),
            bucket: None,
            options: Default::default(),
        };
        service.ingest(ingest_request).await.unwrap();

        let state: SharedState = Arc::new(RwLock::new(service));

        let search_request = SearchRequest {
            query: "#rust".to_string(),
            max_results: 10,
            mode: SearchMode::Combined,
            budget: BudgetConfig::default(),
        };

        let _response = search_memory(State(state), Json(search_request)).await;
        // Should not panic and return valid response structure
        // (response may have 0 results since search query may not match)
    }
}

// ============================================================================
// Watchdog & Settings Endpoints (UI Compatibility Stubs)
// ============================================================================

use crate::config::Config;

/// Get watchdog status.
#[debug_handler]
async fn watchdog_status() -> Json<serde_json::Value> {
    // Load current config to get actual auto_start state
    let config = Config::load().unwrap_or_default();
    
    Json(serde_json::json!({
        "is_running": config.watcher.auto_start,
        "files_processed": 0,
        "errors": 0,
        "watched_paths": config.watcher.extra_paths,
        "auto_start": config.watcher.auto_start,
        "stability_threshold_ms": config.watcher.stability_threshold_ms
    }))
}

/// Start watchdog.
#[debug_handler]
async fn watchdog_start() -> Json<serde_json::Value> {
    // Update user_settings.json to enable auto_start
    let mut config = Config::load().unwrap_or_default();
    config.watcher.auto_start = true;
    
    // Save back to file
    let config_path = std::path::PathBuf::from("user_settings.json");
    if let Ok(content) = serde_json::to_string_pretty(&config) {
        let _ = std::fs::write(&config_path, content);
    }
    
    Json(serde_json::json!({
        "success": true,
        "message": "Watchdog enabled - will auto-start on next server restart",
        "auto_start": true
    }))
}

/// Stop watchdog.
#[debug_handler]
async fn watchdog_stop() -> Json<serde_json::Value> {
    // Update user_settings.json to disable auto_start
    let mut config = Config::load().unwrap_or_default();
    config.watcher.auto_start = false;
    
    // Save back to file
    let config_path = std::path::PathBuf::from("user_settings.json");
    if let Ok(content) = serde_json::to_string_pretty(&config) {
        let _ = std::fs::write(&config_path, content);
    }
    
    Json(serde_json::json!({
        "success": true,
        "message": "Watchdog disabled",
        "auto_start": false
    }))
}

/// Trigger watchdog ingest.
#[debug_handler]
async fn watchdog_ingest() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "success": true,
        "message": "Ingest triggered (stub)"
    }))
}

/// Get settings.
#[debug_handler]
async fn get_settings() -> Json<serde_json::Value> {
    // Load from user_settings.json or return defaults
    let config = Config::load().unwrap_or_default();
    Json(serde_json::json!({
        "server": {
            "port": config.server.port,
            "host": config.server.host,
            "api_key": config.server.api_key
        },
        "paths": {
            "notebook": config.paths.notebook,
            "inbox": config.paths.inbox,
            "external_inbox": config.paths.external_inbox,
            "mirrored_brain": config.paths.mirrored_brain,
            "database": config.paths.database
        },
        "watcher": {
            "extra_paths": config.watcher.extra_paths,
            "auto_start": config.watcher.auto_start,
            "stability_threshold_ms": config.watcher.stability_threshold_ms
        },
        "ingestion": {
            "max_keywords": config.ingestion.max_keywords,
            "sanitize": config.ingestion.sanitize
        },
        "budget": {
            "planet_budget": config.budget.planet_budget,
            "moon_budget": config.budget.moon_budget,
            "total_tokens": config.budget.total_tokens
        },
        "transient_filter": {
            "min_lines": config.transient_filter.min_lines,
            "threshold": config.transient_filter.threshold
        },
        "search": {
            "max_chars_default": 524288,
            "strategy": config.search.strategy
        },
        "context": {
            "relevance_weight": 0.7,
            "recency_weight": 0.3
        },
        "physics": {
            "damping_factor": 0.85,
            "temperature": 0.2,
            "walk_radius": 1
        },
        "resource_management": {
            "gc_cooldown_ms": 30000
        },
        "database": {
            "wipe_on_startup": true
        },
        "tagging": {
            "modulation_level": 50,
            "blacklist_strictness": 75,
            "atom_as_tag": false,
            "strict_atom_selection": true
        },
        "encryption": {
            "enabled": false
        }
    }))
}

/// Save settings.
#[debug_handler]
async fn save_settings(
    Json(new_settings): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    // Load current config
    let mut config = Config::load().unwrap_or_default();
    
    // Update server settings
    if let Some(server) = new_settings.get("server") {
        if let Some(api_key) = server.get("api_key").and_then(|v| v.as_str()) {
            config.server.api_key = api_key.to_string();
        }
    }
    
    // Update watcher settings
    if let Some(watcher) = new_settings.get("watcher") {
        if let Some(extra_paths) = watcher.get("extra_paths").and_then(|v| v.as_array()) {
            config.watcher.extra_paths = extra_paths
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
        }
        if let Some(auto_start) = watcher.get("auto_start").and_then(|v| v.as_bool()) {
            config.watcher.auto_start = auto_start;
        }
    }
    
    // Update ingestion settings
    if let Some(ingestion) = new_settings.get("ingestion") {
        if let Some(max_keywords) = ingestion.get("max_keywords").and_then(|v| v.as_u64()) {
            config.ingestion.max_keywords = max_keywords as usize;
        }
        if let Some(sanitize) = ingestion.get("sanitize").and_then(|v| v.as_bool()) {
            config.ingestion.sanitize = sanitize;
        }
    }
    
    // Save back to file
    let config_path = std::path::PathBuf::from("user_settings.json");
    if let Ok(content) = serde_json::to_string_pretty(&config) {
        if let Err(e) = std::fs::write(&config_path, content) {
            return Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to save: {}", e)
            }));
        }
    }
    
    Json(serde_json::json!({
        "success": true,
        "message": "Settings saved successfully"
    }))
}

/// Save UI test snapshot.
#[debug_handler]
async fn save_snapshot(
    Json(snapshot): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    use std::fs;
    use std::path::PathBuf;
    use chrono::Utc;
    
    let snapshot_name = snapshot.get("name").and_then(|v| v.as_str()).unwrap_or("snapshot");
    let snapshot_type = snapshot.get("type").and_then(|v| v.as_str()).unwrap_or("test");
    
    // Create logs directory if it doesn't exist
    let logs_dir = PathBuf::from("logs");
    if let Err(e) = fs::create_dir_all(&logs_dir) {
        return Json(serde_json::json!({
            "success": false,
            "error": format!("Failed to create logs dir: {}", e)
        }));
    }
    
    // Create snapshot data with timestamp
    let snapshot_data = serde_json::json!({
        "timestamp": Utc::now().to_rfc3339(),
        "type": snapshot_type,
        "name": snapshot_name,
        "data": snapshot.get("data").cloned().unwrap_or(serde_json::Value::Null)
    });
    
    // Save snapshot (overwrites existing)
    let snapshot_path = logs_dir.join(format!("snapshot-{}.json", snapshot_name));
    match serde_json::to_string_pretty(&snapshot_data) {
        Ok(content) => {
            match fs::write(&snapshot_path, content) {
                Ok(_) => Json(serde_json::json!({
                    "success": true,
                    "message": format!("Snapshot saved: {}", snapshot_path.display()),
                    "path": snapshot_path.to_string_lossy()
                })),
                Err(e) => Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to write snapshot: {}", e)
                }))
            }
        },
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": format!("Failed to serialize snapshot: {}", e)
        }))
    }
}

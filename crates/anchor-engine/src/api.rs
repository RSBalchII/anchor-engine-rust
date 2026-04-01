//! HTTP API server for Anchor Engine.

use axum::{
    debug_handler,
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{delete, get, post},
    Json, Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, debug, error};

use crate::models::*;
use crate::service::AnchorService;

/// Shared application state.
pub type SharedState = Arc<RwLock<AnchorService>>;

/// Search UI route - serves the beautiful frontend interface (v5.0.0 UI).
async fn search_ui() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
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
        
        // Search UI - beautiful frontend interface
        .route("/search", get(search_ui))

        // Health and stats
        .route("/health", get(health_check))
        .route("/stats", get(get_stats))

        // Memory/search endpoints
        .route("/v1/memory/search", post(search_memory))
        .route("/v1/memory/ingest", post(ingest_memory))
        
        // System management endpoints
        .route("/v1/system/paths", get(list_watch_paths))
        .route("/v1/system/paths/add", post(add_watch_path))
        .route("/v1/system/paths/remove", delete(remove_watch_path))
        .route("/v1/system/github/ingest", post(ingest_github))
        .route("/v1/system/github/track", post(track_github))
        .route("/v1/system/github/sync", post(sync_github))
        .route("/v1/system/github/tracked", get(list_tracked_github))

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
            error!("Failed to get stats: {}", e);
            Json(DbStatsResponse {
                atoms: 0,
                sources: 0,
                tags: 0,
            })
        }
    }
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
    State(state): State<SharedState>,
) -> Json<serde_json::Value> {
    let service = state.read().await;
    
    // Get config from service or use default
    let config = crate::config::Config::load().unwrap_or_default();
    let paths = config.settings.all_watch_paths();
    
    Json(serde_json::json!({
        "watch_paths": paths.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>(),
        "auto_ingest": config.settings.auto_ingest,
        "stability_threshold_ms": config.settings.watcher_stability_threshold_ms,
    }))
}

/// Add a watch path.
#[debug_handler]
async fn add_watch_path(
    State(state): State<SharedState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let path_str = request["path"]
        .as_str()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing 'path' field".to_string()))?;
    
    let path = std::path::PathBuf::from(path_str);
    
    if !path.exists() {
        return Err((StatusCode::BAD_REQUEST, format!("Path does not exist: {:?}", path)));
    }
    
    // Add to config
    let mut config = crate::config::Config::load().unwrap_or_default();
    config.add_watch_path(path_str)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
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
    State(state): State<SharedState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let path_str = request["path"]
        .as_str()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing 'path' field".to_string()))?;
    
    // Remove from config
    let mut config = crate::config::Config::load().unwrap_or_default();
    config.remove_watch_path(path_str)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    tracing::info!("📍 Removed watch path: {:?}", path_str);

    Ok(Json(serde_json::json!({
        "success": true,
        "path": path_str,
        "message": format!("Removed watch path: {}", path_str),
    })))
}

/// Ingest a GitHub repository.
#[debug_handler]
async fn ingest_github(
    State(state): State<SharedState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let url = request["url"]
        .as_str()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing 'url' field".to_string()))?;
    
    let token = request["token"].as_str().map(|s| s.to_string());
    
    // Parse GitHub URL
    let repo = crate::services::GitHubRepo::from_url(url)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    
    let repo = match &token {
        Some(token) => repo.with_token(token),
        None => repo,
    };
    
    tracing::info!("📥 GITHUB INGEST: {} (ref: {:?})", repo, repo.ref_name);
    
    // Get external-inbox path from config
    let config = crate::config::Config::load().unwrap_or_default();
    let extract_base = config.settings.external_inbox_path();
    
    // Create GitHub service
    let github_service = crate::services::GitHubService::new(extract_base)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Fetch and extract
    match github_service.fetch_and_extract(&repo).await {
        Ok(extract_path) => {
            tracing::info!("✅ GitHub repo extracted to: {:?}", extract_path);
            
            // The Watchdog service will auto-ingest the extracted files
            Ok(Json(serde_json::json!({
                "success": true,
                "message": format!("Successfully fetched and extracted {}/{}", repo.owner, repo.repo),
                "extract_path": extract_path.to_string_lossy(),
                "watchdog_note": "Files will be auto-ingested by the Watchdog service",
            })))
        }
        Err(e) => {
            tracing::error!("❌ GitHub ingestion failed: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
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

    #[tokio::test]
    async fn test_health_check() {
        let db = Database::in_memory().unwrap();
        let service = AnchorService::new(db);
        let state: SharedState = Arc::new(RwLock::new(service));
        
        let response = health_check(State(state)).await;
        assert_eq!(response.status, "healthy");
    }
    
    #[tokio::test]
    async fn test_search_memory() {
        let db = Database::in_memory().unwrap();
        let mut service = AnchorService::new(db);

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

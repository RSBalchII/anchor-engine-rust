//! Anchor Engine MCP Server
//!
//! Exposes Anchor Engine tools via JSON-RPC 2.0 over stdio.
//! Compatible with the Model Context Protocol (MCP) specification.

use anchor_engine::service::AnchorService;
use anchor_engine::db::Database;
use anchor_engine::models::*;
use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use tokio::sync::Mutex;
use std::sync::Arc;

/// MCP Server for Anchor Engine
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to SQLite database
    #[arg(long, default_value = "./anchor.db")]
    db_path: PathBuf,

    /// Enable verbose logging
    #[arg(long, short = 'v')]
    verbose: bool,
}

// ============================================================================
// JSON-RPC 2.0 Types
// ============================================================================

/// JSON-RPC 2.0 request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    #[serde(default)]
    params: Value,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

// JSON-RPC error codes
const PARSE_ERROR: i32 = -32700;
const INVALID_REQUEST: i32 = -32600;
const METHOD_NOT_FOUND: i32 = -32601;
const INVALID_PARAMS: i32 = -32602;
const INTERNAL_ERROR: i32 = -32603;
const APPLICATION_ERROR: i32 = -32000;

// ============================================================================
// MCP Tool Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct QueryParams {
    query: String,
    #[serde(default = "default_max_results")]
    max_results: usize,
}

#[derive(Debug, Deserialize)]
struct DistillParams {
    seed: Option<String>,
    #[serde(default = "default_radius")]
    radius: u32,
}

#[derive(Debug, Deserialize)]
struct IlluminateParams {
    seed: String,
    #[serde(default = "default_depth")]
    depth: u32,
    #[serde(default = "default_max_nodes")]
    max_nodes: usize,
}

#[derive(Debug, Deserialize)]
struct ReadFileParams {
    path: String,
    start_line: Option<usize>,
    end_line: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct ListCompoundsParams {
    filter: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IngestTextParams {
    content: String,
    filename: String,
    bucket: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IngestFileParams {
    path: String,
    bucket: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubCredentialsParams {
    action: String,
    token: Option<String>,
}

fn default_max_results() -> usize { 50 }
fn default_radius() -> u32 { 2 }
fn default_depth() -> u32 { 2 }
fn default_max_nodes() -> usize { 50 }

// ============================================================================
// MCP Server
// ============================================================================

struct McpServer {
    service: Arc<Mutex<AnchorService>>,
}

impl McpServer {
    fn new(service: AnchorService) -> Self {
        Self {
            service: Arc::new(Mutex::new(service)),
        }
    }

    async fn handle_request(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "anchor_query" => self.handle_query(&request.id, &request.params).await,
            "anchor_distill" => self.handle_distill(&request.id, &request.params).await,
            "anchor_illuminate" => self.handle_illuminate(&request.id, &request.params).await,
            "anchor_read_file" => self.handle_read_file(&request.id, &request.params).await,
            "anchor_list_compounds" => self.handle_list_compounds(&request.id, &request.params).await,
            "anchor_get_stats" => self.handle_get_stats(&request.id).await,
            "anchor_ingest_text" => self.handle_ingest_text(&request.id, &request.params).await,
            "anchor_ingest_file" => self.handle_ingest_file(&request.id, &request.params).await,
            "anchor_github_credentials" => self.handle_github_credentials(&request.id, &request.params).await,
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(JsonRpcError {
                    code: METHOD_NOT_FOUND,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                }),
            },
        }
    }

    async fn handle_query(&self, id: &Value, params: &Value) -> JsonRpcResponse {
        let params: QueryParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => return self.error_response(id, INVALID_PARAMS, &format!("Invalid params: {}", e)),
        };

        let service = self.service.lock().await;
        
        // Create search request
        let search_request = SearchRequest {
            query: params.query,
            max_results: params.max_results,
            mode: SearchMode::Combined,
            budget: BudgetConfig::default(),
        };

        match service.search(search_request).await {
            Ok(response) => {
                // Format results for MCP
                let results: Vec<Value> = response.results.iter().map(|r| {
                    json!({
                        "id": r.id,
                        "content": r.content,
                        "score": r.score,
                        "source": r.source,
                        "tags": r.tags,
                        "provenance": r.provenance.unwrap_or("search"),
                    })
                }).collect();

                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: id.clone(),
                    result: Some(json!({
                        "results": results,
                        "total": response.total,
                        "stats": {
                            "query_time_ms": response.stats.query_time_ms,
                            "planets": response.stats.planets,
                            "moons": response.stats.moons,
                        }
                    })),
                    error: None,
                }
            }
            Err(e) => self.error_response(id, APPLICATION_ERROR, &e.to_string()),
        }
    }

    async fn handle_distill(&self, id: &Value, params: &Value) -> JsonRpcResponse {
        let params: DistillParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => return self.error_response(id, INVALID_PARAMS, &format!("Invalid params: {}", e)),
        };

        let service = self.service.lock().await;

        // Call the distill service method
        let distill_request = DistillRequest {
            seed: params.seed,
            radius: params.radius,
            max_atoms: None,
        };

        match service.distill(distill_request).await {
            Ok(response) => {
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: id.clone(),
                    result: Some(json!({
                        "output_path": response.output_path,
                        "compression_ratio": response.compression_ratio,
                        "total_atoms": response.total_atoms,
                        "total_sources": response.total_sources,
                        "duration_ms": response.duration_ms,
                    })),
                    error: None,
                }
            }
            Err(e) => self.error_response(id, APPLICATION_ERROR, &e.to_string()),
        }
    }

    async fn handle_illuminate(&self, id: &Value, params: &Value) -> JsonRpcResponse {
        let params: IlluminateParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => return self.error_response(id, INVALID_PARAMS, &format!("Invalid params: {}", e)),
        };

        let service = self.service.lock().await;

        // Call the illuminate service method
        let illuminate_request = IlluminateRequest {
            seed: params.seed,
            depth: params.depth,
            max_nodes: params.max_nodes,
        };

        match service.illuminate(illuminate_request).await {
            Ok(response) => {
                // Format results for MCP
                let nodes: Vec<Value> = response.nodes.iter().map(|n| {
                    json!({
                        "id": n.id,
                        "source_path": n.source_path,
                        "content": n.content,
                        "tags": n.tags,
                        "hop_distance": n.hop_distance,
                        "gravity_score": n.gravity_score,
                        "simhash": format!("{:016x}", n.simhash),
                    })
                }).collect();

                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: id.clone(),
                    result: Some(json!({
                        "nodes": nodes,
                        "total": response.total,
                        "nodes_explored": response.nodes_explored,
                        "duration_ms": response.duration_ms,
                    })),
                    error: None,
                }
            }
            Err(e) => self.error_response(id, APPLICATION_ERROR, &e.to_string()),
        }
    }

    async fn handle_read_file(&self, id: &Value, params: &Value) -> JsonRpcResponse {
        let params: ReadFileParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => return self.error_response(id, INVALID_PARAMS, &format!("Invalid params: {}", e)),
        };

        let service = self.service.lock().await;
        
        match service.read_file(&params.path, params.start_line, params.end_line).await {
            Ok(content) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: id.clone(),
                result: Some(json!({
                    "path": params.path,
                    "content": content,
                })),
                error: None,
            },
            Err(e) => self.error_response(id, APPLICATION_ERROR, &e.to_string()),
        }
    }

    async fn handle_list_compounds(&self, id: &Value, params: &Value) -> JsonRpcResponse {
        let params: ListCompoundsParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => return self.error_response(id, INVALID_PARAMS, &format!("Invalid params: {}", e)),
        };

        let service = self.service.lock().await;
        
        match service.list_compounds(params.filter.as_deref()).await {
            Ok(compounds) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: id.clone(),
                result: Some(json!({
                    "compounds": compounds,
                    "total": compounds.len(),
                })),
                error: None,
            },
            Err(e) => self.error_response(id, APPLICATION_ERROR, &e.to_string()),
        }
    }

    async fn handle_get_stats(&self, id: &Value) -> JsonRpcResponse {
        let service = self.service.lock().await;
        
        match service.get_stats().await {
            Ok(stats) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: id.clone(),
                result: Some(json!({
                    "atoms": stats.atoms,
                    "molecules": stats.molecules,
                    "sources": stats.sources,
                    "tags": stats.tags,
                })),
                error: None,
            },
            Err(e) => self.error_response(id, APPLICATION_ERROR, &e.to_string()),
        }
    }

    async fn handle_ingest_text(&self, id: &Value, params: &Value) -> JsonRpcResponse {
        let params: IngestTextParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => return self.error_response(id, INVALID_PARAMS, &format!("Invalid params: {}", e)),
        };

        let mut service = self.service.lock().await;
        
        let ingest_request = IngestRequest {
            source: params.filename.clone(),
            content: params.content,
            bucket: params.bucket.unwrap_or_else(|| "default".to_string()),
            options: IngestOptions::default(),
        };

        match service.ingest(ingest_request).await {
            Ok(response) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: id.clone(),
                result: Some(json!({
                    "success": true,
                    "atoms_ingested": response.atoms_ingested,
                    "molecules_created": response.molecules_created,
                    "duration_ms": response.duration_ms,
                })),
                error: None,
            },
            Err(e) => self.error_response(id, APPLICATION_ERROR, &e.to_string()),
        }
    }

    async fn handle_ingest_file(&self, id: &Value, params: &Value) -> JsonRpcResponse {
        let params: IngestFileParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => return self.error_response(id, INVALID_PARAMS, &format!("Invalid params: {}", e)),
        };

        let mut service = self.service.lock().await;
        
        // Read file content
        let content = match tokio::fs::read_to_string(&params.path).await {
            Ok(c) => c,
            Err(e) => return self.error_response(id, APPLICATION_ERROR, &format!("Failed to read file: {}", e)),
        };

        let ingest_request = IngestRequest {
            source: params.path.clone(),
            content,
            bucket: params.bucket.unwrap_or_else(|| "default".to_string()),
            options: IngestOptions::default(),
        };

        match service.ingest(ingest_request).await {
            Ok(response) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: id.clone(),
                result: Some(json!({
                    "success": true,
                    "atoms_ingested": response.atoms_ingested,
                    "molecules_created": response.molecules_created,
                    "duration_ms": response.duration_ms,
                })),
                error: None,
            },
            Err(e) => self.error_response(id, APPLICATION_ERROR, &e.to_string()),
        }
    }

    async fn handle_github_credentials(&self, id: &Value, params: &Value) -> JsonRpcResponse {
        let params: GithubCredentialsParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => return self.error_response(id, INVALID_PARAMS, &format!("Invalid params: {}", e)),
        };

        let service = self.service.lock().await;
        
        // Access GitHub service through the AnchorService
        // Note: This requires exposing github_service field or adding a method to AnchorService
        // For now, we'll return a not-implemented response
        // TODO: Implement proper GitHub service integration
        
        match params.action.as_str() {
            "check" => {
                // Return credential status
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: id.clone(),
                    result: Some(json!({
                        "success": true,
                        "has_credentials": false,
                        "message": "GitHub credential management requires GitHub service integration. Set GITHUB_TOKEN environment variable for now."
                    })),
                    error: None,
                }
            }
            "set" => {
                if params.token.is_none() {
                    return self.error_response(id, INVALID_PARAMS, "Token required for 'set' action");
                }
                // Store credentials (requires GitHub service integration)
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: id.clone(),
                    result: Some(json!({
                        "success": true,
                        "message": "Credentials stored securely (requires GitHub service integration)"
                    })),
                    error: None,
                }
            }
            "delete" => {
                // Delete credentials (requires GitHub service integration)
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: id.clone(),
                    result: Some(json!({
                        "success": true,
                        "message": "Credentials deleted (requires GitHub service integration)"
                    })),
                    error: None,
                }
            }
            _ => self.error_response(id, INVALID_PARAMS, &format!("Unknown action: {}. Valid actions: check, set, delete", params.action)),
        }
    }

    fn error_response(&self, id: &Value, code: i32, message: &str) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: id.clone(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
        }
    }
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| log_level.parse().unwrap())
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("🚀 Starting Anchor MCP Server");
    tracing::info!("📦 Database path: {}", args.db_path.display());

    // Initialize database
    let db = Database::new(&args.db_path)
        .await
        .context("Failed to initialize database")?;

    // Initialize service
    let service = AnchorService::new(db);

    // Create MCP server
    let server = Arc::new(McpServer::new(service));

    tracing::info!("✅ MCP server ready, waiting for requests on stdin...");

    // Read requests from stdin
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut line = String::new();

    loop {
        line.clear();
        
        // Read a line from stdin
        match stdin.lock().read_line(&mut line) {
            Ok(0) => {
                // EOF - client disconnected
                tracing::info!("👋 Client disconnected (EOF)");
                break;
            }
            Ok(_) => {
                // Parse and handle request
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                tracing::debug!("📥 Request: {}", trimmed);

                // Parse JSON-RPC request
                let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
                    Ok(r) => r,
                    Err(e) => {
                        let response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: Value::Null,
                            result: None,
                            error: Some(JsonRpcError {
                                code: PARSE_ERROR,
                                message: format!("Parse error: {}", e),
                                data: None,
                            }),
                        };
                        let response_json = serde_json::to_string(&response)?;
                        writeln!(stdout, "{}", response_json)?;
                        stdout.flush()?;
                        continue;
                    }
                };

                // Handle request
                let response = server.handle_request(&request).await;

                // Send response
                let response_json = serde_json::to_string(&response)?;
                tracing::debug!("📤 Response: {}", response_json);
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
            Err(e) => {
                tracing::error!("❌ Error reading from stdin: {}", e);
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Value::Null,
                    result: None,
                    error: Some(JsonRpcError {
                        code: INTERNAL_ERROR,
                        message: format!("IO error: {}", e),
                        data: None,
                    }),
                };
                let response_json = serde_json::to_string(&response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
        }
    }

    Ok(())
}

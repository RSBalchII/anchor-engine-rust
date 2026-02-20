//! API Client for Anchor Engine

use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

const DEFAULT_BASE_URL: &str = "http://localhost:3160";

/// API client for communicating with Anchor Engine
#[derive(Clone, Debug)]
pub struct ApiClient {
    base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_chars: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_budget: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buckets: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_code: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub source: String,
    pub score: f32,
    pub tags: Option<Vec<String>>,
    pub buckets: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub context: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<ChatContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatContext {
    pub enable_search: bool,
    pub max_atoms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Option<ChatUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbStatsResponse {
    pub atoms: u32,
    pub sources: u32,
    pub tags: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathResponse {
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantinedAtom {
    pub id: String,
    pub content: String,
    pub source: String,
    pub timestamp: f64,
    pub tags: Vec<String>,
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new(DEFAULT_BASE_URL)
    }
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }

    /// Health check
    pub async fn health(&self) -> Result<HealthResponse, ApiError> {
        self.get("/health").await
    }

    /// Get database statistics
    pub async fn stats(&self) -> Result<DbStatsResponse, ApiError> {
        self.get("/stats").await
    }

    /// Search the knowledge base
    pub async fn search(&self, request: SearchRequest) -> Result<SearchResponse, ApiError> {
        self.post("/v1/memory/search", &request).await
    }

    /// Chat with context (RAG)
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ApiError> {
        self.post("/v1/chat/completions", &request).await
    }

    /// Get watched paths
    pub async fn get_paths(&self) -> Result<PathResponse, ApiError> {
        self.get("/v1/system/paths").await
    }

    /// Add a watched path
    pub async fn add_path(&self, path: &str) -> Result<serde_json::Value, ApiError> {
        self.post("/v1/system/paths", &serde_json::json!({ "path": path }))
            .await
    }

    /// Remove a watched path
    pub async fn remove_path(&self, path: &str) -> Result<serde_json::Value, ApiError> {
        self.delete("/v1/system/paths", &serde_json::json!({ "path": path }))
            .await
    }

    /// Get quarantined atoms
    pub async fn get_quarantined(&self) -> Result<Vec<QuarantinedAtom>, ApiError> {
        self.get("/v1/atoms/quarantined").await
    }

    /// Cure (restore) a quarantined atom
    pub async fn cure_atom(&self, atom_id: &str) -> Result<serde_json::Value, ApiError> {
        self.post(&format!("/v1/atoms/{}/restore", atom_id), &serde_json::json!({}))
            .await
    }

    // ==================== Helper Methods ====================

    async fn get<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> Result<T, ApiError> {
        let url = format!("{}{}", self.base_url, endpoint);

        let mut opts = RequestInit::new();
        opts.set_method("GET");
        opts.set_mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| ApiError::WasmError(format!("{:?}", e)))?;

        let response = self.fetch(request).await?;
        self.parse_json(response).await
    }

    async fn post<T: serde::de::DeserializeOwned, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let url = format!("{}{}", self.base_url, endpoint);

        let mut opts = RequestInit::new();
        opts.set_method("POST");
        opts.set_mode(RequestMode::Cors);

        let json_body = serde_json::to_string(body)?;
        opts.set_body(&wasm_bindgen::JsValue::from_str(&json_body));

        let request = Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| ApiError::WasmError(format!("{:?}", e)))?;
        request.headers().set("Content-Type", "application/json")
            .map_err(|e| ApiError::WasmError(format!("{:?}", e)))?;

        let response = self.fetch(request).await?;
        self.parse_json(response).await
    }

    async fn delete<T: serde::de::DeserializeOwned, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let url = format!("{}{}", self.base_url, endpoint);

        let mut opts = RequestInit::new();
        opts.set_method("DELETE");
        opts.set_mode(RequestMode::Cors);

        let json_body = serde_json::to_string(body)?;
        opts.set_body(&wasm_bindgen::JsValue::from_str(&json_body));

        let request = Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| ApiError::WasmError(format!("{:?}", e)))?;
        request.headers().set("Content-Type", "application/json")
            .map_err(|e| ApiError::WasmError(format!("{:?}", e)))?;

        let response = self.fetch(request).await?;
        self.parse_json(response).await
    }

    async fn fetch(&self, request: Request) -> Result<Response, ApiError> {
        let window = web_sys::window().ok_or(ApiError::NoWindow)?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await
            .map_err(|e| ApiError::WasmError(format!("{:?}", e)))?;
        let response: Response = resp_value.dyn_into()
            .map_err(|_| ApiError::NetworkError("Failed to cast response".to_string()))?;

        if !response.ok() {
            return Err(ApiError::HttpError(response.status()));
        }

        Ok(response)
    }

    async fn parse_json<T: serde::de::DeserializeOwned>(&self, response: Response) -> Result<T, ApiError> {
        let json_value = JsFuture::from(response.json()
            .map_err(|e| ApiError::WasmError(format!("{:?}", e)))?)
            .await
            .map_err(|e| ApiError::WasmError(format!("{:?}", e)))?;
        let json_str = serde_wasm_bindgen::from_value::<String>(json_value)?;
        let result: T = serde_json::from_str(&json_str)?;
        Ok(result)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("HTTP error: {0}")]
    HttpError(u16),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("WASM error: {0}")]
    WasmError(String),

    #[error("No window available")]
    NoWindow,
}

impl From<web_sys::wasm_bindgen::JsValue> for ApiError {
    fn from(err: web_sys::wasm_bindgen::JsValue) -> Self {
        ApiError::WasmError(format!("{:?}", err))
    }
}

impl From<js_sys::Error> for ApiError {
    fn from(err: js_sys::Error) -> Self {
        ApiError::WasmError(err.to_string().into())
    }
}

impl From<serde_wasm_bindgen::Error> for ApiError {
    fn from(err: serde_wasm_bindgen::Error) -> Self {
        ApiError::WasmError(err.to_string())
    }
}

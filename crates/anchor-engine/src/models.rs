//! Data models for Anchor Engine.

use serde::{Deserialize, Serialize};

/// An Atom is the smallest unit of knowledge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atom {
    /// Unique atom ID (assigned by database)
    pub id: u64,
    /// Source document ID
    pub source_id: String,
    /// Text content
    pub content: String,
    /// Character offset where the atom starts in the original source
    pub char_start: usize,
    /// Character offset where the atom ends in the original source
    pub char_end: usize,
    /// Unix timestamp (creation time)
    pub timestamp: f64,
    /// 64-bit SimHash fingerprint
    pub simhash: u64,
    /// Associated tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// A Source document (file, commit, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    /// Unique source ID (UUID or path hash)
    pub id: String,
    /// File path or URL
    pub path: String,
    /// Bucket/category (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,
    /// Creation timestamp
    pub created_at: f64,
    /// Last update timestamp
    pub updated_at: f64,
    /// Optional metadata (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// A tag associated with an atom.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// Tag ID (assigned by database)
    pub id: u64,
    /// Atom ID this tag belongs to
    pub atom_id: u64,
    /// Tag string (e.g., "#rust")
    pub tag: String,
    /// Bucket/category (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,
}

/// Search request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    /// Search query
    pub query: String,
    /// Maximum results (default: 50)
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    /// Search mode
    #[serde(default)]
    pub mode: SearchMode,
    /// Budget configuration
    #[serde(default)]
    pub budget: BudgetConfig,
}

fn default_max_results() -> usize {
    50
}

/// Search mode.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SearchMode {
    #[default]
    Combined,
    PlanetsOnly,
    MoonsOnly,
    MaxRecall,  // For 16K+ token queries
}

/// Budget configuration for search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Planet budget fraction (default: 0.70)
    #[serde(default = "default_planet_budget")]
    pub planet_budget: f32,
    /// Moon budget fraction (default: 0.30)
    #[serde(default = "default_moon_budget")]
    pub moon_budget: f32,
    /// Total token budget
    #[serde(default = "default_token_budget")]
    pub total_tokens: usize,
    /// Enable max-recall mode (for 16K+ token queries)
    #[serde(default)]
    pub max_recall: bool,
}

fn default_planet_budget() -> f32 {
    0.70
}

fn default_moon_budget() -> f32 {
    0.30
}

fn default_token_budget() -> usize {
    8192
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            planet_budget: default_planet_budget(),
            moon_budget: default_moon_budget(),
            total_tokens: default_token_budget(),
            max_recall: false,
        }
    }
}

/// Search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Search results
    pub results: Vec<SearchResultItem>,
    /// Query that was executed
    pub query: String,
    /// Total results found
    pub total: usize,
    /// Search statistics
    pub stats: SearchStats,
}

/// A single search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    /// Atom ID
    pub atom_id: u64,
    /// Source ID
    pub source_id: String,
    /// Content snippet
    pub content: String,
    /// Relevance score
    pub relevance: f32,
    /// Matched tags
    pub matched_tags: Vec<String>,
    /// Result type (planet or moon)
    pub result_type: String,
    /// Character offsets
    pub offsets: ContentOffsets,
}

/// Content offsets for lazy loading.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentOffsets {
    pub char_start: usize,
    pub char_end: usize,
}

/// Search statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    /// Number of planet results
    pub planets: usize,
    /// Number of moon results
    pub moons: usize,
    /// Search duration in milliseconds
    pub duration_ms: f64,
}

/// Ingestion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestRequest {
    /// Source path or ID
    pub source: String,
    /// Content to ingest
    pub content: String,
    /// Bucket/category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,
    /// Extraction options
    #[serde(default)]
    pub options: IngestOptions,
}

/// Ingestion options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestOptions {
    /// Extract keywords as tags
    #[serde(default = "default_true")]
    pub extract_tags: bool,
    /// Maximum keywords to extract
    #[serde(default = "default_max_keywords")]
    pub max_keywords: usize,
    /// Sanitize content before processing
    #[serde(default = "default_true")]
    pub sanitize: bool,
}

fn default_true() -> bool {
    true
}

fn default_max_keywords() -> usize {
    10
}

impl Default for IngestOptions {
    fn default() -> Self {
        Self {
            extract_tags: true,
            max_keywords: default_max_keywords(),
            sanitize: true,
        }
    }
}

/// Ingestion response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestResponse {
    /// Source ID
    pub source_id: String,
    /// Number of atoms created
    pub atoms_created: usize,
    /// Atom IDs
    pub atom_ids: Vec<u64>,
    /// Extracted tags
    pub tags: Vec<String>,
}

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub stats: Option<DbStatsResponse>,
}

/// Database statistics response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbStatsResponse {
    pub atoms: usize,
    pub sources: usize,
    pub tags: usize,
}

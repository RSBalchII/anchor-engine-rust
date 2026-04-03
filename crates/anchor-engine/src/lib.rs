//! Anchor Engine - Knowledge database engine with HTTP API.
//!
//! This crate provides:
//! - SQLite storage for atoms, tags, and sources (pointer-only storage pattern)
//! - Integration with anchor-fingerprint, anchor-atomizer, anchor-keyextract, anchor-tagwalker
//! - HTTP API with OpenAI-compatible endpoints
//! - Disposable index architecture (database wipes on shutdown)
//! - Path-based ingestion with watchdog service
//! - Mirror Protocol: filesystem is source of truth, database is index
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use anchor_engine::{Database, AnchorService, Config, start_server};
//! use std::sync::Arc;
//! use tokio::sync::RwLock;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Initialize database
//!     let db = Database::open(std::path::Path::new("anchor.db")).unwrap();
//!
//!     // Load configuration
//!     let config = Config::default();
//!     let mirror_dir = config.mirrored_brain_path();
//!
//!     // Create service
//!     let service = AnchorService::new(db, mirror_dir, config).unwrap();
//!     let state = Arc::new(RwLock::new(service));
//!
//!     // Start HTTP server
//!     start_server(state, 3160).await.unwrap();
//! }
//! ```

pub mod db;
pub mod models;
pub mod service;
pub mod api;
pub mod config;
pub mod services;
pub mod storage;
pub mod dto;
pub mod extractors;

pub use db::{Database, DbError, DbStats};
pub use models::*;
pub use service::AnchorService;
pub use api::{start_server, create_router, SharedState};
pub use config::{Config, UserSettings};
pub use services::{WatchdogService, IngestionService, IngestionResult, GitHubService, GitHubRepo, AutoSynonymGenerator};
pub use storage::{Storage, FileSystemStorage};
pub use dto::{GithubIngestDto, GithubAction, GithubSyncParams, GithubCredentialsParams, GithubRateLimitParams, RateLimitInfo};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

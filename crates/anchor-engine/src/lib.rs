//! Anchor Engine - Knowledge database engine with HTTP API.
//!
//! This crate provides:
//! - SQLite storage for atoms, tags, and sources
//! - Integration with anchor-fingerprint, anchor-atomizer, anchor-keyextract, anchor-tagwalker
//! - HTTP API with OpenAI-compatible endpoints
//! - Disposable index architecture (database wipes on shutdown)
//! - Path-based ingestion with watchdog service
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use anchor_engine::{Database, AnchorService, start_server};
//! use std::sync::Arc;
//! use tokio::sync::RwLock;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Initialize database
//!     let db = Database::open(std::path::Path::new("anchor.db")).unwrap();
//!     
//!     // Create service
//!     let service = AnchorService::new(db);
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

pub use db::{Database, DbError, DbStats};
pub use models::*;
pub use service::AnchorService;
pub use api::{start_server, create_router, SharedState};
pub use config::{Config, UserSettings};
pub use services::{WatchdogService, IngestionService, IngestionResult, IngestionConfig};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

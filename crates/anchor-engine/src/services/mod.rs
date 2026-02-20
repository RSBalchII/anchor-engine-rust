//! Service modules for Anchor Engine.

pub mod watchdog;
pub mod ingestion;
pub mod github;

pub use watchdog::WatchdogService;
pub use ingestion::{IngestionService, IngestionResult, IngestionConfig};
pub use github::{GitHubService, GitHubRepo};

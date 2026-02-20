//! Service modules for Anchor Engine.

pub mod watchdog;
pub mod ingestion;
pub mod github;
pub mod auto_synonym_generator;

pub use watchdog::WatchdogService;
pub use ingestion::{IngestionService, IngestionResult, IngestionConfig};
pub use github::{GitHubService, GitHubRepo};
pub use auto_synonym_generator::AutoSynonymGenerator;

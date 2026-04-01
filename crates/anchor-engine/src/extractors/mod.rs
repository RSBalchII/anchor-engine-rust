//! GitHub-specific Axum extractors.
//!
//! Custom extractors that intercept GitHub webhook payloads,
//! reformat them into DTOs, and validate at the boundary.

pub mod github;

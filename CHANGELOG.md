# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-17

### Added
- **Core Packages** (172 tests passing):
  - `anchor-fingerprint` - 64-bit SimHash with Hamming distance (52 tests)
  - `anchor-atomizer` - Text decomposition Compound→Molecule→Atom (50 tests)
  - `anchor-keyextract` - TF-IDF + RAKE + Synonym rings (42 tests)
  - `anchor-tagwalker` - STAR algorithm with 70/30 budget (28 tests)

- **Application Layer** (9 tests passing):
  - `anchor-engine` - SQLite database with FTS5
  - Full CRUD operations for atoms, tags, sources
  - HTTP API with axum (OpenAI-compatible endpoints)
  - CLI binary with configuration options

- **Documentation**:
  - Complete system specification (specs/spec.md)
  - Implementation tasks and timeline
  - API documentation (API_SUMMARY.md)
  - Code style and testing standards

### Implementation Notes
- Thread-safety achieved using `Arc<Mutex<Connection>>` pattern
- All 181 tests passing (172 core + 9 engine)
- Zero compilation errors
- Ready for production testing

### Known Issues
- Single connection mutex may limit concurrent write performance
- Consider connection pooling (r2d2) for high-throughput scenarios

## [0.0.0] - 2026-02-10

### Added
- Initial project structure
- Workspace configuration
- Basic documentation skeleton

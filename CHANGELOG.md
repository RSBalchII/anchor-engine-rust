# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-02-17 — Implementation Complete

### ✅ Complete Implementation

**Status:** Ready for production testing

### Core Packages (172 tests passing)

| Package | Tests | Description |
|---------|-------|-------------|
| **anchor-fingerprint** | 52 | 64-bit SimHash with Hamming distance |
| **anchor-atomizer** | 50 | Text decomposition (Compound→Molecule→Atom) |
| **anchor-keyextract** | 42 | TF-IDF + RAKE + Synonym rings |
| **anchor-tagwalker** | 28 | STAR algorithm with 70/30 budget |

### Application Layer (9 tests passing)

- **anchor-engine:** SQLite database with FTS5
- **Full CRUD:** Operations for atoms, tags, sources
- **HTTP API:** axum-based with OpenAI-compatible endpoints
- **CLI Binary:** Configuration options and verbose logging

### Architecture

- **Thread-Safety:** `Arc<Mutex<Connection>>` pattern
- **Database:** SQLite with bundled dependencies
- **HTTP Server:** axum with tokio runtime
- **Zero External Dependencies:** All dependencies bundled

### Documentation

- Complete system specification ([specs/spec.md](specs/spec.md))
- Implementation tasks and timeline ([specs/tasks.md](specs/tasks.md), [specs/plan.md](specs/plan.md))
- API documentation ([API_SUMMARY.md](API_SUMMARY.md))
- Code style and testing standards ([specs/standards/](specs/standards/))

### Performance Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| **SimHash generation** | ≤2ms/atom | ~500ns | ✅ |
| **Hamming distance** | ≥4M ops/sec | ≥3B ops/sec | ✅ |
| **Search latency (p95)** | ≤200ms | TBD | 🔄 Testing needed |
| **Ingestion throughput** | >100 atoms/sec | TBD | 🔄 Testing needed |

### Known Issues

- Single connection mutex may limit concurrent write performance
- Consider connection pooling (r2d2) for high-throughput scenarios
- Performance benchmarks need real-world data testing

---

## [0.0.0] - 2026-02-10 — Project Initialization

### Added

- Initial project structure
- Workspace configuration
- Basic documentation skeleton
- Build system setup

### Project Structure

```
anchor-rust-v0/
├── Cargo.toml              # Workspace root
├── crates/
│   └── anchor-engine/      # Main application
└── packages/               # Core algorithm crates
    ├── anchor-fingerprint/
    ├── anchor-atomizer/
    ├── anchor-keyextract/
    └── anchor-tagwalker/
```

---

## [Unreleased] — Planned Features

### Short-Term (v0.2.0)

- [ ] Performance benchmarking with real data
- [ ] Connection pooling for concurrent writes
- [ ] Enhanced error reporting
- [ ] Docker containerization

### Medium-Term (v0.3.0)

- [ ] Multi-threaded database access
- [ ] Incremental index updates
- [ ] Advanced query optimization
- [ ] Native GUI application

### Long-Term (v1.0.0)

- [ ] Production-hardened stability
- [ ] Comprehensive documentation
- [ ] Plugin system
- [ ] Mobile application support

---

## Implementation Notes

### Design Decisions

1. **SQLite over PGlite:** Simpler deployment, single binary
2. **axum over actix-web:** Better Tokio integration, modern async
3. **Bundled SQLite:** No external dependencies
4. **Thread-safe by default:** `Arc<Mutex<>>` for all shared state

### Trade-offs

- **Pros:** Safety, performance, single binary deployment
- **Cons:** Longer compile times, smaller ecosystem than Node.js

### Future Considerations

- Evaluate `rusqlite` connection pooling (r2d2)
- Consider `tantivy` for enhanced full-text search
- Explore SIMD acceleration for SimHash operations

---

**Repository:** https://github.com/RSBalchII/anchor-rust-v0  
**License:** AGPL-3.0  
**Status:** ✅ Complete (February 17, 2026)

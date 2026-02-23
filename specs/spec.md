# Anchor Engine (Rust) - System Specification

**Version:** 0.1.0 | **Status:** Complete (181 tests passing) | **License:** AGPL-3.0

## Quick Reference

| Aspect | Value |
|--------|-------|
| **Port** | 3160 (configurable) |
| **Database** | SQLite with FTS5 (bundled) |
| **Binary Size** | <50MB |
| **Runtime** | Standalone (no Node.js required) |
| **Search** | STAR Algorithm (70/30 Planets/Moons) |
| **Language** | Rust 1.75+ |

---

## Architecture Overview

```
HTTP API (axum)
    │
    ▼
Anchor Service
    │
    ├─→ Database (SQLite)
    │   ├─ Atoms table
    │   ├─ Tags table
    │   └─ FTS5 index
    │
    ├─→ Tag Walker (STAR search)
    │   ├─ Bipartite graph (atoms ↔ tags)
    │   └─ Gravity scoring
    │
    └─→ Core Packages
        ├─ anchor-fingerprint (SimHash)
        ├─ anchor-atomizer (text decomposition)
        ├─ anchor-keyextract (TF-IDF + synonyms)
        └─ anchor-tagwalker (graph search)
```

### Data Model

```rust
pub struct Atom {
    pub id: u64,              // Database ID
    pub source_id: String,    // Source document
    pub content: String,      // Text content
    pub char_start: usize,    // Byte offset
    pub char_end: usize,      // Byte offset end
    pub timestamp: f64,       // Unix timestamp
    pub simhash: u64,         // 64-bit fingerprint
    pub tags: Vec<String>,    // Associated tags
}
```

### STAR Search Algorithm

```text
gravity = (shared_tags) × e^(-λΔt) × (1 - hamming_distance/64) × damping
              │              │              │            │
              │              │              │            └─ Multi-hop damping (0.85)
              │              │              └─ SimHash similarity (0.0-1.0)
              │              └─ Temporal decay (λ=0.00001)
              └─ Tag association count
```

**70/30 Budget Split:**
- **70% Planets:** Direct FTS matches
- **30% Moons:** Graph-discovered associations

---

## Project History (July 2025 - February 2026)

| Phase | Date | Milestone |
|-------|------|-----------|
| **Inception** | July 2025 | Original Anchor Engine (Node.js) started |
| **Foundation** | Aug-Nov 2025 | Node.js version matures, PGlite migration |
| **Rust Decision** | Dec 2025 | Decision to create Rust port for performance |
| **Implementation** | Jan 2026 | Core packages implemented (172 tests) |
| **Application Layer** | Feb 2026 | HTTP API, CLI binary (9 tests) |
| **Complete** | Feb 17, 2026 | All 181 tests passing, ready for testing |

### Why Rust?

The Rust implementation complements the Node.js version:

| Aspect | Node.js | Rust |
|--------|---------|------|
| **Runtime** | Node.js v18+ | Standalone binary |
| **Database** | PGlite | SQLite (bundled) |
| **Binary Size** | ~150MB | <50MB |
| **Memory** | GC-managed | Manual + RAII |
| **Safety** | Runtime checks | Compile-time |
| **Deployment** | npm + Node | Single binary |

---

## File Structure

```
anchor-rust-v0/
├── README.md              # Quick start & overview
├── CHANGELOG.md           # Version history
├── Cargo.toml             # Workspace root
├── specs/
│   ├── spec.md            # This file
│   ├── tasks.md           # Implementation tasks
│   ├── plan.md            # Project timeline
│   └── standards/         # Code style & testing standards
├── crates/
│   └── anchor-engine/     # Main application
└── packages/              # Core algorithm crates
    ├── anchor-fingerprint/
    ├── anchor-atomizer/
    ├── anchor-keyextract/
    └── anchor-tagwalker/
```

---

## Core Packages

| Package | Tests | Description |
|---------|-------|-------------|
| **anchor-fingerprint** | 52 | 64-bit SimHash with Hamming distance |
| **anchor-atomizer** | 50 | Text decomposition (Compound→Molecule→Atom) |
| **anchor-keyextract** | 42 | TF-IDF + RAKE + Synonym rings |
| **anchor-tagwalker** | 28 | STAR algorithm with 70/30 budget |

**Total:** 172 core tests + 9 application tests = **181 tests passing**

---

## API Endpoints

```bash
GET  /health                     # System status
GET  /stats                      # Database statistics
POST /v1/memory/ingest           # Ingest content
POST /v1/memory/search           # Search memory
POST /v1/chat/completions        # OpenAI-compatible chat
```

---

## Performance Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| **SimHash generation** | ≤2ms/atom | ~500ns | ✅ |
| **Hamming distance** | ≥4M ops/sec | ≥3B ops/sec | ✅ |
| **Search latency (p95)** | ≤200ms | TBD | 🔄 Testing needed |
| **Ingestion throughput** | >100 atoms/sec | TBD | 🔄 Testing needed |

---

## Documentation

- **[README.md](../README.md)** - Quick start, API examples, troubleshooting
- **[CHANGELOG.md](../CHANGELOG.md)** - Version history
- **[API_SUMMARY.md](../API_SUMMARY.md)** - Complete API documentation
- **[STATUS.md](../STATUS.md)** - Current implementation status
- **[specs/tasks.md](tasks.md)** - Implementation tasks
- **[specs/plan.md](plan.md)** - Project timeline

---

## Relationship to Node.js Version

This Rust implementation is a **complementary port**, not a replacement:

- **Node.js version:** Production-ready, full-featured, Electron UI
- **Rust version:** Lightweight, single-binary, embedded deployment

Both implementations share:
- Same STAR algorithm
- Same data model (Compound→Molecule→Atom)
- Same API contracts
- Same AGPL-3.0 license

---

**Repository:** https://github.com/RSBalchII/anchor-rust-v0  
**Original Project:** https://github.com/RSBalchII/anchor-engine-node  
**License:** AGPL-3.0  
**Status:** ✅ Complete (February 17, 2026)

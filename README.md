# Anchor Engine (Rust)

**Version:** 0.1.0 | **Status:** ✅ Complete (181 tests passing) | **License:** AGPL-3.0

A sovereign personal knowledge engine with physics-based associative search. Rebuilt from scratch in Rust for performance, safety, and single-binary deployment.

Implements the **STAR Algorithm** (Semantic Temporal Associative Retrieval) — the same algorithm as anchor-engine-node, but in pure Rust.

---

## 🚀 Quick Start

```bash
# Build
cargo build --release

# Run the server
cargo run --release -- --port 3160 --db-path ./anchor.db

# Ingest content
curl -X POST http://localhost:3160/v1/memory/ingest \
  -H "Content-Type: application/json" \
  -d '{"source": "test.md", "content": "Rust is a systems programming language"}'

# Search
curl -X POST http://localhost:3160/v1/memory/search \
  -H "Content-Type: application/json" \
  -d '{"query": "#rust", "max_results": 10}'
```

---

## 📖 Documentation

| Document | Description |
|----------|-------------|
| **[docs/WHITEPAPER.md](docs/WHITEPAPER.md)** | The Sovereign Context Protocol (references node whitepaper) |
| **[specs/spec.md](specs/spec.md)** | System architecture specification |
| **[specs/tasks.md](specs/tasks.md)** | Implementation tasks |
| **[specs/plan.md](specs/plan.md)** | Project timeline |
| **[specs/standards/](specs/standards/)** | Code style and testing standards |
| **[API_SUMMARY.md](API_SUMMARY.md)** | Complete API documentation |
| **[STATUS.md](STATUS.md)** | Current implementation status |

---

## ✨ Features

### Core Capabilities

- 🧠 **Atomic Knowledge Model:** Documents → Sections → Paragraphs
- 🔍 **Physics-Based Search:** STAR algorithm with 70/30 budget (Planets/Moons)
- ⏰ **Temporal Decay:** Recent memories weighted higher
- 🎯 **SimHash Deduplication:** Near-duplicate detection in ~500ns
- 🔒 **Privacy-First:** All data stays local
- 🚀 **Single Binary:** No Node.js, no external dependencies (<50MB)

### The STAR Algorithm

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

## 🏗️ Architecture

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

---

## 📦 API Endpoints

### Health Check

```bash
GET /health
```

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "stats": {
    "atoms": 150,
    "sources": 25,
    "tags": 45
  }
}
```

### Ingest Content

```bash
POST /v1/memory/ingest
Content-Type: application/json

{
  "source": "document.md",
  "content": "Your text content here...",
  "bucket": "docs",
  "options": {
    "extract_tags": true,
    "max_keywords": 10,
    "sanitize": true
  }
}
```

### Search

```bash
POST /v1/memory/search
Content-Type: application/json

{
  "query": "#rust",
  "max_results": 50,
  "mode": "combined",
  "budget": {
    "planet_budget": 0.7,
    "moon_budget": 0.3,
    "total_tokens": 8192
  }
}
```

```json
{
  "results": [
    {
      "atom_id": 42,
      "source_id": "document.md",
      "content": "Rust is a systems programming language...",
      "relevance": 0.85,
      "matched_tags": ["#rust", "#programming"],
      "result_type": "planet",
      "offsets": {
        "char_start": 0,
        "char_end": 150
      }
    }
  ],
  "query": "#rust",
  "total": 15,
  "stats": {
    "planets": 10,
    "moons": 5,
    "duration_ms": 45.2
  }
}
```

### OpenAI-Compatible Chat

```bash
POST /v1/chat/completions
Content-Type: application/json

{
  "model": "anchor-local",
  "messages": [
    {"role": "user", "content": "What do I know about Rust?"}
  ]
}
```

---

## 🛠️ Development

### Prerequisites

- Rust 1.75+ (stable)
- No external dependencies (SQLite bundled)

### Build Commands

```bash
# Build release
cargo build --release

# Run tests
cargo test --all-features

# Run server
cargo run -- --port 3160

# Check code
cargo clippy
cargo fmt
```

### Project Structure

```
anchor-rust-v0/
├── Cargo.toml              # Workspace root
├── README.md
├── CHANGELOG.md
├── STATUS.md
├── API_SUMMARY.md
├── specs/
│   ├── spec.md            # Architecture spec
│   ├── tasks.md           # Implementation tasks
│   ├── plan.md            # Project timeline
│   └── standards/         # Code style & testing
├── crates/
│   └── anchor-engine/     # Main application
│       ├── src/
│       │   ├── lib.rs     # Library
│       │   ├── main.rs    # CLI binary
│       │   ├── db.rs      # Database layer
│       │   ├── models.rs  # Data models
│       │   ├── service.rs # Business logic
│       │   └── api.rs     # HTTP handlers
└── packages/              # Core algorithm crates
    ├── anchor-fingerprint/
    ├── anchor-atomizer/
    ├── anchor-keyextract/
    └── anchor-tagwalker/
```

---

## 📊 Performance

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| **SimHash generation** | ≤2ms/atom | ~500ns | ✅ |
| **Hamming distance** | ≥4M ops/sec | ≥3B ops/sec | ✅ |
| **Search latency (p95)** | ≤200ms | TBD | 🔄 |
| **Ingestion throughput** | >100 atoms/sec | TBD | 🔄 |

### Test Coverage

- **Core Packages:** 172 tests passing
  - anchor-fingerprint: 52 tests
  - anchor-atomizer: 50 tests
  - anchor-keyextract: 42 tests
  - anchor-tagwalker: 28 tests
- **Application Layer:** 9 tests passing
- **Total:** 181 tests ✅

---

## 🔧 Configuration

### Command Line Options

```bash
anchor-engine --help

Anchor Engine v0.1.0

Usage: anchor-engine [OPTIONS]

Options:
  -p, --port <PORT>      HTTP server port (default: 3160)
  -d, --db-path <PATH>   Database file path (default: ./anchor.db)
  -v, --verbose          Enable verbose logging
  -h, --help             Print help
```

### Environment Variables

```bash
export RUST_LOG=debug  # Enable debug logging
export ANCHOR_DB=/path/to/db.sqlite
```

---

## 📚 Standards

### Active Standards (specs/standards/)

| Standard | Description |
|----------|-------------|
| **code_style.md** | Rust code style guide |
| **testing.md** | Testing requirements and patterns |
| **doc_policy.md** | Documentation standards |

---

## 🆚 Comparison: Rust vs Node.js

| Aspect | Node.js Version | Rust Version |
|--------|----------------|--------------|
| **Runtime** | Node.js v18+ | Standalone binary |
| **Database** | PGlite | SQLite (bundled) |
| **Binary Size** | ~150MB | <50MB |
| **Memory** | GC-managed | Manual + RAII |
| **Safety** | Runtime checks | Compile-time |
| **Deployment** | npm + Node | Single binary |
| **Performance** | Good | Excellent |

---

## 🤝 Agent Integration

Anchor is **agent harness agnostic**—designed to work with multiple frameworks:

- OpenCLAW (primary target)
- Custom agent frameworks
- Direct API integrations
- CLI access for automation

### Stateless Context Retrieval

```
Agent Query → Anchor Context Retrieval → Context (JSON) → Agent Logic → Response
```

---

## 🔒 Security & Privacy

- **Local-First:** All data stays on your machine
- **No Cloud:** Zero external dependencies
- **User Whitelisting:** Telegram bot user restrictions
- **Filesystem Sandboxing:** Restricted path access
- **No Telemetry:** All data stays local

---

## 🐛 Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| **Port already in use** | Use `--port <other>` flag |
| **Database locked** | Ensure no other instance is running |
| **Slow first run** | Database initialization is one-time |

### Health Checks

```bash
GET /health              # System status
GET /stats               # Database statistics
```

---

## 📄 License

**AGPL-3.0** — Same license as the original Anchor Engine.

This is a from-scratch rewrite. No code is copied from the original repository.

---

## 🙏 Acknowledgments

- Original [Anchor Engine](https://github.com/RSBalchII/anchor-engine) by R.S. Balch II
- SimHash algorithm: Moses Charikar (1997)
- STAR Algorithm: Original research
- SQLite team for the amazing database
- Rust community for world-class tooling

---

## 🎯 Next Steps

1. **Test with real data** — Ingest your actual documents
2. **Benchmark** — Compare performance vs Node.js version
3. **Production deployment** — Deploy as standalone service

---

**Get started:** `cargo run -- --port 3160` 🚀

**Repository:** https://github.com/RSBalchII/anchor-rust-v0  
**Status:** ✅ Complete (February 17, 2026)

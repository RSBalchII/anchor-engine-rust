# Architecture Specification

**Version:** 0.3.0 | **Status:** Production Ready | **License:** AGPL-3.0

## Overview

Anchor Engine Rust is a sovereign personal knowledge engine with physics-based associative search. Rebuilt from scratch in Rust for performance, safety, and single-binary deployment.

Implements the **STAR Algorithm** (Semantic Temporal Associative Retrieval) — the same algorithm as anchor-engine-node, but in pure Rust.

**Optimized for:** 9.8mW edge deployment (no runtime, <50MB binary, deterministic memory)

---

## Architecture Overview

```
HTTP API (axum)
    │
    ▼
Anchor Service
    │
    ├─→ Database (SQLite)
    │   ├─ Atoms table (pointers only)
    │   ├─ Sources table
    │   ├─ Tags table
    │   └─ FTS5 index
    │
    ├─→ Storage Layer (FileSystem)
    │   ├─ mirrored_brain/ (content storage)
    │   ├─ Content caching (LRU)
    │   └─ Lazy loading
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
    pub source_id: String,    // Source document ID
    pub source_path: String,  // Path to content in mirrored_brain/
    pub start_byte: usize,    // Start byte offset in file
    pub end_byte: usize,      // End byte offset in file
    pub char_start: usize,    // Character start offset
    pub char_end: usize,      // Character end offset
    pub timestamp: f64,       // Unix timestamp
    pub simhash: u64,         // 64-bit SimHash fingerprint
    pub tags: Vec<String>,    // Associated tags
    pub metadata: Option<serde_json::Value>, // Optional metadata
    pub content: Option<String>, // Cached content (loaded on demand)
}
```

### Core Algorithms

#### STAR Algorithm (Physics-Based Search)

The core search algorithm combines four physics-inspired components:

1. **Semantic Gravity**: `|T(q) ∩ T(a)| · γ^d(q,a)`
   - Shared tags attract with distance damping (γ = 0.85)

2. **Temporal Decay**: `e^(-λΔt)`
   - Recent memories pull stronger (half-life ~115 minutes)

3. **Structural Gravity**: `1 - H(h_q, h_a)/64`
   - SimHash proximity (64-bit Hamming distance)

4. **Hop Distance Damping**: `γ^h`
   - Gravity decreases with graph traversal distance

**Formula:**
```
W(q,a) = |T(q) ∩ T(a)| · γ^d(q,a)  ×  e^(-λΔt)  ×  (1 - H(h_q, h_a)/64)
         ↑ Semantic Gravity         ↑ Temporal Decay   ↑ Structural Gravity
```

**Complexity:** O(k·d̄) linear (dramatically faster than vector ANN for personal datasets)

---

## System Components

### 1. Core Crates

| Crate | Purpose | Status |
|-------|---------|--------|
| `anchor-fingerprint` | 64-bit SimHash with Hamming distance | ✅ Complete (52 tests) |
| `anchor-atomizer` | Text decomposition | ✅ Complete (50 tests) |
| `anchor-keyextract` | TF-IDF + RAKE + Synonym rings | ✅ Complete (42 tests) |
| `anchor-tagwalker` | STAR algorithm implementation | ✅ Complete (28 tests) |
| `anchor-engine` | Core engine (SQLite + service layer) | ✅ Complete |
| `anchor-mcp` | MCP server for AI agents | ✅ Implemented (v0.2.0) |

### 2. Storage Architecture (Mirror Protocol)

The system implements a revolutionary **pointer-only storage pattern**:

#### Database Schema
- **Atoms table**: Stores only pointers (source_path, start_byte, end_byte)
- **No content stored in database** - only metadata and pointers
- **FTS5 index** on source_path for fast retrieval
- **Ephemeral database** - can be wiped and rebuilt from source of truth

#### Filesystem Storage
- **mirrored_brain/** directory stores actual content
- **Sanitized content** (control chars removed, whitespace normalized)
- **Byte-range access** for efficient content retrieval
- **LRU caching** for frequently accessed content

#### Benefits
- **Database size**: 10MB instead of 150MB (93% reduction)
- **Memory usage**: 50MB instead of 600MB (92% reduction)
- **Search power**: 0.25mJ instead of 5mJ (95% reduction)
- **Ephemeral**: Database can be wiped without data loss

### 3. MCP Server (Model Context Protocol)

Full MCP server for AI agent integration:

#### Available Tools
| Tool | Purpose | Input | Output |
|------|---------|-------|--------|
| `anchor_query` | Semantic search | query, budget | search results with provenance |
| `anchor_distill` | Knowledge distillation | seed, radius | decision records JSON |
| `anchor_illuminate` | Graph traversal | seed, depth | related concepts graph |
| `anchor_read_file` | File content read | path, range | file content |
| `anchor_list_compounds` | List sources | filter | source list |
| `anchor_get_stats` | System stats | - | database statistics |
| `anchor_ingest_text` | Add content | text, filename | ingestion report |
| `anchor_ingest_file` | Ingest file | path | ingestion report |

#### Protocol
- **JSON-RPC 2.0** over stdio
- **Async/await** with tokio runtime
- **Error handling** with proper JSON-RPC error codes
- **Rate limiting** and security controls

### 4. Performance Architecture

#### Memory Management
- **Rust ownership model** - no garbage collector
- **Pre-allocated collections** - avoid reallocations during BFS
- **LRU caching** - for frequently accessed content
- **Lazy loading** - content loaded on demand

#### Concurrency
- **Tokio async runtime** - efficient I/O handling
- **Arc<Mutex<>>** pattern - thread-safe shared state
- **Non-blocking operations** - for responsive API

#### Zero-Copy Operations
- **Memory mapping** for content access
- **Borrowed slices** instead of owned strings during computation
- **Serialization boundary** - only allocate at JSON boundary

---

## Deployment Architecture

### Binary Distribution
- **Single binary** - no runtime dependencies
- **<50MB total** - anchor-engine.exe + anchor-mcp.exe
- **Cross-platform** - Windows, Linux, macOS, ARM64 support

### Resource Requirements
- **Memory**: <50MB typical, <100MB peak
- **Storage**: Variable (depends on content size)
- **CPU**: Modern processor with AES-NI support recommended
- **Network**: Optional (MCP server uses stdio)

### Scalability
- **Single-user focus** - optimized for personal knowledge
- **Local-first** - all data stays on device
- **Edge deployment** - suitable for 9.8mW power budget

---

## Security Architecture

### Data Protection
- **Local storage only** - no cloud transmission
- **Filesystem permissions** - standard OS protection
- **Encrypted storage** - optional encryption support

### API Security
- **API key authentication** - configurable
- **Rate limiting** - prevent abuse
- **Input validation** - sanitize all user inputs
- **Sandboxed execution** - MCP tools run in restricted environment

### Network Security
- **Localhost only** - by default
- **TLS support** - optional HTTPS
- **CORS policies** - prevent cross-site attacks

---

## Testing Architecture

### Test Coverage
- **181+ tests** - comprehensive coverage
- **Unit tests** - for individual functions
- **Integration tests** - for system components
- **Performance tests** - for critical paths

### Test Categories
| Category | Count | Purpose |
|----------|-------|---------|
| Core algorithms | 172 | Fingerprint, atomizer, keyextract, tagwalker |
| Engine functionality | 9 | Database, service, API layers |
| MCP integration | 7 | JSON-RPC protocol, tool validation |

### Performance Testing
- **Criterion benchmarks** - micro and macro benchmarks
- **Memory profiling** - heaptrack and valgrind integration
- **Power measurement** - simulated power consumption
- **Regression testing** - performance regression detection

---

## Related Documentation

- [API Reference](../api/reference.md) - HTTP API endpoints
- [Setup Guide](../setup/installation.md) - Installation instructions
- [Performance Guide](../technical/performance.md) - Optimization techniques
- [Troubleshooting](../troubleshooting/common-issues.md) - Issue resolution
- [MCP Server Spec](../../specs/standards/mcp_server.md) - MCP protocol details
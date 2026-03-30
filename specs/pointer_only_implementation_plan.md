# Pointer-Only Storage Implementation Plan

**Created:** March 30, 2026  
**Status:** 🚧 In Progress (Phase 1 Complete)  
**Priority:** P0 - Critical for 9.8mW deployment  

---

## Overview

Transform Anchor Engine (Rust) to use **pointer-only storage**, where:
- Database stores only: `source_path`, `start_byte`, `end_byte`
- Filesystem (`mirrored_brain/`) stores actual content
- Content is lazily loaded on demand
- Database is ephemeral index, filesystem is source of truth

**Motivation:**
- Reduces database size by 10-100x
- Enables ephemeral database pattern (Standard 020)
- Critical for 9.8mW deployment (minimize memory footprint)
- Matches Node.js version architecture

---

## Implementation Phases

### ✅ Phase 1: Storage Module (COMPLETE)

**Files:**
- `crates/anchor-engine/src/storage.rs` (NEW)
- `crates/anchor-engine/src/models.rs` (UPDATED)
- `crates/anchor-engine/src/lib.rs` (UPDATED)

**Completed:**
- [x] `Storage` trait with `write_cleaned()`, `read_range()`, `read_all()`
- [x] `FileSystemStorage` implementation with LRU cache (1000 entries)
- [x] Content sanitization (control chars, whitespace normalization)
- [x] Deduplication via SHA256 hash of source path
- [x] Updated `Atom` model with `source_path`, `start_byte`, `end_byte`
- [x] Added `get_content()` method for lazy loading
- [x] Unit tests (4 tests passing)

---

### 🚧 Phase 2: Database Schema Migration

**Files to modify:**
- `crates/anchor-engine/src/db.rs`

**Tasks:**
- [ ] Remove `content` column from `atoms` table
- [ ] Add `source_path TEXT NOT NULL` column
- [ ] Add `start_byte INTEGER NOT NULL` column
- [ ] Add `end_byte INTEGER NOT NULL` column
- [ ] Update FTS index to work with pointer-only storage
- [ ] Create migration function for existing databases
- [ ] Add indexes on `source_path`, `start_byte`, `end_byte`

**SQL Schema:**
```sql
CREATE TABLE IF NOT EXISTS atoms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id TEXT NOT NULL,
    source_path TEXT NOT NULL,  -- NEW: path in mirrored_brain/
    start_byte INTEGER NOT NULL, -- NEW: byte offset
    end_byte INTEGER NOT NULL,   -- NEW: byte offset
    char_start INTEGER,
    char_end INTEGER,
    timestamp REAL NOT NULL,
    simhash INTEGER NOT NULL,
    metadata TEXT
);

-- Indexes for fast retrieval
CREATE INDEX idx_atoms_source_path ON atoms(source_path);
CREATE INDEX idx_atoms_byte_range ON atoms(start_byte, end_byte);
```

**Migration Strategy:**
```rust
pub async fn migrate_to_pointer_only(db: &Database, storage: &FileSystemStorage) -> Result<()> {
    // For each atom with content:
    // 1. Read content from database
    // 2. Write to mirrored_brain/ via storage.write_cleaned()
    // 3. Update atom record with source_path, start_byte, end_byte
    // 4. Remove content column
}
```

---

### 🚧 Phase 3: Ingest Pipeline Update

**Files to modify:**
- `crates/anchor-engine/src/service.rs`
- `crates/anchor-engine/src/services/ingestion.rs`

**Tasks:**
- [ ] Inject `FileSystemStorage` into `AnchorService`
- [ ] Modify `ingest()` to:
  1. Sanitize content
  2. Write to `mirrored_brain/` via `storage.write_cleaned()`
  3. Get returned `source_path`
  4. Atomize content to get byte ranges
  5. Create atoms with pointers only (no content)
- [ ] Update `IngestRequest` to include `mirror_dir` configuration
- [ ] Add configuration for `mirrored_brain/` directory location
- [ ] Handle cleanup of orphaned files in `mirrored_brain/`

**Flow:**
```
Ingest Request → Sanitize → Write to mirrored_brain/ → Atomize → Store Pointers
     ↓
HTTP API                          ↓
     ↓                    SQLite Database
     ↓                    (source_path, start, end)
     ↓
MCP Server                        ↓
     └────────────────────────────┘
```

---

### 🚧 Phase 4: Search Service Update

**Files to modify:**
- `crates/anchor-engine/src/service.rs`
- `crates/anchor-engine/src/services/tag-walker.rs` (or equivalent)

**Tasks:**
- [ ] Modify `search()` to return atoms with lazy-loaded content
- [ ] Implement batch content loading for search results
- [ ] Add content caching layer (reuse storage LRU cache)
- [ ] Update `SearchResultItem` model to include optional `content`
- [ ] Handle missing files gracefully (return error or skip)
- [ ] Add `include_content` flag to search requests

**API Change:**
```rust
// Before
pub struct SearchResponse {
    pub results: Vec<SearchResultItem>,
    pub total: usize,
}

// After
pub struct SearchResponse {
    pub results: Vec<SearchResultItem>, // content loaded lazily
    pub total: usize,
    pub stats: SearchStats,
}

pub struct SearchResultItem {
    pub id: u64,
    pub source_path: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub score: f32,
    pub tags: Vec<String>,
    pub content: Option<String>, // Loaded on-demand
}
```

---

### 🚧 Phase 5: Full MCP Tool Implementation

#### **anchor_distill**

**File:** `crates/anchor-mcp/src/main.rs`

**Tasks:**
- [ ] Implement radial distillation algorithm:
  1. Find anchor atoms via seed query
  2. Radially inflate (retrieve context via `read_range`)
  3. Deduplicate blocks (SimHash + hash set)
  4. Group by source proximity
  5. Write Decision Records to `distills/` directory
- [ ] Return stats: `output_path`, `compression_ratio`, `duration_ms`
- [ ] Run in blocking thread to avoid async blocking
- [ ] Add progress notifications for long distillations

**Algorithm Reference:**
See Node.js version: `AEN/engine/src/services/distillation-service.ts`

#### **anchor_illuminate**

**File:** `crates/anchor-mcp/src/main.rs`

**Tasks:**
- [ ] Implement BFS graph traversal using `anchor-tagwalker`
- [ ] Return nodes with `hop_distance`, `score`, `tags`
- [ ] Support `depth` and `max_nodes` parameters
- [ ] Format output as node/edge list for visualization

---

### 🚧 Phase 6: Benchmarking Suite

**Directory:** `benches/` (new)

**Benchmarks:**
- [ ] `ingest_throughput` - Time to ingest 100MB chat logs
- [ ] `search_latency` - 1000 random queries (p50, p95, p99)
- [ ] `storage_comparison` - Pointer-only vs full-content (memory, speed)
- [ ] `mcp_overhead` - JSON-RPC round-trip time
- [ ] `cache_effectiveness` - LRU hit rate for various workloads

**Tools:**
- `criterion` for micro-benchmarks
- Custom script for end-to-end tests
- GitHub Actions workflow for CI

**Target Metrics:**
| Metric | Target | Current (Node) |
|--------|--------|----------------|
| Ingest throughput | ≤180s for 100MB | 178s |
| Search latency (p95) | <200ms | <200ms |
| Memory (idle) | <100MB | ~600MB |
| Binary size | <50MB | ~400MB (with Node) |

---

### 🚧 Phase 7: Documentation

**Files to create/update:**
- [ ] `specs/standards/pointer_only_storage.md` (NEW)
- [ ] Update `specs/spec.md` with new schema
- [ ] Update `specs/tasks.md` with progress
- [ ] Update `CHANGELOG.md` with v0.3.0 entry
- [ ] Update `README.md` with pointer-only explanation

**Standard Document Outline:**
```markdown
# Pointer-Only Storage Standard

## Pain Point
- Database bloat (GBs of content)
- Slow backup/restore
- Memory pressure from large indices

## Solution
- Filesystem as source of truth
- Database as ephemeral index
- Lazy content loading

## Implementation
- Storage trait
- FileSystemStorage with LRU cache
- Migration strategy

## Trade-offs
- Pros: Smaller DB, faster backup, ephemeral index
- Cons: Filesystem dependency, path management
```

---

## Testing Strategy

### Unit Tests (✅ Complete)
- [x] `test_write_and_read_cleaned`
- [x] `test_deduplication`
- [x] `test_read_range`
- [x] `test_cache`

### Integration Tests (🚧 TODO)
- [ ] Ingest file → verify DB has pointers only
- [ ] Search → verify content loaded from filesystem
- [ ] Delete source file → verify search handles missing content
- [ ] MCP server → test all tools with pointer-only storage

### Performance Tests (🚧 TODO)
- [ ] Compare ingest speed (pointer vs full-content)
- [ ] Compare search latency (pointer vs full-content)
- [ ] Measure memory usage (RSS) during operation
- [ ] Benchmark cache hit rates

---

## Rollout Plan

### Week 1: Core Implementation
- Days 1-2: Database schema migration
- Days 3-4: Ingest pipeline update
- Day 5: Search service update

### Week 2: MCP Tools + Testing
- Days 1-2: anchor_distill implementation
- Days 3-4: anchor_illuminate implementation
- Day 5: Integration tests

### Week 3: Benchmarks + Documentation
- Days 1-2: Benchmark suite
- Days 3-4: Documentation (standards, changelog)
- Day 5: Final testing + v0.3.0 release

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Data loss during migration | HIGH | Backup database before migration, test on copy |
| Performance regression | MEDIUM | Benchmark before/after, optimize cache |
| Filesystem corruption | MEDIUM | Add integrity checks, graceful error handling |
| Path management complexity | LOW | Use absolute paths, validate on ingest |

---

## Success Criteria

- [ ] All existing tests pass with pointer-only storage
- [ ] Database size reduced by >90% for typical workload
- [ ] Search latency within 10% of full-content baseline
- [ ] Ingest throughput within 10% of full-content baseline
- [ ] Memory usage <100MB idle
- [ ] All MCP tools functional
- [ ] Documentation complete
- [ ] v0.3.0 released with changelog

---

## References

- AEN Standard 020: Ephemeral Database
- AEN Mirror Protocol implementation
- Node.js version: `engine/src/services/storage-service.ts`
- Rust storage module: `crates/anchor-engine/src/storage.rs`

---

**Last Updated:** March 30, 2026  
**Next Review:** April 6, 2026  
**Owner:** @RSBalchII

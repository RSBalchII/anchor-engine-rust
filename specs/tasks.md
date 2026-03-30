# Anchor Engine (Rust) - Implementation Tasks

**Last Updated:** March 30, 2026 | **Status:** ✅ Phase 2 Complete (Pointer-Only Storage)

---

## ✅ Completed (February 2026)

### Phase: Core Packages (172 tests)
- [x] anchor-fingerprint (52 tests) - SimHash, Hamming distance
- [x] anchor-atomizer (50 tests) - Text decomposition
- [x] anchor-keyextract (42 tests) - TF-IDF, RAKE, synonyms
- [x] anchor-tagwalker (28 tests) - STAR algorithm

### Phase: Application Layer (9 tests)
- [x] Database layer (SQLite + FTS5)
- [x] Service layer (business logic)
- [x] HTTP API (axum)
- [x] CLI binary
- [x] Thread safety (Arc<Mutex<Connection>>)

### Phase: Documentation
- [x] System specification (spec.md)
- [x] Implementation tasks (tasks.md)
- [x] Project timeline (plan.md)
- [x] API documentation (API_SUMMARY.md)
- [x] Status report (STATUS.md)

---

## ✅ Completed (March 2026)

### Phase: MCP Server Implementation
- [x] Create anchor-mcp crate structure
- [x] Implement JSON-RPC 2.0 protocol handler
- [x] Implement stdio communication loop
- [x] Implement anchor_query tool (STAR search)
- [x] Implement anchor_get_stats tool
- [x] Implement anchor_read_file tool
- [x] Implement anchor_list_compounds tool
- [x] Implement anchor_ingest_text tool
- [x] Implement anchor_ingest_file tool
- [x] Add anchor_distill stub (placeholder)
- [x] Add anchor_illuminate stub (placeholder)
- [x] Create integration test suite (7 tests)
- [x] Write comprehensive README.md
- [x] Update workspace Cargo.toml

### Phase: Documentation Standards
- [x] Enforce doc_policy.md (README + CHANGELOG at root only)
- [x] Archive old backup docs to specs/archive/
- [x] Remove scattered IMPLEMENTATION_SUMMARY.md files
- [x] Centralize documentation in specs/ directory

### Phase 2: Pointer-Only Storage (Mirror Protocol) ✅
- [x] Create Storage trait for pointer-only filesystem
- [x] Implement FileSystemStorage with LRU cache
- [x] Update Atom model (source_path, start_byte, end_byte)
- [x] Update database schema (remove content, add pointers)
- [x] Update all SQL queries (insert, select, FTS)
- [x] Update service layer (ingest with storage)
- [x] Create storage module tests (4 tests)
- [x] Update database tests for pointer-only storage
- [x] Document migration in specs/phase2_database_migration.md
- [x] Create zero-copy storage standard (specs/standards/zero_copy_storage.md)

---

## 🎯 Current Focus

### Phase 3: Service Layer Integration ✅
- [x] Update main.rs to pass mirror_dir to AnchorService
- [x] Update api.rs with mirror_dir configuration
- [x] Update config module for mirror_dir setting
- [x] Integration testing (end-to-end pointer-only)
- [x] Create pointer_only_integration.rs test suite (7 tests)

### Phase 4: MCP Tools Completion ✅
- [x] Implement anchor_distill full functionality
- [x] Implement anchor_illuminate full functionality
- [x] Test MCP tools with pointer-only storage
- [x] Add pre-allocation with_capacity() for BFS (zero reallocations)
- [x] Add illuminate integration tests (3 tests)

### Phase 5: Benchmarking ✅
- [x] Create benchmark suite (criterion)
- [x] Measure ingestion throughput
- [x] Measure search latency (p50, p95, p99)
- [x] Compare memory usage vs Node.js version
- [x] Create profiling standard (specs/standards/profiling.md)
- [x] Add heaptrack/valgrind/perf documentation

---

## 📋 Backlog

### Short-Term (Q2 2026)
- [ ] Enhanced error reporting
- [ ] Docker containerization
- [ ] Incremental index updates
- [ ] Advanced query optimization

### Medium-Term (Q3 2026)
- [ ] Multi-threaded database access
- [ ] Native GUI application
- [ ] Plugin system
- [ ] Mobile application support

### Long-Term (Q4 2026)
- [ ] Production-hardened stability
- [ ] Comprehensive benchmarks
- [ ] Enterprise features
- [ ] Federation protocol

---

## Historical Phases (January - February 2026)

<details>
<summary><strong>Click to expand completed phases</strong></summary>

### Phase 1: Foundation Packages (Week 1) ✅
- [x] anchor-fingerprint implementation
- [x] anchor-atomizer implementation
- [x] anchor-keyextract implementation
- [x] anchor-tagwalker implementation
- [x] All 172 core tests passing

### Phase 2: Engine Core (Week 2) ✅
- [x] Database schema design
- [x] SQLite integration
- [x] CRUD operations
- [x] FTS5 indexing
- [x] Thread-safe connections

### Phase 3: Search + Retrieval (Week 3) ✅
- [x] Tag-Walker DB integration
- [x] Gravity scoring implementation
- [x] Radial inflation
- [x] Context assembly
- [x] Search API endpoint

### Phase 4: Inference + Agent (Week 4) ✅
- [x] HTTP API setup (axum)
- [x] OpenAI-compatible endpoint
- [x] Streaming support
- [x] CLI binary
- [x] Configuration management

### Phase 5: Testing + Documentation (Week 5-6) ✅
- [x] Integration tests
- [x] Performance benchmarks
- [x] Documentation complete
- [x] Deployment scripts
- [x] v0.1.0 release preparation

</details>

---

## Task Priority Legend

| Priority | Description | Timeline |
|----------|-------------|----------|
| **P0** | Critical path - blocks other work | Current sprint |
| **P1** | Important but can parallelize | Next 2-4 weeks |
| **P2** | Nice to have | Backlog |

---

## Definition of Done

Tasks are complete when:
- ✅ Implementation complete and tested
- ✅ Unit tests pass (≥90% coverage)
- ✅ Integration tests pass
- ✅ Benchmarks meet targets
- ✅ Documentation complete
- ✅ Code reviewed
- ✅ No clippy warnings
- ✅ `cargo fmt` applied

---

**Project Specs:** See `spec.md`, `plan.md`  
**API Docs:** See `../API_SUMMARY.md`  
**Status:** See `../STATUS.md`

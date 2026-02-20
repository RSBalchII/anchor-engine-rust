# Project Plan - Anchor OS v0 Rust Rewrite

## Project Overview

**Goal**: Rebuild Anchor OS from scratch in Rust, preserving all core innovations while improving performance and maintainability.

**Timeline**: 6 weeks (February - March 2026)

**Success Criteria**:
- ✅ All features from anchor-os implemented
- ✅ Performance meets or exceeds C++ benchmarks
- ✅ Full test coverage (≥90%)
- ✅ Self-hosting documentation
- ✅ One-command deployment

---

## Week-by-Week Breakdown

### Week 1: Foundation Packages (Feb 17-23)

**Goal**: Build and test core algorithm packages

| Day | Focus | Deliverables |
|-----|-------|--------------|
| Mon | anchor-fingerprint | SimHash, Hamming distance, tests, benchmarks |
| Tue | anchor-atomizer | Tokenizer, splitter, sanitizer |
| Wed | anchor-keyextract | TF-IDF, RAKE, synonym rings |
| Thu | anchor-tagwalker (graph) | Bipartite graph structure |
| Fri | anchor-tagwalker (traversal) | 70/30 budget, gravity scoring |
| Sat | Review + polish | Code review, benchmark validation |
| Sun | Buffer / catch-up | Documentation, edge cases |

**Milestone**: All 4 packages compile, pass tests, meet performance targets

---

### Week 2: Engine Core (Feb 24 - Mar 2)

**Goal**: Build database layer and ingestion pipeline

| Day | Focus | Deliverables |
|-----|-------|--------------|
| Mon | anchor-engine (schema) | PGlite setup, migrations |
| Tue | anchor-engine (CRUD) | Atom + Tag operations |
| Wed | anchor-engine (FTS) | Full-text search indexing |
| Thu | Ingestion pipeline | Watchdog, batch processor |
| Fri | Deduplication service | SimHash-based dedup |
| Sat | Integration testing | End-to-end ingestion flow |
| Sun | Buffer / catch-up | Performance tuning |

**Milestone**: Can ingest documents and query via FTS

---

### Week 3: Search + Retrieval (Mar 3-9)

**Goal**: Implement STAR algorithm and context assembly

| Day | Focus | Deliverables |
|-----|-------|--------------|
| Mon | Tag-Walker DB integration | SQL CTEs for graph traversal |
| Tue | Gravity scoring | Unified field equation |
| Wed | Radial inflation | Multi-hop walking |
| Thu | Context assembly | 70/30 budget allocation |
| Fri | Lazy loading | Context inflation from disk |
| Sat | Search API | Query endpoint |
| Sun | Buffer / catch-up | Optimization |

**Milestone**: Full STAR retrieval working with gravity scoring

---

### Week 4: Inference + Agent (Mar 10-16)

**Goal**: LLM integration and Telegram bot

| Day | Focus | Deliverables |
|-----|-------|--------------|
| Mon | anchor-inference setup | Model loading, inference |
| Tue | OpenAI-compatible API | Chat completions endpoint |
| Wed | Streaming support | SSE responses |
| Thu | nanobot-node setup | Telegram integration |
| Fri | Wake word + DM policy | User authentication |
| Sat | Conversation state | Context management |
| Sun | Buffer / catch-up | Testing |

**Milestone**: Can chat with Anchor via Telegram with memory context

---

### Week 5: UI + Orchestration (Mar 17-23)

**Goal**: Web interface and service orchestration

| Day | Focus | Deliverables |
|-----|-------|--------------|
| Mon | anchor-ui setup | React + Vite |
| Tue | Chat interface | Message history, streaming |
| Wed | Memory browser | Atom viewer, search |
| Thu | Settings panel | Configuration UI |
| Fri | Service orchestration | Multi-service startup |
| Sat | Logging + monitoring | Centralized logs |
| Sun | Buffer / catch-up | Polish |

**Milestone**: All services running, web UI functional

---

### Week 6: Testing + Documentation (Mar 24-30)

**Goal**: Final polish and release preparation

| Day | Focus | Deliverables |
|-----|-------|--------------|
| Mon | E2E tests | Full workflow testing |
| Tue | Performance benchmarks | All metrics validated |
| Wed | Documentation | README, API docs, guides |
| Thu | Deployment scripts | start.sh, start.bat |
| Fri | Bug fixes | Issue resolution |
| Sat | Release candidate | v0.1.0 prep |
| Sun | Launch! | v0.1.0 published |

**Milestone**: v0.1.0 release ready

---

## Risk Management

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| PGlite performance issues | Medium | High | Early benchmarking, fallback to SQLite |
| SIMD optimization complexity | High | Medium | Start without SIMD, add later |
| LLM integration delays | Medium | High | Use node-llama-cpp (proven), defer candle |
| Tag-Walker SQL complexity | High | Medium | Prototype early, iterate |

### Schedule Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Scope creep | High | High | Strict adherence to spec, defer P2 features |
| Burnout | Medium | High | 6-hour work days, buffer time built in |
| Dependency issues | Medium | Medium | Vendor critical deps, pin versions |

---

## Resource Requirements

### Development Environment

- Rust 1.75+ (stable)
- Node.js 22+ (for tooling)
- PGlite (bundled)
- Git
- VS Code + rust-analyzer

### Hardware

- CPU: 8+ cores (for parallel builds)
- RAM: 16GB+ (for LLM testing)
- Storage: 50GB+ (for models + test data)

### External Dependencies

| Dependency | Purpose | Alternative |
|------------|---------|-------------|
| murmur3 | Token hashing | xxhash |
| unicode-segmentation | Text splitting | regex |
| tf-idf | Keyword extraction | Custom impl |
| pglite | Database | SQLite + vectors |
| node-llama-cpp | Inference | candle-transformers |
| grammy | Telegram bot | teloxide |

---

## Quality Gates

### Code Quality

- ✅ `cargo clippy` - 0 warnings
- ✅ `cargo fmt` - Applied
- ✅ `cargo test` - All pass
- ✅ `cargo doc` - No warnings
- ✅ Test coverage ≥90%

### Performance Benchmarks

| Metric | Target | Measurement |
|--------|--------|-------------|
| SimHash generation | ≤2ms/atom | `criterion` benchmark |
| Hamming distance | ≥4M ops/sec | `criterion` benchmark |
| Tag-Walker p95 | ≤200ms | Integration test |
| Ingestion throughput | >100 atoms/sec | Load test |
| Search latency | ≤150ms p95 | Load test |

### Documentation Quality

- ✅ README.md - Quick start works
- ✅ API docs - All public items documented
- ✅ Examples - Each package has usage examples
- ✅ Architecture diagram - Included in README

---

## Release Checklist

### v0.1.0 (MVP)

- [ ] All P0 tasks complete
- [ ] Ingestion working
- [ ] Search working (STAR algorithm)
- [ ] Inference working (local LLM)
- [ ] Telegram bot working
- [ ] Web UI functional
- [ ] Documentation complete
- [ ] Tests passing
- [ ] Benchmarks meet targets

### v0.2.0 (Post-MVP)

- [ ] SIMD acceleration
- [ ] Multi-user support
- [ ] Advanced analytics
- [ ] Plugin system
- [ ] Mobile apps (Compose/SwiftUI)

---

## Communication Plan

### Daily

- Morning: Review tasks.md, update status
- Evening: Commit code, update session log

### Weekly

- Sunday: Weekly review, adjust plan if needed
- Monday: Start week with clear priorities

### Milestone Reviews

After each week's milestone:
1. Demo working features
2. Review benchmarks
3. Identify blockers
4. Adjust next week's plan

---

## Success Metrics

### Technical

- Binary size <50MB (excluding models)
- Memory usage <500MB idle
- Cold start <5 seconds
- Search p95 <200ms

### User Experience

- Onboarding <10 minutes
- First successful query <1 minute
- Zero configuration for defaults
- Clear error messages

### Code Quality

- 90%+ test coverage
- 0 clippy warnings
- All public APIs documented
- No TODO comments in released code

---

## Appendix: Original anchor-os Reference

The following files from anchor-os serve as reference (do not copy):

- `anchor-os/packages/anchor-engine/` - Database schema, search logic
- `anchor-os/packages/anchor-atomizer/` - Text decomposition (C++)
- `anchor-os/packages/native-fingerprint/` - SimHash (C++)
- `anchor-os/packages/physics-tag-walker/` - Tag-Walker implementation
- `anchor-os/packages/context-manager/` - Context assembly
- `anchor-os/specs/` - Architecture documentation

**Rule**: Understand the architecture, then implement from scratch in Rust.

---

## Document History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2026-02-17 | 0.1.0 | Qwen | Initial plan created |

# Implementation Tasks

## Phase 1: Foundation Packages (Week 1)

### Task 1.1: anchor-fingerprint ⏳
**Status**: Pending  
**Priority**: P0  
**Lines**: ~150  
**Complexity**: Low

**Deliverables**:
- [ ] `Cargo.toml` with murmur3 dependency
- [ ] `src/lib.rs` - Public API exports
- [ ] `src/simhash.rs` - SimHash implementation
- [ ] `src/distance.rs` - Hamming distance + similarity
- [ ] Unit tests (≥90% coverage)
- [ ] Benchmark suite
- [ ] README with usage examples

**API**:
```rust
pub fn simhash(text: &str) -> u64;
pub fn hamming_distance(a: u64, b: u64) -> u32;
pub fn similarity(a: u64, b: u64) -> f32;
pub fn hamming_weight(hash: u64) -> u32;
```

**Acceptance Criteria**:
- [ ] Identical texts produce identical hashes
- [ ] Similar texts have Hamming distance < 10
- [ ] Dissimilar texts have Hamming distance > 30
- [ ] Performance: ≥4M distance ops/sec
- [ ] Unicode support verified

---

### Task 1.2: anchor-atomizer ⏳
**Status**: Pending  
**Priority**: P0  
**Lines**: ~300  
**Complexity**: Medium

**Deliverables**:
- [ ] `Cargo.toml` with unicode-segmentation + regex
- [ ] `src/lib.rs` - Public API
- [ ] `src/tokenizer.rs` - Word tokenization
- [ ] `src/splitter.rs` - Compound → Molecule → Atom
- [ ] `src/sanitizer.rs` - Metadata stripping
- [ ] Unit tests
- [ ] README

**API**:
```rust
pub struct Atom {
    pub content: String,
    pub char_start: usize,
    pub char_end: usize,
}

pub struct Molecule {
    pub atoms: Vec<Atom>,
    pub metadata: Option<Value>,
}

pub fn atomize(text: &str) -> Vec<Atom>;
pub fn decompose_to_molecules(text: &str) -> Vec<Molecule>;
pub fn sanitize(text: &str) -> String;
```

**Acceptance Criteria**:
- [ ] Handles Unicode segmentation correctly
- [ ] Strips YAML frontmatter, log lines, code fences
- [ ] Preserves byte offsets for lazy loading
- [ ] Performance: >100 atoms/sec

---

### Task 1.3: anchor-keyextract ⏳
**Status**: Pending  
**Priority**: P1  
**Lines**: ~200  
**Complexity**: Medium

**Deliverables**:
- [ ] `Cargo.toml` with tf-idf + unicode-segmentation
- [ ] `src/lib.rs` - Public API
- [ ] `src/tfidf.rs` - TF-IDF scoring
- [ ] `src/rake.rs` - RAKE algorithm
- [ ] `src/synonym_ring.rs` - Synonym expansion
- [ ] Unit tests
- [ ] README

**API**:
```rust
pub struct Keyword {
    pub term: String,
    pub score: f32,
}

pub fn extract_keywords(text: &str, max_keywords: usize) -> Vec<Keyword>;
pub fn build_synonym_ring() -> HashMap<String, Vec<String>>;
pub fn expand_tag(tag: &str, ring: &HashMap<String, Vec<String>>) -> Vec<String>;
```

**Acceptance Criteria**:
- [ ] Extracts top 10 keywords with meaningful scores
- [ ] Synonym expansion returns related tags
- [ ] Handles multi-word terms

---

### Task 1.4: anchor-tagwalker ⏳
**Status**: Pending  
**Priority**: P0  
**Lines**: ~500  
**Complexity**: High

**Deliverables**:
- [ ] `Cargo.toml` with anchor-fingerprint dependency
- [ ] `src/lib.rs` - Public API
- [ ] `src/graph.rs` - Bipartite graph structure
- [ ] `src/traversal.rs` - Tag-Walker logic
- [ ] `src/budget.rs` - 70/30 budget allocation
- [ ] `src/gravity.rs` - Unified field equation
- [ ] Unit tests
- [ ] Integration tests
- [ ] README

**API**:
```rust
pub struct TagWalkerConfig {
    pub planet_budget: f32,
    pub moon_budget: f32,
    pub max_results: usize,
    pub max_hops: usize,
    pub temporal_decay: f32,
    pub damping: f32,
}

pub struct SearchResult {
    pub atom_id: u64,
    pub relevance: f32,
    pub path: Vec<String>,
}

pub struct TagWalker {
    // Bipartite graph: atoms ↔ tags
}

impl TagWalker {
    pub fn new() -> Self;
    pub fn add_atom(&mut self, id: u64, content: &str, tags: &[String]);
    pub fn search(&self, query: &str, config: TagWalkerConfig) -> Vec<SearchResult>;
    pub fn perform_radial_inflation(&self, anchors: &[u64], max_hops: usize, damping: f32) -> Vec<SearchResult>;
}
```

**Acceptance Criteria**:
- [ ] Planets discovered via FTS match query terms
- [ ] Moons discovered via tag graph traversal
- [ ] Gravity scoring matches equation exactly
- [ ] 70/30 budget enforced in results
- [ ] Radial inflation works with damping
- [ ] Performance: <200ms p95 for 100k atoms

---

## Phase 2: Core Engine (Week 2-3)

### Task 2.1: anchor-engine (Database Layer) ⏳
**Status**: Pending  
**Priority**: P0  
**Complexity**: High

**Deliverables**:
- [ ] PGlite integration
- [ ] Schema migrations
- [ ] Atom CRUD operations
- [ ] Tag index management
- [ ] FTS index setup
- [ ] Connection pooling

---

### Task 2.2: Ingestion Pipeline ⏳
**Status**: Pending  
**Priority**: P0  
**Complexity**: High

**Deliverables**:
- [ ] Filesystem watchdog (notify crate)
- [ ] Batch processor
- [ ] Deduplication service
- [ ] Error handling + retry logic
- [ ] Progress tracking

---

### Task 2.3: Search Service ⏳
**Status**: Pending  
**Priority**: P0  
**Complexity**: High

**Deliverables**:
- [ ] Query parser
- [ ] Tag-Walker integration
- [ ] Context assembly
- [ ] Result ranking
- [ ] Caching layer

---

## Phase 3: Inference + Agent (Week 4)

### Task 3.1: anchor-inference (LLM Layer) ⏳
**Status**: Pending  
**Priority**: P1  
**Complexity**: Medium

**Deliverables**:
- [ ] node-llama-cpp or candle integration
- [ ] OpenAI-compatible API
- [ ] Streaming support
- [ ] Model management

---

### Task 3.2: nanobot-node (Agent Service) ⏳
**Status**: Pending  
**Priority**: P1  
**Complexity**: Medium

**Deliverables**:
- [ ] Telegram bot integration (grammy crate)
- [ ] Wake word detection
- [ ] DM policy enforcement
- [ ] Command handlers
- [ ] Conversation state management

---

## Phase 4: UI + Integration (Week 5)

### Task 4.1: anchor-ui (Web Interface) ⏳
**Status**: Pending  
**Priority**: P2  
**Complexity**: Medium

**Deliverables**:
- [ ] React + Vite setup
- [ ] Chat interface
- [ ] Memory browser
- [ ] Settings panel
- [ ] Real-time status

---

### Task 4.2: System Integration ⏳
**Status**: Pending  
**Priority**: P0  
**Complexity**: High

**Deliverables**:
- [ ] Service orchestration
- [ ] Configuration management
- [ ] Logging + monitoring
- [ ] Health checks
- [ ] Startup scripts (Windows + Unix)

---

## Phase 5: Testing + Documentation (Week 6)

### Task 5.1: End-to-End Tests ⏳
**Status**: Pending  
**Priority**: P1  
**Complexity**: Medium

**Deliverables**:
- [ ] Ingestion → Search → Response workflow
- [ ] Multi-user scenarios
- [ ] Performance benchmarks
- [ ] Stress testing

---

### Task 5.2: Documentation ⏳
**Status**: Pending  
**Priority**: P1  
**Complexity**: Low

**Deliverables**:
- [ ] README.md (quick start, architecture)
- [ ] CHANGELOG.md
- [ ] API documentation (rustdoc)
- [ ] Deployment guide
- [ ] Troubleshooting guide

---

## Task Priority Legend

| Priority | Description | Timeline |
|----------|-------------|----------|
| P0 | Critical path - blocks other work | Week 1-2 |
| P1 | Important but can parallelize | Week 3-4 |
| P2 | Nice to have | Week 5+ |

---

## Definition of Done

Each task is complete when:
- ✅ All deliverables implemented
- ✅ Unit tests pass (≥90% coverage)
- ✅ Integration tests pass (if applicable)
- ✅ Benchmarks meet targets
- ✅ Documentation complete
- ✅ Code reviewed
- ✅ No clippy warnings
- ✅ `cargo fmt` applied

---

## Progress Tracking

Update this section after each work session:

### Session Log

| Date | Task | Status | Notes |
|------|------|--------|-------|
| 2026-02-17 | Task 1.1 (anchor-fingerprint) | Not Started | Awaiting go-ahead |
| 2026-02-17 | Documentation (spec.md) | ✅ Complete | System specification written |

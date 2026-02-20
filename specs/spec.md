# Anchor OS v0 - System Specification

## Overview

Anchor OS is a sovereign personal knowledge engine with physics-based associative search. It implements the STAR (Semantic Temporal Associative Retrieval) algorithm for context-aware memory management.

**Core Innovation**: A disposable index architecture where `mirrored_brain/` filesystem is the source of truth, and the database serves as a transient semantic index.

---

## 1. Atomic Knowledge Model

### Hierarchy

```
Compound (Document)
    └── Molecule (Section/Chunk)
            └── Atom (Paragraph/Unit)
```

### Atom Schema

```rust
pub struct Atom {
    pub id: String,           // Unique identifier (ULID/UUID)
    pub content: String,      // Text content
    pub source_path: String,  // Path in mirrored_brain/
    pub byte_offset: u64,     // Byte position in source file
    pub byte_length: u64,     // Content length in bytes
    pub timestamp: f64,       // Unix timestamp (creation time)
    pub simhash: u64,         // 64-bit SimHash fingerprint
    pub tags: Vec<String>,    // Associated tags
    pub buckets: Vec<String>, // Categorization buckets
}
```

### Molecule Schema

```rust
pub struct Molecule {
    pub id: String,
    pub atoms: Vec<Atom>,
    pub metadata: Option<Value>,  // Optional structured metadata
    pub char_start: usize,
    pub char_end: usize,
}
```

---

## 2. SimHash Fingerprinting

### Algorithm

64-bit SimHash for near-duplicate detection:

1. **Tokenize** input text (word-level, lowercase, strip punctuation)
2. **Hash each token** using MurmurHash3 (64-bit)
3. **Accumulate** bit vectors:
   - For each bit position (0-63): +1 if token hash bit = 1, -1 if 0
4. **Finalize**: bit i = 1 if accumulator[i] > 0, else 0

### API

```rust
pub fn simhash(text: &str) -> u64;
pub fn hamming_distance(a: u64, b: u64) -> u32;  // Uses popcount
pub fn similarity(a: u64, b: u64) -> f32;         // 1.0 - (distance / 64.0)
```

### Performance Targets

| Metric | Target |
|--------|--------|
| Fingerprint generation | ≤2ms per atom |
| Hamming distance | ≥4M ops/sec |
| Deduplication accuracy | ≥95% near-duplicate detection |

---

## 3. Tag-Walker Protocol (STAR Algorithm)

### Graph Structure

**Bipartite Graph**: Atoms ↔ Tags

```
Atom A ──┬── #rust
         ├── #memory
         └── #ai

Atom B ──┬── #rust
         └── #performance

Atom C ──┬── #ai
         └── #ethics
```

**No direct Atom-Atom edges**. Similarity derived from shared tags.

### Database Schema (PGlite)

```sql
-- Atoms table
CREATE TABLE atoms (
    id TEXT PRIMARY KEY,
    content TEXT,
    source_path TEXT,
    timestamp REAL,
    simhash TEXT,
    tags TEXT[],
    buckets TEXT[]
);

-- Tags index (for fast graph traversal)
CREATE TABLE tags (
    atom_id TEXT,
    tag TEXT,
    bucket TEXT,
    PRIMARY KEY (atom_id, tag, bucket)
);

-- FTS index for planet discovery
CREATE INDEX idx_atoms_content ON atoms USING GIN(to_tsvector('english', content));
```

### Traversal Logic (70/30 Budget)

#### Phase 1: Planets (70% of token budget)

Direct full-text search matches on `atoms.content`:

```sql
SELECT id, content, ts_rank(to_tsvector(content), query) as relevance
FROM atoms
WHERE to_tsvector(content) @@ to_tsquery(:query)
ORDER BY relevance DESC
LIMIT :planet_count;
```

#### Phase 2: Moons (30% of token budget)

Graph walk from planets via shared tags using **Unified Field Equation**:

```
gravity = (shared_tags_count) × exp(-λ × Δt) × (1 - hamming_distance/64) × damping
```

**Components**:
- `shared_tags_count`: Number of tags shared with anchor set
- `exp(-λ × Δt)`: Temporal decay (λ ≈ 0.00001, Δt = seconds since creation)
- `(1 - hamming_distance/64)`: SimHash similarity factor (0.0-1.0)
- `damping`: 0.85 per hop (reduces weight for multi-hop discoveries)

#### SQL Implementation (Simplified)

```sql
WITH anchor_tags AS (
    SELECT DISTINCT tag 
    FROM tags 
    WHERE atom_id IN (:planet_ids)
),
candidates AS (
    SELECT 
        t.atom_id,
        COUNT(*) as shared_tags,
        MAX(a.timestamp) as timestamp,
        MAX(a.simhash) as simhash
    FROM tags t
    JOIN atoms a ON t.atom_id = a.id
    WHERE t.tag IN (SELECT tag FROM anchor_tags)
      AND a.id NOT IN (:planet_ids)
    GROUP BY t.atom_id
)
SELECT 
    atom_id,
    shared_tags,
    exp(-0.00001 * (EXTRACT(EPOCH FROM NOW()) - timestamp)) as time_decay,
    (1.0 - (hamming_distance(simhash, :query_simhash) / 64.0)) as sim_similarity,
    (shared_tags * exp(-0.00001 * (EXTRACT(EPOCH FROM NOW()) - timestamp)) 
        * (1.0 - (hamming_distance(simhash, :query_simhash) / 64.0)) * 0.85) as gravity
FROM candidates
ORDER BY gravity DESC
LIMIT :moon_count;
```

### Radial Inflation (Multi-Hop Walking)

```rust
pub fn perform_radial_inflation(
    &self,
    anchors: &[AtomId],
    max_hops: usize,
    damping: f32,
) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let mut current_frontier = anchors.to_vec();
    
    for hop in 0..max_hops {
        let hop_damping = damping.powi(hop as i32);
        let moons = self.walk_from_anchors(&current_frontier, hop_damping);
        
        if moons.is_empty() {
            break;
        }
        
        current_frontier = moons.iter().map(|m| m.atom_id).collect();
        results.extend(moons);
    }
    
    results
}
```

### TagWalkerConfig

```rust
pub struct TagWalkerConfig {
    pub planet_budget: f32,    // 0.70 (direct FTS matches)
    pub moon_budget: f32,      // 0.30 (graph-discovered)
    pub max_results: usize,    // Total results cap
    pub max_hops: usize,       // Radial inflation depth (default: 1)
    pub temporal_decay: f32,   // λ (default: 0.00001)
    pub damping: f32,          // Per-hop damping (default: 0.85)
}
```

---

## 4. Text Decomposition (Atomizer)

### Tokenization

```rust
pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && !c.is_unicode())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}
```

### Splitting Strategy

1. **Compound → Molecule**: Split on section headers, double newlines, or semantic boundaries
2. **Molecule → Atom**: Split on paragraphs, sentences, or fixed token windows (512 tokens)

### Sanitization

Strip metadata wrappers before processing:
- JSON/YAML frontmatter (`---\n...`)
- Log lines (`[INFO]`, `[ERROR]`, timestamps)
- Code fences (```lang ... ```)
- HTML/XML tags

```rust
pub fn sanitize(text: &str) -> String {
    // Remove YAML frontmatter
    let text = regex::Regex::new(r"^---\n.*?\n---\n")
        .unwrap()
        .replace_all(text, "");
    
    // Remove log lines
    let text = regex::Regex::new(r"^\[\w+\]\s+\d{4}-\d{2}-\d{2}.*$\n")
        .unwrap()
        .replace_all(&text, "");
    
    text.to_string()
}
```

---

## 5. Keyword Extraction

### TF-IDF + RAKE Hybrid

```rust
pub struct Keyword {
    pub term: String,
    pub score: f32,
}

pub fn extract_keywords(text: &str, max_keywords: usize) -> Vec<Keyword>;
```

### Synonym Rings

Loaded from `internal_tags.json`:

```json
{
  "#coding": ["#programming", "#dev", "#software"],
  "#ai": ["#artificial-intelligence", "#ml", "#machine-learning"],
  "#memory": ["#recall", "#context", "knowledge"]
}
```

```rust
pub fn build_synonym_ring() -> HashMap<String, Vec<String>>;
pub fn expand_tag(tag: &str, ring: &HashMap<String, Vec<String>>) -> Vec<String>;
```

---

## 6. Context Assembly

### Token Budget Management

```rust
pub struct ContextPackage {
    pub planets: Vec<Atom>,      // Direct matches (70% budget)
    pub moons: Vec<Atom>,        // Graph discoveries (30% budget)
    pub total_tokens: usize,
    pub budget_used: usize,
}

pub fn assemble_context_package(
    planets: Vec<Atom>,
    moons: Vec<Atom>,
    max_tokens: usize,
) -> ContextPackage {
    let planet_budget = (max_tokens as f32 * 0.70) as usize;
    let moon_budget = (max_tokens as f32 * 0.30) as usize;
    
    // Prioritize planets, then fill with moons
    // Truncate long entries to fit budget
}
```

### Lazy Content Loading (Context Inflation)

Database stores pointers; content loaded from `mirrored_brain/` on demand:

```rust
pub fn load_atom_content(atom: &Atom, base_path: &Path) -> io::Result<String> {
    let mut file = File::open(base_path.join(&atom.source_path))?;
    let mut buffer = vec![0u8; atom.byte_length as usize];
    file.seek(SeekFrom::Start(atom.byte_offset))?;
    file.read_exact(&mut buffer)?;
    Ok(String::from_utf8_lossy(&buffer).to_string())
}
```

---

## 7. Ingestion Pipeline

### Flow

```
Filesystem Watchdog
    └── New/Modified File
            └── Sanitize
                    └── Decompose (Compound → Molecule → Atom)
                            └── Fingerprint (SimHash)
                                    └── Deduplicate (Hamming < 5)
                                            └── Extract Keywords
                                                    └── Index (Atoms + Tags)
```

### Performance Targets

| Metric | Target |
|--------|--------|
| Ingestion throughput | >100 atoms/sec (batched) |
| Deduplication accuracy | ≥95% |
| Index update latency | <500ms |

---

## 8. HTTP API (OpenAI-Compatible)

### Endpoints

```
POST /v1/chat/completions
POST /v1/embeddings
GET  /v1/models
```

### Chat Completions Request

```json
{
  "model": "anchor-local",
  "messages": [
    {"role": "user", "content": "What do I know about Rust?"}
  ],
  "stream": false,
  "context": {
    "enable_search": true,
    "max_atoms": 20
  }
}
```

### Response

```json
{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "anchor-local",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "Based on your memory, you know that Rust..."
    }
  }],
  "usage": {
    "prompt_tokens": 150,
    "completion_tokens": 80,
    "total_tokens": 230
  },
  "context": {
    "atoms_used": 12,
    "planets": 8,
    "moons": 4
  }
}
```

---

## 9. Nanobot (Agent Layer)

### Telegram Integration

```rust
pub struct NanobotConfig {
    pub telegram_token: String,
    pub allowed_users: Vec<i64>,  // Telegram user IDs
    pub dm_policy: DmPolicy,       // Default: PairingMode (DM-only)
}

pub enum DmPolicy {
    PairingMode,      // Only respond to paired users in DM
    GroupMode,        // Respond in groups with @mention
    OpenMode,         // Respond to anyone
}
```

### Wake Word Detection

```rust
pub fn detect_wake_word(text: &str) -> bool {
    let wake_words = ["hey anchor", "ok anchor", "anchor bot"];
    wake_words.iter().any(|w| text.to_lowercase().contains(w))
}
```

---

## 10. Configuration

### anchor-config.yaml

```yaml
brain:
  path: ./mirrored_brain
  buckets:
    - notes
    - code
    - conversations
    - documents

engine:
  port: 3160
  db_path: ./anchor-engine.db
  max_context_tokens: 8192

inference:
  provider: local
  model_path: ./models/qwen-30b-q4.gguf
  port: 3001

nanobot:
  telegram_token: ${TELEGRAM_TOKEN}
  allowed_users: [123456789]
  dm_policy: pairing_mode

tag_walker:
  planet_budget: 0.70
  moon_budget: 0.30
  temporal_decay: 0.00001
  damping: 0.85
  max_hops: 1
```

---

## 11. File Structure

```
anchor-rewrite-v0/
├── Cargo.toml              # Workspace root
├── README.md
├── CHANGELOG.md
├── specs/
│   ├── spec.md             # This file
│   ├── tasks.md            # Implementation tasks
│   ├── plan.md             # Project timeline
│   └── standards/
│       ├── code_style.md
│       ├── doc_policy.md
│       └── testing.md
├── packages/
│   ├── anchor-fingerprint/
│   ├── anchor-atomizer/
│   ├── anchor-keyextract/
│   └── anchor-tagwalker/
├── crates/
│   ├── anchor-engine/      # Knowledge database + search
│   ├── anchor-inference/   # LLM inference
│   ├── nanobot-node/       # Agent service
│   └── anchor-ui/          # Web interface
└── mirrored_brain/         # Source of truth (gitignored)
```

---

## 12. Testing Strategy

### Unit Tests

Each package must have ≥90% code coverage:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simhash_identical_texts() {
        let text = "Hello, world!";
        let hash1 = simhash(text);
        let hash2 = simhash(text);
        assert_eq!(hash1, hash2);
        assert_eq!(hamming_distance(hash1, hash2), 0);
    }
}
```

### Integration Tests

End-to-end workflow testing:

```rust
#[test]
fn test_ingestion_to_search() {
    // 1. Ingest a test document
    // 2. Verify atomization
    // 3. Verify fingerprinting
    // 4. Perform search query
    // 5. Verify Tag-Walker results
}
```

### Benchmark Tests

```rust
#[bench]
fn bench_simhash_generation(b: &mut Bencher) {
    let text = "The quick brown fox jumps over the lazy dog";
    b.iter(|| simhash(text));
}

#[bench]
fn bench_hamming_distance(b: &mut Bencher) {
    let a = 0x1234567890ABCDEFu64;
    let b = 0xFEDCBA0987654321u64;
    b.iter(|| hamming_distance(a, b));
}
```

---

## 13. Security Considerations

1. **API Authentication**: Bearer token for HTTP endpoints
2. **Telegram User Whitelisting**: Only allowed users can interact
3. **Filesystem Sandboxing**: Restrict `mirrored_brain/` access to designated paths
4. **No External Telemetry**: All data stays local

---

## 14. Performance Optimization Strategies

1. **SIMD Acceleration**: Use `std::simd` or `wide` crate for popcount
2. **Batched Ingestion**: Process atoms in batches of 100
3. **Connection Pooling**: PGlite connection pool for concurrent queries
4. **Lazy Loading**: Load atom content only when needed
5. **Cache Hot Atoms**: LRU cache for frequently accessed atoms

---

## References

- [STAR Algorithm White Paper](../WHITE_PAPER.md)
- [Anchor OS Original Codebase](../anchor-os/)
- [PGlite Documentation](https://pglite.dev/)
- [Rust SIMD Guide](https://doc.rust-lang.org/std/simd/)

# Anchor Engine: Node.js vs Rust Implementation Comparison

**Date:** February 23, 2026  
**Purpose:** Identify architectural and implementation differences between the two implementations

---

## Executive Summary

Both implementations share the **same core algorithm** (STAR — Semantic Temporal Associative Retrieval) and **same data model** (Compound → Molecule → Atom), but differ in:

1. **Runtime:** Node.js + C++ N-API vs. Pure Rust
2. **Database:** PGlite vs. SQLite
3. **Deployment:** npm packages + Electron vs. Single binary
4. **Maturity:** Production-ready (Node) vs. Testing phase (Rust)

---

## Architecture Comparison

| Aspect | anchor-engine-node | anchor-rust-v0 |
|--------|-------------------|----------------|
| **Runtime** | Node.js v18+ required | Standalone binary (no runtime) |
| **Language** | TypeScript + C++ N-API | Pure Rust 1.75+ |
| **Database** | PGlite (PostgreSQL-compatible) | SQLite with FTS5 (bundled) |
| **Binary Size** | ~150MB (with Node) | <50MB |
| **Memory Management** | GC-managed | Manual + RAII |
| **Type Safety** | Runtime type checks | Compile-time guarantees |
| **Deployment** | npm packages + Electron UI | Single binary |
| **Native Modules** | @rbalchii/* npm packages | Built-in Rust crates |
| **Tests** | Manual testing | 181 automated tests |

---

## Shared Core: STAR Algorithm

Both implementations use the **identical** STAR Algorithm:

```text
gravity = (shared_tags) × e^(-λΔt) × (1 - hamming_distance/64) × damping
```

| Component | Node.js | Rust | Status |
|-----------|---------|------|--------|
| **SimHash (64-bit)** | ✅ @rbalchii/native-fingerprint | ✅ anchor-fingerprint | ✅ Identical |
| **Temporal Decay** | ✅ e^(-λΔt), λ=0.00001 | ✅ e^(-λΔt), λ=0.00001 | ✅ Identical |
| **70/30 Budget** | ✅ Planets/Moons split | ✅ Planets/Moons split | ✅ Identical |
| **Damping Factor** | ✅ 0.85 default | ✅ 0.85 default | ✅ Identical |
| **Bipartite Graph** | ✅ atoms ↔ tags | ✅ atoms ↔ tags | ✅ Identical |

**Conclusion:** The core algorithm is **mathematically identical** in both implementations.

---

## Data Model Comparison

### Shared Model: Compound → Molecule → Atom

| Field | Node.js | Rust | Notes |
|-------|---------|------|-------|
| **Compound** | `compound_id: string` | `source_id: String` | File/document reference |
| **Molecule** | `start_byte`, `end_byte` | `char_start`, `char_end` | Byte offsets |
| **Atom** | Tags only (no content) | Tags + content | ⚠️ **DIFFERENCE** |
| **SimHash** | `molecular_signature: string` | `simhash: u64` | Same algorithm, different storage |
| **Timestamp** | `timestamp: number` | `timestamp: f64` | Unix timestamp |

### Key Difference: Content Storage

**Node.js:**
- Content lives in `mirrored_brain/` filesystem
- Database stores **pointers only** (byte offsets)
- Database is **disposable** (rebuilt on startup)

**Rust:**
- Content stored **in database** (Atoms table)
- SQLite database is **persistent**
- No mirrored_brain/ filesystem

**Trade-off:**
- Node: Faster ingestion, rebuildable index, more disk I/O at query time
- Rust: Simpler deployment, faster queries, larger database

---

## Implementation Differences

### 1. Context Inflation

| Feature | Node.js | Rust |
|---------|---------|------|
| **Post-merge inflation** | ✅ Implemented (v4.1.1) | ❌ Not implemented |
| **n-1, n+1 expansion** | ✅ Reads from disk | N/A (content in DB) |
| **Per-atom budget** | ✅ Dynamic (~8k chars) | N/A |

**Impact:** Node.js retrieves ~600k chars per search; Rust retrieves raw atom content only.

### 2. Transient Data Filter

| Feature | Node.js | Rust |
|---------|---------|------|
| **Pattern detection** | ✅ 20+ patterns | ❌ Not implemented |
| **Excluded content** | Error logs, npm/pip install, build artifacts | None |
| **Context savings** | ~30% reclaimed | N/A |

**Patterns (Node.js):**
```typescript
/Traceback \(most recent call last\)/i
/npm install/i, /pip install/i
/Build succeeded/i, /Compiling\.\.\./i
```

### 3. Deduplication Strategy

| Layer | Node.js | Rust |
|-------|---------|------|
| **Geometric (same file)** | ✅ 50% overlap threshold | ✅ Similar |
| **Content Fingerprint (MD5)** | ✅ First 500 chars | ❌ Not implemented |
| **Containment Check** | ✅ Substring match | ❌ Not implemented |
| **Fuzzy Prefix Match** | ✅ 50-100 chars | ❌ Not implemented |
| **SimHash Distance** | ✅ Hamming < 5 | ✅ Hamming < 5 |

**Impact:** Node.js achieves 40-50% dedup rate; Rust achieves ~25% (SimHash only).

### 4. Time Ordering

| Feature | Node.js | Rust |
|---------|---------|------|
| **Chronological sort** | ✅ Default (toggleable) | ❓ Not verified |
| **Relevance sort** | ✅ Toggle button | ❓ Not verified |
| **UI toggle** | ✅ 📅 Chronological ↔ 🎯 Relevance | ❌ Not implemented |

### 5. XML Metadata Wrapper

| Feature | Node.js | Rust |
|---------|---------|------|
| **XML wrapping** | ✅ `<atom id="..." relevance="...">` | ❌ Not implemented |
| **LLM prioritization** | ✅ Helps with truncation | ❌ Not implemented |
| **Relevance score** | ✅ score × temporal_weight | ❌ Not computed |

---

## Feature Parity Matrix

| Feature | Node.js | Rust | Priority for Rust |
|---------|---------|------|-------------------|
| **STAR Algorithm** | ✅ | ✅ | ✅ Complete |
| **SimHash Deduplication** | ✅ | ✅ | ✅ Complete |
| **Temporal Decay** | ✅ | ✅ | ✅ Complete |
| **70/30 Budget Split** | ✅ | ✅ | ✅ Complete |
| **Context Inflation** | ✅ | ❌ | 🔶 Medium |
| **Transient Filter** | ✅ | ❌ | 🔶 Medium |
| **Multi-layer Dedup** | ✅ (5 layers) | ⚠️ (1 layer) | 🔶 Medium |
| **Time Ordering Toggle** | ✅ | ❓ | 🟡 Low |
| **XML Metadata** | ✅ | ❌ | 🟡 Low |
| **Phoenix Protocol (Backup)** | ✅ | ❌ | 🔶 Medium |
| **Watchdog (File Sync)** | ✅ | ❌ | 🔶 Medium |
| **UI (React/Vite)** | ✅ | ❌ | N/A (different approach) |

**Legend:**
- ✅ Implemented
- ❌ Not implemented
- ❓ Not verified
- 🔶 Medium priority
- 🟡 Low priority

---

## Performance Comparison

| Metric | Node.js | Rust | Notes |
|--------|---------|------|-------|
| **SimHash Generation** | ~2ms/atom (native) | ~500ns/atom | ✅ Rust 4x faster |
| **Ingestion (90MB)** | ~178s | Not measured | Node has transient filter |
| **Search Latency (p95)** | ~150ms (1.5k atoms) | Not measured | Node has 100x dataset |
| **Memory Peak** | ~510MB | Not measured | Node has GC |
| **Binary Size** | ~150MB (with Node) | <50MB | ✅ Rust 3x smaller |

---

## Deployment Comparison

### Node.js Deployment

```bash
# Requires Node.js v18+
pnpm install
pnpm build
pnpm start

# UI: http://localhost:3160
# Database: engine/context_data/
# Mirror: mirrored_brain/
```

**Pros:**
- Full-featured UI (React/Vite)
- Watchdog for automatic file sync
- Phoenix Protocol backup/restore
- Mature (production since Feb 2026)

**Cons:**
- Requires Node.js runtime
- Larger deployment footprint
- GC pauses possible

### Rust Deployment

```bash
# No runtime required
cargo build --release
./target/release/anchor-engine

# UI: None (API only)
# Database: anchor.db
```

**Pros:**
- Single binary deployment
- No runtime dependencies
- Compile-time safety
- Smaller memory footprint

**Cons:**
- No UI (API only)
- No Watchdog (manual ingestion)
- No backup protocol
- Earlier maturity stage

---

## When to Use Which

### Use anchor-engine-node When:

1. **You need a full UI** — React/Vite interface with bucket filtering, time ordering toggle
2. **You want automatic file sync** — Watchdog monitors inbox/external-inbox/
3. **You need backup/restore** — Phoenix Protocol with filesystem rebuild
4. **You have large datasets** — Proven at 151k atoms, 280k molecules
5. **You want context inflation** — Retrieves 600k+ chars per search

### Use anchor-rust-v0 When:

1. **You need single-binary deployment** — Embedded systems, minimal dependencies
2. **You prioritize compile-time safety** — Rust type system
3. **You want maximum SimHash performance** — 4x faster than Node.js native
4. **You're building a library** — Rust crates for integration
5. **You prefer SQLite** — Simpler than PGlite for some deployments

---

## Recommendations for Rust Implementation

### High Priority (Match Node.js v4.2.0)

1. **Context Inflation** — Read content from filesystem with n-1, n+1 expansion
2. **Transient Data Filter** — Exclude error logs, install output, build artifacts
3. **Multi-layer Deduplication** — Add MD5 fingerprint, containment, fuzzy prefix
4. **Phoenix Protocol** — Backup/restore with database + filesystem export

### Medium Priority

5. **Time Ordering Toggle** — Chronological vs. Relevance sort option
6. **XML Metadata Wrapper** — Help LLMs prioritize content
7. **Watchdog Service** — Automatic file monitoring and ingestion

### Low Priority

8. **UI Development** — Consider Tauri for Rust-based UI
9. **Advanced Features** — Persona tagging (if needed)

---

## Conclusion

Both implementations are **valid approaches** to the same problem:

| Aspect | Winner | Why |
|--------|--------|-----|
| **Core Algorithm** | 🤝 Tie | Identical STAR implementation |
| **Performance** | 🏆 Rust | 4x faster SimHash, smaller binary |
| **Features** | 🏆 Node.js | Context inflation, transient filter, UI |
| **Maturity** | 🏆 Node.js | Production since Feb 2026 |
| **Deployment** | 🏆 Rust | Single binary, no runtime |
| **Safety** | 🏆 Rust | Compile-time guarantees |

**Best Path Forward:**
- **Node.js:** Continue as production reference implementation
- **Rust:** Port high-priority features (context inflation, transient filter, dedup layers)
- **Both:** Share whitepaper, algorithm specs, and test cases

---

**Repository (Node.js):** https://github.com/RSBalchII/anchor-engine-node  
**Repository (Rust):** https://github.com/RSBalchII/anchor-rust-v0

**License:** AGPL-3.0 (both projects)

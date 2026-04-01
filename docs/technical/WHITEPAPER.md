# The Sovereign Context Protocol

**Note:** This is a reference file. The authoritative whitepaper is maintained in the Node.js project.

## Read the Whitepaper

📄 **[The Sovereign Context Protocol: Decoupling Intelligence from Infrastructure via Data Atomization and Cross-Platform Sharding](https://github.com/RSBalchII/anchor-engine-node/blob/main/docs/whitepaper.md)**

## Abstract

The Anchor Engine implements a "Browser Paradigm" for AI memory systems. Just as web browsers download only the shards needed for the current view, Anchor loads only the atoms required for the current thought—enabling 4GB RAM laptops to navigate 10TB datasets.

## Key Concepts

### 1. Browser Paradigm
- **Old Way (Vector Monoliths):** Load entire HNSW index into RAM
- **Anchor Way (Sharded Atomization):** Load only relevant graph nodes

### 2. Data Model: Compound → Molecule → Atom
- **Compound:** File/document reference
- **Molecule:** Semantic chunk with byte offsets
- **Atom:** Tag/concept (content lives in filesystem)

### 3. STAR Search Algorithm
```
Gravity = (SharedTags) × e^(-λΔt) × (1 - SimHashDistance/64)

70% Planets: Direct FTS matches
30% Moons: Graph-discovered associations
```

### 4. Architecture
- **Node.js Version:** Production-ready, full-featured, Electron UI
- **Rust Version:** Lightweight, single-binary, embedded deployment

## Production Benchmarks (Node.js)

| Metric | Result | Target | Status |
|--------|--------|--------|--------|
| **90MB Ingestion** | ~178s | <200s | ✅ |
| **Memory Peak** | <1GB | <1GB | ✅ |
| **Search Latency (p95)** | ~150ms | <200ms | ✅ |
| **SimHash Speed** | ~2ms/atom | <5ms | ✅ |

## Citation

```bibtex
@techreport{balch2026sovereign,
  title={The Sovereign Context Protocol: Decoupling Intelligence from Infrastructure via Data Atomization and Cross-Platform Sharding},
  author={Balch II, R.S.},
  year={2026},
  institution={Anchor Project},
  url={https://github.com/RSBalchII/anchor-engine-node/blob/main/docs/whitepaper.md}
}
```

---

**License:** AGPL-3.0  
**Repository:** https://github.com/RSBalchII/anchor-engine-node  
**Rust Implementation:** https://github.com/RSBalchII/anchor-rust-v0

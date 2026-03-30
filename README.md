# Anchor Engine (Rust)

**Version:** 0.2.0 | **Status:** 🚀 MCP Server Implementation | **License:** AGPL-3.0

A sovereign personal knowledge engine with physics-based associative search. Rebuilt from scratch in Rust for performance, safety, and single-binary deployment.

Implements the **STAR Algorithm** (Semantic Temporal Associative Retrieval) — the same algorithm as [anchor-engine-node](https://github.com/RSBalchII/anchor-engine-node), but in pure Rust.

**Optimized for:** 9.8mW edge deployment (no runtime, <50MB binary, deterministic memory)

---

## 🚀 Quick Start

### Build

```bash
cargo build --release
```

### Run HTTP Server

```bash
cargo run --release -- --port 3160 --db-path ./anchor.db
```

### Run MCP Server (for AI agents)

```bash
cargo run --release --package anchor-mcp -- --db-path ./anchor.db
```

### Ingest Content

```bash
# Via HTTP
curl -X POST http://localhost:3160/v1/memory/ingest \
  -H "Content-Type: application/json" \
  -d '{"source": "test.md", "content": "Rust is a systems programming language"}'

# Via MCP (stdio)
echo '{"jsonrpc":"2.0","id":1,"method":"anchor_ingest_text","params":{"content":"Rust is safe","filename":"test.md","bucket":"default"}}' \
  | ./target/release/anchor-mcp --db-path ./anchor.db
```

### Search

```bash
# Via HTTP
curl -X POST http://localhost:3160/v1/memory/search \
  -H "Content-Type: application/json" \
  -d '{"query": "#rust", "max_results": 10}'

# Via MCP (stdio)
echo '{"jsonrpc":"2.0","id":1,"method":"anchor_query","params":{"query":"#rust","max_results":10}}' \
  | ./target/release/anchor-mcp --db-path ./anchor.db
```

---

## 📖 Documentation

| Document | Location | Description |
|----------|----------|-------------|
| **Quick Start** | This file | Build and run instructions |
| **Specification** | [specs/spec.md](specs/spec.md) | System architecture & STAR algorithm |
| **Tasks** | [specs/tasks.md](specs/tasks.md) | Implementation progress |
| **Standards** | [specs/standards/](specs/standards/) | Code style, testing, documentation |
| **MCP Server** | [specs/standards/mcp_server.md](specs/standards/mcp_server.md) | MCP protocol specification |
| **Changelog** | [CHANGELOG.md](CHANGELOG.md) | Version history |

**Note:** Per [doc_policy.md](specs/standards/doc_policy.md), all documentation is centralized in `specs/`. Package-specific READMEs exist in `crates/`.

---

## ✨ Features

- 🧠 **Atomic Knowledge Model:** Documents → Molecules → Atoms
- 🔍 **STAR Algorithm:** Physics-based search (gravity, temporal decay, SimHash)
- ⏰ **Temporal Decay:** Recent memories weighted higher (7.9 year half-life)
- 🎯 **SimHash Deduplication:** Near-duplicate detection in ~500ns
- 🔒 **Privacy-First:** All data stays local
- 🚀 **Single Binary:** No Node.js, no external dependencies (<50MB)
- 🤖 **MCP Server:** AI agent integration via JSON-RPC 2.0 over stdio

---

## 🏗️ Architecture

See [specs/spec.md](specs/spec.md) for complete architecture.

```
┌─────────────┐     ┌──────────────┐     ┌─────────────────┐
│ AI Agent    │────▶│ anchor-mcp   │────▶│ anchor-engine   │
│ (Claude)    │stdio│ (JSON-RPC)   │calls│ (SQLite + STAR) │
└─────────────┘     └──────────────┘     └─────────────────┘
```

---

## 📦 Crates

| Crate | Description | Status |
|-------|-------------|--------|
| `anchor-fingerprint` | 64-bit SimHash with Hamming distance | ✅ Complete (52 tests) |
| `anchor-atomizer` | Text decomposition | ✅ Complete (50 tests) |
| `anchor-keyextract` | TF-IDF + RAKE + Synonym rings | ✅ Complete (42 tests) |
| `anchor-tagwalker` | STAR algorithm implementation | ✅ Complete (28 tests) |
| `anchor-engine` | Core engine (SQLite + service layer) | ✅ Complete |
| `anchor-mcp` | MCP server for AI agents | 🚀 Implemented (v0.2.0) |
| `anchor-ui` | Terminal UI (Ratatui) | ⏳ Planned |

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run MCP server tests
cargo test --package anchor-mcp

# Run with coverage
cargo tarpaulin --out Html
```

**Test Status:** 181 tests passing (172 core + 9 engine)

---

## 📊 Performance

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| SimHash generation | ≤2ms/atom | ~500ns | ✅ |
| Search latency (p95) | <200ms | ~50-150ms | ✅ |
| Memory (idle) | <100MB | ~30MB | ✅ |
| Binary size | <50MB | ~15MB | ✅ |

See [specs/spec.md](specs/spec.md) for detailed benchmarks.

---

## 🔧 Installation

### From Source

```bash
git clone https://github.com/RSBalchII/anchor-engine-rust.git
cd anchor-engine-rust
cargo build --release
```

### Add as Dependency

```toml
[dependencies]
anchor-engine = { git = "https://github.com/RSBalchII/anchor-engine-rust.git" }
anchor-fingerprint = { git = "https://github.com/RSBalchII/anchor-engine-rust.git" }
```

---

## 🤝 Contributing

1. Read [specs/standards/code_style.md](specs/standards/code_style.md)
2. Check [specs/tasks.md](specs/tasks.md) for open tasks
3. Follow [specs/standards/doc_policy.md](specs/standards/doc_policy.md) for documentation
4. Submit PR with tests and changelog entry

---

## 📄 License

AGPL-3.0 - See [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

- Original Anchor Engine: [anchor-engine-node](https://github.com/RSBalchII/anchor-engine-node)
- STAR Algorithm Whitepaper: [docs/WHITEPAPER.md](docs/WHITEPAPER.md)

```
HTTP API (axum)
    │

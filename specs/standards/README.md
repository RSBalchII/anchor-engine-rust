# Architecture Standards Index

**Last Updated:** February 23, 2026 | **Total Standards:** 3

---

## Active Standards (Current Implementation)

These standards define the current Rust implementation:

### Core Standards

| # | Standard | Description | Status |
|---|----------|-------------|--------|
| **Code Style** | [code_style.md](code_style.md) | Rust formatting, naming, structure | ✅ ACTIVE |
| **Documentation** | [doc_policy.md](doc_policy.md) | Documentation-driven development | ✅ ACTIVE |
| **Testing** | [testing.md](testing.md) | Unit tests, integration tests, doc tests | ✅ ACTIVE |

---

## Alignment with anchor-engine-node

This project aligns with **anchor-engine-node** documentation structure:

| Aspect | anchor-engine-node | anchor-rust-v0 |
|--------|-------------------|----------------|
| **Standards Index** | specs/standards/README.md | specs/standards/README.md |
| **Code Style** | ESLint + Prettier | rustfmt + clippy |
| **Documentation** | JSDoc + Markdown | Rust doc comments + Markdown |
| **Testing** | Manual + Integration | 181 automated tests |

**Shared Standards:**
- STAR Algorithm specification (identical)
- Data model (Compound → Molecule → Atom)
- API endpoint structure (OpenAI-compatible)
- Keep a Changelog format

**Different Standards:**
- Language-specific formatting (rustfmt vs. Prettier)
- Database (SQLite vs. PGlite)
- Deployment (binary vs. npm packages)

---

## Standards by Domain

### CODE Domain (Implementation)

- **code_style.md** — Rust formatting, naming conventions, module structure
- **testing.md** — Test organization, coverage requirements, doc tests

### DOC Domain (Documentation)

- **doc_policy.md** — Documentation structure, writing guidelines, API docs

---

## Writing Guidelines

### Tone

- **Direct**: "The function returns..." not "The function should return..."
- **Active voice**: "The system validates..." not "Validation is performed..."
- **Confident**: State requirements, don't hedge

### Formatting

```markdown
# Use ATX headings (# not ##)

**Bold** for emphasis, not italics

`inline code` for:
  - Function names
  - File paths
  - Configuration keys

```rust
// Code blocks with language
pub fn example() -> Result<()> {
    Ok(())
}
```

| Tables | For | Comparisons |
|--------|-----|-------------|

- Bullet lists for non-sequential items
- Numbered lists for steps
```

### Cross-Referencing

```markdown
See [STAR Algorithm](../spec.md#star-search-algorithm)
Refer to [Task 1.4](../tasks.md#task-14)
As defined in [code_style.md](code_style.md)
```

---

## Review Checklist

Before merging any PR:

- [ ] spec.md updated (if architecture changed)
- [ ] tasks.md updated (if task status changed)
- [ ] CHANGELOG.md entry added
- [ ] All public APIs documented
- [ ] Code examples in docs compile
- [ ] No TODO comments in released code
- [ ] Cross-references valid

---

## Tools

### Automated Checks

```bash
# Formatting check
cargo fmt -- --check

# Linting
cargo clippy -- -D warnings

# Documentation tests
cargo test --doc

# Check for undocumented items
cargo doc --document-private-items
```

### Recommended VS Code Extensions

- rust-analyzer (required)
- rustfmt
- markdownlint

---

## Anti-Patterns

### ❌ Don't Do This

```markdown
# README.md (root)
[Full architecture essay]
[Installation instructions duplicated]
[Tutorial content]
```

```markdown
# Scattered docs
project/
├── docs/
├── README.md
├── INSTALL.md
├── ARCHITECTURE.md
└── TODO.md
```

### ✅ Do This

```markdown
# README.md (root)
[Quick start only]
[Link to specs/ for details]
```

```markdown
# Centralized docs
anchor-rust-v0/
├── docs/
│   ├── WHITEPAPER.md
│   └── COMPARISON_WITH_NODE.md
└── specs/
    ├── spec.md
    ├── tasks.md
    ├── plan.md
    └── standards/
```

---

## References

- [Keep a Changelog](https://keepachangelog.com/)
- [Rust API Documentation Guidelines](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)
- [anchor-engine-node Documentation](https://github.com/RSBalchII/anchor-engine-node/tree/main/docs)

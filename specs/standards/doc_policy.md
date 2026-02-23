# Documentation Policy

## Philosophy

Documentation is **executable specification**. It should be:

1. **Discoverable**: Find it in `specs/`, not scattered
2. **Actionable**: Tells you exactly what to do
3. **Living**: Updated with every code change
4. **Minimal**: No duplication, single source of truth
5. **Aligned**: Follows anchor-engine-node documentation structure where applicable

---

## Directory Structure

```
anchor-rust-v0/
├── README.md              # Quick start + overview (links to specs/)
├── CHANGELOG.md           # Version history (Keep a Changelog format)
├── Cargo.toml             # Workspace configuration
├── docs/
│   ├── WHITEPAPER.md      # References anchor-engine-node whitepaper
│   └── COMPARISON_WITH_NODE.md  # Architecture comparison
├── specs/
│   ├── spec.md            # System specification (THE truth)
│   ├── tasks.md           # Implementation tasks + status
│   ├── plan.md            # Project timeline + milestones
│   └── standards/
│       ├── README.md      # Standards index
│       ├── code_style.md  # Coding standards
│       ├── doc_policy.md  # This file
│       └── testing.md     # Testing standards
```

**Root directory** contains only:
- `README.md` - Quick start + overview
- `CHANGELOG.md` - User-facing changes
- `Cargo.toml` - Workspace configuration
- `start.sh` / `start.bat` - Launch scripts

**No other docs in root.**

### Alignment with anchor-engine-node

This project follows the documentation structure of **anchor-engine-node** where applicable:

| Document Type | anchor-engine-node | anchor-rust-v0 | Status |
|---------------|-------------------|----------------|--------|
| **Whitepaper** | docs/whitepaper.md | docs/WHITEPAPER.md (reference) | ✅ Aligned |
| **Spec** | specs/spec.md | specs/spec.md | ✅ Aligned |
| **Changelog** | CHANGELOG.md | CHANGELOG.md | ✅ Aligned (Keep a Changelog) |
| **Standards Index** | specs/standards/README.md | specs/standards/README.md | ✅ Aligned |
| **Standards** | specs/standards/*.md | specs/standards/*.md | ✅ Aligned |
| **Comparison** | N/A | docs/COMPARISON_WITH_NODE.md | 🆕 Rust-specific |

**Key Differences:**
- Rust version references Node.js whitepaper (authoritative source)
- Rust version includes comparison document (architecture differences)
- Both use identical STAR Algorithm specification

---

## Document Types

### 1. Specification (spec.md)

**Purpose**: Authoritative system description

**Audience**: Future implementers (including future you)

**Content**:
- Architecture diagrams
- Data models (structs, schemas)
- Algorithm descriptions (with equations)
- API contracts
- Performance targets

**Style**:
- Precise, technical language
- Code blocks for all APIs
- Equations in LaTeX format
- Tables for comparisons

**Update Rule**: Change spec BEFORE changing code

---

### 2. Tasks (tasks.md)

**Purpose**: Work tracking

**Audience**: Current implementer

**Content**:
- Task breakdown with acceptance criteria
- Status indicators (⏳, ✅, ❌)
- Session log
- Definition of Done

**Style**:
- Checkbox format
- Clear ownership
- Time estimates

**Update Rule**: Update AFTER each work session

---

### 3. Plan (plan.md)

**Purpose**: Project timeline + risk management

**Audience**: Project manager (you)

**Content**:
- Week-by-week breakdown
- Milestones
- Risk register
- Resource requirements
- Success metrics

**Style**:
- Tables for schedules
- Bullet points for risks
- Measurable metrics

**Update Rule**: Review weekly, adjust as needed

---

### 4. Standards (standards/*.md)

**Purpose**: Enforce consistency

**Audience**: All contributors

**Content**:
- Code style rules
- Documentation templates
- Testing requirements
- Review checklists

**Style**:
- Prescriptive ("must", "must not")
- Examples of correct/incorrect
- Automated checks where possible

**Update Rule**: Change when pain point discovered

---

### 5. Changelog (CHANGELOG.md)

**Purpose**: User-facing change log

**Audience**: End users, deployers

**Content**:
- New features
- Breaking changes
- Bug fixes
- Performance improvements

**Style**:
- [Keep a Changelog](https://keepachangelog.com/) format
- Semantic versioning
- Links to PRs/issues

**Update Rule**: Update WITH each PR

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
See [Tag-Walker Protocol](spec.md#tag-walker-protocol)
Refer to [Task 1.4](tasks.md#task-14-anchor-tagwalker)
As defined in [standards/code_style.md](code_style.md)
```

---

## Documentation-Driven Development

### Workflow

1. **Before coding**: Read spec.md, understand requirements
2. **During planning**: Update tasks.md with new tasks
3. **Before implementation**: Write doc comments in code
4. **After implementation**: Update CHANGELOG.md
5. **Before commit**: Verify docs match code

### Commit Message Format

```
<type>(<scope>): <subject>

[body - references spec section]

Fixes #123
See spec.md#tag-walker-protocol
```

**Types**: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

**Examples**:
```
feat(tagwalker): implement gravity scoring

As specified in spec.md#traversal-logic-7030-budget

Fixes #45
```

```
docs(spec): update atom schema

Add byte_offset and byte_length fields for lazy loading
```

---

## API Documentation Standards

### Public Items Must Be Documented

```rust
/// Compute the 64-bit SimHash fingerprint of a text string.
///
/// This function tokenizes the input text and computes a 64-bit fingerprint
/// using the SimHash algorithm. Similar texts will produce similar fingerprints
/// (small Hamming distance).
///
/// # Arguments
///
/// * `text` - The input text to fingerprint
///
/// # Returns
///
/// A 64-bit fingerprint as a `u64`
///
/// # Example
///
/// ```rust
/// let fingerprint = anchor_fingerprint::simhash("Hello, world!");
/// println!("Fingerprint: {:016x}", fingerprint);
/// ```
pub fn simhash(text: &str) -> u64;
```

### Required Sections

1. **One-line summary**: What it does
2. **Description**: How it works (if non-obvious)
3. **Arguments**: Each parameter
4. **Returns**: Return value
5. **Errors**: Error conditions (if Result)
6. **Panics**: Panic conditions (if applicable)
7. **Example**: Usage example
8. **See Also**: Related functions (optional)

---

## README Template

```markdown
# Package Name

One-sentence description.

## Quick Start

```rust
use package_name;

fn main() {
    // Minimal working example
}
```

## Features

- Feature 1
- Feature 2
- Feature 3

## API

### `function_name()`

Description.

```rust
pub fn function_name(args) -> ReturnType;
```

**Example**:
```rust
// Usage example
```

## Installation

```toml
[dependencies]
package_name = "0.1.0"
```

## License

AGPL-3.0
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
# Documentation tests
cargo test --doc

# Check for undocumented items
cargo doc --document-private-items

# Validate code examples
cargo +nightly rustdoc --all-features
```

### Recommended VS Code Extensions

- rust-analyzer (required)
- markdownlint
- Docs Writer

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
# In code comments
// This function does stuff
// TODO: document what this actually does
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
# In doc comments
/// Validates user authentication token.
/// See spec.md#authentication for protocol details.
```

```markdown
# Centralized docs
specs/
├── spec.md
├── tasks.md
├── plan.md
└── standards/
```

---

## Enforcement

**CI Checks** (future):
- `cargo doc` succeeds
- `cargo test --doc` passes
- All public items have doc comments
- No broken cross-references

**Manual Review**:
- PR reviewer verifies documentation completeness
- Spec changes reviewed by architecture owner

---

## Versioning

Documents are versioned with the code:

| Document | Version Strategy |
|----------|------------------|
| spec.md | Updated every release |
| tasks.md | Living document (no version) |
| plan.md | Updated weekly |
| CHANGELOG.md | Per-release |
| standards/* | Updated as needed |

---

## References

- [Keep a Changelog](https://keepachangelog.com/)
- [Rust API Documentation Guidelines](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)
- [Diátaxis Technical Documentation Framework](https://diataxis.fr/)

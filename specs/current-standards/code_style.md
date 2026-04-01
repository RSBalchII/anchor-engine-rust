# Code Style Guide

## Philosophy

**Consistency over preference.** Code should look like it was written by one person.

---

## Rust-Specific Rules

### 1. Formatting

**Tool**: `rustfmt` (default settings)

**Enforcement**:
```bash
cargo fmt -- --check  # CI check
cargo fmt             # Auto-fix
```

**Key Settings** (`.rustfmt.toml`):
```toml
max_width = 100
tab_spaces = 4
newline_style = "Unix"
reorder_imports = true
reorder_modules = true
```

---

### 2. Naming Conventions

| Item | Convention | Example |
|------|------------|---------|
| Crates | `kebab-case` | `anchor-fingerprint` |
| Modules | `snake_case` | `mod tag_walker;` |
| Types | `PascalCase` | `pub struct SearchResult;` |
| Functions | `snake_case` | `pub fn extract_keywords();` |
| Methods | `snake_case` | `impl Walker { pub fn search() }` |
| Variables | `snake_case` | `let atom_count = 0;` |
| Constants | `SCREAMING_SNAKE` | `const MAX_TOKENS: usize;` |
| Lifetimes | Lowercase letter | `fn get<'a>(&'a self)` |
| Traits | `PascalCase` | `pub trait Indexable;` |

**Exceptions**:
- Acronyms in types: `SimHash` (not `Simhash`)
- Test modules: `mod tests;`

---

### 3. File Organization

**Module Structure**:
```rust
// lib.rs
pub mod simhash;
pub mod distance;

pub use simhash::simhash;
pub use distance::hamming_distance;

#[cfg(test)]
mod tests;
```

**File Contents Order**:
1. Module-level doc comment
2. Imports (`std`, external crates, parent/sibling modules)
3. Constants/types
4. Public functions
5. Private functions
6. Tests

**Example**:
```rust
//! SimHash fingerprinting algorithm.
//!
//! See spec.md#simhash-fingerprinting for details.

use murmur3::murmur3_64;

use crate::tokenizer::tokenize;

const HASH_SEED: u64 = 0;

/// Compute the 64-bit SimHash fingerprint.
pub fn simhash(text: &str) -> u64 {
    // Implementation
}

/// Internal helper function.
fn accumulate_bits(hash: u64, accumulator: &mut [i32; 64]) {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simhash_basic() {
        // Test code
    }
}
```

---

### 4. Import Organization

**Order**:
1. `std` / `core` / `alloc`
2. External crates
3. Parent modules (`super::`)
4. Sibling modules
5. Current module (`self::`)

**Example**:
```rust
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};

use murmur3::murmur3_64;
use regex::Regex;

use super::tokenizer::tokenize;
use crate::atom::Atom;
use crate::utils::sanitize;

use self::constants::MAX_TOKENS;
```

**Grouping**: Separate groups with blank lines.

**Alphabetical**: Within each group, sort alphabetically.

---

### 5. Error Handling

**Use `Result` for recoverable errors**:
```rust
pub fn load_atom(path: &Path) -> io::Result<Atom> {
    let mut file = File::open(path)?;
    // ...
    Ok(atom)
}
```

**Use `Option` for absence**:
```rust
pub fn find_atom_by_id(&self, id: &str) -> Option<&Atom> {
    self.atoms.iter().find(|a| a.id == id)
}
```

**Use `panic!` only for bugs**:
```rust
pub fn get_atom(&self, index: usize) -> &Atom {
    &self.atoms[index]  // Will panic if out of bounds - that's OK
}
```

**Custom Error Types**:
```rust
#[derive(Debug)]
pub enum AnchorError {
    IoError(io::Error),
    ParseError(String),
    NotFound(String),
}

impl From<io::Error> for AnchorError {
    fn from(err: io::Error) -> Self {
        AnchorError::IoError(err)
    }
}

pub type Result<T> = std::result::Result<T, AnchorError>;
```

---

### 6. Documentation Comments

**Public items MUST have doc comments**:
```rust
/// Compute the 64-bit SimHash fingerprint of a text string.
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
/// let hash = simhash("Hello, world!");
/// ```
pub fn simhash(text: &str) -> u64;
```

**Module-level docs**:
```rust
//! SimHash fingerprinting module.
//!
//! This module implements the SimHash algorithm for near-duplicate
//! detection. See spec.md#simhash-fingerprinting for the full specification.
```

**Inner doc comments** (`//!` and `/*! */`):
- For modules, crates
- At the top of files

**Outer doc comments** (`///` and `/** */`):
- For items (functions, structs, etc.)
- Before the item

---

### 7. Testing Standards

**Test Module Structure**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = "test";
        
        // Act
        let result = function(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

**Test Naming**:
```rust
#[test]
fn test_simhash_identical_texts_produce_same_hash() {}

#[test]
fn test_hamming_distance_symmetric() {}

#[test]
fn test_similarity_bounds_0_to_1() {}
```

**Test Organization**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod simhash {
        use super::*;

        #[test]
        fn test_empty_text() {}

        #[test]
        fn test_single_token() {}
    }

    mod distance {
        use super::*;

        #[test]
        fn test_identical_hashes() {}

        #[test]
        fn test_opposite_hashes() {}
    }
}
```

**Property-Based Testing** (when applicable):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;

    quickcheck! {
        fn simhash_deterministic(text: String) -> bool {
            let hash1 = simhash(&text);
            let hash2 = simhash(&text);
            hash1 == hash2
        }
    }
}
```

---

### 8. Performance-Critical Code

**Use `#[inline]` for small functions**:
```rust
#[inline]
pub fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}
```

**Use iterators over loops**:
```rust
// ✅ Good
let sum: i32 = values.iter().map(|v| v * 2).sum();

// ❌ Avoid
let mut sum = 0;
for v in values {
    sum += v * 2;
}
```

**Avoid unnecessary allocations**:
```rust
// ✅ Good: borrows
pub fn tokenize<'a>(text: &'a str) -> impl Iterator<Item = &'a str> {
    text.split(char::is_whitespace)
}

// ❌ Avoid: allocates
pub fn tokenize(text: &str) -> Vec<String> {
    text.split(char::is_whitespace)
        .map(|s| s.to_string())
        .collect()
}
```

**Benchmark critical paths**:
```rust
#[cfg(test)]
mod benches {
    use test::Bencher;

    #[bench]
    fn bench_simhash_100_chars(b: &mut Bencher) {
        let text = "x".repeat(100);
        b.iter(|| simhash(&text));
    }
}
```

---

### 9. Type System Usage

**Newtype Pattern** for type safety:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AtomId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TagId(pub u64);

// Prevents mixing up IDs
pub fn find_atom(id: AtomId) -> Option<Atom> { /* ... */ }
pub fn find_tag(id: TagId) -> Option<Tag> { /* ... */ }
```

**Builder Pattern** for complex construction:
```rust
pub struct AtomBuilder {
    id: Option<String>,
    content: String,
    tags: Vec<String>,
}

impl AtomBuilder {
    pub fn new(content: String) -> Self {
        Self { id: None, content, tags: Vec::new() }
    }

    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn build(self) -> Result<Atom> {
        Ok(Atom {
            id: self.id.ok_or("ID required")?,
            content: self.content,
            tags: self.tags,
        })
    }
}
```

---

### 10. Concurrency

**Use `Send + Sync` bounds explicitly**:
```rust
pub trait Indexable: Send + Sync {
    fn index(&self) -> u64;
}
```

**Prefer message passing over shared state**:
```rust
use std::sync::mpsc;

let (tx, rx) = mpsc::channel();

// Producer
tx.send(atom).unwrap();

// Consumer
for atom in rx {
    process(atom);
}
```

**Use `Arc` for shared ownership**:
```rust
use std::sync::Arc;

let shared_data: Arc<Mutex<HashMap<String, Atom>>> = Arc::new(Mutex::new(HashMap::new()));
```

---

## Anti-Patterns

### ❌ Don't Do This

```rust
// Unnecessary Box
fn create_atom() -> Box<Atom> {
    Box::new(Atom { /* ... */ })
}

// Should be:
fn create_atom() -> Atom {
    Atom { /* ... */ }
}
```

```rust
// Overuse of clone
fn process_atoms(atoms: Vec<Atom>) {
    for atom in atoms.clone() {  // ❌
        process(atom);
    }
}

// Should be:
fn process_atoms(atoms: &[Atom]) {
    for atom in atoms {
        process(atom);
    }
}
```

```rust
// Magic numbers
if score > 0.7 {  // ❌
    // ...
}

// Should be:
const RELEVANCE_THRESHOLD: f32 = 0.7;
if score > RELEVANCE_THRESHOLD {
    // ...
}
```

```rust
// TODO comments in released code
// TODO: fix this later  // ❌

// Should be:
// Create an issue and reference it
// See: https://github.com/org/repo/issues/123
```

---

## Enforcement

### Pre-commit Hooks

```bash
#!/bin/bash
# .git/hooks/pre-commit

cargo fmt -- --check || exit 1
cargo clippy -- -D warnings || exit 1
cargo test --all-features || exit 1
```

### CI Checks

```yaml
# .github/workflows/ci.yml
- name: Check formatting
  run: cargo fmt -- --check

- name: Lint
  run: cargo clippy -- -D warnings

- name: Test
  run: cargo test --all-features
```

### Editor Configuration

**VS Code** (`.vscode/settings.json`):
```json
{
  "editor.formatOnSave": true,
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.cargo.allFeatures": true
}
```

---

## Review Checklist

Before submitting code:

- [ ] `cargo fmt` applied
- [ ] `cargo clippy` passes with no warnings
- [ ] All public APIs documented
- [ ] Tests added/updated
- [ ] No TODO comments
- [ ] No magic numbers (constants defined)
- [ ] Error types are descriptive
- [ ] Import order correct
- [ ] Test names are descriptive

---

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust By Example](https://doc.rust-lang.org/rust-by-example/)
- [Effective Rust](https://www.lurklurk.org/effective-rust/)

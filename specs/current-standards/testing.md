# Testing Standards

## Philosophy

**Tests are production code.** They must be:

1. **Correct**: Test the right thing
2. **Complete**: Cover all cases
3. **Clear**: Obvious what's being tested
4. **Fast**: Run in milliseconds
5. **Isolated**: No inter-test dependencies

---

## Test Pyramid

```
        /\
       /  \      E2E Tests (10%)
      /----\     - Full workflow
     /      \    - External integrations
    /--------\
   /          \   Integration Tests (30%)
  /------------\  - Module interactions
 /              \ - API contracts
/----------------\
|  Unit Tests    |  Unit Tests (60%)
|  (Majority)    |  - Pure functions
|                |  - Edge cases
------------------
```

---

## Unit Tests

### Structure (AAA Pattern)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange: Set up inputs
        let input = "test data";
        let expected = 42;

        // Act: Call the function
        let result = function_under_test(input);

        // Assert: Verify the output
        assert_eq!(result, expected);
    }
}
```

### Naming Convention

```rust
#[test]
fn test_<unit>_<scenario>_<expected_result>() {
    // ...
}

// Examples:
#[test]
fn test_simhash_identical_texts_produce_same_hash() {}

#[test]
fn test_hamming_distance_empty_input_returns_zero() {}

#[test]
fn test_search_no_matches_returns_empty_vec() {}
```

### Coverage Requirements

**Target**: ≥90% line coverage

**Critical Paths** (100% required):
- SimHash algorithm
- Hamming distance calculation
- Gravity scoring equation
- Budget allocation
- Sanitization logic

**Example**:
```rust
#[cfg(test)]
mod simhash_tests {
    use super::*;

    #[test]
    fn test_empty_text() {
        let hash = simhash("");
        assert_eq!(hash, 0);
    }

    #[test]
    fn test_single_character() {
        let hash = simhash("a");
        assert_ne!(hash, 0);
    }

    #[test]
    fn test_unicode_text() {
        let text = "你好世界";
        let hash = simhash(text);
        assert_ne!(hash, 0);
    }

    #[test]
    fn test_identical_texts() {
        let text = "Hello, world!";
        assert_eq!(simhash(text), simhash(text));
    }

    #[test]
    fn test_similar_texts_have_small_distance() {
        let text1 = "The quick brown fox";
        let text2 = "The quick brown fox jumps";
        let dist = hamming_distance(simhash(text1), simhash(text2));
        assert!(dist < 10);
    }

    #[test]
    fn test_different_texts_have_large_distance() {
        let text1 = "The quick brown fox";
        let text2 = "Completely unrelated text";
        let dist = hamming_distance(simhash(text1), simhash(text2));
        assert!(dist > 20);
    }
}
```

---

## Integration Tests

### Purpose

Test module interactions and API contracts.

### Structure

```rust
// tests/integration_test.rs

use anchor_fingerprint::{simhash, hamming_distance};
use anchor_atomizer::{atomize, sanitize};

#[test]
fn test_ingestion_pipeline() {
    // 1. Sanitize input
    let raw = r#"---
title: Test
---
Hello, world!"#;
    let clean = sanitize(raw);
    assert!(!clean.contains("---"));

    // 2. Atomize
    let atoms = atomize(&clean);
    assert!(!atoms.is_empty());

    // 3. Fingerprint
    for atom in &atoms {
        let hash = simhash(&atom.content);
        assert_ne!(hash, 0);
    }

    // 4. Verify deduplication
    let hash1 = simhash(&atoms[0].content);
    let hash2 = simhash(&atoms[0].content);
    assert_eq!(hamming_distance(hash1, hash2), 0);
}
```

### API Contract Tests

```rust
#[test]
fn test_tagwalker_search_api() {
    let mut walker = TagWalker::new();
    
    // Add test data
    walker.add_atom(1, "Rust is great", &["rust", "programming"]);
    walker.add_atom(2, "Python is also great", &["python", "programming"]);
    
    // Search
    let config = TagWalkerConfig::default();
    let results = walker.search("rust", config);
    
    // Verify contract
    assert!(!results.is_empty());
    assert!(results[0].atom_id == 1);
    assert!(results[0].relevance > 0.0);
}
```

---

## Property-Based Tests

### Use QuickCheck for Invariants

```rust
#[cfg(test)]
mod props {
    use super::*;
    use quickcheck::quickcheck;

    quickcheck! {
        fn simhash_is_deterministic(text: String) -> bool {
            simhash(&text) == simhash(&text)
        }

        fn hamming_distance_is_symmetric(a: u64, b: u64) -> bool {
            hamming_distance(a, b) == hamming_distance(b, a)
        }

        fn similarity_is_bounded(a: u64, b: u64) -> bool {
            let sim = similarity(a, b);
            sim >= 0.0 && sim <= 1.0
        }

        fn similarity_identical_is_one(text: String) -> bool {
            let hash = simhash(&text);
            (similarity(hash, hash) - 1.0).abs() < f32::EPSILON
        }
    }
}
```

---

## Benchmark Tests

### Structure

```rust
#![feature(test)]
extern crate test;

#[cfg(test)]
mod benches {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_simhash_100_chars(b: &mut Bencher) {
        let text = "x".repeat(100);
        b.iter(|| simhash(&text));
    }

    #[bench]
    fn bench_simhash_1000_chars(b: &mut Bencher) {
        let text = "x".repeat(1000);
        b.iter(|| simhash(&text));
    }

    #[bench]
    fn bench_hamming_distance(b: &mut Bencher) {
        let a = 0x1234567890ABCDEFu64;
        let b = 0xFEDCBA0987654321u64;
        b.iter(|| hamming_distance(a, b));
    }

    #[bench]
    fn bench_tagwalker_search_1000_atoms(b: &mut Bencher) {
        let mut walker = TagWalker::new();
        for i in 0..1000 {
            walker.add_atom(
                i,
                &format!("Atom {} with some content", i),
                &["tag1", "tag2"],
            );
        }
        let config = TagWalkerConfig::default();
        b.iter(|| walker.search("query", config));
    }
}
```

### Performance Targets

| Test | Target | Measurement |
|------|--------|-------------|
| `bench_simhash_100_chars` | ≤1ms | `cargo bench` |
| `bench_simhash_1000_chars` | ≤2ms | `cargo bench` |
| `bench_hamming_distance` | ≥4M ops/sec | `cargo bench` |
| `bench_tagwalker_search_1000_atoms` | ≤50ms | `cargo bench` |

---

## End-to-End Tests

### Full Workflow

```rust
// tests/e2e_test.rs

use std::fs;
use std::path::PathBuf;
use anchor_engine::{Engine, Config};
use anchor_atomizer::atomize;
use anchor_fingerprint::simhash;

#[test]
fn test_full_ingestion_to_search() {
    // Setup: Create temp directory
    let temp_dir = tempfile::tempdir().unwrap();
    let config = Config {
        brain_path: temp_dir.path().to_path_buf(),
        db_path: temp_dir.path().join("test.db"),
        ..Default::default()
    };

    // 1. Initialize engine
    let mut engine = Engine::new(config).unwrap();

    // 2. Create test document
    let doc_path = temp_dir.path().join("test.md");
    fs::write(&doc_path, "# Test\n\nThis is test content about Rust programming.").unwrap();

    // 3. Ingest
    engine.ingest_file(&doc_path).unwrap();

    // 4. Search
    let results = engine.search("rust programming", 10).unwrap();

    // 5. Verify
    assert!(!results.is_empty());
    assert!(results[0].relevance > 0.5);

    // Cleanup
    temp_dir.close().unwrap();
}
```

---

## Mocking and Fixtures

### Test Fixtures

```rust
// tests/fixtures/mod.rs

use anchor_atomizer::Atom;

pub fn sample_atom() -> Atom {
    Atom {
        id: "test-1".to_string(),
        content: "This is test content".to_string(),
        char_start: 0,
        char_end: 20,
    }
}

pub fn sample_atoms(count: usize) -> Vec<Atom> {
    (0..count)
        .map(|i| Atom {
            id: format!("test-{}", i),
            content: format!("Content for atom {}", i),
            char_start: 0,
            char_end: 20,
        })
        .collect()
}

pub fn sample_tags() -> Vec<String> {
    vec!["rust".to_string(), "programming".to_string()]
}
```

### Mocking Traits

```rust
// For testing with mock implementations
pub trait Storage: Send + Sync {
    fn get_atom(&self, id: &str) -> Option<Atom>;
    fn save_atom(&mut self, atom: &Atom) -> Result<()>;
}

// Production implementation
pub struct PgStorage { /* ... */ }
impl Storage for PgStorage { /* ... */ }

// Mock for tests
#[cfg(test)]
pub struct MockStorage {
    pub atoms: HashMap<String, Atom>,
}

#[cfg(test)]
impl Storage for MockStorage {
    fn get_atom(&self, id: &str) -> Option<Atom> {
        self.atoms.get(id).cloned()
    }

    fn save_atom(&mut self, atom: &Atom) -> Result<()> {
        self.atoms.insert(atom.id.clone(), atom.clone());
        Ok(())
    }
}
```

---

## Test Data Management

### Test Files

```
tests/
├── fixtures/
│   ├── sample_document.md
│   ├── yaml_frontmatter.md
│   ├── code_fences.md
│   └── log_file.txt
├── integration_test.rs
└── e2e_test.rs
```

**Example Fixture** (`sample_document.md`):
```markdown
---
title: Test Document
tags: [rust, test]
---

# Introduction

This is a test document for the Anchor OS ingestion pipeline.

## Section 1

Content about Rust programming and memory management.

## Section 2

More content with keywords like search, retrieval, and STAR algorithm.
```

---

## Coverage Requirements

### Minimum Thresholds

| Component | Line Coverage | Branch Coverage |
|-----------|---------------|-----------------|
| Core algorithms | 100% | 100% |
| Public APIs | ≥95% | ≥90% |
| Internal modules | ≥90% | ≥85% |
| Tests | N/A | N/A |

### Measuring Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/tarpaulin-report.html
```

### CI Enforcement

```yaml
# .github/workflows/coverage.yml
- name: Check coverage
  run: |
    cargo tarpaulin --out Xml
    # Fail if coverage < 90%
    cargo tarpaulin --out Stdout | grep -q "total coverage: [9][0-9]\|100"
```

---

## Continuous Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Lint
        run: cargo clippy -- -D warnings

      - name: Run tests
        run: cargo test --all-features --verbose

      - name: Run benchmarks (compile check only)
        run: cargo bench --no-run

      - name: Check documentation
        run: cargo doc --all-features --no-deps
```

---

## Test Organization

### Module Structure

```rust
// src/lib.rs
pub mod simhash;
pub mod distance;

#[cfg(test)]
mod tests {
    mod simhash;
    mod distance;
    mod integration;
}
```

```rust
// src/tests/simhash.rs
use super::*;

#[test]
fn test_empty_text() {}

#[test]
fn test_unicode() {}
```

```rust
// src/tests/integration.rs
use super::*;

#[test]
fn test_full_pipeline() {}
```

---

## Common Patterns

### Testing Error Cases

```rust
#[test]
fn test_invalid_input_returns_error() {
    let result = parse_atom("");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Empty input");
}

#[test]
fn test_malformed_json() {
    let result = parse_json("{ invalid json");
    assert!(result.is_err());
}
```

### Testing Panics

```rust
#[test]
#[should_panic(expected = "Index out of bounds")]
fn test_out_of_bounds_access() {
    let atoms = vec![sample_atom()];
    atoms[5];  // Should panic
}
```

### Testing with Time

```rust
#[test]
fn test_temporal_decay() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();

    let old_timestamp = now - 86400.0;  // 1 day ago
    let decay = exp_decay(old_timestamp, now, 0.00001);

    assert!(decay < 1.0);
    assert!(decay > 0.0);
}
```

---

## Anti-Patterns

### ❌ Don't Do This

```rust
// Testing implementation details
#[test]
fn test_loop_runs_correct_times() {  // ❌
    // Don't test how, test what
}

// Should be:
#[test]
fn test_output_is_correct() {  // ✅
    // Test the result
}
```

```rust
// Flaky tests with timing
#[test]
fn test_something_fast() {  // ❌
    thread::sleep(Duration::from_millis(10));
    // Timing-dependent tests are flaky
}
```

```rust
// Tests with side effects
#[test]
fn test_writes_to_file() {  // ❌
    // Don't modify filesystem in tests
}

// Should be:
#[test]
fn test_with_mock_storage() {  // ✅
    // Use MockStorage instead
}
```

```rust
// Ignoring test failures
#[test]
#[ignore]  // ❌
fn test_something_broken() {}

// Should be:
// Fix the test or create an issue
```

---

## Review Checklist

Before merging:

- [ ] All new code has tests
- [ ] Unit tests cover edge cases
- [ ] Integration tests verify API contracts
- [ ] Benchmarks meet performance targets
- [ ] No flaky tests
- [ ] Test names are descriptive
- [ ] Fixtures are reusable
- [ ] Coverage meets thresholds
- [ ] CI passes

---

## References

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust By Example - Testing](https://doc.rust-lang.org/rust-by-example/testing.html)
- [Google Testing Blog](https://testing.googleblog.com/)

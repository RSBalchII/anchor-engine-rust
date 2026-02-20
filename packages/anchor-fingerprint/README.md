# anchor-fingerprint

> 64-bit SimHash for text deduplication with Hamming distance calculations

[![Crates.io](https://img.shields.io/crates/v/anchor-fingerprint.svg)](https://crates.io/crates/anchor-fingerprint)
[![Documentation](https://docs.rs/anchor-fingerprint/badge.svg)](https://docs.rs/anchor-fingerprint)
[![License](https://img.shields.io/crates/l/anchor-fingerprint)](https://github.com/your-org/anchor-rewrite-v0/blob/main/LICENSE)

## Features

- **64-bit SimHash**: Near-duplicate detection for text
- **Hamming Distance**: Efficient bit-difference counting (uses POPCNT instruction)
- **Similarity Scoring**: 0.0 to 1.0 similarity metric
- **Unicode Support**: Handles international text correctly
- **Zero Dependencies**: Only uses `murmur3` for token hashing
- **Optional SIMD**: Enable the `simd` feature for potential acceleration

## Performance

| Operation | Target | Measured |
|-----------|--------|----------|
| SimHash (50 chars) | ≤1ms | See `cargo bench` |
| SimHash (500 chars) | ≤2ms | See `cargo bench` |
| Hamming Distance | ≥4M ops/sec | See `cargo bench` |

## Quick Start

```rust
use anchor_fingerprint::{simhash, hamming_distance, similarity};

// Fingerprint some text
let hash1 = simhash("The quick brown fox jumps over the lazy dog");
let hash2 = simhash("The quick brown fox leaps over the lazy dog");

// Compute distance (number of differing bits)
let dist = hamming_distance(hash1, hash2);
println!("Hamming distance: {}", dist);

// Compute similarity (0.0 to 1.0)
let sim = similarity(hash1, hash2);
println!("Similarity: {:.2}", sim);
```

## API

### `simhash(text: &str) -> u64`

Compute the 64-bit SimHash fingerprint of a text string.

```rust
let hash = simhash("Hello, world!");
println!("Fingerprint: {:016x}", hash);
```

### `hamming_distance(a: u64, b: u64) -> u32`

Compute the number of differing bits between two fingerprints.

```rust
let hash1 = simhash("Hello");
let hash2 = simhash("Hello");
assert_eq!(hamming_distance(hash1, hash2), 0);
```

### `similarity(a: u64, b: u64) -> f32`

Compute similarity score between two fingerprints (0.0 to 1.0).

```rust
let hash1 = simhash("Hello");
let hash2 = simhash("Hello");
assert_eq!(similarity(hash1, hash2), 1.0);
```

### `tokenize(text: &str) -> Vec<String>`

Tokenize text into lowercase words (public for custom pipelines).

```rust
let tokens = tokenize("Hello, World! 123");
assert_eq!(tokens, vec!["hello", "world", "123"]);
```

### `simhash_with_tokens(tokens: &[String]) -> u64`

Compute SimHash from pre-tokenized words.

```rust
let tokens = vec!["hello".to_string(), "world".to_string()];
let hash = simhash_with_tokens(&tokens);
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
anchor-fingerprint = "0.1.0"
```

Or install via cargo:

```bash
cargo add anchor-fingerprint
```

## Usage Examples

### Deduplication

```rust
use anchor_fingerprint::{simhash, hamming_distance};

fn is_duplicate(existing: &[u64], new_text: &str, threshold: u32) -> bool {
    let new_hash = simhash(new_text);
    existing.iter().any(|&hash| hamming_distance(hash, new_hash) < threshold)
}

// Usage
let existing_hashes = vec![/* ... */];
if is_duplicate(&existing_hashes, "New document text", 5) {
    println!("This is a near-duplicate!");
}
```

### Similarity Search

```rust
use anchor_fingerprint::{simhash, similarity};

fn find_most_similar(query: &str, candidates: &[(&str, u64)]) -> Option<&str> {
    let query_hash = simhash(query);
    candidates
        .iter()
        .max_by(|a, b| {
            let sim_a = similarity(query_hash, a.1);
            let sim_b = similarity(query_hash, b.1);
            sim_a.partial_cmp(&sim_b).unwrap()
        })
        .map(|(text, _)| *text)
}
```

### Batch Processing

```rust
use anchor_fingerprint::simhash;

fn fingerprint_documents(docs: &[&str]) -> Vec<u64> {
    docs.iter().map(|&doc| simhash(doc)).collect()
}
```

## Algorithm

The SimHash algorithm works as follows:

1. **Tokenize**: Split input into lowercase words
2. **Hash tokens**: Compute 64-bit MurmurHash3 for each token
3. **Accumulate**: For each bit position (0-63), add +1 if token hash bit is 1, -1 if 0
4. **Finalize**: Bit i in fingerprint is 1 if accumulator[i] > 0, else 0

This produces a fingerprint where similar texts have similar bit patterns (small Hamming distance).

## Benchmarks

Run benchmarks with:

```bash
cargo bench
```

Sample output:

```
simhash_short_50_chars    time:   [500 ns 520 ns 540 ns]
simhash_medium_500_chars  time:   [1.5 µs 1.6 µs 1.7 µs]
simhash_long_2000_chars   time:   [5.0 µs 5.2 µs 5.4 µs]
hamming_distance          time:   [0.3 ns 0.4 ns 0.5 ns]  (~2B ops/sec)
```

## Testing

```bash
cargo test --all-features
```

Coverage report:

```bash
cargo tarpaulin --out Html
```

## License

AGPL-3.0 - See [LICENSE](../../LICENSE) for details.

## Contributing

1. Read the [specification](https://github.com/your-org/anchor-rewrite-v0/blob/main/specs/spec.md)
2. Follow [code style](https://github.com/your-org/anchor-rewrite-v0/blob/main/specs/standards/code_style.md)
3. Write tests per [testing standards](https://github.com/your-org/anchor-rewrite-v0/blob/main/specs/standards/testing.md)
4. Submit a PR

## Acknowledgments

- SimHash algorithm: Moses Charikar (1997)
- MurmurHash3: Austin Appleby

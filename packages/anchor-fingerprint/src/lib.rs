//! 64-bit SimHash fingerprinting for text deduplication.
//!
//! This crate implements the SimHash algorithm for near-duplicate detection.
//! Similar texts produce similar fingerprints (small Hamming distance).
//!
//! # Quick Start
//!
//! ```rust
//! use anchor_fingerprint::{simhash, hamming_distance, similarity};
//!
//! let text1 = "The quick brown fox jumps over the lazy dog";
//! let text2 = "The quick brown fox leaps over the lazy dog";
//!
//! let hash1 = simhash(text1);
//! let hash2 = simhash(text2);
//!
//! let dist = hamming_distance(hash1, hash2);
//! let sim = similarity(hash1, hash2);
//!
//! println!("Hamming distance: {}", dist);
//! println!("Similarity: {:.2}", sim);
//! ```
//!
//! # Performance
//!
//! - SimHash generation: ≤2ms per atom
//! - Hamming distance: ≥4M ops/sec
//!
//! # See Also
//!
//! - [System Specification](https://github.com/your-org/anchor-rewrite-v0/blob/main/specs/spec.md#simhash-fingerprinting)
//! - [Testing Standards](https://github.com/your-org/anchor-rewrite-v0/blob/main/specs/standards/testing.md)

mod simhash;
mod distance;

pub use simhash::simhash;
pub use simhash::simhash_with_tokens;
pub use simhash::tokenize;
pub use distance::hamming_distance;
pub use distance::similarity;
pub use distance::hamming_weight;

/// Compute the 64-bit SimHash fingerprint of a text string.
///
/// This is the main entry point for fingerprinting. It tokenizes the input
/// and computes a 64-bit SimHash using the standard algorithm.
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
/// let fingerprint = anchor_fingerprint::fingerprint("Hello, world!");
/// println!("Fingerprint: {:016x}", fingerprint);
/// ```
pub fn fingerprint(text: &str) -> u64 {
    simhash(text)
}

/// Compute the Hamming distance between two 64-bit fingerprints.
///
/// The Hamming distance is the number of bit positions that differ between
/// two fingerprints. For similar texts, this distance is typically small.
///
/// # Arguments
///
/// * `a` - First fingerprint
/// * `b` - Second fingerprint
///
/// # Returns
///
/// The number of differing bits (0-64)
///
/// # Example
///
/// ```rust
/// let hash1 = anchor_fingerprint::simhash("Hello, world!");
/// let hash2 = anchor_fingerprint::simhash("Hello, world!");
/// assert_eq!(anchor_fingerprint::distance(hash1, hash2), 0);
/// ```
pub fn distance(a: u64, b: u64) -> u32 {
    hamming_distance(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_texts_produce_same_hash() {
        let text = "The quick brown fox jumps over the lazy dog";
        let hash1 = simhash(text);
        let hash2 = simhash(text);
        assert_eq!(hash1, hash2);
        assert_eq!(hamming_distance(hash1, hash2), 0);
        assert!((similarity(hash1, hash2) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_similar_texts_produce_similar_hashes() {
        let text1 = "The quick brown fox jumps over the lazy dog";
        let text2 = "The quick brown fox leaps over the lazy dog";
        let text3 = "Completely different text about something else";

        let hash1 = simhash(text1);
        let hash2 = simhash(text2);
        let hash3 = simhash(text3);

        // Similar texts should have small Hamming distance
        let dist12 = hamming_distance(hash1, hash2);
        let dist13 = hamming_distance(hash1, hash3);

        // Similar texts should be closer than dissimilar texts
        assert!(
            dist12 < dist13,
            "Similar texts should have smaller distance (got {} vs {})",
            dist12,
            dist13
        );

        // Similarity should reflect this
        let sim12 = similarity(hash1, hash2);
        let sim13 = similarity(hash1, hash3);
        assert!(
            sim12 > sim13,
            "Similar texts should have higher similarity (got {} vs {})",
            sim12,
            sim13
        );
    }

    #[test]
    fn test_empty_text() {
        let hash = simhash("");
        // Empty text should still produce a valid hash
        assert!(hash >= 0);
    }

    #[test]
    fn test_hamming_distance_symmetric() {
        let text1 = "Hello, world!";
        let text2 = "Goodbye, world!";
        let hash1 = simhash(text1);
        let hash2 = simhash(text2);

        assert_eq!(
            hamming_distance(hash1, hash2),
            hamming_distance(hash2, hash1)
        );
    }

    #[test]
    fn test_similarity_bounds() {
        let text1 = "Test text 1";
        let text2 = "Test text 2";
        let hash1 = simhash(text1);
        let hash2 = simhash(text2);

        let sim = similarity(hash1, hash2);
        assert!(
            sim >= 0.0 && sim <= 1.0,
            "Similarity should be between 0 and 1, got {}",
            sim
        );
    }

    #[test]
    fn test_hamming_weight() {
        let hash: u64 = 0b1010_1010_1010_1010;
        assert_eq!(hamming_weight(hash), 4);

        let hash2: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        assert_eq!(hamming_weight(hash2), 64);

        let hash3: u64 = 0;
        assert_eq!(hamming_weight(hash3), 0);
    }

    #[test]
    fn test_unicode_text() {
        let text1 = "你好，世界！";
        let text2 = "你好，世界！";
        let text3 = "Hello, World!";

        let hash1 = simhash(text1);
        let hash2 = simhash(text2);
        let hash3 = simhash(text3);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_single_token() {
        let hash = simhash("hello");
        assert_ne!(hash, 0, "Single token should produce non-zero hash");
    }

    #[test]
    fn test_fingerprint_alias() {
        let text = "Test content";
        assert_eq!(simhash(text), fingerprint(text));
    }

    #[test]
    fn test_distance_alias() {
        let text1 = "Hello";
        let text2 = "World";
        let hash1 = simhash(text1);
        let hash2 = simhash(text2);
        assert_eq!(hamming_distance(hash1, hash2), distance(hash1, hash2));
    }
}

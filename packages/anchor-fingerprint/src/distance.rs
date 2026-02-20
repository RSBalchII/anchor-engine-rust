//! Hamming distance and similarity calculations.
//!
//! This module provides functions for computing the Hamming distance
//! between two 64-bit fingerprints and converting that to a similarity score.
//!
//! # Performance
//!
//! The `hamming_distance` function uses the built-in `count_ones()` intrinsic
//! which compiles to a POPCNT instruction on modern CPUs, providing excellent
//! performance (≥4M ops/sec).
//!
//! # Example
//!
//! ```rust
//! use anchor_fingerprint::{hamming_distance, similarity};
//!
//! let a = 0x1234567890ABCDEFu64;
//! let b = 0xFEDCBA0987654321u64;
//!
//! let dist = hamming_distance(a, b);
//! let sim = similarity(a, b);
//!
//! println!("Distance: {}", dist);
//! println!("Similarity: {:.2}", sim);
//! ```

/// Compute the Hamming distance between two 64-bit fingerprints.
///
/// The Hamming distance is the number of bit positions that differ between
/// the two fingerprints. This is computed efficiently using XOR and POPCNT.
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
/// # Performance
///
/// This function compiles to a single XOR + POPCNT instruction on modern
/// CPUs, achieving ≥4M operations per second.
///
/// # Example
///
/// ```rust
/// use anchor_fingerprint::hamming_distance;
///
/// let a = 0b1010_1010;
/// let b = 0b1001_1001;
/// let dist = hamming_distance(a, b);
/// assert_eq!(dist, 4);
/// ```
#[inline]
pub fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

/// Compute the Hamming weight (population count) of a 64-bit value.
///
/// The Hamming weight is the number of 1-bits in the value.
///
/// # Arguments
///
/// * `hash` - The value to count bits in
///
/// # Returns
///
/// The number of 1-bits (0-64)
///
/// # Example
///
/// ```rust
/// use anchor_fingerprint::hamming_weight;
///
/// let hash = 0b1010_1010;
/// let weight = hamming_weight(hash);
/// assert_eq!(weight, 4);
/// ```
#[inline]
pub fn hamming_weight(hash: u64) -> u32 {
    hash.count_ones()
}

/// Compute the similarity between two 64-bit fingerprints.
///
/// Similarity is defined as `1.0 - (distance / 64.0)`, where distance
/// is the Hamming distance. A similarity of 1.0 means the fingerprints
/// are identical, 0.0 means they differ in all bits.
///
/// # Arguments
///
/// * `a` - First fingerprint
/// * `b` - Second fingerprint
///
/// # Returns
///
/// A similarity score between 0.0 and 1.0
///
/// # Example
///
/// ```rust
/// use anchor_fingerprint::similarity;
///
/// let a = 0x1234567890ABCDEFu64;
/// let b = 0x1234567890ABCDEFu64;
/// let sim = similarity(a, b);
/// assert_eq!(sim, 1.0);
/// ```
#[inline]
pub fn similarity(a: u64, b: u64) -> f32 {
    let dist = hamming_distance(a, b);
    1.0 - (dist as f32 / 64.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hamming_distance_identical() {
        let a = 0x1234567890ABCDEFu64;
        let b = 0x1234567890ABCDEFu64;
        assert_eq!(hamming_distance(a, b), 0);
    }

    #[test]
    fn test_hamming_distance_opposite() {
        let a = 0x0000000000000000u64;
        let b = 0xFFFFFFFFFFFFFFFFu64;
        assert_eq!(hamming_distance(a, b), 64);
    }

    #[test]
    fn test_hamming_distance_single_bit() {
        let a = 0b0000_0000u64;
        let b = 0b0000_0001u64;
        assert_eq!(hamming_distance(a, b), 1);

        let c = 0b0000_0100u64;
        assert_eq!(hamming_distance(a, c), 1);
    }

    #[test]
    fn test_hamming_distance_symmetric() {
        let a = 0x1234567890ABCDEFu64;
        let b = 0xFEDCBA0987654321u64;
        assert_eq!(hamming_distance(a, b), hamming_distance(b, a));
    }

    #[test]
    fn test_hamming_distance_known_value() {
        let a = 0b1010_1010_1010_1010u64;
        let b = 0b0101_0101_0101_0101u64;
        assert_eq!(hamming_distance(a, b), 16);
    }

    #[test]
    fn test_hamming_weight_zero() {
        assert_eq!(hamming_weight(0u64), 0);
    }

    #[test]
    fn test_hamming_weight_all_ones() {
        assert_eq!(hamming_weight(0xFFFF_FFFF_FFFF_FFFFu64), 64);
    }

    #[test]
    fn test_hamming_weight_alternating() {
        assert_eq!(hamming_weight(0xAAAA_AAAA_AAAA_AAAAu64), 32);
        assert_eq!(hamming_weight(0x5555_5555_5555_5555u64), 32);
    }

    #[test]
    fn test_hamming_weight_single_bit() {
        assert_eq!(hamming_weight(1u64), 1);
        assert_eq!(hamming_weight(2u64), 1);
        assert_eq!(hamming_weight(4u64), 1);
        assert_eq!(hamming_weight(8u64), 1);
    }

    #[test]
    fn test_similarity_identical() {
        let a = 0x1234567890ABCDEFu64;
        assert_eq!(similarity(a, a), 1.0);
    }

    #[test]
    fn test_similarity_opposite() {
        let a = 0x0000000000000000u64;
        let b = 0xFFFFFFFFFFFFFFFFu64;
        assert_eq!(similarity(a, b), 0.0);
    }

    #[test]
    fn test_similarity_half_different() {
        let a = 0x0000000000000000u64;
        let b = 0xFFFF_FFFF_0000_0000u64;
        let sim = similarity(a, b);
        assert!((sim - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_similarity_bounds() {
        let a = 0x1234567890ABCDEFu64;
        let b = 0xFEDCBA0987654321u64;
        let sim = similarity(a, b);
        assert!(sim >= 0.0 && sim <= 1.0);
    }

    #[test]
    fn test_similarity_symmetric() {
        let a = 0x1234567890ABCDEFu64;
        let b = 0xFEDCBA0987654321u64;
        assert_eq!(similarity(a, b), similarity(b, a));
    }

    #[test]
    fn test_distance_similarity_relationship() {
        let a = 0x1234567890ABCDEFu64;
        let b = 0xFEDCBA0987654321u64;

        let dist = hamming_distance(a, b) as f32;
        let sim = similarity(a, b);

        let expected_sim = 1.0 - (dist / 64.0);
        assert!((sim - expected_sim).abs() < f32::EPSILON);
    }

    #[test]
    fn test_hamming_distance_with_simhash() {
        use crate::simhash;

        let text1 = "Hello, world!";
        let text2 = "Hello, world!";
        let text3 = "Goodbye, world!";

        let hash1 = simhash(text1);
        let hash2 = simhash(text2);
        let hash3 = simhash(text3);

        // Identical texts should have zero distance
        assert_eq!(hamming_distance(hash1, hash2), 0);

        // Different texts should have non-zero distance
        assert!(hamming_distance(hash1, hash3) > 0);
    }

    #[test]
    fn test_similarity_with_simhash() {
        use crate::simhash;

        let text1 = "The quick brown fox";
        let text2 = "The quick brown fox";
        let text3 = "Something completely different";

        let hash1 = simhash(text1);
        let hash2 = simhash(text2);
        let hash3 = simhash(text3);

        // Identical texts should have similarity 1.0
        assert_eq!(similarity(hash1, hash2), 1.0);

        // Different texts should have lower similarity
        let sim13 = similarity(hash1, hash3);
        assert!(sim13 < 1.0);

        // Similarity should still be positive (some bits may match)
        assert!(sim13 > 0.0);
    }
}

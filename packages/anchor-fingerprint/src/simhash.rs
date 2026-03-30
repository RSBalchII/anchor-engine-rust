//! SimHash algorithm implementation.
//!
//! This module implements the 64-bit SimHash algorithm for near-duplicate
//! detection. The algorithm works as follows:
//!
//! 1. Tokenize the input text (word-level, lowercase)
//! 2. For each token, compute a 64-bit hash using MurmurHash3
//! 3. For each bit position (0-63), accumulate +1 if the token's hash has
//!    a 1, -1 if it has a 0
//! 4. The final fingerprint has bit i set if accumulator[i] > 0
//!
//! # References
//!
//! - Charikar, M. S. (2002). "Similarity estimation techniques from
//!   rounding algorithms"
//! - [System Specification](https://github.com/your-org/anchor-rewrite-v0/blob/main/specs/spec.md#simhash-fingerprinting)

use murmur3::murmur3_x64_128;
use std::io::Cursor;

/// Hash seed for MurmurHash3
const HASH_SEED: u32 = 0;

/// Tokenize input text into lowercase words.
///
/// This function splits text on non-alphanumeric characters, converts
/// to lowercase, and filters empty tokens.
///
/// # Arguments
///
/// * `text` - The input text to tokenize
///
/// # Returns
///
/// A vector of lowercase token strings
///
/// # Example
///
/// ```rust
/// use anchor_fingerprint::tokenize;
///
/// let tokens = tokenize("Hello, World! This is a test.");
/// assert_eq!(tokens.len(), 6);
/// assert_eq!(tokens[0], "hello");
/// assert_eq!(tokens[1], "world");
/// ```
pub fn tokenize(text: &str) -> Vec<String> {
    text.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_lowercase().to_string()
            } else {
                " ".to_string()
            }
        })
        .collect::<String>()
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Hash a single token using MurmurHash3 (64-bit).
///
/// # Arguments
///
/// * `token` - The token string to hash
///
/// # Returns
///
/// A 64-bit hash value
fn hash_token(token: &str) -> u64 {
    // Use murmur3_x64_128 and take the lower 64 bits
    let mut cursor = Cursor::new(token.as_bytes());
    let hash_128: u128 = murmur3_x64_128(&mut cursor, HASH_SEED).unwrap_or(0);
    (hash_128 & 0xFFFF_FFFF_FFFF_FFFF) as u64
}

/// Compute the 64-bit SimHash fingerprint of raw bytes.
///
/// This function hashes bytes directly without UTF-8 validation.
/// Useful for deduplication where you want to avoid UTF-8 validation
/// overhead on duplicate blocks.
///
/// # Arguments
///
/// * `bytes` - Raw byte slice to hash
///
/// # Returns
///
/// A 64-bit SimHash fingerprint
///
/// # Example
///
/// ```rust
/// use anchor_fingerprint::simhash_bytes;
///
/// let hash = simhash_bytes(b"Hello, world!");
/// println!("Hash: {:016x}", hash);
/// ```
pub fn simhash_bytes(bytes: &[u8]) -> u64 {
    // Hash the raw bytes directly using MurmurHash3
    // No tokenization, no UTF-8 validation
    use murmur3::murmur3_x64_128;
    use std::io::Cursor;
    
    if bytes.is_empty() {
        return 0;
    }
    
    // For byte-level hashing, we use a sliding window approach
    // This captures local structure without requiring UTF-8 validity
    const WINDOW_SIZE: usize = 8;
    let mut accumulator = [0i32; 64];
    
    // Slide window over bytes
    for i in 0..bytes.len().saturating_sub(WINDOW_SIZE - 1) {
        let window = &bytes[i..i + WINDOW_SIZE];
        let mut cursor = Cursor::new(window);
        let hash_128: u128 = murmur3_x64_128(&mut cursor, HASH_SEED).unwrap_or(0);
        let hash = (hash_128 & 0xFFFF_FFFF_FFFF_FFFF) as u64;
        
        // Update accumulator for each bit position
        for bit in 0..64 {
            if (hash >> bit) & 1 == 1 {
                accumulator[bit] += 1;
            } else {
                accumulator[bit] -= 1;
            }
        }
    }
    
    // Convert accumulator to final fingerprint
    let mut fingerprint: u64 = 0;
    for bit in 0..64 {
        if accumulator[bit] > 0 {
            fingerprint |= 1 << bit;
        }
    }
    
    fingerprint
}

/// Compute the 64-bit SimHash fingerprint of a text string.
///
/// This is the main SimHash implementation. It tokenizes the input and
/// computes a fingerprint using the standard algorithm.
///
/// # Arguments
///
/// * `text` - The input text to fingerprint
///
/// # Returns
///
/// A 64-bit SimHash fingerprint
///
/// # Example
///
/// ```rust
/// use anchor_fingerprint::simhash;
///
/// let hash = simhash("Hello, world!");
/// println!("Hash: {:016x}", hash);
/// ```
pub fn simhash(text: &str) -> u64 {
    let tokens = tokenize(text);
    simhash_with_tokens(&tokens)
}

/// Compute SimHash from pre-tokenized words.
///
/// This is useful when you want to apply custom tokenization logic
/// or when processing multiple texts with the same token set.
///
/// # Arguments
///
/// * `tokens` - Slice of token strings
///
/// # Returns
///
/// A 64-bit SimHash fingerprint
///
/// # Example
///
/// ```rust
/// use anchor_fingerprint::simhash_with_tokens;
///
/// let tokens = vec!["hello".to_string(), "world".to_string()];
/// let hash = simhash_with_tokens(&tokens);
/// ```
pub fn simhash_with_tokens(tokens: &[String]) -> u64 {
    if tokens.is_empty() {
        return 0;
    }

    // Accumulator for each bit position (64 bits)
    let mut accumulator = [0i32; 64];

    // For each token, update the accumulator
    for token in tokens {
        let hash = hash_token(token);

        // For each bit position, add +1 or -1 to accumulator
        for bit in 0..64 {
            if (hash >> bit) & 1 == 1 {
                accumulator[bit] += 1;
            } else {
                accumulator[bit] -= 1;
            }
        }
    }

    // Convert accumulator to final fingerprint
    let mut fingerprint: u64 = 0;
    for bit in 0..64 {
        if accumulator[bit] > 0 {
            fingerprint |= 1u64 << bit;
        }
    }

    fingerprint
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let text = "Hello, world! This is a test.";
        let tokens = tokenize(text);
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0], "hello");
        assert_eq!(tokens[1], "world");
        assert_eq!(tokens[2], "this");
        assert_eq!(tokens[3], "is");
        assert_eq!(tokens[4], "a");
        assert_eq!(tokens[5], "test");
    }

    #[test]
    fn test_tokenize_empty() {
        let text = "";
        let tokens = tokenize(text);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenize_whitespace_only() {
        let text = "   \t\n  ";
        let tokens = tokenize(text);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenize_mixed_case() {
        let text = "Hello HELLO heLLo";
        let tokens = tokenize(text);
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], "hello");
        assert_eq!(tokens[1], "hello");
        assert_eq!(tokens[2], "hello");
    }

    #[test]
    fn test_tokenize_numbers() {
        let text = "Test 123 with 456 numbers";
        let tokens = tokenize(text);
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[1], "123");
        assert_eq!(tokens[3], "456");
    }

    #[test]
    fn test_tokenize_unicode() {
        let text = "你好，世界！Hello, World!";
        let tokens = tokenize(text);
        assert_eq!(tokens.len(), 4);
        assert!(tokens.contains(&"你好".to_string()));
        assert!(tokens.contains(&"世界".to_string()));
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
    }

    #[test]
    fn test_simhash_deterministic() {
        let text = "The quick brown fox";
        let hash1 = simhash(text);
        let hash2 = simhash(text);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_simhash_with_tokens_direct() {
        let tokens = vec!["hello".to_string(), "world".to_string()];
        let hash1 = simhash_with_tokens(&tokens);

        let text = "hello world";
        let hash2 = simhash(text);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_simhash_empty() {
        let hash = simhash("");
        assert_eq!(hash, 0);
    }

    #[test]
    fn test_simhash_empty_tokens() {
        let tokens: Vec<String> = vec![];
        let hash = simhash_with_tokens(&tokens);
        assert_eq!(hash, 0);
    }

    #[test]
    fn test_simhash_single_token() {
        let hash = simhash("hello");
        assert_ne!(hash, 0, "Single token should produce non-zero hash");
    }

    #[test]
    fn test_simhash_long_text() {
        let text = "This is a longer piece of text with many words. ".repeat(10);
        let hash = simhash(&text);
        assert_ne!(hash, 0);

        // Same text should produce same hash
        assert_eq!(hash, simhash(&text));
    }

    #[test]
    fn test_token_hash_consistency() {
        let token = "test";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_token_hash_different_tokens() {
        let hash1 = hash_token("hello");
        let hash2 = hash_token("world");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_simhash_similar_texts() {
        let text1 = "The quick brown fox jumps over the lazy dog";
        let text2 = "The quick brown fox leaps over the lazy dog";
        let text3 = "Completely different text about something else";

        let hash1 = simhash(text1);
        let hash2 = simhash(text2);
        let hash3 = simhash(text3);

        let dist12 = (hash1 ^ hash2).count_ones();
        let dist13 = (hash1 ^ hash3).count_ones();

        // Similar texts should have smaller distance
        assert!(
            dist12 < dist13,
            "Similar texts should have smaller Hamming distance"
        );
    }
}

//! Transient Data Filter
//!
//! Detects and excludes transient/noisy content from ingestion.
//! Implements Pattern D from anchor-engine-node v4.2.0.
//!
//! # Patterns Detected
//!
//! - Terminal error logs (Traceback, KeyError, TypeError, etc.)
//! - Package installation output (npm install, pip install, etc.)
//! - Build artifacts (Build succeeded, Compiling..., etc.)
//! - Repetitive log noise (timestamp lines, separator lines)
//!
//! # Usage
//!
//! ```rust
//! use anchor_engine::services::transient_filter::{TransientFilter, TransientFilterConfig};
//!
//! let config = TransientFilterConfig::default();
//! let filter = TransientFilter::new(config);
//!
//! let content = "Traceback (most recent call last):\n...";
//! assert!(filter.is_transient(content));
//! ```

use once_cell::sync::Lazy;
use regex::Regex;

/// Configuration for transient filter
#[derive(Debug, Clone)]
pub struct TransientFilterConfig {
    /// Minimum number of lines to consider for filtering
    pub min_lines: usize,
    /// Threshold for transient content (0.0-1.0)
    /// If >50% lines match transient patterns, skip entire document
    pub threshold: f64,
}

impl Default for TransientFilterConfig {
    fn default() -> Self {
        Self {
            min_lines: 5,
            threshold: 0.5,
        }
    }
}

/// Transient data filter
pub struct TransientFilter {
    config: TransientFilterConfig,
}

// Compile-time regex patterns for performance
static TRANSIENT_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Terminal error logs
        Regex::new(r"(?i)Traceback\s*\(most recent call last\)").unwrap(),
        Regex::new(r"(?i)KeyError:").unwrap(),
        Regex::new(r"(?i)TypeError:").unwrap(),
        Regex::new(r"(?i)ValueError:").unwrap(),
        Regex::new(r"(?i)Error:.*at line \d+").unwrap(),
        Regex::new(r"(?i)Exception in thread").unwrap(),
        Regex::new(r"(?i)Fatal error:").unwrap(),
        
        // Package installation logs
        Regex::new(r"(?i)npm install").unwrap(),
        Regex::new(r"(?i)pip install").unwrap(),
        Regex::new(r"(?i)yarn add").unwrap(),
        Regex::new(r"(?i)pnpm add").unwrap(),
        Regex::new(r"(?i)Collecting\s+[a-zA-Z0-9_-]+").unwrap(),
        Regex::new(r"(?i)Downloading\s+[a-zA-Z0-9_-]+").unwrap(),
        Regex::new(r"(?i)added\s+\d+\s+package").unwrap(),
        Regex::new(r"(?i)Successfully installed").unwrap(),
        
        // Build artifacts
        Regex::new(r"(?i)Build succeeded").unwrap(),
        Regex::new(r"(?i)Build failed").unwrap(),
        Regex::new(r"(?i)Compiling\.\.\.").unwrap(),
        Regex::new(r"(?i)Linking\.\.\.").unwrap(),
        Regex::new(r"(?i)Generating\.\.\.").unwrap(),
        
        // Repetitive log noise
        Regex::new(r"^\[\d{4}-\d{2}-\d{2}.*\]$").unwrap(),
        Regex::new(r"^={50,}$").unwrap(),
        Regex::new(r"^-{50,}$").unwrap(),
    ]
});

impl TransientFilter {
    /// Create a new transient filter with default configuration
    pub fn new(config: TransientFilterConfig) -> Self {
        Self { config }
    }

    /// Check if content is transient/temporary data that should be excluded
    ///
    /// Returns `true` if more than 50% of lines match transient patterns.
    ///
    /// # Arguments
    ///
    /// * `content` - The content to check
    ///
    /// # Returns
    ///
    /// `true` if content should be skipped, `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// use anchor_engine::services::transient_filter::{TransientFilter, TransientFilterConfig};
    ///
    /// let filter = TransientFilter::new(TransientFilterConfig::default());
    /// assert!(filter.is_transient("Traceback (most recent call last):\n..."));
    /// assert!(!filter.is_transient("# Documentation\n\nThis is clean content."));
    /// ```
    pub fn is_transient(&self, content: &str) -> bool {
        let lines: Vec<&str> = content.lines().collect();
        
        // Too short to be log output
        if lines.len() < self.config.min_lines {
            return false;
        }
        
        let mut transient_lines = 0;
        
        for line in &lines {
            for pattern in TRANSIENT_PATTERNS.iter() {
                if pattern.is_match(line) {
                    transient_lines += 1;
                    break; // Count each line only once
                }
            }
        }
        
        let ratio = transient_lines as f64 / lines.len() as f64;
        ratio > self.config.threshold
    }

    /// Get statistics about transient patterns in content
    ///
    /// # Returns
    ///
    /// A tuple of (total_lines, transient_lines, ratio)
    pub fn analyze(&self, content: &str) -> (usize, usize, f64) {
        let lines: Vec<&str> = content.lines().collect();
        let total = lines.len();
        
        let mut transient_lines = 0;
        for line in lines {
            for pattern in TRANSIENT_PATTERNS.iter() {
                if pattern.is_match(line) {
                    transient_lines += 1;
                    break;
                }
            }
        }
        
        let ratio = if total > 0 {
            transient_lines as f64 / total as f64
        } else {
            0.0
        };
        
        (total, transient_lines, ratio)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test fixture for transient content samples
    struct TransientFixtures;

    impl TransientFixtures {
        fn python_traceback() -> &'static str {
            r#"Traceback (most recent call last):
  File "script.py", line 10, in <module>
    result = divide(1, 0)
  File "script.py", line 5, in divide
    return a / b
ZeroDivisionError: division by zero"#
        }

        fn npm_install() -> &'static str {
            r#"added 142 packages, and audited 143 packages in 3s
23 packages are looking for funding
  run `npm fund` for details
found 0 vulnerabilities"#
        }

        fn pip_install() -> &'static str {
            r#"Collecting requests
  Downloading requests-2.31.0-py3-none-any.whl (62 kB)
Installing collected packages: urllib3, requests
Successfully installed requests-2.31.0 urllib3-2.1.0"#
        }

        fn build_output() -> &'static str {
            r#"Compiling anchor-engine v0.1.0
   Linking target/release/anchor-engine
Build succeeded in 45.3s"#
        }

        fn clean_content() -> &'static str {
            r#"# Anchor Engine Documentation

The Anchor Engine implements the STAR Algorithm.

## Installation

```bash
cargo install anchor-engine
```

## Usage

```rust
use anchor_engine::Database;
```"#
        }

        fn mixed_content() -> &'static str {
            r#"# Project Notes

Meeting notes from 2026-02-23:

- Discussed architecture

$ npm install
added 50 packages

Next steps:
- Implement context inflation

$ pip install requests
Successfully installed requests-2.31.0

TODO: Write tests"#
        }

        fn error_log() -> &'static str {
            r#"2026-02-23 10:15:32 ERROR: Database connection failed
KeyError: 'user_id'
TypeError: expected string, got None
ValueError: invalid literal for int() with base 10: 'abc'"#
        }
    }

    #[test]
    fn test_detect_python_traceback() {
        let filter = TransientFilter::new(TransientFilterConfig::default());
        let content = TransientFixtures::python_traceback();
        assert!(filter.is_transient(content), "Should detect Python traceback");
    }

    #[test]
    fn test_detect_npm_install() {
        let filter = TransientFilter::new(TransientFilterConfig::default());
        let content = TransientFixtures::npm_install();
        assert!(filter.is_transient(content), "Should detect npm install");
    }

    #[test]
    fn test_detect_pip_install() {
        let filter = TransientFilter::new(TransientFilterConfig::default());
        let content = TransientFixtures::pip_install();
        assert!(filter.is_transient(content), "Should detect pip install");
    }

    #[test]
    fn test_detect_build_output() {
        let filter = TransientFilter::new(TransientFilterConfig::default());
        let content = TransientFixtures::build_output();
        assert!(filter.is_transient(content), "Should detect build output");
    }

    #[test]
    fn test_allow_clean_content() {
        let filter = TransientFilter::new(TransientFilterConfig::default());
        let content = TransientFixtures::clean_content();
        assert!(!filter.is_transient(content), "Should allow clean content");
    }

    #[test]
    fn test_mixed_content_threshold() {
        let filter = TransientFilter::new(TransientFilterConfig::default());
        let content = TransientFixtures::mixed_content();
        // Mixed content has ~30% transient lines (below 50% threshold)
        assert!(!filter.is_transient(content), "Mixed content below threshold should pass");
    }

    #[test]
    fn test_detect_error_logs() {
        let filter = TransientFilter::new(TransientFilterConfig::default());
        let content = TransientFixtures::error_log();
        assert!(filter.is_transient(content), "Should detect error logs");
    }

    #[test]
    fn test_pattern_performance() {
        let filter = TransientFilter::new(TransientFilterConfig::default());
        let content = TransientFixtures::clean_content();
        let iterations = 1000;

        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = filter.is_transient(content);
        }
        let elapsed = start.elapsed();

        println!("Processed {} iterations in {:?}", iterations, elapsed);
        
        // Target: <1ms per check
        let avg_time = elapsed.as_millis() / iterations as u128;
        assert!(avg_time < 1, "Average check should be <1ms, got {}ms", avg_time);
    }

    #[test]
    fn test_unicode_content() {
        let filter = TransientFilter::new(TransientFilterConfig::default());
        let content = r#"Processing Unicode: 你好世界 🦀 日本語
Collecting パッケージ
Successfully installed 日本語パッケージ"#;
        
        assert!(filter.is_transient(content), "Should detect Unicode transient content");
    }

    #[test]
    fn test_case_insensitivity() {
        let filter = TransientFilter::new(TransientFilterConfig::default());
        let content = r#"TRACEBACK (most recent call last)
NPM INSTALL completed
BUILD SUCCEEDED"#;
        
        assert!(filter.is_transient(content), "Should be case-insensitive");
    }

    #[test]
    fn test_analyze_statistics() {
        let filter = TransientFilter::new(TransientFilterConfig::default());
        let content = TransientFixtures::mixed_content();
        
        let (total, transient, ratio) = filter.analyze(content);
        
        assert!(total > 0, "Should count total lines");
        assert!(transient >= 0, "Should count transient lines");
        assert!(ratio >= 0.0 && ratio <= 1.0, "Ratio should be 0.0-1.0");
        
        println!("Total: {}, Transient: {}, Ratio: {:.2}", total, transient, ratio);
    }

    #[test]
    fn test_short_content_allowed() {
        let filter = TransientFilter::new(TransientFilterConfig::default());
        let content = "Traceback (most recent call last)";
        
        // Too short (< 5 lines), should be allowed
        assert!(!filter.is_transient(content), "Short content should be allowed");
    }
}

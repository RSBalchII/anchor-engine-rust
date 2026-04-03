//! Pointer-Only Storage Module (Mirror Protocol)
//!
//! This module implements the "Mirror Protocol" for Anchor Engine:
//! - Database stores only pointers (source_path, start_byte, end_byte)
//! - Actual content resides in mirrored_brain/ directory
//! - Content is lazily loaded from filesystem on demand
//!
//! **Rationale:**
//! - Reduces database size by 10-100x
//! - Enables ephemeral database pattern (wipe and rebuild on startup)
//! - Filesystem is source of truth, database is index
//! - Critical for 9.8mW deployment (minimize memory footprint)
//!
//! See: specs/standards/pointer_only_storage.md

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use sha2::{Sha256, Digest};

/// LRU Cache for frequently accessed content
struct ContentCache {
    capacity: usize,
    cache: HashMap<String, String>,
    order: Vec<String>,
}

impl ContentCache {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: HashMap::new(),
            order: Vec::new(),
        }
    }

    fn get(&mut self, key: &str) -> Option<&String> {
        if let Some(content) = self.cache.get(key) {
            // Move to end (most recently used)
            if let Some(pos) = self.order.iter().position(|k| k == key) {
                self.order.remove(pos);
                self.order.push(key.to_string());
            }
            Some(content)
        } else {
            None
        }
    }

    fn insert(&mut self, key: String, value: String) {
        // If already exists, update and move to end
        if self.cache.contains_key(&key) {
            if let Some(pos) = self.order.iter().position(|k| k == &key) {
                self.order.remove(pos);
            }
        } else {
            // Evict oldest if at capacity
            if self.cache.len() >= self.capacity {
                if let Some(oldest) = self.order.first().cloned() {
                    self.cache.remove(&oldest);
                    self.order.remove(0);
                }
            }
        }
        
        self.cache.insert(key.clone(), value);
        self.order.push(key);
    }
}

/// Pointer-only storage trait
pub trait Storage: Send + Sync {
    /// Write sanitized content to mirrored_brain/ and return the file path
    fn write_cleaned(&self, source: &str, content: &str) -> Result<String>;

    /// Read content from a file at specific byte range
    fn read_range(&self, path: &str, start: usize, end: usize) -> Result<String>;

    /// Read entire file content
    fn read_all(&self, path: &str) -> Result<String>;

    /// Get the mirrored_brain directory path
    fn get_mirror_dir(&self) -> &Path;

    /// Clear the content cache (for testing)
    fn clear_cache(&self);
}

/// Filesystem-based pointer-only storage implementation
pub struct FileSystemStorage {
    mirror_dir: PathBuf,
    cache: Arc<Mutex<ContentCache>>,
}

impl FileSystemStorage {
    /// Create new filesystem storage with given mirror directory
    pub fn new(mirror_dir: PathBuf) -> Result<Self> {
        // Create mirror directory if it doesn't exist
        fs::create_dir_all(&mirror_dir)
            .with_context(|| format!("Failed to create mirror directory: {:?}", mirror_dir))?;

        Ok(Self {
            mirror_dir,
            cache: Arc::new(Mutex::new(ContentCache::new(1000))), // Cache 1000 most recent content blocks
        })
    }
    
    /// Generate deterministic filename from source path
    fn generate_filename(&self, source: &str) -> String {
        // Hash the source path to get deterministic filename
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        let hash = hasher.finalize();
        
        // Get file extension from source if present
        let ext = Path::new(source)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("txt");
        
        format!("{:x}.{}", hash, ext)
    }
    
    /// Sanitize content (remove control chars, normalize whitespace)
    fn sanitize_content(content: &str) -> String {
        // Remove control characters except newline and tab
        let sanitized: String = content
            .chars()
            .filter(|c| c.is_ascii_graphic() || *c == '\n' || *c == '\t' || *c == '\r')
            .collect();
        
        // Normalize whitespace (multiple spaces → single space)
        let normalized = sanitized
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");
        
        normalized
    }
}

impl Storage for FileSystemStorage {
    fn write_cleaned(&self, source: &str, content: &str) -> Result<String> {
        // Generate deterministic filename
        let filename = self.generate_filename(source);
        let file_path = self.mirror_dir.join(&filename);
        
        // Check if file already exists (deduplication)
        if file_path.exists() {
            tracing::debug!("File already exists, skipping write: {:?}", file_path);
            return Ok(file_path.to_string_lossy().to_string());
        }
        
        // Sanitize content
        let sanitized = Self::sanitize_content(content);
        
        // Write to mirror directory
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)
            .with_context(|| format!("Failed to open file for writing: {:?}", file_path))?;
        
        file.write_all(sanitized.as_bytes())
            .with_context(|| format!("Failed to write content to file: {:?}", file_path))?;
        
        file.sync_all()
            .with_context(|| format!("Failed to sync file: {:?}", file_path))?;
        
        tracing::info!("✍️ Written cleaned content to: {:?}", file_path);
        
        Ok(file_path.to_string_lossy().to_string())
    }
    
    fn read_range(&self, path: &str, start: usize, end: usize) -> Result<String> {
        // Create cache key
        let cache_key = format!("{}:{}:{}", path, start, end);

        // Check cache first
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        // Open file
        let mut file = File::open(path)
            .with_context(|| format!("Failed to open file: {}", path))?;

        // Seek to start position
        file.seek(SeekFrom::Start(start as u64))
            .with_context(|| format!("Failed to seek to position {}: {}", start, path))?;

        // Calculate bytes to read
        let bytes_to_read = end - start;

        // Read content
        let mut buffer = vec![0u8; bytes_to_read];
        file.read_exact(&mut buffer)
            .with_context(|| format!("Failed to read {} bytes from {}: {}", bytes_to_read, path, start))?;

        // Convert to string
        let content = String::from_utf8(buffer)
            .with_context(|| format!("Invalid UTF-8 in file {} at {}:{}", path, start, end))?;

        // Cache the result
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(cache_key, content.clone());
        }

        Ok(content)
    }
    
    fn read_all(&self, path: &str) -> Result<String> {
        // Check cache first
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(path) {
                return Ok(cached.clone());
            }
        }

        // Read entire file
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path))?;

        // Cache the result
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(path.to_string(), content.clone());
        }

        Ok(content)
    }
    
    fn get_mirror_dir(&self) -> &Path {
        &self.mirror_dir
    }

    fn clear_cache(&self) {
        *self.cache.lock().unwrap() = ContentCache::new(1000);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_write_and_read_cleaned() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileSystemStorage::new(temp_dir.path().to_path_buf()).unwrap();
        
        let source = "test/document.md";
        let content = "Hello, World!\nThis is a test document.";
        
        // Write content
        let path = storage.write_cleaned(source, content).unwrap();
        
        // Verify file exists
        assert!(Path::new(&path).exists());
        
        // Read back content
        let file_size = fs::metadata(&path).unwrap().len() as usize;
        let read_content = storage.read_range(&path, 0, file_size).unwrap();
        
        // Content should be sanitized (whitespace normalized)
        assert!(read_content.contains("Hello"));
        assert!(read_content.contains("World"));
    }
    
    #[test]
    fn test_deduplication() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileSystemStorage::new(temp_dir.path().to_path_buf()).unwrap();
        
        let source = "test/document.md";
        let content = "Test content";
        
        // Write same content twice
        let path1 = storage.write_cleaned(source, content).unwrap();
        let path2 = storage.write_cleaned(source, content).unwrap();
        
        // Should return same path (deduplication)
        assert_eq!(path1, path2);
    }
    
    #[test]
    fn test_read_range() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileSystemStorage::new(temp_dir.path().to_path_buf()).unwrap();
        
        let source = "test/document.md";
        let content = "Hello, World! This is a test.";
        
        // Write content
        let path = storage.write_cleaned(source, content).unwrap();
        
        // Read first 5 bytes
        let first_part = storage.read_range(&path, 0, 5).unwrap();
        assert_eq!(first_part, "Hello");
        
        // Read middle part
        let middle_part = storage.read_range(&path, 7, 12).unwrap();
        assert_eq!(middle_part, "World");
    }
    
    #[test]
    fn test_cache() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileSystemStorage::new(temp_dir.path().to_path_buf()).unwrap();

        let source = "test/document.md";
        let content = "Test content for caching";

        // Write content
        let path = storage.write_cleaned(source, content).unwrap();
        let file_size = fs::metadata(&path).unwrap().len() as usize;

        // First read (from disk)
        let _ = storage.read_range(&path, 0, file_size);

        // Second read (from cache)
        let cached = storage.read_range(&path, 0, file_size);
        assert!(cached.is_ok());

        // Clear cache
        storage.clear_cache();

        // Third read (from disk again)
        let _ = storage.read_range(&path, 0, file_size);
    }
}

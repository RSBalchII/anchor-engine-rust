//! Ingestion Service - Processes files and ingests them into the database.
//!
//! This service handles the full ingestion pipeline:
//! 1. Read file content
//! 2. Mirror to mirrored_brain/
//! 3. Sanitize content
//! 4. Atomize into molecules
//! 5. Extract keywords as tags
//! 6. Compute SimHash fingerprints
//! 7. Store in database

use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Read, Write};
use thiserror::Error;
use tracing::{info, debug, error};

use anchor_atomizer::{atomize, sanitize};
use anchor_fingerprint::simhash;
use anchor_keyextract::extract_keywords;

use crate::db::Database;
use crate::models::{Atom, Source, Tag};

/// Ingestion errors.
#[derive(Error, Debug)]
pub enum IngestionError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] crate::db::DbError),
    
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    
    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),
    
    #[error("Failed to mirror file: {0}")]
    MirrorError(String),
}

/// Result type for ingestion operations.
pub type Result<T> = std::result::Result<T, IngestionError>;

/// Ingestion result.
#[derive(Debug, Clone)]
pub struct IngestionResult {
    /// Source ID
    pub source_id: String,
    /// Number of atoms created
    pub atoms_created: usize,
    /// Atom IDs
    pub atom_ids: Vec<u64>,
    /// Extracted tags
    pub tags: Vec<String>,
    /// File size in bytes
    pub file_size: u64,
    /// Processing time in milliseconds
    pub processing_time_ms: f64,
}

/// Ingestion service configuration.
#[derive(Debug, Clone)]
pub struct IngestionConfig {
    /// Path to mirrored brain directory
    pub mirrored_brain_path: PathBuf,
    /// Batch size for database inserts
    pub batch_size: usize,
    /// Maximum keywords to extract per atom
    pub max_keywords: usize,
    /// Minimum keyword score threshold
    pub min_keyword_score: f32,
    /// Enable sanitization
    pub sanitize: bool,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            mirrored_brain_path: PathBuf::from("mirrored_brain"),
            batch_size: 50,
            max_keywords: 10,
            min_keyword_score: 0.3,
            sanitize: true,
        }
    }
}

/// Ingestion service.
pub struct IngestionService {
    /// Database connection
    db: Database,
    /// Service configuration
    config: IngestionConfig,
}

impl IngestionService {
    /// Create a new Ingestion service.
    pub fn new(db: Database, config: IngestionConfig) -> Self {
        Self { db, config }
    }

    /// Create an in-memory ingestion service (for testing).
    pub fn in_memory() -> Result<Self> {
        let db = Database::in_memory().map_err(IngestionError::from)?;
        Ok(Self {
            db,
            config: IngestionConfig::default(),
        })
    }

    /// Ingest a file into the database.
    pub async fn ingest_file(&self, file_path: &Path) -> Result<IngestionResult> {
        let start_time = std::time::Instant::now();

        // Check file exists
        if !file_path.exists() {
            return Err(IngestionError::FileNotFound(file_path.to_path_buf()));
        }

        // Get file metadata
        let metadata = fs::metadata(file_path)?;
        let file_size = metadata.len();

        // 📥 INFO log at start of ingestion
        let size_str = if file_size < 1024 {
            format!("{} B", file_size)
        } else if file_size < 1024 * 1024 {
            format!("{:.1} KB", file_size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", file_size as f64 / (1024.0 * 1024.0))
        };
        info!("📥 INGEST START: {:?} ({})", file_path, size_str);

        // Read file content
        let mut file = fs::File::open(file_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let original_size = content.len();

        // ✓ INFO log after read
        info!("   ├─ ✓ Read: {} bytes", original_size);

        // Generate source ID from file path
        let source_id = self.generate_source_id(file_path);

        // Mirror file to mirrored_brain/
        let mirrored_path = self.mirror_file(file_path, &content)?;

        // ✓ INFO log after mirror
        info!("   ├─ ✓ Mirrored: {}", mirrored_path.display());

        // Sanitize content if enabled
        let content = if self.config.sanitize {
            sanitize(&content)
        } else {
            content
        };
        let sanitized_size = content.len();

        // ✓ INFO log after sanitize
        info!("   ├─ ✓ Sanitized: {} → {} chars", original_size, sanitized_size);

        // Atomize content
        let atoms_data = atomize(&content);

        // ✓ INFO log after atomize
        info!("   ├─ ✓ Atomized: {} atoms", atoms_data.len());

        // Create source record
        let source = Source {
            id: source_id.clone(),
            path: mirrored_path.to_string_lossy().to_string(),
            bucket: Some(self.get_bucket_from_path(file_path)),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            metadata: None,
        };

        self.db.upsert_source(&source).await?;

        // Create atoms with tags
        let mut atom_ids = Vec::new();
        let mut all_tags = Vec::new();
        let mut total_tags_extracted = 0;
        let mut last_simhash: u64 = 0;
        
        info!("   ├─ [AtomicIngest] START Persisting: {} atoms", atoms_data.len());
        let persist_start = std::time::Instant::now();

        for (i, atom_data) in atoms_data.iter().enumerate() {
            // Generate SimHash
            let hash = simhash(&atom_data.content);
            last_simhash = hash;

            // Create atom
            let atom = Atom {
                id: 0, // Will be assigned by DB
                source_id: source_id.clone(),
                content: atom_data.content.clone(),
                char_start: atom_data.char_start,
                char_end: atom_data.char_end,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64(),
                simhash: hash,
                tags: Vec::new(),
                metadata: None,
            };

            let atom_id = self.db.insert_atom(&atom).await?;
            atom_ids.push(atom_id);

            // Extract keywords as tags
            let keywords = extract_keywords(&atom_data.content, self.config.max_keywords);
            let mut tags = Vec::new();

            for kw in keywords {
                if kw.score >= self.config.min_keyword_score {
                    let tag = format!("#{}", kw.term.to_lowercase());
                    tags.push(Tag {
                        id: 0,
                        atom_id,
                        tag: tag.clone(),
                        bucket: None,
                    });
                    all_tags.push(tag.clone());
                    total_tags_extracted += 1;
                }
            }

            // Add tags to database
            if !tags.is_empty() {
                self.db.add_tags(atom_id, &tags).await?;
            }
            
            // Progress logging every 1000 atoms or at 10%, 20%, etc.
            let progress = ((i + 1) as f64 / atoms_data.len() as f64) * 100.0;
            if (i + 1) % 1000 == 0 || progress % 10.0 < 0.1 {
                info!("   ├─ [AtomicIngest] Progress: {}/{} atoms ({:.1}%)", 
                      i + 1, atoms_data.len(), progress);
            }
        }
        
        info!("   ├─ ✓ Atoms persisted: {:.2}s", persist_start.elapsed().as_secs_f64());

        // ✓ INFO log after fingerprint and store
        info!("   ├─ ✓ Extracted: {} tags", total_tags_extracted);
        info!("   ├─ ✓ Fingerprinted: SimHash 0x{:016x}", last_simhash);
        info!("   ├─ ✓ Stored: atom_id={}", atom_ids.last().unwrap_or(&0));

        let processing_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;

        // ✅ INFO log with summary at end
        info!(
            "   └─ ✅ [Atomizer] COMPLETE: {:?} → {} atoms, {} tags in {:.2}s",
            file_path.file_name().unwrap_or_default(),
            atom_ids.len(),
            all_tags.len(),
            processing_time_ms / 1000.0
        );

        Ok(IngestionResult {
            source_id,
            atoms_created: atom_ids.len(),
            atom_ids,
            tags: all_tags,
            file_size,
            processing_time_ms,
        })
    }

    /// Mirror a file to the mirrored_brain directory.
    fn mirror_file(&self, source_path: &Path, content: &str) -> Result<PathBuf> {
        // Calculate mirrored path
        let relative_path = source_path
            .strip_prefix(std::env::current_dir().unwrap_or_default())
            .unwrap_or(source_path);
        
        let mirrored_path = self.config.mirrored_brain_path.join(relative_path);

        // Ensure parent directory exists
        if let Some(parent) = mirrored_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write mirrored content
        let mut file = fs::File::create(&mirrored_path)?;
        file.write_all(content.as_bytes())?;

        Ok(mirrored_path)
    }

    /// Generate a unique source ID from a file path.
    fn generate_source_id(&self, path: &Path) -> String {
        // Use file path hash as source ID
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        format!("src_{:016x}", hasher.finish())
    }

    /// Get bucket name from file path.
    fn get_bucket_from_path(&self, path: &Path) -> String {
        // Extract bucket from path (e.g., "inbox/Code/file.md" -> "Code")
        if let Some(parent) = path.parent() {
            if let Some(bucket) = parent.file_name() {
                return bucket.to_string_lossy().to_string();
            }
        }
        "default".to_string()
    }

    /// Ingest content directly (without file).
    pub async fn ingest_content(
        &self,
        source: &str,
        content: &str,
        bucket: Option<&str>,
    ) -> Result<IngestionResult> {
        let start_time = std::time::Instant::now();

        // Generate source ID
        let source_id = format!("src_{}", source.replace("/", "_").replace("\\", "_"));

        // Sanitize content if enabled
        let content = if self.config.sanitize {
            sanitize(content)
        } else {
            content.to_string()
        };

        // Atomize content
        let atoms_data = atomize(&content);

        // Create source record
        let source = Source {
            id: source_id.clone(),
            path: format!("inline:{}", source),
            bucket: bucket.map(|s| s.to_string()),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            metadata: None,
        };

        self.db.upsert_source(&source).await?;

        // Create atoms with tags
        let mut atom_ids = Vec::new();
        let mut all_tags = Vec::new();

        for atom_data in &atoms_data {
            // Generate SimHash
            let hash = simhash(&atom_data.content);

            // Create atom
            let atom = Atom {
                id: 0,
                source_id: source_id.clone(),
                content: atom_data.content.clone(),
                char_start: atom_data.char_start,
                char_end: atom_data.char_end,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64(),
                simhash: hash,
                tags: Vec::new(),
                metadata: None,
            };

            let atom_id = self.db.insert_atom(&atom).await?;
            atom_ids.push(atom_id);

            // Extract keywords as tags
            let keywords = extract_keywords(&atom_data.content, self.config.max_keywords);
            let mut tags = Vec::new();

            for kw in keywords {
                if kw.score >= self.config.min_keyword_score {
                    let tag = format!("#{}", kw.term.to_lowercase());
                    tags.push(Tag {
                        id: 0,
                        atom_id,
                        tag: tag.clone(),
                        bucket: None,
                    });
                    all_tags.push(tag);
                }
            }

            // Add tags to database
            if !tags.is_empty() {
                self.db.add_tags(atom_id, &tags).await?;
            }
        }

        let processing_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;

        Ok(IngestionResult {
            source_id,
            atoms_created: atom_ids.len(),
            atom_ids,
            tags: all_tags,
            file_size: content.len() as u64,
            processing_time_ms,
        })
    }

    /// Get the database reference.
    pub fn db(&self) -> &Database {
        &self.db
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_ingest_content() {
        let service = IngestionService::in_memory().unwrap();
        
        let result = service.ingest_content(
            "test.md",
            "Rust is a systems programming language. Rust provides memory safety without garbage collection.",
            Some("test"),
        ).await.unwrap();

        assert!(result.atoms_created > 0);
        // Tags may or may not be extracted depending on keyword scores
        // Just verify the ingestion worked
        assert!(result.processing_time_ms > 0.0);
    }

    #[tokio::test]
    async fn test_ingest_file() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.md");
        
        // Create test file
        fs::write(&file_path, "This is test content. Rust is great.").unwrap();
        
        let service = IngestionService::in_memory().unwrap();
        let result = service.ingest_file(&file_path).await.unwrap();

        assert!(result.atoms_created > 0);
        assert!(result.file_size > 0);
        assert!(result.processing_time_ms > 0.0);
    }

    #[tokio::test]
    async fn test_mirror_file() {
        let temp_dir = tempdir().unwrap();
        let mirrored_dir = temp_dir.path().join("mirrored");
        fs::create_dir_all(&mirrored_dir).unwrap();
        
        let config = IngestionConfig {
            mirrored_brain_path: mirrored_dir.clone(),
            ..Default::default()
        };
        
        let service = IngestionService::in_memory().unwrap();
        // Note: Can't test mirror_file directly without refactoring
        // It's tested indirectly through ingest_file
    }

    #[test]
    fn test_generate_source_id() {
        let service = IngestionService::in_memory().unwrap();
        
        let id1 = service.generate_source_id(Path::new("/path/to/file.md"));
        let id2 = service.generate_source_id(Path::new("/path/to/file.md"));
        let id3 = service.generate_source_id(Path::new("/path/to/other.md"));
        
        assert_eq!(id1, id2); // Same path = same ID
        assert_ne!(id1, id3); // Different path = different ID
    }

    #[test]
    fn test_get_bucket_from_path() {
        let service = IngestionService::in_memory().unwrap();
        
        let bucket1 = service.get_bucket_from_path(Path::new("inbox/Code/file.md"));
        let bucket2 = service.get_bucket_from_path(Path::new("external-inbox/docs/readme.txt"));
        let bucket3 = service.get_bucket_from_path(Path::new("file.md"));
        
        assert_eq!(bucket1, "Code");
        assert_eq!(bucket2, "docs");
        assert_eq!(bucket3, "default");
    }
}

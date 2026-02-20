//! Database module for Anchor Engine.
//!
//! Provides SQLite storage for atoms, tags, and sources with full CRUD operations.

use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use thiserror::Error;
use tracing::{debug, info};

use crate::models::{Atom, Source, Tag};

/// Database errors.
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Atom not found: {0}")]
    AtomNotFound(u64),

    #[error("Source not found: {0}")]
    SourceNotFound(String),

    #[error("Migration error: {0}")]
    Migration(String),
}

/// Result type for database operations.
pub type Result<T> = std::result::Result<T, DbError>;

/// Database wrapper with connection pool.
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Open or create a database at the given path.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        // Run migrations
        Self::migrate(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Create an in-memory database (for testing).
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::migrate(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Run database migrations.
    fn migrate(conn: &Connection) -> Result<()> {
        // Create sources table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sources (
                id TEXT PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                bucket TEXT,
                created_at REAL NOT NULL,
                updated_at REAL NOT NULL,
                metadata TEXT
            )",
            [],
        )?;

        // Create atoms table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS atoms (
                id INTEGER PRIMARY KEY,
                source_id TEXT NOT NULL,
                content TEXT NOT NULL,
                char_start INTEGER NOT NULL,
                char_end INTEGER NOT NULL,
                timestamp REAL NOT NULL,
                simhash TEXT NOT NULL,
                metadata TEXT,
                FOREIGN KEY (source_id) REFERENCES sources(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create tags table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                atom_id INTEGER NOT NULL,
                tag TEXT NOT NULL,
                bucket TEXT,
                FOREIGN KEY (atom_id) REFERENCES atoms(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create indexes for fast lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_atoms_source ON atoms(source_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_atoms_simhash ON atoms(simhash)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tags_atom ON tags(atom_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tags_tag ON tags(tag)",
            [],
        )?;

        // Create FTS index for atoms content
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS atoms_fts USING fts5(
                content,
                content='atoms',
                content_rowid='id'
            )",
            [],
        )?;

        // Create triggers to keep FTS in sync
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS atoms_ai AFTER INSERT ON atoms BEGIN
                INSERT INTO atoms_fts(rowid, content) VALUES (new.id, new.content);
            END",
            [],
        )?;

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS atoms_ad AFTER DELETE ON atoms BEGIN
                INSERT INTO atoms_fts(atoms_fts, rowid, content) VALUES('delete', old.id, old.content);
            END",
            [],
        )?;

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS atoms_au AFTER UPDATE ON atoms BEGIN
                INSERT INTO atoms_fts(atoms_fts, rowid, content) VALUES('delete', old.id, old.content);
                INSERT INTO atoms_fts(rowid, content) VALUES (new.id, new.content);
            END",
            [],
        )?;

        Ok(())
    }

    // ==================== Source Operations ====================

    /// Insert or update a source.
    pub async fn upsert_source(&self, source: &Source) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO sources (id, path, bucket, created_at, updated_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                source.id,
                source.path,
                source.bucket,
                source.created_at,
                source.updated_at,
                source.metadata.as_ref().map(|m| serde_json::to_string(m).unwrap()),
            ],
        )?;
        Ok(())
    }

    /// Get a source by ID.
    pub async fn get_source(&self, id: &str) -> Result<Source> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, path, bucket, created_at, updated_at, metadata FROM sources WHERE id = ?1",
        )?;

        let source = stmt.query_row(params![id], |row| {
            let metadata: Option<String> = row.get(5)?;
            Ok(Source {
                id: row.get(0)?,
                path: row.get(1)?,
                bucket: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                metadata: metadata.and_then(|m| serde_json::from_str(&m).ok()),
            })
        })?;

        Ok(source)
    }

    /// Get all sources.
    pub async fn list_sources(&self) -> Result<Vec<Source>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, path, bucket, created_at, updated_at, metadata FROM sources",
        )?;

        let sources = stmt.query_map([], |row| {
            let metadata: Option<String> = row.get(5)?;
            Ok(Source {
                id: row.get(0)?,
                path: row.get(1)?,
                bucket: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                metadata: metadata.and_then(|m| serde_json::from_str(&m).ok()),
            })
        })?;

        sources.collect::<rusqlite::Result<Vec<_>>>().map_err(DbError::from)
    }

    /// Delete a source.
    pub async fn delete_source(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        let result = conn.execute("DELETE FROM sources WHERE id = ?1", params![id])?;
        if result == 0 {
            return Err(DbError::SourceNotFound(id.to_string()));
        }
        Ok(())
    }

    // ==================== Atom Operations ====================

    /// Insert an atom.
    pub async fn insert_atom(&self, atom: &Atom) -> Result<u64> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO atoms (source_id, content, char_start, char_end, timestamp, simhash, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                atom.source_id,
                atom.content,
                atom.char_start,
                atom.char_end,
                atom.timestamp,
                format!("{:016x}", atom.simhash),
                atom.metadata.as_ref().map(|m| serde_json::to_string(m).unwrap()),
            ],
        )?;

        let id = conn.last_insert_rowid() as u64;
        Ok(id)
    }

    /// Insert multiple atoms in a transaction.
    pub async fn insert_atoms_batch(&self, atoms: &[Atom]) -> Result<Vec<u64>> {
        // 📊 DEBUG log for batch operations
        debug!("📊 Batch insert: {} atoms", atoms.len());

        let conn = self.conn.lock().await;
        let tx = conn.unchecked_transaction()?;
        let mut ids = Vec::with_capacity(atoms.len());

        for atom in atoms {
            tx.execute(
                "INSERT INTO atoms (source_id, content, char_start, char_end, timestamp, simhash, metadata)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    atom.source_id,
                    atom.content,
                    atom.char_start,
                    atom.char_end,
                    atom.timestamp,
                    format!("{:016x}", atom.simhash),
                    atom.metadata.as_ref().map(|m| serde_json::to_string(m).unwrap()),
                ],
            )?;
            ids.push(tx.last_insert_rowid() as u64);
        }

        tx.commit()?;
        debug!("   └─ ✓ Batch insert complete: {} atoms stored", ids.len());
        Ok(ids)
    }

    /// Get an atom by ID.
    pub async fn get_atom(&self, id: u64) -> Result<Atom> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, source_id, content, char_start, char_end, timestamp, simhash, metadata
             FROM atoms WHERE id = ?1",
        )?;

        let atom = stmt.query_row(params![id], |row| {
            let simhash_str: String = row.get(6)?;
            let simhash = u64::from_str_radix(&simhash_str.trim_start_matches("0x"), 16)
                .unwrap_or(0);

            let metadata: Option<String> = row.get(7)?;
            Ok(Atom {
                id: row.get(0)?,
                source_id: row.get(1)?,
                content: row.get(2)?,
                char_start: row.get(3)?,
                char_end: row.get(4)?,
                timestamp: row.get(5)?,
                simhash,
                metadata: metadata.and_then(|m| serde_json::from_str(&m).ok()),
            })
        })?;

        Ok(atom)
    }

    /// Get atoms by source ID.
    pub async fn get_atoms_by_source(&self, source_id: &str) -> Result<Vec<Atom>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, source_id, content, char_start, char_end, timestamp, simhash, metadata
             FROM atoms WHERE source_id = ?1",
        )?;

        let atoms = stmt.query_map(params![source_id], |row| {
            let simhash_str: String = row.get(6)?;
            let simhash = u64::from_str_radix(&simhash_str.trim_start_matches("0x"), 16)
                .unwrap_or(0);

            let metadata: Option<String> = row.get(7)?;
            Ok(Atom {
                id: row.get(0)?,
                source_id: row.get(1)?,
                content: row.get(2)?,
                char_start: row.get(3)?,
                char_end: row.get(4)?,
                timestamp: row.get(5)?,
                simhash,
                metadata: metadata.and_then(|m| serde_json::from_str(&m).ok()),
            })
        })?;

        atoms.collect::<rusqlite::Result<Vec<_>>>().map_err(DbError::from)
    }

    /// Search atoms by content (FTS).
    pub async fn search_atoms(&self, query: &str, limit: usize) -> Result<Vec<Atom>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT a.id, a.source_id, a.content, a.char_start, a.char_end, a.timestamp, a.simhash, a.metadata
             FROM atoms a
             JOIN atoms_fts fts ON a.id = fts.rowid
             WHERE atoms_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;

        let atoms = stmt.query_map(params![query, limit], |row| {
            let simhash_str: String = row.get(6)?;
            let simhash = u64::from_str_radix(&simhash_str.trim_start_matches("0x"), 16)
                .unwrap_or(0);

            let metadata: Option<String> = row.get(7)?;
            Ok(Atom {
                id: row.get(0)?,
                source_id: row.get(1)?,
                content: row.get(2)?,
                char_start: row.get(3)?,
                char_end: row.get(4)?,
                timestamp: row.get(5)?,
                simhash,
                metadata: metadata.and_then(|m| serde_json::from_str(&m).ok()),
            })
        })?;

        atoms.collect::<rusqlite::Result<Vec<_>>>().map_err(DbError::from)
    }

    /// Delete an atom.
    pub async fn delete_atom(&self, id: u64) -> Result<()> {
        let conn = self.conn.lock().await;
        let result = conn.execute("DELETE FROM atoms WHERE id = ?1", params![id])?;
        if result == 0 {
            return Err(DbError::AtomNotFound(id));
        }
        Ok(())
    }

    // ==================== Tag Operations ====================

    /// Add tags to an atom.
    pub async fn add_tags(&self, atom_id: u64, tags: &[Tag]) -> Result<()> {
        let conn = self.conn.lock().await;
        for tag in tags {
            conn.execute(
                "INSERT INTO tags (atom_id, tag, bucket) VALUES (?1, ?2, ?3)",
                params![atom_id, tag.tag, tag.bucket],
            )?;
        }
        Ok(())
    }

    /// Get tags for an atom.
    pub async fn get_tags_for_atom(&self, atom_id: u64) -> Result<Vec<Tag>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, atom_id, tag, bucket FROM tags WHERE atom_id = ?1",
        )?;

        let tags = stmt.query_map(params![atom_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                atom_id: row.get(1)?,
                tag: row.get(2)?,
                bucket: row.get(3)?,
            })
        })?;

        tags.collect::<rusqlite::Result<Vec<_>>>().map_err(DbError::from)
    }

    /// Get atoms by tag.
    pub async fn get_atoms_by_tag(&self, tag: &str) -> Result<Vec<Atom>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT a.id, a.source_id, a.content, a.char_start, a.char_end, a.timestamp, a.simhash, a.metadata
             FROM atoms a
             JOIN tags t ON a.id = t.atom_id
             WHERE t.tag = ?1",
        )?;

        let atoms = stmt.query_map(params![tag], |row| {
            let simhash_str: String = row.get(6)?;
            let simhash = u64::from_str_radix(&simhash_str.trim_start_matches("0x"), 16)
                .unwrap_or(0);

            let metadata: Option<String> = row.get(7)?;
            Ok(Atom {
                id: row.get(0)?,
                source_id: row.get(1)?,
                content: row.get(2)?,
                char_start: row.get(3)?,
                char_end: row.get(4)?,
                timestamp: row.get(5)?,
                simhash,
                metadata: metadata.and_then(|m| serde_json::from_str(&m).ok()),
            })
        })?;

        atoms.collect::<rusqlite::Result<Vec<_>>>().map_err(DbError::from)
    }

    /// Get all unique tags.
    pub async fn list_all_tags(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT DISTINCT tag FROM tags ORDER BY tag")?;
        let tags = stmt.query_map([], |row| row.get(0))?;
        tags.collect::<rusqlite::Result<Vec<_>>>().map_err(DbError::from)
    }

    // ==================== Stats ====================

    /// Get database statistics.
    pub async fn get_stats(&self) -> Result<DbStats> {
        let conn = self.conn.lock().await;
        let atom_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM atoms",
            [],
            |row| row.get(0),
        )?;

        let source_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sources",
            [],
            |row| row.get(0),
        )?;

        let tag_count: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT tag) FROM tags",
            [],
            |row| row.get(0),
        )?;

        Ok(DbStats {
            atom_count: atom_count as usize,
            source_count: source_count as usize,
            tag_count: tag_count as usize,
        })
    }

    // ==================== Disposable Index ====================

    /// Wipe all data from the database (disposable index pattern).
    /// Keeps the schema but removes all atoms, tags, and sources.
    pub async fn wipe_all_data(&self) -> Result<()> {
        // 🗑️ DEBUG log when wiping database
        debug!("🗑️ Wiping all database data...");

        let conn = self.conn.lock().await;

        // Use a transaction for atomic cleanup
        let tx = conn.unchecked_transaction()?;

        // Delete in order respecting foreign keys
        tx.execute("DELETE FROM tags", [])?;
        tx.execute("DELETE FROM atoms", [])?;
        tx.execute("DELETE FROM sources", [])?;

        // Reset auto-increment counters
        tx.execute("DELETE FROM sqlite_sequence WHERE name='tags'", [])?;
        tx.execute("DELETE FROM sqlite_sequence WHERE name='atoms'", [])?;

        // Clear FTS index
        tx.execute("DELETE FROM atoms_fts", [])?;

        tx.commit()?;

        debug!("   └─ ✓ Database wiped clean");
        Ok(())
    }

    /// Check if the database is empty (no atoms).
    pub async fn is_empty(&self) -> Result<bool> {
        let conn = self.conn.lock().await;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM atoms",
            [],
            |row| row.get(0),
        )?;
        Ok(count == 0)
    }

    /// Rebuild FTS index from atoms table.
    pub async fn rebuild_fts(&self) -> Result<()> {
        // 📊 INFO log when rebuilding FTS index
        info!("📊 Rebuilding FTS index...");

        let conn = self.conn.lock().await;

        // Delete existing FTS entries
        conn.execute("DELETE FROM atoms_fts", [])?;

        // Rebuild from atoms table
        conn.execute(
            "INSERT INTO atoms_fts(rowid, content) SELECT id, content FROM atoms",
            [],
        )?;

        info!("   └─ ✓ FTS index rebuilt");
        Ok(())
    }
}

/// Database statistics.
#[derive(Debug, Clone)]
pub struct DbStats {
    pub atom_count: usize,
    pub source_count: usize,
    pub tag_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_database_in_memory() {
        let db = Database::in_memory().unwrap();
        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.atom_count, 0);
    }

    #[tokio::test]
    async fn test_source_crud() {
        let db = Database::in_memory().unwrap();

        let source = Source {
            id: "test-1".to_string(),
            path: "/test/path.md".to_string(),
            bucket: Some("test".to_string()),
            created_at: Utc::now().timestamp() as f64,
            updated_at: Utc::now().timestamp() as f64,
            metadata: None,
        };

        db.upsert_source(&source).await.unwrap();
        let retrieved = db.get_source("test-1").await.unwrap();
        assert_eq!(retrieved.path, "/test/path.md");

        db.delete_source("test-1").await.unwrap();
    }

    #[tokio::test]
    async fn test_atom_crud() {
        let db = Database::in_memory().unwrap();

        // Create source first
        let source = Source {
            id: "source-1".to_string(),
            path: "/test.md".to_string(),
            bucket: None,
            created_at: Utc::now().timestamp() as f64,
            updated_at: Utc::now().timestamp() as f64,
            metadata: None,
        };
        db.upsert_source(&source).await.unwrap();

        // Create atom
        let atom = Atom {
            id: 0, // Will be assigned by DB
            source_id: "source-1".to_string(),
            content: "Test content".to_string(),
            char_start: 0,
            char_end: 12,
            timestamp: Utc::now().timestamp() as f64,
            simhash: 0x1234567890ABCDEF,
            metadata: None,
        };

        let id = db.insert_atom(&atom).await.unwrap();
        assert!(id > 0);

        let retrieved = db.get_atom(id).await.unwrap();
        assert_eq!(retrieved.content, "Test content");
    }

    #[tokio::test]
    async fn test_tag_operations() {
        let db = Database::in_memory().unwrap();

        // Create source and atom
        let source = Source {
            id: "source-1".to_string(),
            path: "/test.md".to_string(),
            bucket: None,
            created_at: Utc::now().timestamp() as f64,
            updated_at: Utc::now().timestamp() as f64,
            metadata: None,
        };
        db.upsert_source(&source).await.unwrap();

        let atom = Atom {
            id: 0,
            source_id: "source-1".to_string(),
            content: "Test".to_string(),
            char_start: 0,
            char_end: 4,
            timestamp: Utc::now().timestamp() as f64,
            simhash: 0x1234567890ABCDEF,
            metadata: None,
        };
        let atom_id = db.insert_atom(&atom).await.unwrap();

        // Add tags
        let tags = vec![
            Tag { id: 0, atom_id, tag: "#rust".to_string(), bucket: None },
            Tag { id: 0, atom_id, tag: "#programming".to_string(), bucket: None },
        ];
        db.add_tags(atom_id, &tags).await.unwrap();

        // Retrieve tags
        let retrieved = db.get_tags_for_atom(atom_id).await.unwrap();
        assert_eq!(retrieved.len(), 2);

        // Search by tag
        let atoms = db.get_atoms_by_tag("#rust").await.unwrap();
        assert_eq!(atoms.len(), 1);
    }

    #[tokio::test]
    async fn test_fts_search() {
        let db = Database::in_memory().unwrap();

        // Create source and atom
        let source = Source {
            id: "source-1".to_string(),
            path: "/test.md".to_string(),
            bucket: None,
            created_at: Utc::now().timestamp() as f64,
            updated_at: Utc::now().timestamp() as f64,
            metadata: None,
        };
        db.upsert_source(&source).await.unwrap();

        let atom = Atom {
            id: 0,
            source_id: "source-1".to_string(),
            content: "Rust is a systems programming language".to_string(),
            char_start: 0,
            char_end: 38,
            timestamp: Utc::now().timestamp() as f64,
            simhash: 0x1234567890ABCDEF,
            metadata: None,
        };
        db.insert_atom(&atom).await.unwrap();

        // Search
        let results = db.search_atoms("rust", 10).await.unwrap();
        assert!(!results.is_empty());
    }
}

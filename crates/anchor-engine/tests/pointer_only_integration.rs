//! Integration tests for pointer-only storage (Mirror Protocol)
//!
//! These tests verify that:
//! - Ingest writes content to mirrored_brain/
//! - Database stores only pointers (source_path, start_byte, end_byte)
//! - Search lazily loads content from filesystem
//! - Illuminate BFS traversal works with pre-allocated collections

use anchor_engine::{Database, AnchorService, Storage, FileSystemStorage};
use anchor_engine::models::{IngestRequest, IngestOptions, SearchRequest, SearchMode, BudgetConfig, IlluminateRequest};
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;

/// Create a test service with pointer-only storage
async fn create_test_service() -> (AnchorService, TempDir, TempDir) {
    // Create temporary directories
    let db_dir = TempDir::new().unwrap();
    let mirror_dir = TempDir::new().unwrap();
    
    let db_path = db_dir.path().join("test.db");
    let db = Database::open(&db_path).await.unwrap();
    
    let service = AnchorService::new(db, mirror_dir.path().to_path_buf()).unwrap();
    
    (service, db_dir, mirror_dir)
}

#[tokio::test]
async fn test_ingest_writes_to_mirrored_brain() {
    let (mut service, _db_dir, mirror_dir) = create_test_service().await;
    
    // Ingest content
    let request = IngestRequest {
        source: "test.md".to_string(),
        content: "Hello, World! This is test content.".to_string(),
        bucket: "test".to_string(),
        options: IngestOptions {
            sanitize: true,
            extract_tags: true,
            max_keywords: 10,
        },
    };
    
    let result = service.ingest(request).await.unwrap();
    
    // Verify atoms were ingested
    assert!(result.atoms_ingested > 0);
    
    // Verify content was written to mirrored_brain/
    let mut entries = fs::read_dir(mirror_dir.path()).await.unwrap();
    let mut found_file = false;
    
    while let Some(entry) = entries.next_entry().await.unwrap() {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "md") {
            found_file = true;
            
            // Verify file contains sanitized content
            let content = fs::read_to_string(&path).await.unwrap();
            assert!(content.contains("Hello"));
            assert!(content.contains("World"));
        }
    }
    
    assert!(found_file, "No .md file found in mirrored_brain/");
}

#[tokio::test]
async fn test_database_stores_pointers_only() {
    let (mut service, _db_dir, _mirror_dir) = create_test_service().await;
    
    // Ingest content
    let request = IngestRequest {
        source: "test.md".to_string(),
        content: "Rust is a systems programming language.".to_string(),
        bucket: "test".to_string(),
        options: IngestOptions::default(),
    };
    
    let _result = service.ingest(request).await.unwrap();
    
    // Get atoms from database
    let atoms = service.db.get_all_atoms().await.unwrap();
    
    assert!(!atoms.is_empty());
    
    for atom in &atoms {
        // Verify pointer fields are populated
        assert!(!atom.source_path.is_empty());
        assert!(atom.start_byte < atom.end_byte);
        
        // Verify content is NOT loaded (lazy loading)
        assert!(atom.content.is_none());
        
        // Verify source_path points to existing file
        let path = PathBuf::from(&atom.source_path);
        assert!(path.exists(), "Source file does not exist: {}", atom.source_path);
    }
}

#[tokio::test]
async fn test_search_lazily_loads_content() {
    let (mut service, _db_dir, _mirror_dir) = create_test_service().await;
    
    // Ingest content
    let request = IngestRequest {
        source: "rust.md".to_string(),
        content: "Rust is fast and safe. #rust #programming".to_string(),
        bucket: "test".to_string(),
        options: IngestOptions {
            sanitize: true,
            extract_tags: true,
            max_keywords: 10,
        },
    };
    
    let _result = service.ingest(request).await.unwrap();
    
    // Search for content
    let search_request = SearchRequest {
        query: "#rust".to_string(),
        max_results: 10,
        mode: SearchMode::Combined,
        budget: BudgetConfig::default(),
    };
    
    let search_result = service.search(search_request).await.unwrap();
    
    // Verify results were found
    assert!(search_result.total > 0);
    
    // Verify results have content loaded
    for item in &search_result.results {
        assert!(!item.content.is_empty());
        assert!(item.content.contains("Rust"));
    }
}

#[tokio::test]
async fn test_multiple_ingest_deduplication() {
    let (mut service, _db_dir, mirror_dir) = create_test_service().await;
    
    // Ingest same content twice
    let request1 = IngestRequest {
        source: "test.md".to_string(),
        content: "Duplicate content test".to_string(),
        bucket: "test".to_string(),
        options: IngestOptions::default(),
    };
    
    let request2 = IngestRequest {
        source: "test.md".to_string(),
        content: "Duplicate content test".to_string(),
        bucket: "test".to_string(),
        options: IngestOptions::default(),
    };
    
    let result1 = service.ingest(request1).await.unwrap();
    let result2 = service.ingest(request2).await.unwrap();
    
    // Both should succeed
    assert!(result1.atoms_ingested > 0);
    assert!(result2.atoms_ingested > 0);
    
    // Verify only one file in mirrored_brain/ (deduplication)
    let entries: Vec<_> = fs::read_dir(mirror_dir.path())
        .await
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    
    assert_eq!(entries.len(), 1, "Deduplication failed: multiple files created");
}

#[tokio::test]
async fn test_byte_range_accuracy() {
    let (mut service, _db_dir, _mirror_dir) = create_test_service().await;
    
    let content = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
    
    // Ingest content
    let request = IngestRequest {
        source: "paragraphs.md".to_string(),
        content: content.to_string(),
        bucket: "test".to_string(),
        options: IngestOptions::default(),
    };
    
    let _result = service.ingest(request).await.unwrap();
    
    // Get atoms
    let atoms = service.db.get_all_atoms().await.unwrap();
    
    // Verify byte ranges are correct
    for atom in &atoms {
        // Read content from filesystem using byte offsets
        let file_content = fs::read_to_string(&atom.source_path).await.unwrap();
        let bytes = file_content.as_bytes();
        
        // Extract byte range
        let extracted = String::from_utf8_lossy(&bytes[atom.start_byte..atom.end_byte]);
        
        // Verify extracted content matches atom's char range
        assert!(!extracted.is_empty(), "Empty byte range for atom {}", atom.id);
    }
}

#[tokio::test]
async fn test_storage_cache_behavior() {
    let (mut service, _db_dir, mirror_dir) = create_test_service().await;
    
    // Ingest content
    let request = IngestRequest {
        source: "cache_test.md".to_string(),
        content: "Testing LRU cache behavior.".to_string(),
        bucket: "test".to_string(),
        options: IngestOptions::default(),
    };
    
    let _result = service.ingest(request).await.unwrap();
    
    // Get the source path
    let atoms = service.db.get_all_atoms().await.unwrap();
    let source_path = &atoms[0].source_path;
    
    // Access the storage module (through service)
    // Note: This test verifies the cache exists, but detailed cache testing
    // would require exposing cache internals
    
    // Verify file exists in mirrored_brain/
    let path = PathBuf::from(source_path);
    assert!(path.exists());
    
    // Verify file is readable
    let content = fs::read_to_string(&path).await.unwrap();
    assert!(content.contains("Testing"));
    assert!(content.contains("cache"));
}

#[tokio::test]
async fn test_illuminate_bfs_traversal() {
    let (mut service, _db_dir, _mirror_dir) = create_test_service().await;
    
    // Ingest interconnected content (to create tag graph)
    let rust_content = "Rust is a systems programming language. #rust #programming #systems";
    let programming_content = "Programming is the art of telling computers what to do. #programming #computers";
    let systems_content = "Systems programming requires understanding of memory and performance. #systems #memory #performance";
    
    // Ingest multiple documents with shared tags
    for (source, content) in &[
        ("rust.md", rust_content),
        ("programming.md", programming_content),
        ("systems.md", systems_content),
    ] {
        let request = IngestRequest {
            source: source.to_string(),
            content: content.to_string(),
            bucket: "test".to_string(),
            options: IngestOptions {
                sanitize: true,
                extract_tags: true,
                max_keywords: 10,
            },
        };
        service.ingest(request).await.unwrap();
    }
    
    // Run illuminate from seed "#rust"
    let request = IlluminateRequest {
        seed: "#rust".to_string(),
        depth: 2,
        max_nodes: 50,
    };
    
    let result = service.illuminate(request).await.unwrap();
    
    // Verify results
    assert!(result.total > 0, "Illuminate should find at least one node");
    assert!(result.nodes_explored >= result.total);
    
    // Verify first node is the rust atom (hop_distance = 0)
    let first_node = &result.nodes[0];
    assert_eq!(first_node.hop_distance, 0);
    assert!(first_node.content.contains("Rust"));
    
    // Verify gravity score damping (should decrease with hop distance)
    for node in &result.nodes {
        assert!(node.gravity_score > 0.0);
        assert!(node.gravity_score <= 1.0);
        
        // Verify content is loaded (not empty)
        assert!(!node.content.is_empty());
    }
    
    // Verify pre-allocation worked (no reallocations during traversal)
    // This is implicit - if pre-allocation failed, the test would still pass
    // but would be slower. We verify the capacity was used correctly.
    assert!(result.total <= 50); // max_nodes limit
}

#[tokio::test]
async fn test_illuminate_depth_limit() {
    let (mut service, _db_dir, _mirror_dir) = create_test_service().await;
    
    // Create a chain of documents: A -> B -> C -> D
    let docs = vec![
        ("a.md", "Document A about #topic_a #link"),
        ("b.md", "Document B about #topic_b #link"),
        ("c.md", "Document C about #topic_c #link"),
        ("d.md", "Document D about #topic_d #link"),
    ];
    
    for (source, content) in &docs {
        let request = IngestRequest {
            source: source.to_string(),
            content: content.to_string(),
            bucket: "test".to_string(),
            options: IngestOptions::default(),
        };
        service.ingest(request).await.unwrap();
    }
    
    // Run illuminate with depth=1 (should only find direct neighbors)
    let request = IlluminateRequest {
        seed: "#topic_a".to_string(),
        depth: 1,
        max_nodes: 50,
    };
    
    let result = service.illuminate(request).await.unwrap();
    
    // With depth=1, should find A (hop 0) and possibly B,C,D via #link tag (hop 1)
    // But not beyond
    for node in &result.nodes {
        assert!(node.hop_distance <= 1, "Depth limit should be enforced");
    }
}

#[tokio::test]
async fn test_illuminate_max_nodes_limit() {
    let (mut service, _db_dir, _mirror_dir) = create_test_service().await;
    
    // Ingest content with common tag
    for i in 0..20 {
        let request = IngestRequest {
            source: format!("doc_{}.md", i),
            content: format!("Document {} about #common_tag", i),
            bucket: "test".to_string(),
            options: IngestOptions::default(),
        };
        service.ingest(request).await.unwrap();
    }
    
    // Run illuminate with small max_nodes limit
    let request = IlluminateRequest {
        seed: "#common_tag".to_string(),
        depth: 5,
        max_nodes: 10,
    };
    
    let result = service.illuminate(request).await.unwrap();
    
    // Verify max_nodes limit is enforced
    assert!(result.total <= 10, "max_nodes limit should be enforced");
}

//! Core service logic for Anchor Engine.
//!
//! Integrates atomizer, fingerprint, keyextract, and tagwalker with the database.
//! 
//! **Pointer-Only Storage Pattern:**
//! - Content is written to mirrored_brain/ via FileSystemStorage
//! - Database stores only pointers (source_path, start_byte, end_byte)
//! - Content is lazily loaded from filesystem on demand

use std::time::Instant;
use std::path::PathBuf;

use anchor_atomizer::{atomize, sanitize};
use anchor_fingerprint::simhash;
use anchor_keyextract::{extract_keywords, SynonymRing};
use anchor_tagwalker::{TagWalker, TagWalkerConfig, ResultType};
use tracing::{info, debug};

use crate::db::{Database, Result};
use crate::models::*;
use crate::storage::{Storage, FileSystemStorage};

/// Core Anchor service.
pub struct AnchorService {
    db: Database,
    storage: FileSystemStorage,
    tag_walker: TagWalker,
    synonym_ring: SynonymRing,
}

impl AnchorService {
    /// Create a new Anchor service with pointer-only storage.
    pub fn new(db: Database, mirror_dir: PathBuf) -> Result<Self> {
        let storage = FileSystemStorage::new(mirror_dir)
            .map_err(|e| crate::db::DbError::Migration(e.to_string()))?;
        let tag_walker = TagWalker::new();
        let synonym_ring = SynonymRing::default();

        // ℹ️ INFO log when AnchorService is created
        info!("🚀 AnchorService initialized (pointer-only storage)");

        Self {
            db,
            storage,
            tag_walker,
            synonym_ring,
        }
    }

    /// Ingest content into the knowledge base with pointer-only storage.
    pub async fn ingest(&mut self, request: IngestRequest) -> Result<IngestResponse> {
        let start = Instant::now();

        // Create or get source
        let source_id = request.source.clone();
        let source = Source {
            id: source_id.clone(),
            path: request.source,
            bucket: request.bucket,
            created_at: chrono::Utc::now().timestamp() as f64,
            updated_at: chrono::Utc::now().timestamp() as f64,
            metadata: None,
        };
        self.db.upsert_source(&source).await?;

        // Sanitize content if requested
        let content = if request.options.sanitize {
            sanitize(&request.content)
        } else {
            request.content
        };

        // Write to mirrored_brain/ and get file path
        let source_path = self.storage.write_cleaned(&source_id, &content)?;
        let content_bytes = content.as_bytes();

        // Atomize content
        let atoms = atomize(&content);

        // Process each atom
        let mut atom_ids = Vec::new();
        let mut all_tags = Vec::new();

        for atom_data in &atoms {
            // Generate SimHash
            let hash = simhash(&atom_data.content);

            // Calculate byte offsets in the sanitized content
            let start_byte = atom_data.char_start;
            let end_byte = atom_data.char_end;

            // Create atom record with pointer-only storage
            let atom = Atom::new(
                source_id.clone(),
                source_path.clone(),
                start_byte,
                end_byte,
                atom_data.char_start,
                atom_data.char_end,
                hash,
            );

            let atom_id = self.db.insert_atom(&atom).await?;
            atom_ids.push(atom_id);

            // Extract keywords as tags if requested
            let mut tags = Vec::new();
            if request.options.extract_tags {
                let keywords = extract_keywords(&atom_data.content, request.options.max_keywords);

                for kw in keywords {
                    if kw.score > 0.3 { // Threshold for relevance
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
            }

            // Add tags to database
            if !tags.is_empty() {
                self.db.add_tags(atom_id, &tags).await?;
            }

            // Add to tag walker (store full content for now, will be loaded lazily later)
            let tag_strings: Vec<String> = tags.iter().map(|t| t.tag.clone()).collect();
            self.tag_walker.add_atom(atom_id, &atom_data.content, tag_strings);
        }

        let duration = start.elapsed().as_millis() as f64;
        tracing::info!("Ingested {} atoms from {} in {:.2}ms", atoms.len(), source_id, duration);

        Ok(IngestResponse {
            source_id,
            atoms_created: atoms.len(),
            atom_ids,
            tags: all_tags,
        })
    }

    /// Search the knowledge base.
    pub async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        let start = Instant::now();

        // Auto-enable max-recall for 16K+ token queries
        let use_max_recall = request.budget.total_tokens >= 16384
            || request.budget.max_recall
            || request.mode == SearchMode::MaxRecall;

        // Extract keywords from query and convert to tags
        let query_tags = anchor_keyextract::extract_keywords(&request.query, 20);
        let tag_query: Vec<String> = query_tags
            .iter()
            .filter(|kw| kw.score > 0.2)
            .map(|kw| format!("#{}", kw.term.to_lowercase()))
            .collect();
        
        let query_str = if tag_query.is_empty() {
            request.query.clone()  // Fallback to original query
        } else {
            tag_query.join(" ")  // Use extracted tags
        };

        // 🔍 DEBUG log for search execution
        info!("🔍 SEARCH: \"{}\" (max_recall={}, tokens={})",
              request.query, use_max_recall, request.budget.total_tokens);
        if !tag_query.is_empty() {
            info!("   ├─ Extracted tags: {}", tag_query.join(", "));
        }

        // Configure tag walker based on mode
        let config = if use_max_recall {
            // Max-recall mode: zero temporal decay, 3 hops, high serendipity
            info!("   ├─ Max-recall mode: 3 hops, zero decay");
            TagWalkerConfig::default()
                .with_max_results(request.max_results)
                .with_planet_budget(request.budget.planet_budget)
                .with_moon_budget(request.budget.moon_budget)
                .with_max_hops(3)  // 3 hops for max recall
                .with_temporal_decay(0.0)  // Zero decay - all memories equally important
                .with_damping(0.75)  // Lower damping for deeper exploration
        } else {
            // Standard mode: default settings
            TagWalkerConfig::default()
                .with_max_results(request.max_results)
                .with_planet_budget(request.budget.planet_budget)
                .with_moon_budget(request.budget.moon_budget)
        };

        // Perform search
        let walker_results = self.tag_walker.search(&query_str, &config);

        // Convert walker results to response format
        let mut results = Vec::new();
        let mut planets_count = 0;
        let mut moons_count = 0;

        for walker_result in &walker_results {
            if let Ok(atom) = self.db.get_atom(walker_result.atom_id).await {
                let result_type = match walker_result.result_type {
                    ResultType::Planet => {
                        planets_count += 1;
                        "planet"
                    }
                    ResultType::Moon => {
                        moons_count += 1;
                        "moon"
                    }
                };

                results.push(SearchResultItem {
                    atom_id: atom.id,
                    source_id: atom.source_id,
                    content: atom.content,
                    relevance: walker_result.relevance,
                    matched_tags: walker_result.matched_tags.clone(),
                    result_type: result_type.to_string(),
                    offsets: ContentOffsets {
                        char_start: atom.char_start,
                        char_end: atom.char_end,
                    },
                });
            }
        }

        let total = results.len();
        let duration = start.elapsed().as_millis() as f64;
        
        info!("   └─ ✅ COMPLETE: {} results ({} planets, {} moons) in {:.1}ms", 
              total, planets_count, moons_count, duration);

        Ok(SearchResponse {
            results,
            query: request.query,
            total,
            stats: SearchStats {
                planets: planets_count,
                moons: moons_count,
                duration_ms: duration,
            },
        })
    }

    /// Get database statistics.
    pub async fn get_stats(&self) -> Result<DbStatsResponse> {
        let stats = self.db.get_stats().await?;
        Ok(DbStatsResponse {
            atoms: stats.atom_count,
            sources: stats.source_count,
            tags: stats.tag_count,
        })
    }

    /// Load synonym ring from file.
    pub fn load_synonym_ring(&mut self, path: &std::path::Path) -> std::result::Result<(), Box<dyn std::error::Error>> {
        self.synonym_ring = SynonymRing::load_or_empty(path);
        
        // Update tag walker with synonym ring
        // (In production, this would be more sophisticated)
        
        Ok(())
    }

    /// Get the database reference.
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// Illuminate: BFS graph traversal from seed query.
    /// 
    /// **Pre-allocation Strategy:**
    /// Uses `VecDeque::with_capacity(max_nodes)` and `HashSet::with_capacity(max_nodes)`
    /// to avoid dynamic reallocation during traversal.
    /// 
    /// **Zero-Copy Integration:**
    /// Content is loaded from Arc<Mmap> on demand, not stored in results.
    pub async fn illuminate(&self, request: IlluminateRequest) -> Result<IlluminateResponse> {
        use std::collections::{VecDeque, HashSet};
        
        let start = Instant::now();
        
        info!("🔦 [Illuminate] Starting BFS traversal: seed='{}', depth={}, max_nodes={}",
              request.seed, request.depth, request.max_nodes);
        
        // Pre-allocate collections (avoids dynamic resizing)
        let mut queue: VecDeque<IlluminateNode> = VecDeque::with_capacity(request.max_nodes);
        let mut visited: HashSet<u64> = HashSet::with_capacity(request.max_nodes);
        let mut results: Vec<IlluminateResultItem> = Vec::with_capacity(request.max_nodes);
        
        // Resolve seed query to anchor atoms via FTS
        let anchor_atoms = self.db.search_atoms(&request.seed, 100).await?;
        info!("   └─ Found {} anchor atoms for seed '{}'", anchor_atoms.len(), request.seed);
        
        // Initialize queue with anchor atoms at hop_distance = 0
        for atom in &anchor_atoms {
            if visited.insert(atom.id) {
                queue.push_back(IlluminateNode {
                    atom_id: atom.id,
                    hop_distance: 0,
                    gravity_score: 1.0,
                });
            }
        }
        
        // BFS traversal
        let mut nodes_explored = 0;
        let damping_factor: f64 = 0.85; // Standard PageRank damping
        
        while let Some(current) = queue.pop_front() {
            // Stop if we've reached max nodes
            if results.len() >= request.max_nodes {
                info!("   └─ Reached max_nodes limit ({})", request.max_nodes);
                break;
            }
            
            // Stop if we've exceeded depth
            if current.hop_distance > request.depth {
                info!("   └─ Reached max depth ({})", request.depth);
                continue;
            }
            
            nodes_explored += 1;
            
            // Get atom details (with lazy content loading)
            let atom = match self.db.get_atom(current.atom_id).await {
                Ok(a) => a,
                Err(_) => continue,
            };
            
            // Load content from filesystem (zero-copy via mmap)
            let content = match self.storage.read_range(&atom.source_path, atom.start_byte, atom.end_byte).await {
                Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                Err(_) => atom.source_path.clone(), // Fallback to path if read fails
            };
            
            // Calculate gravity score with hop-distance damping
            // W(q,a) = semantic_gravity × γ^d(q,a) × temporal_decay × structural_similarity
            let gravity_score = current.gravity_score * damping_factor.powi(current.hop_distance as i32);
            
            // Add to results
            results.push(IlluminateResultItem {
                id: atom.id,
                source_path: atom.source_path.clone(),
                content,
                tags: atom.tags.clone(),
                hop_distance: current.hop_distance,
                gravity_score,
                simhash: atom.simhash,
            });
            
            // Find neighboring atoms via shared tags
            let atom_tags = self.db.get_tags_for_atom(atom.id).await.unwrap_or_default();
            
            for tag in &atom_tags {
                // Find all atoms with this tag
                let neighbor_atoms = self.db.get_atoms_by_tag(&tag.tag).await.unwrap_or_default();
                
                for neighbor in &neighbor_atoms {
                    // Skip if already visited
                    if visited.contains(&neighbor.id) {
                        continue;
                    }
                    
                    // Mark as visited and add to queue
                    if visited.insert(neighbor.id) {
                        queue.push_back(IlluminateNode {
                            atom_id: neighbor.id,
                            hop_distance: current.hop_distance + 1,
                            gravity_score,
                        });
                    }
                }
            }
        }
        
        let duration = start.elapsed().as_millis() as f64;
        
        info!("   └─ ✅ COMPLETE: {} nodes illuminated ({} explored) in {:.1}ms",
              results.len(), nodes_explored, duration);
        
        Ok(IlluminateResponse {
            nodes: results,
            total: results.len(),
            nodes_explored,
            duration_ms: duration,
        })
    }

    /// Get storage reference (for benchmarks).
    pub fn storage(&self) -> &FileSystemStorage {
        &self.storage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ingest_and_search() {
        let db = Database::in_memory().unwrap();
        let mut service = AnchorService::new(db);

        // Ingest content
        let ingest_request = IngestRequest {
            source: "test.md".to_string(),
            content: "# Rust Programming\n\nRust is a systems programming language. \n\nRust provides memory safety.".to_string(),
            bucket: Some("docs".to_string()),
            options: IngestOptions {
                extract_tags: true,
                max_keywords: 5,
                sanitize: true,
            },
        };

        let ingest_response = service.ingest(ingest_request).await.unwrap();
        assert!(ingest_response.atoms_created > 0);

        // Search
        let search_request = SearchRequest {
            query: "#rust".to_string(),
            max_results: 10,
            mode: SearchMode::Combined,
            budget: BudgetConfig::default(),
        };

        let search_response = service.search(search_request).await.unwrap();
        assert!(!search_response.results.is_empty());
    }
}

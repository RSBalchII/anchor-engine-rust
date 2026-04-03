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
use tracing::info;

use crate::db::{Database, Result};
use crate::models::*;
use crate::storage::{Storage, FileSystemStorage};
use crate::config::Config;

/// Core Anchor service.
pub struct AnchorService {
    db: Database,
    storage: FileSystemStorage,
    tag_walker: TagWalker,
    synonym_ring: SynonymRing,
    config: Config,
}

impl AnchorService {
    /// Create a new Anchor service with pointer-only storage.
    pub fn new(db: Database, mirror_dir: PathBuf, config: Config) -> Result<Self> {
        let storage = FileSystemStorage::new(mirror_dir)
            .map_err(|e| crate::db::DbError::Migration(e.to_string()))?;
        let tag_walker = TagWalker::new();
        let synonym_ring = SynonymRing::default();

        // ℹ️ INFO log when AnchorService is created
        info!("🚀 AnchorService initialized (pointer-only storage)");

        Ok(Self {
            db,
            storage,
            tag_walker,
            synonym_ring,
            config,
        })
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
                let max_keywords = if request.options.max_keywords > 0 {
                    request.options.max_keywords
                } else {
                    self.config.ingestion.max_keywords
                };
                let keywords = extract_keywords(&atom_data.content, max_keywords);

                for keyword in keywords {
                    // Simple keyword extraction - just use the keyword string directly
                    let tag = format!("#{}", keyword.to_lowercase());
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
        let max_keywords = self.config.ingestion.max_keywords;
        let query_tags = anchor_keyextract::extract_keywords(&request.query, max_keywords);
        let tag_query: Vec<String> = query_tags
            .iter()
            .map(|kw| format!("#{}", kw.to_lowercase()))
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
        let max_results = if request.max_results > 0 {
            request.max_results
        } else {
            self.config.search.max_results
        };

        let config = if use_max_recall {
            // Max-recall mode: zero temporal decay, 3 hops, high serendipity
            info!("   ├─ Max-recall mode: 3 hops, zero decay");
            TagWalkerConfig::default()
                .with_max_results(max_results)
                .with_planet_budget(request.budget.planet_budget)
                .with_moon_budget(request.budget.moon_budget)
                .with_max_hops(3)  // 3 hops for max recall
                .with_temporal_decay(0.0)  // Zero decay - all memories equally important
                .with_damping(0.75)  // Lower damping for deeper exploration
        } else {
            // Standard mode: default settings
            TagWalkerConfig::default()
                .with_max_results(max_results)
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

                // Load content from filesystem
                let content = self.storage.read_range(&atom.source_path, atom.start_byte, atom.end_byte)
                    .unwrap_or_else(|_| atom.source_path.clone());

                results.push(SearchResultItem {
                    atom_id: atom.id,
                    source_id: atom.source_id,
                    content,
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
            let content = match self.storage.read_range(&atom.source_path, atom.start_byte, atom.end_byte) {
                Ok(content_str) => content_str,
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
        let total = results.len();

        info!("   └─ ✅ COMPLETE: {} nodes illuminated ({} explored) in {:.1}ms",
              total, nodes_explored, duration);

        Ok(IlluminateResponse {
            nodes: results,
            total,
            nodes_explored,
            duration_ms: duration,
        })
    }

    /// Get storage reference (for benchmarks).
    pub fn storage(&self) -> &FileSystemStorage {
        &self.storage
    }

    /// Distill: Radial distillation to compress knowledge into Decision Records.
    ///
    /// **Zero-Copy Streaming:**
    /// Content is loaded lazily from Arc<Mmap>, not all at once.
    /// Blocks are deduplicated via SimHash before assembly.
    ///
    /// **Algorithm:**
    /// 1. Find anchor atoms via seed query (FTS)
    /// 2. For each anchor, retrieve surrounding atoms within radius hops
    /// 3. Deduplicate blocks using SimHash
    /// 4. Group by source proximity for coherent Decision Records
    /// 5. Write output to distills/ directory
    pub async fn distill(&self, request: DistillRequest) -> Result<DistillResponse> {
        use std::collections::{HashMap, HashSet};
        use std::fs::{self, OpenOptions};
        use std::io::Write;

        let start = Instant::now();

        info!("🔮 [Distill] Starting radial distillation: seed='{:?}', radius={}",
              request.seed, request.radius);

        // Pre-allocate collections (avoids dynamic resizing)
        let mut visited: HashSet<u64> = HashSet::with_capacity(request.max_atoms.unwrap_or(1000));
        let mut blocks_by_source: HashMap<String, Vec<DistillBlock>> = HashMap::new();
        let mut seen_hashes: HashSet<u64> = HashSet::with_capacity(500);

        // Step 1: Find anchor atoms via seed query
        let seed_query = request.seed.as_deref().unwrap_or("#");
        let anchor_atoms = self.db.search_atoms(seed_query, 100).await?;
        info!("   └─ Found {} anchor atoms for seed '{}'", anchor_atoms.len(), seed_query);

        // Step 2: For each anchor, retrieve surrounding atoms within radius
        let mut total_atoms_collected = 0;
        let max_atoms = request.max_atoms.unwrap_or(1000);

        for anchor in &anchor_atoms {
            if total_atoms_collected >= max_atoms {
                break;
            }

            // BFS to find atoms within radius
            let mut queue: Vec<(u64, u32)> = vec![(anchor.id, 0)]; // (atom_id, hop_distance)
            visited.insert(anchor.id);

            while let Some((atom_id, hop)) = queue.pop() {
                if hop > request.radius || total_atoms_collected >= max_atoms {
                    continue;
                }

                // Get atom with lazy content loading
                let atom = match self.db.get_atom(atom_id).await {
                    Ok(a) => a,
                    Err(_) => continue,
                };

                // Zero-copy content load from filesystem
                let content_str = match self.storage.read_range(&atom.source_path, atom.start_byte, atom.end_byte) {
                    Ok(content) => content,
                    Err(_) => continue,
                };

                // CRITICAL: Hash content for deduplication
                let content_hash = anchor_fingerprint::simhash(&content_str);
                if !seen_hashes.insert(content_hash) {
                    continue; // Duplicate - skipped
                }

                // Calculate gravity score (damped by hop distance)
                let damping_factor: f64 = 0.85;
                let gravity_score = damping_factor.powi(hop as i32);

                // Group by source for coherent assembly
                blocks_by_source
                    .entry(atom.source_path.clone())
                    .or_insert_with(Vec::new)
                    .push(DistillBlock {
                        atom_id: atom.id,
                        content: content_str.to_string(),
                        hop_distance: hop,
                        gravity_score,
                        tags: atom.tags.clone(),
                        char_start: atom.char_start,
                        char_end: atom.char_end,
                    });

                total_atoms_collected += 1;

                // Find neighboring atoms via shared tags
                let atom_tags = self.db.get_tags_for_atom(atom.id).await.unwrap_or_default();
                for tag in &atom_tags {
                    let neighbor_atoms = self.db.get_atoms_by_tag(&tag.tag).await.unwrap_or_default();
                    for neighbor in &neighbor_atoms {
                        if visited.insert(neighbor.id) {
                            queue.push((neighbor.id, hop + 1));
                        }
                    }
                }
            }
        }

        info!("   └─ Collected {} unique blocks from {} sources",
              total_atoms_collected, blocks_by_source.len());

        // Step 4: Assemble Decision Records
        let distills_dir = PathBuf::from("distills");
        fs::create_dir_all(&distills_dir)
            .map_err(|e| crate::db::DbError::Migration(e.to_string()))?;

        // Generate output filename based on seed
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let seed_safe = seed_query.replace(|c: char| !c.is_alphanumeric(), "_");
        let output_filename = format!("distill_{}_{}.json", seed_safe, timestamp);
        let output_path = distills_dir.join(&output_filename);

        // Calculate token counts for compression ratio
        let mut original_tokens = 0;
        let mut distilled_tokens = 0;

        // Build output structure
        let mut decision_records: Vec<DecisionRecord> = Vec::new();

        for (source_path, blocks) in &blocks_by_source {
            // Sort blocks by char position for coherent narrative
            let mut sorted_blocks = blocks.clone();
            sorted_blocks.sort_by_key(|b| b.char_start);

            // Concatenate blocks for this source
            let mut record_content = String::new();
            for block in &sorted_blocks {
                record_content.push_str(&block.content);
                record_content.push('\n');
                original_tokens += block.content.split_whitespace().count();
            }
            distilled_tokens += record_content.split_whitespace().count();

            decision_records.push(DecisionRecord {
                source: source_path.clone(),
                content: record_content,
                blocks: sorted_blocks.len(),
                total_hops: sorted_blocks.iter().map(|b| b.hop_distance).max().unwrap_or(0),
            });
        }

        // Step 5: Write output to distills/ directory
        let output_json = serde_json::to_string_pretty(&DecisionRecordsOutput {
            seed: seed_query.to_string(),
            radius: request.radius,
            total_atoms: total_atoms_collected,
            total_sources: blocks_by_source.len(),
            compression_ratio: if original_tokens > 0 {
                distilled_tokens as f64 / original_tokens as f64
            } else {
                1.0
            },
            records: decision_records.clone(),
            duration_ms: start.elapsed().as_millis() as f64,
        })?;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&output_path)
            .map_err(|e| crate::db::DbError::Migration(e.to_string()))?;

        file.write_all(output_json.as_bytes())
            .map_err(|e| crate::db::DbError::Migration(e.to_string()))?;

        let duration = start.elapsed().as_millis() as f64;
        let total_records = decision_records.len();

        info!("   └─ ✅ COMPLETE: {} records, {} compression ratio in {:.1}ms",
              total_records,
              if original_tokens > 0 { format!("{:.1}%", (distilled_tokens as f64 / original_tokens as f64) * 100.0) } else { "N/A".to_string() },
              duration);

        Ok(DistillResponse {
            output_path: output_path.to_string_lossy().to_string(),
            compression_ratio: if original_tokens > 0 {
                distilled_tokens as f64 / original_tokens as f64
            } else {
                1.0
            },
            total_atoms: total_atoms_collected,
            total_sources: blocks_by_source.len(),
            duration_ms: duration,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_ingest_and_search() {
        let db = Database::in_memory().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let config = Config::default();
        let mut service = AnchorService::new(db, temp_dir.path().to_path_buf(), config).unwrap();

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

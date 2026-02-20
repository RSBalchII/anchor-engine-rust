//! Core service logic for Anchor Engine.
//!
//! Integrates atomizer, fingerprint, keyextract, and tagwalker with the database.

use std::time::Instant;

use anchor_atomizer::{atomize, sanitize};
use anchor_fingerprint::simhash;
use anchor_keyextract::{extract_keywords, SynonymRing};
use anchor_tagwalker::{TagWalker, TagWalkerConfig, ResultType};
use tracing::{info, debug};

use crate::db::{Database, Result};
use crate::models::*;

/// Core Anchor service.
pub struct AnchorService {
    db: Database,
    tag_walker: TagWalker,
    synonym_ring: SynonymRing,
}

impl AnchorService {
    /// Create a new Anchor service.
    pub fn new(db: Database) -> Self {
        let tag_walker = TagWalker::new();
        let synonym_ring = SynonymRing::default();

        // Load existing atoms into tag walker
        // (In production, this would be done lazily or on startup)

        // ℹ️ INFO log when AnchorService is created
        info!("🚀 AnchorService initialized");

        Self {
            db,
            tag_walker,
            synonym_ring,
        }
    }

    /// Ingest content into the knowledge base.
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

        // Atomize content
        let atoms = atomize(&content);
        
        // Process each atom
        let mut atom_ids = Vec::new();
        let mut all_tags = Vec::new();
        
        for atom_data in &atoms {
            // Generate SimHash
            let hash = simhash(&atom_data.content);
            
            // Create atom record
            let atom = Atom {
                id: 0, // Will be assigned by DB
                source_id: source_id.clone(),
                content: atom_data.content.clone(),
                char_start: atom_data.char_start,
                char_end: atom_data.char_end,
                timestamp: chrono::Utc::now().timestamp() as f64,
                simhash: hash,
                metadata: None,
            };
            
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
            
            // Add to tag walker
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
        
        // 🔍 DEBUG log for search execution
        info!("🔍 SEARCH: \"{}\" (max_recall={}, tokens={})", 
              request.query, use_max_recall, request.budget.total_tokens);

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
        let walker_results = self.tag_walker.search(&request.query, &config);

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

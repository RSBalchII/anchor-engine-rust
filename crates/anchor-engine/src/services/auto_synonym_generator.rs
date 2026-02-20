//! Automatic Synonym Ring Generator
//!
//! Mines synonym rings from the knowledge base using three strategies:
//! 1. Co-occurrence patterns (tags that appear together)
//! 2. Tag neighborhood similarity (tags with similar neighborhoods)
//! 3. SimHash proximity (atoms with similar fingerprints)

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};

/// Synonym ring type for auto-generated synonyms
pub type SynonymRing = HashMap<String, Vec<String>>;

/// Automatic synonym generator
pub struct AutoSynonymGenerator {
    /// Minimum co-occurrence threshold
    min_cooccurrence: usize,
    /// Minimum Jaccard similarity for tag neighborhoods
    min_jaccard_similarity: f32,
    /// Maximum Hamming distance for SimHash proximity
    max_hamming_distance: u32,
}

impl AutoSynonymGenerator {
    /// Create a new auto synonym generator
    pub fn new() -> Self {
        Self {
            min_cooccurrence: 2,
            min_jaccard_similarity: 0.5,
            max_hamming_distance: 5,
        }
    }

    /// Generate synonym rings from all strategies
    pub async fn generate_all(&self, db: &crate::db::Database) -> SynonymRing {
        info!("🔄 [SynonymGenerator] Generating synonym rings from all strategies...");
        
        let mut synonym_rings = SynonymRing::new();
        
        // Strategy 1: Co-occurrence patterns
        info!("   ├─ Strategy 1: Mining co-occurrence patterns...");
        let cooccurrence = self.generate_cooccurrence_synonyms(db).await;
        info!("   │  Found {} terms with co-occurrence synonyms", cooccurrence.len());
        synonym_rings.extend(cooccurrence);
        
        // Strategy 2: Tag neighborhood similarity
        info!("   ├─ Strategy 2: Mining tag neighborhood similarity...");
        let neighborhoods = self.generate_neighborhood_synonyms(db).await;
        info!("   │  Found {} terms with neighborhood synonyms", neighborhoods.len());
        synonym_rings.extend(neighborhoods);
        
        // Strategy 3: SimHash proximity
        info!("   ├─ Strategy 3: Mining SimHash proximity...");
        let simhash = self.generate_simhash_synonyms(db).await;
        info!("   │  Found {} terms with SimHash synonyms", simhash.len());
        synonym_rings.extend(simhash);
        
        info!("   └─ ✅ Generated {} total synonym rings", synonym_rings.len());
        
        synonym_rings
    }

    /// Strategy 1: Mine co-occurrence patterns
    /// Tags that frequently appear together on the same atoms
    async fn generate_cooccurrence_synonyms(&self, db: &crate::db::Database) -> SynonymRing {
        let mut synonyms = SynonymRing::new();
        
        // Get all atoms with their tags
        let atoms = match db.get_all_atoms().await {
            Ok(atoms) => atoms,
            Err(e) => {
                error!("   └─ ❌ Failed to get atoms: {}", e);
                return synonyms;
            }
        };
        
        // Build co-occurrence map: tag -> (tag -> count)
        let mut cooccurrence_map: HashMap<String, HashMap<String, usize>> = HashMap::new();
        
        for atom in &atoms {
            let tags = &atom.tags;
            
            // For each pair of tags on this atom
            for (i, tag1) in tags.iter().enumerate() {
                for tag2 in tags.iter().skip(i + 1) {
                    if tag1 != tag2 {
                        *cooccurrence_map
                            .entry(tag1.clone())
                            .or_insert_with(HashMap::new)
                            .entry(tag2.clone())
                            .or_insert(0) += 1;
                        
                        *cooccurrence_map
                            .entry(tag2.clone())
                            .or_insert_with(HashMap::new)
                            .entry(tag1.clone())
                            .or_insert(0) += 1;
                    }
                }
            }
        }
        
        // Convert to synonym ring
        for (tag1, related) in cooccurrence_map {
            let mut synonyms_for_tag = Vec::new();
            
            for (tag2, count) in related {
                if count >= self.min_cooccurrence {
                    synonyms_for_tag.push(tag2);
                }
            }
            
            if !synonyms_for_tag.is_empty() {
                synonyms.insert(tag1, synonyms_for_tag);
            }
        }
        
        synonyms
    }

    /// Strategy 2: Mine tag neighborhood similarity
    /// Tags with similar neighborhoods (Jaccard similarity)
    async fn generate_neighborhood_synonyms(&self, db: &crate::db::Database) -> SynonymRing {
        let mut synonyms = SynonymRing::new();
        
        // Get all atoms and extract unique tags
        let atoms = match db.get_all_atoms().await {
            Ok(atoms) => atoms,
            Err(e) => {
                error!("   └─ ❌ Failed to get atoms: {}", e);
                return synonyms;
            }
        };
        
        // Build tag neighborhoods: tag -> set of atom IDs it appears on
        let mut tag_neighborhoods: HashMap<String, HashSet<u64>> = HashMap::new();
        
        for atom in &atoms {
            for tag in &atom.tags {
                tag_neighborhoods
                    .entry(tag.clone())
                    .or_insert_with(HashSet::new)
                    .insert(atom.id);
            }
        }
        
        if tag_neighborhoods.len() < 2 {
            warn!("   └─ ⚠️  Insufficient tag data for neighborhood analysis");
            return synonyms;
        }
        
        // Calculate Jaccard similarity between tag pairs
        let tags_vec: Vec<&String> = tag_neighborhoods.keys().collect();
        
        for (i, tag1) in tags_vec.iter().enumerate() {
            let mut similar_tags = Vec::new();
            
            for tag2 in tags_vec.iter().skip(i + 1) {
                if let (Some(neigh1), Some(neigh2)) = (
                    tag_neighborhoods.get(*tag1),
                    tag_neighborhoods.get(*tag2),
                ) {
                    let jaccard = self.jaccard_similarity(neigh1, neigh2);
                    
                    if jaccard >= self.min_jaccard_similarity {
                        similar_tags.push((*tag2).clone());
                    }
                }
            }
            
            if !similar_tags.is_empty() {
                synonyms.insert((*tag1).clone(), similar_tags);
            }
        }
        
        synonyms
    }

    /// Strategy 3: Mine SimHash proximity
    /// Atoms with similar SimHashes (Hamming distance <= threshold)
    async fn generate_simhash_synonyms(&self, db: &crate::db::Database) -> SynonymRing {
        let mut synonyms = SynonymRing::new();
        
        // Get all atoms
        let atoms = match db.get_all_atoms().await {
            Ok(atoms) => atoms,
            Err(e) => {
                error!("   └─ ❌ Failed to get atoms: {}", e);
                return synonyms;
            }
        };
        
        if atoms.is_empty() {
            warn!("   └─ ⚠️  No SimHash data available");
            return synonyms;
        }
        
        // Group atoms by SimHash proximity
        let mut hash_to_tags: HashMap<u64, HashSet<String>> = HashMap::new();
        
        for atom in &atoms {
            hash_to_tags
                .entry(atom.simhash)
                .or_insert_with(HashSet::new)
                .extend(atom.tags.iter().cloned());
        }
        
        // Find similar hashes
        let hashes: Vec<u64> = hash_to_tags.keys().copied().collect();
        
        for (i, &hash1) in hashes.iter().enumerate() {
            let mut similar_tags = HashSet::new();
            
            for &hash2 in hashes.iter().skip(i + 1) {
                let distance = anchor_fingerprint::hamming_distance(hash1, hash2);
                
                if distance <= self.max_hamming_distance {
                    // Merge tags from similar hashes
                    if let Some(tags2) = hash_to_tags.get(&hash2) {
                        similar_tags.extend(tags2.iter().cloned());
                    }
                }
            }
            
            if let Some(tags1) = hash_to_tags.get(&hash1) {
                for tag1 in tags1 {
                    let mut synonyms_for_tag: Vec<String> = similar_tags
                        .iter()
                        .filter(|t| t != &tag1)
                        .cloned()
                        .collect();
                    
                    if !synonyms_for_tag.is_empty() {
                        synonyms_for_tag.sort();
                        synonyms_for_tag.dedup();
                        synonyms.insert(tag1.clone(), synonyms_for_tag);
                    }
                }
            }
        }
        
        synonyms
    }

    /// Calculate Jaccard similarity between two sets
    fn jaccard_similarity<T: Eq + std::hash::Hash>(&self, set1: &HashSet<T>, set2: &HashSet<T>) -> f32 {
        if set1.is_empty() && set2.is_empty() {
            return 1.0;
        }
        
        let intersection = set1.intersection(set2).count();
        let union = set1.union(set2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// Save synonym rings to a JSON file
    pub fn save_synonym_rings(&self, synonyms: &SynonymRing, output_path: &Path) -> std::io::Result<()> {
        info!("💾 [SynonymGenerator] Saving synonym rings to {:?}", output_path);
        
        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Write JSON
        let json = serde_json::to_string_pretty(synonyms)?;
        fs::write(output_path, json)?;
        
        info!("   └─ ✅ Saved {} synonym rings", synonyms.len());
        
        Ok(())
    }

    /// Generate summary markdown
    pub fn generate_summary(&self, synonyms: &SynonymRing, output_path: &Path) -> std::io::Result<()> {
        info!("📝 [SynonymGenerator] Generating summary at {:?}", output_path);
        
        let mut summary = String::new();
        summary.push_str("# Auto-Generated Synonym Rings\n\n");
        summary.push_str(&format!("**Generated**: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        summary.push_str(&format!("**Total Terms**: {}\n\n", synonyms.len()));
        
        // Group by strategy (approximate)
        let mut by_size: Vec<_> = synonyms.iter().collect();
        by_size.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        
        summary.push_str("## Top Synonym Rings\n\n");
        
        for (term, synonyms) in by_size.iter().take(20) {
            summary.push_str(&format!("### {}\n", term));
            summary.push_str(&format!("**Synonyms**: {}\n\n", synonyms.join(", ")));
        }
        
        fs::write(output_path, summary)?;
        
        info!("   └─ ✅ Summary generated");
        
        Ok(())
    }
}

impl Default for AutoSynonymGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaccard_similarity() {
        let gen = AutoSynonymGenerator::new();
        
        let set1: HashSet<i32> = [1, 2, 3].iter().copied().collect();
        let set2: HashSet<i32> = [2, 3, 4].iter().copied().collect();
        
        let similarity = gen.jaccard_similarity(&set1, &set2);
        assert!((similarity - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_empty_sets() {
        let gen = AutoSynonymGenerator::new();
        
        let set1: HashSet<i32> = HashSet::new();
        let set2: HashSet<i32> = HashSet::new();
        
        let similarity = gen.jaccard_similarity(&set1, &set2);
        assert!((similarity - 1.0).abs() < f32::EPSILON);
    }
}

use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

#[derive(Debug, Clone, Copy)]
pub struct TranspositionEntry {
    pub score: f32,
    pub depth: u32,
    pub best_move: Option<crate::game::Direction>,
    pub node_type: NodeType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeType {
    Exact,      // Exact score
    LowerBound, // Alpha cutoff (score >= beta)
    UpperBound, // Beta cutoff (score <= alpha)
}

lazy_static! {
    pub static ref ENHANCED_TABLE: Mutex<HashMap<u64, TranspositionEntry>> = Mutex::new(HashMap::new());
    pub static ref CACHE_HITS: Mutex<u64> = Mutex::new(0);
    pub static ref CACHE_MISSES: Mutex<u64> = Mutex::new(0);
    pub static ref CACHE_COLLISIONS: Mutex<u64> = Mutex::new(0);
}

pub struct EnhancedTranspositionTable;

impl EnhancedTranspositionTable {
    pub fn lookup(hash: u64, depth: u32, alpha: f32, beta: f32) -> Option<f32> {
        let table = ENHANCED_TABLE.lock().unwrap();
        
        if let Some(entry) = table.get(&hash) {
            let mut hits = CACHE_HITS.lock().unwrap();
            *hits += 1;
            
            // Only use entry if it was computed at equal or greater depth
            if entry.depth >= depth {
                match entry.node_type {
                    NodeType::Exact => return Some(entry.score),
                    NodeType::LowerBound if entry.score >= beta => return Some(entry.score),
                    NodeType::UpperBound if entry.score <= alpha => return Some(entry.score),
                    _ => {}
                }
            }
        } else {
            let mut misses = CACHE_MISSES.lock().unwrap();
            *misses += 1;
        }
        
        None
    }

    pub fn store(hash: u64, score: f32, depth: u32, alpha: f32, beta: f32, best_move: Option<crate::game::Direction>) {
        let node_type = if score <= alpha {
            NodeType::UpperBound
        } else if score >= beta {
            NodeType::LowerBound
        } else {
            NodeType::Exact
        };

        let entry = TranspositionEntry {
            score,
            depth,
            best_move,
            node_type,
        };

        let mut table = ENHANCED_TABLE.lock().unwrap();
        
        // Always replace if new entry has greater depth, or if slot is empty
        if let Some(existing) = table.get(&hash) {
            if existing.depth > depth {
                let mut collisions = CACHE_COLLISIONS.lock().unwrap();
                *collisions += 1;
                return; // Don't replace deeper search with shallow one
            }
        }
        
        table.insert(hash, entry);
        
        // Clear cache if it gets too large
        if table.len() > 2_000_000 {
            Self::clear_cache();
        }
    }

    pub fn get_best_move(hash: u64) -> Option<crate::game::Direction> {
        let table = ENHANCED_TABLE.lock().unwrap();
        table.get(&hash).and_then(|entry| entry.best_move)
    }

    pub fn clear_cache() {
        ENHANCED_TABLE.lock().unwrap().clear();
        *CACHE_HITS.lock().unwrap() = 0;
        *CACHE_MISSES.lock().unwrap() = 0;
        *CACHE_COLLISIONS.lock().unwrap() = 0;
    }

    pub fn get_cache_stats() -> (u64, u64, u64, usize) {
        let hits = *CACHE_HITS.lock().unwrap();
        let misses = *CACHE_MISSES.lock().unwrap();
        let collisions = *CACHE_COLLISIONS.lock().unwrap();
        let size = ENHANCED_TABLE.lock().unwrap().len();
        (hits, misses, collisions, size)
    }

    // Selective cache clearing - remove entries with low depth
    pub fn selective_clear(min_depth: u32) {
        let mut table = ENHANCED_TABLE.lock().unwrap();
        table.retain(|_, entry| entry.depth >= min_depth);
    }

    // Get cache hit rate
    pub fn hit_rate() -> f64 {
        let hits = *CACHE_HITS.lock().unwrap() as f64;
        let misses = *CACHE_MISSES.lock().unwrap() as f64;
        let total = hits + misses;
        if total > 0.0 { hits / total } else { 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_transposition_table() {
        EnhancedTranspositionTable::clear_cache();
        
        // Test storing and retrieving
        let hash = 12345;
        let score = 100.0;
        let depth = 5;
        let alpha = -1000.0;
        let beta = 1000.0;
        
        EnhancedTranspositionTable::store(hash, score, depth, alpha, beta, None);
        
        let retrieved = EnhancedTranspositionTable::lookup(hash, depth, alpha, beta);
        assert_eq!(retrieved, Some(score));
    }

    #[test]
    fn test_depth_replacement() {
        EnhancedTranspositionTable::clear_cache();
        
        let hash = 12345;
        
        // Store shallow search
        EnhancedTranspositionTable::store(hash, 50.0, 3, -1000.0, 1000.0, None);
        
        // Try to store deeper search
        EnhancedTranspositionTable::store(hash, 100.0, 5, -1000.0, 1000.0, None);
        
        // Should get the deeper search result
        let retrieved = EnhancedTranspositionTable::lookup(hash, 5, -1000.0, 1000.0);
        assert_eq!(retrieved, Some(100.0));
    }
} 
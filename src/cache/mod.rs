mod transposition;
pub mod enhanced_transposition;

pub use transposition::{get_cache_stats, clear_cache, TRANSPOSITION_TABLE, CACHE_HITS, CACHE_MISSES};
pub use enhanced_transposition::{EnhancedTranspositionTable, TranspositionEntry, NodeType}; 
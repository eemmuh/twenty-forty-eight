use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref TRANSPOSITION_TABLE: Mutex<HashMap<u64, f32>> = Mutex::new(HashMap::new());
    pub static ref CACHE_HITS: Mutex<u64> = Mutex::new(0);
    pub static ref CACHE_MISSES: Mutex<u64> = Mutex::new(0);
}

pub fn get_cache_stats() -> (u64, u64, usize) {
    let hits = *CACHE_HITS.lock().unwrap();
    let misses = *CACHE_MISSES.lock().unwrap();
    let size = TRANSPOSITION_TABLE.lock().unwrap().len();
    (hits, misses, size)
}

pub fn clear_cache() {
    TRANSPOSITION_TABLE.lock().unwrap().clear();
    *CACHE_HITS.lock().unwrap() = 0;
    *CACHE_MISSES.lock().unwrap() = 0;
} 
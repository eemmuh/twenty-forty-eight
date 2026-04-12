use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

/// Lookup key: same board can have different values depending on how much
/// lookahead remains and whether the next event is a player move or a spawn.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct TtKey {
    hash: u64,
    depth: u32,
    /// `true` = maximizing (player) node, `false` = chance (spawn) node.
    max_node: bool,
}

struct TranspositionState {
    map: HashMap<TtKey, f32>,
    hits: u64,
    misses: u64,
}

lazy_static! {
    static ref TT: Mutex<TranspositionState> = Mutex::new(TranspositionState {
        map: HashMap::new(),
        hits: 0,
        misses: 0,
    });
}

pub fn tt_probe(hash: u64, depth: u32, max_node: bool) -> Option<f32> {
    let key = TtKey {
        hash,
        depth,
        max_node,
    };
    let mut state = TT.lock().unwrap();
    if let Some(&score) = state.map.get(&key) {
        state.hits += 1;
        Some(score)
    } else {
        state.misses += 1;
        None
    }
}

pub fn tt_store(hash: u64, depth: u32, max_node: bool, score: f32) {
    let key = TtKey {
        hash,
        depth,
        max_node,
    };
    TT.lock().unwrap().map.insert(key, score);
}

pub fn get_cache_stats() -> (u64, u64, usize) {
    let state = TT.lock().unwrap();
    (state.hits, state.misses, state.map.len())
}

pub fn clear_cache() {
    let mut state = TT.lock().unwrap();
    state.map.clear();
    state.hits = 0;
    state.misses = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tt_keys_differ_by_depth_and_phase() {
        // Reserved test-only hash so we never clear the global TT (other tests run in parallel).
        let h = 0x7e57_7e57_0000_0001_u64;
        tt_store(h, 5, true, 10.0);
        tt_store(h, 3, true, 20.0);
        tt_store(h, 5, false, 30.0);

        assert_eq!(tt_probe(h, 5, true), Some(10.0));
        assert_eq!(tt_probe(h, 3, true), Some(20.0));
        assert_eq!(tt_probe(h, 5, false), Some(30.0));
        assert_eq!(tt_probe(h, 4, true), None);
    }
}

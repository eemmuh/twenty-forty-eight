use std::cell::RefCell;
use std::collections::HashMap;

/// Lookup key: same board can have different values depending on how much
/// lookahead remains and whether the next event is a player move or a spawn.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct TtKey {
    hash: u64,
    depth: u32,
    /// `true` = maximizing (player) node, `false` = chance (spawn) node.
    max_node: bool,
}

/// Transposition table for expectimax. Pass `&mut TranspositionState` through
/// the search so probes/stores avoid global synchronization.
pub struct TranspositionState {
    map: HashMap<TtKey, f32>,
    hits: u64,
    misses: u64,
}

impl TranspositionState {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    pub fn probe(&mut self, hash: u64, depth: u32, max_node: bool) -> Option<f32> {
        let key = TtKey {
            hash,
            depth,
            max_node,
        };
        if let Some(&score) = self.map.get(&key) {
            self.hits += 1;
            Some(score)
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn store(&mut self, hash: u64, depth: u32, max_node: bool, score: f32) {
        let key = TtKey {
            hash,
            depth,
            max_node,
        };
        self.map.insert(key, score);
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.hits = 0;
        self.misses = 0;
    }

    pub fn stats(&self) -> (u64, u64, usize) {
        (self.hits, self.misses, self.map.len())
    }
}

thread_local! {
    static THREAD_TT: RefCell<TranspositionState> = RefCell::new(TranspositionState::new());
}

/// Runs `f` with the current thread’s persistent transposition table (used
/// across moves in the solver). For parallel search, use one `TranspositionState`
/// per thread instead of sharing this.
pub fn with_thread_tt<F, R>(f: F) -> R
where
    F: FnOnce(&mut TranspositionState) -> R,
{
    THREAD_TT.with(|cell| f(&mut *cell.borrow_mut()))
}

pub fn get_cache_stats() -> (u64, u64, usize) {
    THREAD_TT.with(|cell| {
        let s = cell.borrow();
        s.stats()
    })
}

pub fn clear_cache() {
    THREAD_TT.with(|cell| {
        cell.borrow_mut().clear();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tt_keys_differ_by_depth_and_phase() {
        let mut tt = TranspositionState::new();
        let h = 0x7e57_7e57_0000_0001_u64;
        tt.store(h, 5, true, 10.0);
        tt.store(h, 3, true, 20.0);
        tt.store(h, 5, false, 30.0);

        assert_eq!(tt.probe(h, 5, true), Some(10.0));
        assert_eq!(tt.probe(h, 3, true), Some(20.0));
        assert_eq!(tt.probe(h, 5, false), Some(30.0));
        assert_eq!(tt.probe(h, 4, true), None);
    }
}

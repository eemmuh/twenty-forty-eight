mod transposition;

pub(crate) use transposition::{tt_probe, tt_store};
pub use transposition::{get_cache_stats, clear_cache}; 
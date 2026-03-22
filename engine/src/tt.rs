use crate::results::Score;
use chess_lib::board::{BoardHash, Color};
use lru::LruCache;
use std::num::NonZeroUsize;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TTEntryType {
    LowerBound,
    UpperBound,
    Exact,
}

impl TTEntryType {
    pub fn color_bound(color: Color) -> TTEntryType {
        // Might have been able to find something better for itself if it wasn't cut off
        if color.is_white() {
            TTEntryType::LowerBound
        } else {
            TTEntryType::UpperBound
        }
    }
}

// TODO: Store hashes from initial to eval to detect repetition
// e.g. if A's final eval is A -> B -> C -> D then B, C, D must be stored to test whether they
// can be used without causing threefold
pub struct TTEntry {
    pub depth_searched: usize,
    pub white_score: Score,
    pub entry_type: TTEntryType,
}

pub struct TranspositionTable {
    inner: LruCache<BoardHash, TTEntry>,
}

impl TranspositionTable {
    pub fn new() -> TranspositionTable {
        TranspositionTable {
            // TODO: Sensible way to size this
            inner: LruCache::new(
                NonZeroUsize::try_from(800_000_000 / size_of::<TTEntry>()).unwrap(),
            ),
        }
    }

    pub fn push(&mut self, hash: BoardHash, entry: TTEntry) {
        self.inner.push(hash, entry);
    }

    pub fn get(&mut self, hash: &BoardHash) -> Option<&TTEntry> {
        self.inner.get(hash)
    }
}

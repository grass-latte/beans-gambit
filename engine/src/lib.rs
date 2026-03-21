#![allow(dead_code)]
#![allow(unused)]

mod constant_heuristics;
pub(crate) mod eval;
mod minimax;
mod tt;

use crate::minimax::search_minimax;
use crate::tt::TranspositionTable;
use chess_lib::board::{Board, Move};

pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub struct InterMoveCache {
    // (depth searched, eval)
    // Evals from perspective of white
    pub(crate) transposition_table: TranspositionTable,
}

impl Default for InterMoveCache {
    fn default() -> Self {
        Self::new()
    }
}

impl InterMoveCache {
    pub fn new() -> InterMoveCache {
        InterMoveCache {
            transposition_table: TranspositionTable::new(),
        }
    }
}

pub fn search(
    board: &mut Board,
    cache: &mut InterMoveCache,
    stop_fn: fn() -> bool,
) -> Option<Move> {
    search_minimax(board, cache, stop_fn)
}

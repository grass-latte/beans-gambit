#![allow(dead_code)]
#![allow(unused)]

mod constant_heuristics;
pub(crate) mod eval;
mod minimax;
pub mod results;
mod tt;

use crate::minimax::{TimeManagementStrat, search_minimax};
use crate::results::Score;
use crate::tt::TranspositionTable;
use chess_lib::board::{Board, Move};
use std::cmp::min;
use std::time::Duration;

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
    time_remaining: Duration,
) -> (Option<Move>, Score) {
    search_minimax(
        board,
        cache,
        stop_fn,
        min(Duration::from_secs(15), time_remaining / 10),
        TimeManagementStrat::TargetLimit,
    )
}

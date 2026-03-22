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
use log::info;
use opening_book::OpeningBook;
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

    pub fn size_bytes(&self) -> usize {
        self.transposition_table.size_bytes()
    }
}

pub fn search(
    board: &mut Board,
    cache: &mut InterMoveCache,
    stop_fn: fn() -> bool,
    time_remaining: Duration,
    opening_book: Option<&dyn OpeningBook>,
) -> (Option<Move>, Score) {
    let target_move_time = min(Duration::from_secs(20), time_remaining / 10);
    let time_management_strat = TimeManagementStrat::TargetLimit;
    info!(
        "FEN {} | Target time {:?} | Strat: {:?}",
        board.to_fen(),
        target_move_time,
        time_management_strat
    );

    if let Some(opening_book) = opening_book {
        if let Some(mv) = opening_book.get_weighted(board.hash()) {
            info!("Playing book move {:?}", mv);
            return (Some(mv), Score::ZERO);
        }
    }

    search_minimax(
        board,
        cache,
        stop_fn,
        target_move_time,
        time_management_strat,
    )
}

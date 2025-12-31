#![allow(dead_code)]
#![allow(unused)]

use chess_lib::board::{Board, Move};
use chess_lib::movegen::MoveGenerator;
use derive_new::new;

pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(new)]
pub struct InterMoveCache;

pub fn search(board: &mut Board, cache: &mut InterMoveCache) -> Option<Move> {
    Some(MoveGenerator::new().compute_legal_moves(board)[0])
}

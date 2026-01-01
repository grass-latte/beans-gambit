#![allow(dead_code)]
#![allow(unused)]

use chess_lib::board::{Board, Move};
use chess_lib::movegen::{MoveGenerator, MoveList};
use derive_new::new;
use rand::{Rng, rng};

pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(new)]
pub struct InterMoveCache;

pub fn search(board: &mut Board, cache: &mut InterMoveCache) -> Option<Move> {
    let mut rng = rng();
    let mut mg = MoveGenerator::new();
    let mut options = MoveList::new();
    mg.compute_legal_moves(&mut options, board);
    Some(options[rng.random_range(0..options.len())])
}

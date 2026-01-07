#![allow(dead_code)]
#![allow(unused)]

use chess_lib::board::{Board, Move};
use chess_lib::movegen::{MoveGenerator, MoveList};
use derive_new::new;
use rand::rng;

pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(new)]
pub struct InterMoveCache;

pub fn eval(board: &mut Board) -> f32 {
    let mut mg = MoveGenerator::new();
    let mut options = MoveList::new();
    mg.compute_legal_moves(&mut options, board);
    options.len() as f32
}

pub fn search(board: &mut Board, cache: &mut InterMoveCache) -> Option<Move> {
    let mut rng = rng();
    let mut mg = MoveGenerator::new();
    let mut options = MoveList::new();
    mg.compute_legal_moves(&mut options, board);

    let mut best_move = options[0];
    let um = board.make_move(options[0]);
    let mut best_eval = eval(board);
    board.unmake_last_move(um);

    for mv in options.into_iter().skip(1) {
        let um = board.make_move(mv);
        let ev = eval(board);

        if ev < best_eval {
            best_move = mv;
            best_eval = ev;
        }

        board.unmake_last_move(um);
    }

    Some(best_move)
}

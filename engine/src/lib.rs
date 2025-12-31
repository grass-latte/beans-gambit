#![allow(dead_code)]
#![allow(unused)]

use chess_lib::board::{Board, Move};
use derive_new::new;

pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(new)]
pub struct InterMoveCache;

pub fn search(board: Board, cache: &mut InterMoveCache, stop_check: fn() -> bool) -> Option<Move> {
    todo!()
}

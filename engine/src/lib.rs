use chess_lib::board::{Board, Move};
use derive_new::new;

pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(new)]
struct InterMoveCache;

pub fn search(board: Board, cache: &mut InterMoveCache, stop_check: fn() -> bool) -> Option<Move> {
    todo!()
}

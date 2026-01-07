#![allow(dead_code)]
#![allow(unused)]

use chess_lib::board::{Board, Move, PieceKind};
use chess_lib::movegen::{compute_legal_moves, MoveList};
use derive_new::new;
use rand::rng;

pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(new)]
pub struct InterMoveCache;

pub fn eval(board: &Board) -> f32 {
    let color_to_move = board.color_to_move();

    let mut score = 0f32;

    for (_, piece) in board.pieces().iter() {
        let mut s = match piece.kind() {
            PieceKind::Pawn => 1.0,
            PieceKind::Knight => 3.0,
            PieceKind::Bishop => 3.5,
            PieceKind::Rook => 5.0,
            PieceKind::Queen => 8.0,
            PieceKind::King => 0.0,
        };

        if piece.color() != color_to_move {
            s *= -1.0;
        }
        score += s;
    }

    score
}

pub fn minimax(
    board: &mut Board,
    depth_remaining: usize,
    prune: f32,
    stop_fn: fn() -> bool,
) -> f32 {
    if depth_remaining == 0 {
        return eval(board);
    }

    // Both players are maximising so if -eval < prune, this branch will be pruned

    let mut options = MoveList::new();
    compute_legal_moves(&mut options, board);

    if options.is_empty() {
        return f32::NEG_INFINITY; // Checkmate
    }

    let mut best_move = options[0];
    let um = board.make_move(options[0]);
    // Minimax returns opponent's score
    let mut best_eval = -minimax(board, depth_remaining - 1, f32::NEG_INFINITY, stop_fn);
    board.unmake_last_move(um);

    if best_eval > -prune {
        // Branch will be pruned
        return best_eval;
    }

    for mv in options.into_iter().skip(1) {
        if stop_fn() {
            return best_eval;
        }

        let um = board.make_move(mv);
        // Minimax returns opponent's score
        let ev = -minimax(board, depth_remaining - 1, best_eval, stop_fn);
        board.unmake_last_move(um);

        if ev == f32::INFINITY {
            return ev;
        }

        if ev > best_eval {
            best_move = mv;
            best_eval = ev;

            if best_eval > -prune {
                // Branch will be pruned
                return best_eval;
            }
        }
    }

    best_eval
}

pub fn search(
    board: &mut Board,
    cache: &mut InterMoveCache,
    stop_fn: fn() -> bool,
) -> Option<Move> {
    const SEARCH_DEPTH: usize = 4;

    let mut rng = rng();
    let mut options = MoveList::new();
    compute_legal_moves(&mut options, board);

    let mut best_move = options[0];
    let um = board.make_move(options[0]);
    // Minimax returns opponent's score
    let mut best_eval = -minimax(board, SEARCH_DEPTH, f32::NEG_INFINITY, stop_fn);
    board.unmake_last_move(um);

    for mv in options.into_iter().skip(1) {
        if stop_fn() {
            return Some(best_move);
        }

        let um = board.make_move(mv);
        // Minimax returns opponent's score
        let ev = -minimax(board, SEARCH_DEPTH, best_eval, stop_fn);
        board.unmake_last_move(um);

        if ev > best_eval {
            best_move = mv;
            best_eval = ev;
        }
    }

    Some(best_move)
}

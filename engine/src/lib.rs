#![allow(dead_code)]
#![allow(unused)]

use chess_lib::board::{Board, BoardHash, Move, PieceKind};
use chess_lib::movegen::{MoveList, compute_legal_moves};
use rand::rng;
use std::collections::HashMap;

pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub struct InterMoveCache {
    // (depth searched, eval)
    // Evals from perspective of white
    saved_positions: HashMap<BoardHash, (usize, f32)>,
}

impl InterMoveCache {
    pub fn new() -> InterMoveCache {
        InterMoveCache {
            saved_positions: HashMap::new(),
        }
    }
}

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
    cache: &mut InterMoveCache,
    depth_remaining: usize,
    prune: f32,
    stop_fn: fn() -> bool,
) -> f32 {
    if depth_remaining == 0 {
        return eval(board);
    }

    if let Some((depth_searched, white_score)) = cache.saved_positions.get(&board.hash()) {
        if *depth_searched < depth_remaining {
            // Will be replaced by deeper search later
        } else if board.color_to_move().is_white() {
            return *white_score;
        } else {
            return -white_score;
        }
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
    let mut best_eval = -minimax(
        board,
        cache,
        depth_remaining - 1,
        f32::NEG_INFINITY,
        stop_fn,
    );
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
        let ev = -minimax(board, cache, depth_remaining - 1, best_eval, stop_fn);
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

    let best_white_eval = if board.color_to_move().is_white() {
        best_eval
    } else {
        -best_eval
    };

    cache
        .saved_positions
        .insert(board.hash(), (depth_remaining, best_white_eval));

    best_eval
}

pub fn search(
    board: &mut Board,
    cache: &mut InterMoveCache,
    stop_fn: fn() -> bool,
) -> Option<Move> {
    const SEARCH_DEPTH: usize = 6;

    let mut rng = rng();
    let mut options = MoveList::new();
    compute_legal_moves(&mut options, board);

    let mut best_move = options[0];
    let um = board.make_move(options[0]);
    // Minimax returns opponent's score
    let mut best_eval = -minimax(board, cache, SEARCH_DEPTH, f32::NEG_INFINITY, stop_fn);
    board.unmake_last_move(um);

    for mv in options.into_iter().skip(1) {
        if stop_fn() {
            return Some(best_move);
        }

        let um = board.make_move(mv);
        // Minimax returns opponent's score
        let ev = -minimax(board, cache, SEARCH_DEPTH, best_eval, stop_fn);
        board.unmake_last_move(um);

        if ev > best_eval {
            best_move = mv;
            best_eval = ev;
        }
    }

    Some(best_move)
}

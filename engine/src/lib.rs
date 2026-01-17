#![allow(dead_code)]
#![allow(unused)]

mod heatmaps;

use crate::heatmaps::{
    BISHOP_HEATMAP, KING_HEATMAP, KNIGHT_HEATMAP, PAWN_HEATMAP, QUEEN_HEATMAP, ROOK_HEATMAP,
};
use chess_lib::board::{Board, BoardHash, Move, PieceKind, Square};
use chess_lib::movegen::{MoveList, compute_legal_moves};
use lru::LruCache;
use rand::rng;
use std::num::NonZeroUsize;

pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

type HashData = (usize, f32);
pub struct InterMoveCache {
    // (depth searched, eval)
    // Evals from perspective of white
    saved_positions: LruCache<BoardHash, HashData>,
}

impl Default for InterMoveCache {
    fn default() -> Self {
        Self::new()
    }
}

impl InterMoveCache {
    pub fn new() -> InterMoveCache {
        InterMoveCache {
            // 8 GB
            saved_positions: LruCache::new(
                NonZeroUsize::try_from(8_000_000_000 / size_of::<HashData>()).unwrap(),
            ),
        }
    }
}

pub fn eval(board: &Board) -> f32 {
    let color_to_move = board.color_to_move();

    let mut score = 0f32;

    for (position, piece) in board.pieces().iter() {
        let position = if piece.color().is_white() {
            position.as_u8()
        } else {
            Square::at(position.file(), position.rank().flipped()).as_u8()
        };

        let mut s = match piece.kind() {
            PieceKind::Pawn => 1.0 + PAWN_HEATMAP[position as usize],
            PieceKind::Knight => 3.0 + KNIGHT_HEATMAP[position as usize],
            PieceKind::Bishop => 3.5 + BISHOP_HEATMAP[position as usize],
            PieceKind::Rook => 5.0 + ROOK_HEATMAP[position as usize],
            PieceKind::Queen => 8.0 + QUEEN_HEATMAP[position as usize],
            PieceKind::King => 0.0 + KING_HEATMAP[position as usize],
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
            // Will be replaced by deeper search
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
        .push(board.hash(), (depth_remaining, best_white_eval));

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

mod minimax_result;

use crate::InterMoveCache;
use crate::eval::eval;
use crate::minimax::minimax_result::MinimaxResult;
use crate::tt::{TTEntry, TTEntryType};
use chess_lib::board::{Board, Move};
use chess_lib::movegen::{MoveList, compute_legal_moves};
use rand::rng;
use std::cmp::{Ordering, PartialOrd};

fn minimax(
    board: &mut Board,
    cache: &mut InterMoveCache,
    depth_remaining: usize,
    prune: f32,
    stop_fn: fn() -> bool,
) -> MinimaxResult {
    if board.halfmoves_since_event() >= 150 {
        // 75 move rule
        return MinimaxResult::poisoned(0f32); // immediate draw
    }

    let score = minimax_inner(board, cache, depth_remaining, prune, stop_fn);

    if board.halfmoves_since_event() >= 100 && score.score <= 0f32 {
        // Assume current player will choose draw if in a bad position
        MinimaxResult::poisoned(0f32)
    } else {
        score
    }
}

impl PartialEq<f32> for MinimaxResult {
    fn eq(&self, other: &f32) -> bool {
        other == &self.score
    }
}

impl PartialOrd<f32> for MinimaxResult {
    fn partial_cmp(&self, other: &f32) -> Option<Ordering> {
        if other < &self.score {
            Some(Ordering::Less)
        } else if other == &self.score {
            Some(Ordering::Equal)
        } else {
            Some(Ordering::Greater)
        }
    }
}

fn minimax_inner(
    board: &mut Board,
    cache: &mut InterMoveCache,
    depth_remaining: usize,
    prune: f32,
    stop_fn: fn() -> bool,
) -> MinimaxResult {
    if depth_remaining == 0 {
        let score = eval(board);

        // Accept three-fold if position is bad
        if board.is_threefold() && score < 0f32 {
            return MinimaxResult::poisoned(0f32);
        }

        return MinimaxResult::normal(score);
    }

    if let Some(hd) = cache.transposition_table.get(&board.hash())
        && hd.depth_searched > depth_remaining
    // Otherwise will be replaced by deeper search
    {
        let score = if board.color_to_move().is_white() {
            hd.white_score
        } else {
            -hd.white_score
        };

        // TODO: With a full AB window rather than just prune, LowerBound and UpperBound could both allow for early returns
        if (hd.entry_type == TTEntryType::Exact
            || (board.color_to_move().is_white()
                && hd.entry_type == TTEntryType::LowerBound // Score being at least this good gets pruned
                && score > -prune)
            || (!board.color_to_move().is_white()
                && hd.entry_type == TTEntryType::UpperBound // Score being at least this good (black) gets pruned
                && score > -prune))
        {
            return MinimaxResult::normal(score);
        }
    }

    let mut options = MoveList::new();
    compute_legal_moves(&mut options, board);

    if options.is_empty() {
        cache.transposition_table.push(
            board.hash(),
            TTEntry {
                depth_searched: usize::MAX, // Checkmate is checkmate
                white_score: if board.color_to_move().is_white() {
                    f32::NEG_INFINITY
                } else {
                    f32::INFINITY
                },
                entry_type: TTEntryType::Exact,
            },
        );

        return MinimaxResult::normal(f32::NEG_INFINITY); // Checkmate
    }

    let mut best_move = options[0];
    let um = board.make_move(options[0]);
    // Minimax returns opponent's score
    let MinimaxResult {
        score: mut best_eval,
        mut poisoned,
    } = -minimax(
        board,
        cache,
        depth_remaining - 1,
        f32::NEG_INFINITY,
        stop_fn,
    );
    board.unmake_last_move(um);

    if best_eval > -prune {
        // Branch will be pruned
        cache.transposition_table.push(
            board.hash(),
            TTEntry {
                depth_searched: depth_remaining, // Remaining depth is what was searched to obtain eval
                white_score: board.color_to_move().apply_color_to_score(best_eval),
                entry_type: TTEntryType::color_bound(board.color_to_move()),
            },
        );
        return MinimaxResult::new(best_eval, poisoned);
    }

    for mv in options.into_iter().skip(1) {
        if stop_fn() {
            return MinimaxResult::new(best_eval, poisoned);
        }

        let um = board.make_move(mv);

        // Minimax returns opponent's score
        let MinimaxResult {
            score: ev,
            poisoned: np,
        } = -minimax(board, cache, depth_remaining - 1, best_eval, stop_fn);
        poisoned |= np;
        board.unmake_last_move(um);

        if ev == f32::INFINITY {
            return MinimaxResult::new(ev, poisoned);
        }

        if ev > best_eval {
            best_move = mv;
            best_eval = ev;

            if best_eval > -prune {
                // Branch will be pruned
                cache.transposition_table.push(
                    board.hash(),
                    TTEntry {
                        depth_searched: depth_remaining, // Remaining depth is what was searched to obtain eval
                        white_score: board.color_to_move().apply_color_to_score(best_eval),
                        entry_type: TTEntryType::color_bound(board.color_to_move()),
                    },
                );
                return MinimaxResult::new(best_eval, poisoned);
            }
        }
    }

    if !poisoned {
        cache.transposition_table.push(
            board.hash(),
            TTEntry {
                depth_searched: depth_remaining, // Remaining depth is what was searched to obtain eval
                white_score: board.color_to_move().apply_color_to_score(best_eval),
                entry_type: TTEntryType::Exact,
            },
        );
    }

    // Take draw if position is bad and three-fold is available
    if best_eval < 0f32 && board.is_threefold() {
        MinimaxResult::poisoned(0f32);
    }

    MinimaxResult::new(best_eval, poisoned)
}

pub fn search_minimax(
    board: &mut Board,
    cache: &mut InterMoveCache,
    stop_fn: fn() -> bool,
) -> Option<Move> {
    const SEARCH_DEPTH: usize = 5;

    let mut rng = rng();
    let mut options = MoveList::new();
    compute_legal_moves(&mut options, board);

    let mut best_move = options[0]; // Panic if given checkmate
    let um = board.make_move(options[0]);

    // Minimax returns opponent's score
    let MinimaxResult {
        score: mut best_eval,
        mut poisoned,
    } = -minimax_inner(board, cache, SEARCH_DEPTH, f32::NEG_INFINITY, stop_fn);
    board.unmake_last_move(um);

    for mv in options.into_iter().skip(1) {
        if stop_fn() {
            return Some(best_move);
        }

        let um = board.make_move(mv);

        // Minimax returns opponent's score
        let MinimaxResult {
            score: mut ev,
            poisoned: np,
        } = -minimax_inner(board, cache, SEARCH_DEPTH, best_eval, stop_fn);

        poisoned |= np;
        board.unmake_last_move(um);

        if ev > best_eval {
            best_move = mv;
            best_eval = ev;
        }
    }

    if !poisoned {
        cache.transposition_table.push(
            board.hash(),
            TTEntry {
                depth_searched: SEARCH_DEPTH, // Remaining depth is what was searched to obtain eval
                white_score: board.color_to_move().apply_color_to_score(best_eval),
                entry_type: todo!(),
            },
        );
    }

    // Try to force draw if position is losing
    if best_eval < 0f32 {
        if board.halfmoves_since_event() >= 100 {
            return None;
        }

        if board.is_threefold() {
            // Accept threefold draw
            return None;
        }
    }

    Some(best_move)
}

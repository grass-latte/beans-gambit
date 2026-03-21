mod minimax_result;

use crate::InterMoveCache;
use crate::eval::eval;
use crate::minimax::TimeManagementStrat::StrictLimit;
use crate::minimax::minimax_result::MinimaxResult;
use crate::tt::{TTEntry, TTEntryType};
use chess_lib::board::{Board, Move};
use chess_lib::movegen::{MoveList, compute_legal_moves};
use log::{info, log};
use std::cmp::PartialOrd;
use std::time::{Duration, Instant};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum TimeManagementStrat {
    StrictLimit,
    TargetLimit,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum MoveType {
    Move(Move),
    Draw,
    Pruned,
    Eval,
    Interrupted,
}

fn minimax<F>(
    toplevel: bool,
    board: &mut Board,
    cache: &mut InterMoveCache,
    depth_remaining: usize,
    prune: f32,
    stop_fn: &F,
) -> (MinimaxResult, MoveType)
where
    F: Fn() -> bool,
{
    if board.halfmoves_since_event() >= 150 {
        // 75 move rule
        return (MinimaxResult::poisoned(0f32), MoveType::Draw); // immediate draw
    }

    let (score, mv) = minimax_inner(toplevel, board, cache, depth_remaining, prune, stop_fn);

    if board.halfmoves_since_event() >= 100 && score.score <= 0f32 {
        // Assume current player will choose draw if in a bad position
        (MinimaxResult::poisoned(0f32), MoveType::Draw)
    } else {
        (score, mv)
    }
}

fn minimax_inner<F>(
    toplevel: bool,
    board: &mut Board,
    cache: &mut InterMoveCache,
    depth_remaining: usize,
    prune: f32,
    stop_fn: &F,
) -> (MinimaxResult, MoveType)
where
    F: Fn() -> bool,
{
    if depth_remaining == 0 {
        let score = eval(board);

        // Accept three-fold if position is bad
        if board.is_threefold() && score < 0f32 {
            return (MinimaxResult::poisoned(0f32), MoveType::Draw);
        }

        return (MinimaxResult::normal(score), MoveType::Eval);
    }

    // Force search if toplevel - probably not worth storing moves with evals to speed up
    // move selection in previously seen position
    if !toplevel
        && let Some(hd) = cache.transposition_table.get(&board.hash())
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
            return (MinimaxResult::normal(score), MoveType::Eval);
        }
    }

    // TODO: Consider randomising move order
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

        return (MinimaxResult::normal(f32::NEG_INFINITY), MoveType::Eval); // Checkmate
    }

    let mut best_move = options[0];
    let um = board.make_move(options[0]);
    // Minimax returns opponent's score
    let (mr, _) = minimax(
        false,
        board,
        cache,
        depth_remaining - 1,
        f32::NEG_INFINITY,
        stop_fn,
    );
    let MinimaxResult {
        score: mut best_eval,
        mut poisoned,
    } = -mr;
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
        return (MinimaxResult::new(best_eval, poisoned), MoveType::Pruned);
    }

    for mv in options.into_iter().skip(1) {
        if stop_fn() {
            return (
                MinimaxResult::new(best_eval, poisoned),
                MoveType::Interrupted,
            );
        }

        let um = board.make_move(mv);

        // Minimax returns opponent's score
        let (mr, _) = minimax(false, board, cache, depth_remaining - 1, best_eval, stop_fn);
        let MinimaxResult {
            score: ev,
            poisoned: np,
        } = -mr;
        poisoned |= np;
        board.unmake_last_move(um);

        if ev == f32::INFINITY {
            return (MinimaxResult::new(ev, poisoned), MoveType::Move(mv));
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
                return (MinimaxResult::new(best_eval, poisoned), MoveType::Pruned);
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

    (
        MinimaxResult::new(best_eval, poisoned),
        MoveType::Move(best_move),
    )
}

pub fn search_minimax(
    board: &mut Board,
    cache: &mut InterMoveCache,
    stop_fn: fn() -> bool,
    target_move_time: Duration,
    time_management_strat: TimeManagementStrat,
) -> Option<Move> {
    info!(
        "FEN {} | Target time {:?} | Strat: {:?}",
        board.to_fen(),
        target_move_time,
        time_management_strat
    );

    let limit = Instant::now() + target_move_time;

    let mm_stop_fn: Box<dyn Fn() -> bool> = if time_management_strat == StrictLimit {
        Box::new(move || -> bool { stop_fn() || Instant::now() > limit })
    } else {
        Box::new(move || -> bool { stop_fn() })
    };

    let mut options = MoveList::new();
    compute_legal_moves(&mut options, board);
    let mut best_move = Some(options[0]); // If we fail first search

    let mut search_depth: usize = 2; // Keep even to eval on our turn

    loop {
        info!("Starting search at depth {search_depth}");

        let start = Instant::now();

        let (_, best_move_at_sd) = minimax(
            true,
            board,
            cache,
            search_depth,
            f32::NEG_INFINITY,
            &mm_stop_fn,
        );

        let time_taken = start.elapsed();

        info!("Completed depth {} in {:?}", search_depth, start.elapsed());
        match best_move_at_sd {
            MoveType::Move(mv) => best_move = Some(mv),
            MoveType::Draw => best_move = None,
            MoveType::Pruned => panic!(),
            MoveType::Eval => panic!(),
            MoveType::Interrupted => {
                info!("Search interrupted");
                break;
            }
        };

        // Assume next iteration will take 120x current iteration
        const ITERATION_COST_FACTOR: u32 = 120;
        if Instant::now() + (time_taken * ITERATION_COST_FACTOR) > limit {
            info!("Next depth expected to take to long");
            break;
        }

        search_depth += 2;
    }

    info!("Best move: {:?}", best_move);

    best_move
}

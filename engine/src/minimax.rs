use crate::eval::eval;
use crate::minimax::TimeManagementStrat::StrictLimit;
use crate::results::{Score, SearchResult};
use crate::tt::{TTEntry, TTEntryType};
use crate::{InterMoveCache, results};
use chess_lib::board::{Board, Move};
use chess_lib::movegen::{MoveList, compute_legal_moves};
use log::{debug, info, log};
#[cfg(debug_assertions)]
use std::backtrace::Backtrace;
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
    depth_remaining: u8,
    prune: Score,
    stop_fn: &F,
) -> (SearchResult, MoveType)
where
    F: Fn() -> bool,
{
    if board.halfmoves_since_event() >= 150 {
        // 75 move rule
        return (SearchResult::poisoned(Score::ZERO), MoveType::Draw); // immediate draw
    }

    // TODO: Should this be forced?
    if board.is_threefold() {
        return (SearchResult::poisoned(Score::ZERO), MoveType::Draw);
    }

    let (score, mt) = minimax_inner(toplevel, board, cache, depth_remaining, prune, stop_fn);
    if mt == MoveType::Interrupted {
        return (score, mt);
    }

    if board.halfmoves_since_event() >= 100 && score.score <= Score::ZERO {
        // Assume current player will choose draw if in a bad position
        (SearchResult::poisoned(Score::ZERO), MoveType::Draw)
    } else {
        (score, mt)
    }
}

fn minimax_inner<F>(
    toplevel: bool,
    board: &mut Board,
    cache: &mut InterMoveCache,
    depth_remaining: u8,
    prune: Score,
    stop_fn: &F,
) -> (SearchResult, MoveType)
where
    F: Fn() -> bool,
{
    if depth_remaining == 0 {
        let score = eval(board);

        // Accept three-fold if position is bad
        if board.is_threefold() && score < Score::ZERO {
            return (SearchResult::poisoned(Score::ZERO), MoveType::Draw);
        }

        #[cfg(debug_assertions)]
        return (
            SearchResult::new_backtrace(score, false, Backtrace::capture(), board.to_fen()),
            MoveType::Eval,
        );
        #[cfg(not(debug_assertions))]
        return (SearchResult::normal(score), MoveType::Eval);
    }

    // Force search if toplevel - probably not worth storing moves with evals to speed up
    // move selection in previously seen position
    if !toplevel
        && let Some(hd) = cache.transposition_table.get(&board.hash())
        && hd.depth_searched >= depth_remaining
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
            return (SearchResult::normal(score), MoveType::Eval);
        }
    }

    // TODO: Consider randomising move order
    let mut options = MoveList::new();
    let is_check = compute_legal_moves(&mut options, board);

    if options.is_empty() {
        let score = if is_check {
            Score::BADE_MATE_IN_ZERO
        } else {
            Score::ZERO
        };

        cache.transposition_table.push(
            board.hash(),
            TTEntry {
                depth_searched: u8::MAX, // End of game
                white_score: board.color_to_move().apply_color_to_score(score),
                entry_type: TTEntryType::Exact,
            },
        );

        debug!("{} {score:?} {is_check}", board.to_fen());
        return (SearchResult::normal(score), MoveType::Eval); // Checkmate
    }

    let mut best_move = options[0];
    let mut best_eval = Score::NEG_INF;
    let mut poisoned = false;
    #[cfg(debug_assertions)]
    let (mut best_backtrace, mut best_fen) = {
        let mut best_backtrace = Backtrace::capture();
        let um = board.make_move(best_move);
        let mut best_fen = board.to_fen();
        board.unmake_last_move(um);
        (best_backtrace, best_fen)
    };

    for mv in options {
        if stop_fn() {
            return (
                SearchResult::new(best_eval, poisoned),
                MoveType::Interrupted,
            );
        }

        let um = board.make_move(mv);

        // Minimax returns opponent's score
        let (sr, mt) = minimax(false, board, cache, depth_remaining - 1, best_eval, stop_fn);
        board.unmake_last_move(um);

        if mt == MoveType::Interrupted {
            return (-sr, mt);
        }

        #[cfg(debug_assertions)]
        let SearchResult {
            score: ev,
            poisoned: np,
            backtrace,
            fen,
        } = -sr;
        #[cfg(not(debug_assertions))]
        let SearchResult {
            score: ev,
            poisoned: np,
            ..
        } = -sr;
        let ev = ev.increment_mate_in();
        poisoned |= np;

        // TODO: Should we early return for checkmates?

        if ev > best_eval {
            best_move = mv;
            best_eval = ev;
            #[cfg(debug_assertions)]
            {
                best_backtrace = backtrace;
                best_fen = fen;
            }

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
                return (SearchResult::new(best_eval, poisoned), MoveType::Pruned);
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
    if best_eval < Score::ZERO && board.is_threefold() {
        return (SearchResult::poisoned(Score::ZERO), MoveType::Draw);
    }

    #[cfg(debug_assertions)]
    return (
        SearchResult::new_backtrace(best_eval, poisoned, best_backtrace, best_fen),
        MoveType::Move(best_move),
    );
    #[cfg(not(debug_assertions))]
    (
        SearchResult::new(best_eval, poisoned),
        MoveType::Move(best_move),
    )
}

// TODO: Write tests
// TODO: Account for material draws
pub fn search_minimax(
    board: &mut Board,
    cache: &mut InterMoveCache,
    stop_fn: fn() -> bool,
    target_move_time: Duration,
    time_management_strat: TimeManagementStrat,
) -> (Option<Move>, Score) {
    let limit = Instant::now() + target_move_time;

    let mm_stop_fn: Box<dyn Fn() -> bool> = if time_management_strat == StrictLimit {
        Box::new(move || -> bool { stop_fn() || Instant::now() > limit })
    } else {
        Box::new(move || -> bool { stop_fn() })
    };

    let mut options = MoveList::new();
    compute_legal_moves(&mut options, board);
    let mut best_move = Some(options[0]); // If we fail first search
    let um = board.make_move(options[0]);
    let mut eval_after_move = eval(board);
    board.unmake_last_move(um);

    let mut search_depth: u8 = 2; // Keep even to eval on our turn

    while search_depth < 250 {
        info!("Starting search at depth {search_depth}");

        let start = Instant::now();

        let (sr, best_move_at_sd) = minimax(
            true,
            board,
            cache,
            search_depth,
            Score::NEG_INF,
            &mm_stop_fn,
        );

        let time_taken = start.elapsed();

        info!("Completed depth {} in {:?}", search_depth, start.elapsed());

        match best_move_at_sd {
            MoveType::Move(mv) => best_move = Some(mv),
            MoveType::Draw => best_move = None, // TODO: UCI doesn't support choosing to draw
            MoveType::Pruned => panic!(),
            MoveType::Eval => panic!(),
            MoveType::Interrupted => {
                info!("Search interrupted");
                break;
            }
        };

        debug!("Minimax Result {:#?} | {:?}", sr, best_move);
        eval_after_move = sr.score;

        // Assume next iteration will take 600x current iteration
        const ITERATION_COST_FACTOR: u32 = 10;
        if Instant::now() + (time_taken * ITERATION_COST_FACTOR) > limit {
            info!("Next depth expected to take to long");
            break;
        }

        search_depth += 1;
    }

    info!("Best move: {:?}", best_move);

    (best_move, eval_after_move)
}

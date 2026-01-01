//! Debugging utility for narrowing down failing Perft runs to find the exact position where
//! the move generator gets the incorrect number of moves.
//! Prints each move from the provided position alongside the size of the sub-tree after making
//! that move. Then prompts for the next move to make and repeats.
//! Used as follows:
//! `util-divide "{fen}" {depth}`

use std::{
    env::args,
    io::{stdin, stdout, Write},
    process::exit,
};

use chess_lib::{
    board::{Board, Move},
    movegen::{MoveGenerator, MoveList},
};
use color_print::{ceprintln, cprint, cprintln};

/// Perft implemented using bulk counting.
fn perft(board: &mut Board, movegen: &MoveGenerator, depth: u64) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut move_list = MoveList::new();
    movegen.compute_legal_moves(&mut move_list, &board);

    if depth == 1 {
        return move_list.len() as u64;
    }

    let mut nodes: u64 = 0;
    for &mv in move_list.iter() {
        let unmake = board.make_move(mv);
        nodes += perft(board, movegen, depth - 1);
        board.unmake_last_move(unmake);
    }

    nodes
}

fn user_error(message: impl AsRef<str>) -> ! {
    ceprintln!("<red>Error</red>: {}", message.as_ref());
    exit(1);
}

fn main() {
    let mut args = args().skip(1);
    let fen = args
        .next()
        .unwrap_or_else(|| user_error("Expected 2 arguments - FEN string and search depth."));
    let depth: u64 = args
        .next()
        .unwrap_or_else(|| user_error("Expected 2 arguments - FEN string and search depth."))
        .parse()
        .unwrap_or_else(|_| user_error("Couldn't parse depth."));

    let mut board = Board::from_fen(&fen).expect("couldn't parse fen string");
    let mg = MoveGenerator::new();
    let mut moves = MoveList::new();

    loop {
        moves.clear();
        mg.compute_legal_moves(&mut moves, &board);
        moves.sort_by_key(Move::as_uci);

        let mut total = 0;
        for &mv in &moves {
            let um = board.make_move(mv);
            let nodes = perft(&mut board, &mg, depth - 1);
            total += nodes;
            board.unmake_last_move(um);
            cprintln!("<green>{}</green>: {}", mv.as_uci(), nodes);
        }
        cprintln!("Total nodes searched: {total}");

        let mut next_move = String::new();
        cprint!("<yellow>Next move</yellow>: ");
        stdout().flush().unwrap();
        stdin()
            .read_line(&mut next_move)
            .expect("couldn't read stdin");
        let next_move = next_move.trim();
        let next_move = Move::from_uci(&next_move)
            .unwrap_or_else(|| user_error(&format!("Couldn't parse move: '{}'", &next_move)));

        if !moves.contains(&next_move) {
            user_error(format!("Illegal move: {}", next_move.as_uci()));
        }

        board.make_move(next_move);
    }
}

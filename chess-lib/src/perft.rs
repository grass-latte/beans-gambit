//! https://www.chessprogramming.org/Perft
//! Perft is a function that counts the total number of leaf nodes to a certain depth.
//! This can be used to verify make/unmake and move generation and as a benchmark for these
//! functions.
//! We include Perft to depth 4 as unit tests, to verify the library, and higher depths as
//! benchmarks.

#![cfg(test)]

use crate::{
    board::Board,
    movegen::{MoveGenerator, MoveList},
};

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

/// Compare perft against expected results.
fn test_perft(label: &str, fen: &str, expected_results: &[u64]) {
    let mg = MoveGenerator::new();
    for (depth, &expected_result) in (1..).zip(expected_results) {
        let mut board = Board::from_fen(fen).unwrap();
        let result = perft(&mut board, &mg, depth);
        assert_eq!(result, expected_result, "perft {label} depth {depth}");
    }
}

/*
#[test]
fn perft_starting_position() {
    test_perft(
        "starting position",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        &[20, 400, 8902, 197_281, 4_865_609],
    );
}
*/

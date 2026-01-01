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

// Test positions and Perft results are from https://www.chessprogramming.org/Perft_Results.

#[test]
fn test_perft_starting_position() {
    test_perft(
        "starting position",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        &[20, 400, 8902, 197_281, 4_865_609],
    );
}

#[test]
fn test_perft_kiwipete() {
    test_perft(
        "kiwipete",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        &[48, 2039, 97_862, 4_085_603, 193_690_690],
    );
}

#[test]
fn test_perft_position_3() {
    test_perft(
        "position 3",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        &[14, 191, 2812, 43_238, 674_624],
    );
}

#[test]
fn test_perft_position_4() {
    test_perft(
        "position 4",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        &[6, 264, 9_467, 422_333],
    );
}

#[test]
fn test_perft_position_5() {
    test_perft(
        "position 5",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        &[44, 1_486, 62_379, 2_103_487, 89_941_194],
    );
}

#[test]
fn test_perft_position_6() {
    test_perft(
        "position 6",
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        &[46, 2_079, 89_890, 3_894_594, 164_075_551],
    );
}

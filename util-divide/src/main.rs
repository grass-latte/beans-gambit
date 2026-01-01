//! Debugging utility for narrowing down failing Perft runs to find the exact position where
//! the move generator gets the incorrect number of moves.
//! Prints each move from the provided position alongside the size of the sub-tree after making
//! that move. Then prompts for the next move to make and repeats.
//! Also has an "auto" mode that compares moves against Stockfish.

use std::{
    collections::HashSet,
    io::{stdin, stdout, Read, Write},
    process::{exit, ChildStdout, Command, Stdio},
};

use chess_lib::{
    board::{Board, Move},
    movegen::{MoveGenerator, MoveList},
};
use clap::Parser;
use clap_derive::Parser;
use color_print::{ceprintln, cprint, cprintln};

#[derive(Parser)]
#[command(version, about, long_about = Some("Utility for debugging incorrect move generation."))]
struct Cli {
    #[arg(long)]
    fen: String,
    #[arg(long)]
    depth: u64,
    #[arg(long)]
    auto: bool,
}

fn read_lines_until(stdout: &mut ChildStdout, end: impl Fn(&str) -> bool) -> Vec<String> {
    let mut lines = Vec::new();
    let mut next_byte = [0u8];
    let mut current_line_bytes = Vec::new();

    loop {
        stdout.read_exact(&mut next_byte).unwrap();

        if next_byte[0] == '\n'.to_ascii_uppercase() as u8 {
            // Got a line.
            let line = String::from_utf8(current_line_bytes.clone()).unwrap();

            lines.push(line.clone());
            current_line_bytes.clear();

            if end(&line) {
                // End.
                return lines;
            }
        } else {
            current_line_bytes.push(next_byte[0]);
        }
    }
}

/// Perft implemented using bulk counting.
fn perft(board: &mut Board, movegen: &MoveGenerator, depth: u64) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut move_list = MoveList::new();
    movegen.compute_legal_moves(&mut move_list, board);

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
    ceprintln!("<bold><red>error:</red></bold> {}", message.as_ref());
    exit(1);
}

fn divide_manual(fen: &str, depth: u64) {
    let mut board = Board::from_fen(fen).expect("couldn't parse fen string");
    let mg = MoveGenerator::new();
    let mut moves = MoveList::new();

    for depth in (1..depth).rev() {
        moves.clear();
        mg.compute_legal_moves(&mut moves, &board);
        moves.sort_by_key(Move::as_uci);

        let mut total = 0;
        for &mv in &moves {
            let um = board.make_move(mv);
            let nodes = perft(&mut board, &mg, depth);
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
        let next_move = Move::from_uci(next_move)
            .unwrap_or_else(|| user_error(format!("Couldn't parse move: '{}'", &next_move)));

        if !moves.contains(&next_move) {
            user_error(format!("Illegal move: {}", next_move.as_uci()));
        }

        board.make_move(next_move);
    }
}

fn divide_auto(fen: &str, depth: u64) {
    // Check that Stockfish is present.
    let ok = Command::new("stockfish").arg("help").output().is_ok();
    if !ok {
        user_error("stockfish binary not found.");
    }

    let mut board = Board::from_fen(fen).expect("couldn't parse fen string");
    let mg = MoveGenerator::new();
    let mut moves = MoveList::new();

    for depth in (0..depth).rev() {
        moves.clear();
        mg.compute_legal_moves(&mut moves, &board);
        let our_moves = moves
            .iter()
            .copied()
            .map(|mv| {
                let um = board.make_move(mv);
                let nodes = perft(&mut board, &mg, depth);
                board.unmake_last_move(um);

                (mv, nodes)
            })
            .collect::<HashSet<_>>();

        // Consult with the fish.
        let mut stockfish = Command::new("stockfish")
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .expect("Stockfish failed to start.");

        let mut stdin = stockfish.stdin.take().unwrap();
        let mut stdout = stockfish.stdout.take().unwrap();
        let mut stockfish_moves = HashSet::new();

        stdin
            .write_all(format!("position fen {}\n", board.to_fen()).as_bytes())
            .unwrap();
        stdin
            .write_all(format!("go perft {}\n", depth + 1).as_bytes())
            .unwrap();

        for line in read_lines_until(&mut stdout, |line| line.trim().is_empty()) {
            let Some((before_colon, after_colon)) = line.split_once(':') else {
                continue;
            };
            let Some(mv) = Move::from_uci(before_colon) else {
                continue;
            };
            let Ok(nodes) = after_colon.trim().parse::<u64>() else {
                continue;
            };

            stockfish_moves.insert((mv, nodes));
        }

        stockfish.kill().unwrap();
        stockfish.wait().unwrap();

        // Compare board after each move.
        for &(mv, _) in &stockfish_moves {
            let initial_fen = board.to_fen();
            let um = board.make_move(mv);
            let board_fen = board.to_fen();
            board.unmake_last_move(um);

            let mut stockfish = Command::new("stockfish")
                .stdout(Stdio::piped())
                .stdin(Stdio::piped())
                .spawn()
                .expect("Stockfish failed to start.");

            let mut stdin = stockfish.stdin.take().unwrap();
            let mut stdout = stockfish.stdout.take().unwrap();

            stdin
                .write_all(
                    format!("position fen {} moves {}\n", board.to_fen(), mv.as_uci()).as_bytes(),
                )
                .unwrap();
            stdin.write_all("d\n".as_bytes()).unwrap();

            let fen_line = read_lines_until(&mut stdout, |line| line.starts_with("Fen:"))
                .into_iter()
                .last()
                .unwrap();
            let stockfish_fen = fen_line.strip_prefix("Fen: ").unwrap().trim();

            stockfish.kill().unwrap();
            stockfish.wait().unwrap();

            if board_fen != stockfish_fen {
                cprintln!("<red>Disagreement on board state</red>");
                cprintln!("From position <green>{initial_fen}</green>");
                cprintln!("After move <green>{}</green>", mv.as_uci());
                cprintln!("We got <green>{board_fen}</green>");
                cprintln!("Stockfish got <green>{stockfish_fen}</green>");
                return;
            }
        }

        // Compare moves.

        if our_moves == stockfish_moves {
            continue;
        }

        // Find the different move.
        let &(mv, count) = stockfish_moves
            .symmetric_difference(&our_moves)
            .next()
            .unwrap();

        if count == 1 {
            if stockfish_moves.contains(&(mv, count)) {
                cprintln!(
                    "<red>Found incorrect move</red>\nPosition <green>{}</green> Stockfish got <green>{}</green>, we didn't.",
                    board.to_fen(),
                    mv.as_uci()
                );
            } else {
                cprintln!(
                    "<red>Found incorrect move</red>\nPosition <green>{}</green> We got <green>{}</green>, Stockfish didn't.",
                    board.to_fen(),
                    mv.as_uci()
                );
            }
            return;
        }

        board.make_move(mv);
    }
    cprintln!("<green>Correct.</green>");
}

fn main() {
    let cli = Cli::parse();

    if cli.auto {
        divide_auto(&cli.fen, cli.depth)
    } else {
        divide_manual(&cli.fen, cli.depth)
    }
}

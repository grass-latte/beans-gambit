#[path = "src/shared.rs"]
mod shared;

use chess_lib::board::{Board, BoardFile, PieceKind};
use std::fs;

use crate::shared::{SerialisedBookMove, serialise_book_move};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use shakmaty::fen::Fen;
use shakmaty::zobrist::Zobrist64;
use shakmaty::{Chess, File, Move, Position, Rank, Role, Square};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{Cursor, Write};
// AI Slop?

#[derive(Debug, Clone)]
pub struct PolyglotEntry {
    pub key: u64,
    pub mv: u16,
    pub weight: u16,
    pub learn: u32,
}

#[derive(Debug, Clone)]
pub struct BookMove {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Role>,
    pub weight: u16,
}

// ── Parse the raw move u16 into fields ────────────────────────────────────
//
//  bits 0–2   to-file
//  bits 3–5   to-rank
//  bits 6–8   from-file
//  bits 9–11  from-rank
//  bits 12–14 promotion piece (0=none 1=knight 2=bishop 3=rook 4=queen)
fn decode_move(mv: u16, weight: u16) -> Option<BookMove> {
    let to_file = (mv & 0x0007) as u8;
    let to_rank = ((mv >> 3) & 0x0007) as u8;
    let from_file = ((mv >> 6) & 0x0007) as u8;
    let from_rank = ((mv >> 9) & 0x0007) as u8;
    let promo = (mv >> 12) & 0x0007;

    let from = Square::from_coords(File::new(from_file as u32), Rank::new(from_rank as u32));
    let to = Square::from_coords(File::new(to_file as u32), Rank::new(to_rank as u32));

    let promotion = match promo {
        1 => Some(Role::Knight),
        2 => Some(Role::Bishop),
        3 => Some(Role::Rook),
        4 => Some(Role::Queen),
        _ => None,
    };

    Some(BookMove {
        from,
        to,
        promotion,
        weight,
    })
}

pub fn load_polyglot(path: &str) -> HashMap<u64, Vec<BookMove>> {
    let bytes = fs::read(path).expect("failed to read polyglot file");
    assert!(
        bytes.len().is_multiple_of(16),
        "file size must be a multiple of 16 bytes"
    );

    let mut map: HashMap<u64, Vec<BookMove>> = HashMap::new();
    let mut cur = Cursor::new(bytes);

    while let (Ok(key), Ok(mv), Ok(weight), Ok(learn)) = (
        cur.read_u64::<BigEndian>(),
        cur.read_u16::<BigEndian>(),
        cur.read_u16::<BigEndian>(),
        cur.read_u32::<BigEndian>(),
    ) {
        let _ = learn;
        if let Some(book_move) = decode_move(mv, weight) {
            map.entry(key).or_default().push(book_move);
        }
    }

    for moves in map.values_mut() {
        moves.sort_by(|a, b| b.weight.cmp(&a.weight));
    }

    map
}

fn book_move_to_legal(pos: &Chess, bm: &BookMove) -> Option<Move> {
    let legals = pos.legal_moves();

    // Polyglot encodes castling as king-captures-rook; shakmaty uses
    // king-to-castling-destination.  We match on from/to covering both
    // representations.
    legals.into_iter().find(|m| {
        match m {
            Move::Normal {
                from,
                to,
                promotion,
                ..
            } => *from == bm.from && *to == bm.to && promotion.map(|r| r) == bm.promotion,
            Move::Castle { king, rook } => {
                // king-captures-rook style
                *king == bm.from && *rook == bm.to
            }
            Move::EnPassant { from, to } => *from == bm.from && *to == bm.to,
            Move::Put { .. } => false,
        }
    })
}

pub fn explore(start: &Chess, book: &HashMap<u64, Vec<BookMove>>, book_file: &mut fs::File) {
    let mut visited = HashSet::new();
    let mut queue: VecDeque<Chess> = VecDeque::new();
    queue.push_back(start.clone());
    visited.insert(
        start
            .zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal)
            .0,
    );

    while let Some(pos) = queue.pop_front() {
        let hash: u64 = pos
            .zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal)
            .0;

        let Some(book_moves) = book.get(&hash) else {
            continue; // Leaf
        };

        // Per position logic

        // Write hash of location
        let mut pos_board =
            Board::from_fen(&Fen::from_position(&pos, shakmaty::EnPassantMode::Legal).to_string())
                .unwrap();
        book_file
            .write_u64::<LittleEndian>(pos_board.hash().u64())
            .unwrap();

        let mut legal_moves = Vec::new();
        for bm in book_moves {
            let Some(legal_move) = book_move_to_legal(&pos, bm) else {
                panic!(
                    "Illegal move: {} | {:?}",
                    Fen::from_position(&pos, shakmaty::EnPassantMode::Legal),
                    bm
                );
                // continue;
            };

            let mut next = pos.clone();
            next.play_unchecked(legal_move);

            legal_moves.push((
                bm.from,
                bm.to,
                bm.promotion,
                bm.weight,
                Fen::from_position(&next, shakmaty::EnPassantMode::Legal).to_string(),
            ));

            if visited.insert(
                next.zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal)
                    .0,
            ) {
                queue.push_back(next);
            }
        }

        // Write number of possible moves
        let mut legal_moves: &[_] = &legal_moves;
        if legal_moves.len() > u8::MAX as usize {
            legal_moves = &legal_moves[..u8::MAX as usize];
        }
        book_file.write_u8(legal_moves.len() as u8).unwrap();

        for (from, to, promotion, weight, expected_fen) in legal_moves {
            fn convert_square(square: &Square) -> chess_lib::board::Square {
                unsafe {
                    chess_lib::board::Square::at_xy_unchecked(
                        square.file().to_u32() as u8,
                        square.rank().to_u32() as u8,
                    )
                }
            }

            fn convert_role(role: &Role) -> PieceKind {
                match role {
                    Role::Pawn => PieceKind::Pawn,
                    Role::Knight => PieceKind::Knight,
                    Role::Bishop => PieceKind::Bishop,
                    Role::Rook => PieceKind::Rook,
                    Role::Queen => PieceKind::Queen,
                    Role::King => PieceKind::King,
                }
            }

            let from = convert_square(from);
            let original_to = *to;
            let mut to = convert_square(to);

            // TODO: Remove after board is made tolerant of castling by moving king to edge
            // Castling represented as moving to corner for some reason
            if pos_board.pieces().get(from).unwrap().kind() == PieceKind::King
                && let delta = (to.file().as_u8() as i8) - (from.file().as_u8() as i8)
                && delta.abs() > 2
            {
                // -4 * 2 = -8
                // -8 / 4 = -2
                let delta = (delta * 2) / delta.abs();
                to = chess_lib::board::Square::at(
                    BoardFile::from_u8(((from.file().as_u8() as i8) + delta) as u8).unwrap(),
                    to.rank(),
                )
            }

            let promotion = promotion.as_ref().map(convert_role);
            let weight = *weight;

            let pre_fen = pos_board.to_fen();
            let um = pos_board.make_move(chess_lib::board::Move {
                source: from,
                destination: to,
                promotion,
            });
            assert_eq!(
                pos_board.hash(),
                Board::from_fen(expected_fen).unwrap().hash(),
                "\n{pre_fen}\n{from:?} {to:?}[{original_to}] {promotion:?}\n{} vs. {expected_fen}",
                pos_board.to_fen(),
            );
            pos_board.unmake_last_move(um);

            book_file
                .write_all(&serialise_book_move(SerialisedBookMove {
                    from,
                    to,
                    promotion,
                    weight,
                }))
                .unwrap();
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/shared.rs");
    println!("cargo:rerun-if-changed=static/source");

    eprintln!("Running engine build.rs");

    let characteristic_hash = Board::starting().hash();

    fs::create_dir_all("static/gen").unwrap();
    fs::write("static/gen/hash.txt", format!("{characteristic_hash}")).unwrap();

    // Build graph by starting at initial position
    // let book = load_polyglot("static/source/Cerebellum/Cerebellum3Merge.bin");
    let book = load_polyglot("static/source/gm2001.bin");
    eprintln!("Loaded {} unique book positions", book.len());

    let mut book_file = fs::File::create("static/gen/book.bin").unwrap();

    book_file
        .write_u64::<LittleEndian>(characteristic_hash.u64())
        .unwrap();

    let initial: Chess = Chess::default();
    explore(&initial, &book, &mut book_file);
}

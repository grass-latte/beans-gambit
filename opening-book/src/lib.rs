mod shared;

use crate::shared::{SERIALISED_MOVE_SIZE, deserialise_book_move};
use chess_lib::board::{Board, BoardHash, Move};
use human_bytes::human_bytes;
use rand::RngExt;
use std::collections::HashMap;

const BOOK_BYTES: &[u8] = include_bytes!("../static/gen/book.bin");

pub trait OpeningBook {
    fn statistics(&self) -> String;
    fn get_fast(&self, position: BoardHash) -> Option<Move>;
    fn get_weighted(&self, position: BoardHash) -> Option<Move>;
}

pub type MoveEntry = (u32, Move);
pub type OpeningBookEntry = (u32, Vec<MoveEntry>);
pub struct DefaultOpeningBook {
    book: HashMap<BoardHash, OpeningBookEntry>,
    total_moves: usize,
}

impl DefaultOpeningBook {
    pub fn initialise() -> DefaultOpeningBook {
        let characteristic_hash = Board::starting().hash().u64().to_le_bytes();
        assert!(BOOK_BYTES.starts_with(&characteristic_hash));

        let mut book = HashMap::new();
        let mut total_moves = 0;

        let mut i = 8;
        while i < BOOK_BYTES.len() {
            let position_hash = u64::from_le_bytes(BOOK_BYTES[i..i + 8].try_into().unwrap());
            i += 8;
            let move_options = BOOK_BYTES[i];
            total_moves += move_options as usize;
            i += 1;

            let mut moves = Vec::new();
            let mut weight_total: u32 = 0;

            for _ in 0..move_options {
                let mv = deserialise_book_move(&BOOK_BYTES[i..i + SERIALISED_MOVE_SIZE]);
                weight_total += mv.weight as u32;
                moves.push(mv);
                i += SERIALISED_MOVE_SIZE;
            }

            moves.sort_by(|mv1, mv2| mv2.weight.cmp(&mv1.weight));

            book.insert(
                BoardHash::from_u64(position_hash),
                (
                    weight_total,
                    moves
                        .iter()
                        .map(|mv| {
                            (
                                mv.weight as u32,
                                Move {
                                    source: mv.from,
                                    destination: mv.to,
                                    promotion: mv.promotion,
                                },
                            )
                        })
                        .collect(),
                ),
            );
        }

        DefaultOpeningBook { book, total_moves }
    }
}

impl OpeningBook for DefaultOpeningBook {
    fn statistics(&self) -> String {
        format!(
            "Opening Book: [{} positions | {} moves | {}]",
            self.book.len(),
            self.total_moves,
            human_bytes(
                (size_of::<OpeningBookEntry>()
                    + self.book.len() * (size_of::<BoardHash>() + size_of::<OpeningBookEntry>())
                    + self.total_moves * size_of::<MoveEntry>()) as f32
            )
        )
    }

    fn get_fast(&self, position: BoardHash) -> Option<Move> {
        self.book.get(&position).map(|v| v.1[0].1)
    }

    fn get_weighted(&self, position: BoardHash) -> Option<Move> {
        let mut rng = rand::rng();
        let Some((total, options)) = self.book.get(&position) else {
            return None;
        };

        let mut selection = rng.random_range(0..*total);

        for (weight, mv) in options {
            if selection < *weight {
                return Some(*mv);
            }
            selection -= *weight;
        }

        panic!()
    }
}

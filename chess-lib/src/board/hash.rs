use crate::board::{BoardFile, CastlingRights, Piece, Square};
use get_random_const::random;

struct Dummy {
    a: i32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct BoardHash(u64);

const PIECE_HASHES: [u64; 12 * 64] = random!([u64; 768]);
const BLACK_TO_MOVE_HASH: u64 = random!(u64);
const CASTLING_RIGHTS: [u64; 16] = random!([u64; 16]);
const EN_PASSANT_FILE: [u64; 8] = random!([u64; 8]);

impl BoardHash {
    pub const fn zero() -> Self {
        BoardHash(0)
    }

    pub const fn toggle_piece(self, piece: Piece, square: Square) -> Self {
        BoardHash(self.0 ^ PIECE_HASHES[(piece.as_u8() as usize) * 64 + square.as_u8() as usize])
    }

    pub const fn update_castling_rights(
        self,
        prev_rights: CastlingRights,
        rights: CastlingRights,
    ) -> Self {
        let without_prev = self.0 ^ CASTLING_RIGHTS[prev_rights.as_u8() as usize];
        BoardHash(without_prev ^ CASTLING_RIGHTS[rights.as_u8() as usize])
    }

    pub const fn toggle_move(self) -> Self {
        BoardHash(self.0 ^ BLACK_TO_MOVE_HASH)
    }

    pub const fn set_en_passant_file(
        self,
        prev_file: Option<BoardFile>,
        file: Option<BoardFile>,
    ) -> Self {
        let hash = if let Some(prev_file) = prev_file {
            self.0 ^ EN_PASSANT_FILE[prev_file.as_u8() as usize]
        } else {
            self.0
        };

        let hash = if let Some(file) = file {
            hash ^ EN_PASSANT_FILE[file.as_u8() as usize]
        } else {
            hash
        };

        BoardHash(hash)
    }
}

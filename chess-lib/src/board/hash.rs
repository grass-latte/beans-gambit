use crate::board::{BoardFile, CastlingRights, Piece, Square};
use const_random::const_random;
use std::fmt::Display;
use std::mem;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct BoardHash(u64);

#[rustfmt::skip]
const EXAMPLE: [u8; 30] = const_random!([u8 ; 30]);

#[rustfmt::skip]
const PIECE_HASHES: [u64; 12 * 64] = unsafe { mem::transmute(const_random!([u8 ; 6144])) };
const BLACK_TO_MOVE_HASH: u64 = const_random!(u64);
#[rustfmt::skip]
const CASTLING_RIGHTS: [u64; 16] = unsafe { mem::transmute(const_random!([u8 ; 128])) };
#[rustfmt::skip]
const EN_PASSANT_FILE: [u64; 8] = unsafe { mem::transmute(const_random!([u8 ; 64])) };

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

impl Display for BoardHash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{:016x}", self.0)
    }
}

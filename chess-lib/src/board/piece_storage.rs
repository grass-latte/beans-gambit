use crate::board::{Piece, bitboard::Bitboard, square::Square};

#[derive(Clone, Debug)]
pub struct PieceStorage {
    piece_bitboards: [Bitboard; 12],
    square_contents: [Option<Piece>; 64],
}

impl PieceStorage {
    pub const fn new() -> Self {
        Self {
            square_contents: [None; 64],
            piece_bitboards: [Bitboard::empty(); 12],
        }
    }

    pub const fn get(&self, sq: Square) -> Option<Piece> {
        self.square_contents[sq.as_u8() as usize]
    }

    pub const fn set(&mut self, sq: Square, contents: Option<Piece>) {
        // update old piece bitboard
        if let Some(piece) = self.get(sq) {
            self.piece_bitboards[piece.as_u8() as usize].unset(sq);
        }

        // update new piece bitboard
        if let Some(piece) = contents {
            self.piece_bitboards[piece.as_u8() as usize].set(sq);
        }

        // update square contents
        self.square_contents[sq.as_u8() as usize] = contents;
    }
}

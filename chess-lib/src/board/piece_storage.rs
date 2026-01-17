use crate::board::hash::BoardHash;
use crate::board::{Color, Piece, PieceKind, bitboard::Bitboard, square::Square};
use strum::IntoEnumIterator;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PieceStorage {
    piece_bitboards: [Bitboard; 12],
    square_contents: [Option<Piece>; 64],
}

impl Default for PieceStorage {
    fn default() -> Self {
        Self::new()
    }
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

    pub const fn set(
        &mut self,
        mut hash: BoardHash,
        sq: Square,
        contents: Option<Piece>,
    ) -> BoardHash {
        // update old piece bitboard
        if let Some(piece) = self.get(sq) {
            hash = hash.toggle_piece(piece, sq);
            self.piece_bitboards[piece.as_u8() as usize].remove(sq);
        }

        // update new piece bitboard
        if let Some(piece) = contents {
            hash = hash.toggle_piece(piece, sq);
            self.piece_bitboards[piece.as_u8() as usize].insert(sq);
        }

        // update square contents
        self.square_contents[sq.as_u8() as usize] = contents;

        hash
    }

    pub fn iter(&self) -> impl Iterator<Item = (Square, Piece)> {
        Piece::iter().flat_map(|p| {
            self.piece_bitboards[p.as_u8() as usize]
                .iter()
                .map(move |s| (s, p))
        })
    }

    pub fn iter_single_color(&self, color: Color) -> impl Iterator<Item = (Square, Piece)> {
        PieceKind::iter().flat_map(move |kind| {
            let piece = Piece::new(kind, color);
            self.piece_bitboards[piece.as_u8() as usize]
                .iter()
                .map(move |s| (s, piece))
        })
    }

    pub fn piece_bitboard(&self, piece: Piece) -> Bitboard {
        self.piece_bitboards[piece.as_u8() as usize]
    }
}

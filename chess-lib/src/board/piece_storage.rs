use crate::board::{bitboard::Bitboard, color::Color, piece::Piece, square::Square};

#[derive(Clone, Debug)]
pub struct PieceStorage {
    piece_bitboards: [Bitboard; 12],
    square_contents: [Option<(Piece, Color)>; 64],
}

impl PieceStorage {
    pub fn new() -> Self {
        Self {
            square_contents: [None; 64],
            piece_bitboards: [Bitboard::EMPTY; 12],
        }
    }

    pub fn get(&self, sq: Square) -> Option<(Piece, Color)> {
        self.square_contents[sq.as_index()]
    }

    pub fn set(&mut self, sq: Square, piece: Option<(Piece, Color)>) {
        // update square contents
        self.square_contents[sq.as_index()] = piece;

        // update old piece bitboard
        if let Some((old_piece, old_color)) = self.get(sq) {
            self.piece_bitboards[get_piece_color_index(old_piece, old_color)].unset(sq);
        }

        // update new piece bitboard
        if let Some((new_piece, new_color)) = piece {
            self.piece_bitboards[get_piece_color_index(new_piece, new_color)].set(sq);
        }
    }
}

/// Given a piece-color combination, return a unique index for this combination in 0..12
fn get_piece_color_index(piece: Piece, color: Color) -> usize {
    (piece as usize) + (color as usize) * 6
}

pub mod bitboard;
pub mod color;
pub mod moves;
pub mod piece;
pub mod square;

use bitboard::Bitboard;
use moves::Move;

use self::{color::Color, piece::Piece, square::Square};

#[derive(Clone, Debug, thiserror::Error)]
#[error("invalid move")]
pub struct InvalidMove;

#[derive(Clone, Debug)]
pub struct Board {
    /// Occupancy bitboards for each piece type (0..6 for white, 6..12 for black)
    piece_bitboards: [Bitboard; 12],
    /// Mailbox representation of chess board
    square_contents: [Option<(Piece, Color)>; 64],
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    pub fn new() -> Board {
        Self {
            square_contents: [None; 64],
            piece_bitboards: [Bitboard::EMPTY; 12],
        }
    }

    pub fn get_piece_at(&self, sq: Square) -> Option<(Piece, Color)> {
        self.square_contents[sq.as_index()]
    }

    pub fn set_piece_at(&mut self, sq: Square) -> Option<(Piece, Color)> {
        let old = self.get_piece_at(sq);
        todo!()
    }

    pub fn make_move(&mut self, mv: Move) -> Result<(), InvalidMove> {
        todo!()
    }

    pub fn unmake_last_move(&mut self) {
        todo!();
    }
}

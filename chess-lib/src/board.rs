pub mod bitboard;
pub mod color;
pub mod moves;
pub mod piece;
pub mod square;

use bitboard::Bitboard;
use moves::Move;

use self::{color::Color, piece::Piece, square::Square};

#[derive(Clone, Debug)]
pub struct Board {
    /// Occupancy bitboards for each piece type (0..6 for white, 6..12 for black)
    piece_bitboards: [Bitboard; 12],
    /// Mailbox representation of the chess board
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

    pub fn set_piece_at(&mut self, sq: Square, piece: Option<(Piece, Color)>) {
        // update mailbox

        self.square_contents[sq.as_index()] = piece;

        // update old piece bitboard
        let old = self.get_piece_at(sq);
        if let Some((old_piece, old_color)) = old {
            self.piece_bitboards[get_piece_color_index(old_piece, old_color)].unset(sq);
        }

        // update new piece bitboard
        if let Some((new_piece, new_color)) = piece {
            self.piece_bitboards[get_piece_color_index(new_piece, new_color)].set(sq);
        }
    }

    pub fn make_move(&mut self, mv: Move) -> Result<(), InvalidMove> {
        todo!()
    }

    pub fn unmake_last_move(&mut self) {
        todo!();
    }

    pub fn iter_pieces<'board>(&'board self) -> PieceIterator<'board> {
        PieceIterator::new(self, None)
    }

    pub fn iter_pieces_for_color<'board>(&'board self, color: Color) -> PieceIterator<'board> {
        PieceIterator::new(self, Some(color))
    }
}

#[derive(Clone, Debug, thiserror::Error)]
#[error("invalid move")]
pub struct InvalidMove;

pub struct PieceIterator<'board> {
    board: &'board Board,
    filter_color: Option<Color>,
    sq_index: usize,
}

impl<'board> PieceIterator<'board> {
    fn new(board: &'board Board, filter_color: Option<Color>) -> Self {
        Self {
            board,
            filter_color,
            sq_index: 0,
        }
    }
}

impl<'board> Iterator for PieceIterator<'board> {
    type Item = (Piece, Color, Square);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.sq_index += 1;
            if self.sq_index >= 64 {
                return None;
            }

            if let Some((piece, color)) = self.board.square_contents[self.sq_index]
                && Some(color) == self.filter_color
            {
                return Some((piece, color, Square::from_index(self.sq_index)));
            }
        }
    }
}

/// Given a piece-color combination, return a unique index for this combination in 0..12
fn get_piece_color_index(piece: Piece, color: Color) -> usize {
    (piece as usize) + (color as usize) * 6
}

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
    piece_bitboards: [Bitboard; 12],
    square_contents: [Option<(Piece, Color)>; 64],
    color_to_move: Color,
    en_passant_destination: Option<Square>,
    white_castling_rights: CastlingRights,
    black_castling_rights: CastlingRights,
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
            color_to_move: Color::White,
            en_passant_destination: None,
            white_castling_rights: Default::default(),
            black_castling_rights: Default::default(),
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

    pub fn make_null_move(&mut self) {
        todo!();
    }

    pub fn unmake_last_move(&mut self) {
        todo!();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CastlingRights {
    kingside: bool,
    queenside: bool,
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self {
            kingside: true,
            queenside: true,
        }
    }
}

#[derive(Clone, Debug, thiserror::Error)]
#[error("invalid move")]
pub struct InvalidMove;

/// Given a piece-color combination, return a unique index for this combination in 0..12
fn get_piece_color_index(piece: Piece, color: Color) -> usize {
    (piece as usize) + (color as usize) * 6
}

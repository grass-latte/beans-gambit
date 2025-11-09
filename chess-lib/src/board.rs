pub mod bitboard;
pub mod color;
pub mod moves;
pub mod piece;
pub mod piece_storage;
pub mod square;

use bitboard::Bitboard;
use moves::Move;

use self::{color::Color, piece::Piece, piece_storage::PieceStorage, square::Square};

#[derive(Clone, Debug)]
pub struct Board {
    pieces: PieceStorage,
    color_to_move: Color,
    en_passant_destination: Option<Square>,
    white_castling_rights: CastlingRights,
    black_castling_rights: CastlingRights,
    unmake_stack: Vec<UnmakeInfo>,
}

impl Board {
    pub fn new() -> Board {
        Self {
            pieces: PieceStorage::new(),
            color_to_move: Color::White,
            en_passant_destination: None,
            white_castling_rights: Default::default(),
            black_castling_rights: Default::default(),
            unmake_stack: Vec::new(),
        }
    }

    pub fn make_move(&mut self, mv: Move) -> Result<(), InvalidMove> {
        todo!();
    }

    pub fn make_null_move(&mut self) {
        todo!();
    }

    pub fn unmake_last_move(&mut self) {
        todo!();
    }

    pub fn pieces(&self) -> &PieceStorage {
        &self.pieces
    }

    pub fn pieces_mut(&mut self) -> &mut PieceStorage {
        &mut self.pieces
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

/// Information necessary to unmake the last move
#[derive(Clone, Copy, Debug)]
struct UnmakeInfo {}

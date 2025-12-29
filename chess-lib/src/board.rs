mod bitboard;
mod color;
mod mv;
mod piece;
mod piece_storage;
mod square;

pub use bitboard::*;
pub use color::*;
use derive_getters::Getters;
pub use mv::*;
pub use piece::*;
pub use piece_storage::*;
pub use square::*;

#[derive(Clone, Debug, Getters)]
pub struct Board {
    pieces: PieceStorage,
    color_to_move: Color,
    en_passant_destination: Option<Square>,
    white_castling_rights: CastlingRights,
    black_castling_rights: CastlingRights,
    #[getter(skip)]
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

    pub fn make_move(&mut self, mv: Move) {
        todo!();
    }

    pub fn make_null_move(&mut self) {
        self.color_to_move = !self.color_to_move;
        self.unmake_stack.push(UnmakeInfo::NullMove);
    }

    pub fn unmake_last_move(&mut self) {
        todo!();
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
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

/// Information necessary to unmake the last move
#[derive(Clone, Copy, Debug)]
enum UnmakeInfo {
    Move {
        /// original piece kind, in case of promotion
        piece: PieceKind,
        source: Square,
        destination: Square,
        captured: Option<PieceKind>,
        old_castling_rights: CastlingRights,
        old_en_passant_destination: Option<Square>,
    },
    NullMove,
}

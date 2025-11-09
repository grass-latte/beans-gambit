use crate::{color::Color, piece::Piece, square::Square};

#[derive(Clone, Debug)]
pub struct Board {
    white_pieces: Vec<(Piece, Square)>,
    black_pieces: Vec<(Piece, Square)>,
    square_contents: [Option<(Piece, Color)>; 64],
}

impl Board {
    pub fn new() -> Board {
        Self {
            white_pieces: vec![],
            black_pieces: vec![],
            square_contents: [None; 64],
        }
    }

    pub fn get_piece_at(sq: Square) -> Option<(Piece, Color)> {
        todo!()
    }

    pub fn set_piece_at(sq: Square) -> Option<(Piece, Color)> {
        todo!()
    }

    pub fn unmake_last_move() {
        todo!();
    }
}

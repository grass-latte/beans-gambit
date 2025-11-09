use crate::piece::Piece;
use crate::square::Square;

#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub source: Square,
    pub destination: Square,
    pub promotion: Option<Piece>,
}

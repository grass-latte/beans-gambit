use crate::piece::Piece;
use crate::square::Square;

pub struct Move {
    source: Square,
    destination: Square,
    promotion: Option<Piece>,
}

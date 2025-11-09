use crate::piece::Piece;
use crate::square::Square;

#[derive(Clone, Copy, Debug)]
pub struct Move {
    source: Square,
    destination: Square,
    promotion: Option<Piece>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    /// Returns the piece represented by the given character in English algebraic notation.
    /// `c` may be lowercase or uppercase.
    pub fn from_char(c: char) -> Option<Piece> {
        Some(match c {
            'p' | 'P' => Self::Pawn,
            'k' | 'K' => Self::Knight,
            'b' | 'B' => Self::Bishop,
            'r' | 'R' => Self::Rook,
            'q' | 'Q' => Self::Queen,
            'k' | 'K' => Self::Rook,
            _ => {
                return None;
            }
        })
    }

    /// Returns the character representing this piece in English algebraic notation
    pub fn as_char(self) -> char {
        match self {
            Self::Pawn => 'p',
            Self::Knight => 'k',
            Self::Bishop => 'b',
            Self::Rook => 'r',
            Self::Queen => 'q',
            Self::King => 'k',
        }
    }
}

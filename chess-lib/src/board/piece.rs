use std::num::NonZero;

use crate::board::Color;

/// Combination of piece kind and color.
// The contained u8 is NonZero so that Option<Piece> is one byte. This requires that the type is
// repr(transparent).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece(NonZero<u8>);

impl Piece {
    /// Number of different pieces - 6 white, 6 black.
    pub const COUNT: usize = 12;

    pub const fn new(kind: PieceKind, color: Color) -> Self {
        let kind_index = kind.as_u8();
        let color_index = color.is_white() as u8;

        // SAFETY: Index is less than 12.
        unsafe { Self::from_u8_unchecked(kind_index + color_index * 6) }
    }

    pub const unsafe fn from_u8(v: u8) -> Option<Self> {
        if v < 12 {
            // SAFETY: Index is less than 12.
            Some(unsafe { Self::from_u8_unchecked(v) })
        } else {
            None
        }
    }

    /// SAFETY: Index must be less than 12 (Piece::COUNT).
    pub const unsafe fn from_u8_unchecked(v: u8) -> Self {
        // SAFETY: We shift the index and add one so that 0 is not a valid value.
        Self(unsafe { NonZero::new_unchecked((v << 1) | 1) })
    }

    /// Returns a u8 in 0..12 unique to this piece.
    pub const fn as_u8(self) -> u8 {
        self.0.get() >> 1
    }

    pub const fn kind(self) -> PieceKind {
        let piece_index = (self.0.get() >> 1) % 6;
        // SAFETY: piece_index < 6.
        unsafe { PieceKind::from_u8_unchecked(piece_index) }
    }

    pub const fn color(self) -> Color {
        Color::from_is_white((self.0.get() >> 1) > 6)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceKind {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl PieceKind {
    pub const COUNT: usize = 6;

    pub const fn from_u8(index: u8) -> Option<Self> {
        if index < Self::COUNT as u8 {
            // SAFETY: `v` is a valid value for `Self`.
            Some(unsafe { Self::from_u8_unchecked(index) })
        } else {
            None
        }
    }

    pub const unsafe fn from_u8_unchecked(v: u8) -> Self {
        // SAFETY: Self is repr(u8).
        unsafe { std::mem::transmute(v) }
    }

    pub const fn as_u8(self) -> u8 {
        // SAFETY: Self is repr(u8).
        unsafe { std::mem::transmute(self) }
    }

    /// Returns the piece represented by the given character in English algebraic notation.
    /// `c` may be lowercase or uppercase.
    pub const fn from_char(c: char) -> Option<PieceKind> {
        Some(match c {
            'p' | 'P' => Self::Pawn,
            'n' | 'N' => Self::Knight,
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
    pub const fn as_char(self) -> char {
        match self {
            Self::Pawn => 'p',
            Self::Knight => 'n',
            Self::Bishop => 'b',
            Self::Rook => 'r',
            Self::Queen => 'q',
            Self::King => 'k',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece() {
        assert_eq!(
            Piece::new(PieceKind::Bishop, Color::White).kind(),
            PieceKind::Bishop
        );
        assert_eq!(
            Piece::new(PieceKind::Bishop, Color::White).color(),
            Color::White
        );
        assert_eq!(
            Piece::new(PieceKind::Knight, Color::Black).kind(),
            PieceKind::Knight
        );
        assert_eq!(
            Piece::new(PieceKind::Knight, Color::Black).color(),
            Color::Black
        );
    }
}

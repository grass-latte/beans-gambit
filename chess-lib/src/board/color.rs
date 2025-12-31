use std::ops::Not;

use crate::board::BoardRank;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub const fn from_is_white(is_white: bool) -> Self {
        if is_white {
            Self::White
        } else {
            Self::Black
        }
    }

    pub const fn is_white(self) -> bool {
        match self {
            Self::White => true,
            Self::Black => false,
        }
    }

    pub const fn as_char(self) -> char {
        match self {
            Self::White => 'w',
            Self::Black => 'b',
        }
    }

    /// Returns the rank on which pieces of this colour start.
    pub const fn back_rank(self) -> BoardRank {
        match self {
            Self::White => BoardRank::R1,
            Self::Black => BoardRank::R8,
        }
    }

    /// Returns the rank on which pawns of this color are promoted.
    pub const fn promotion_rank(self) -> BoardRank {
        match self {
            Self::White => BoardRank::R8,
            Self::Black => BoardRank::R1,
        }
    }

    /// Returns the rank on which pawns of this start.
    pub const fn pawn_starting_rank(self) -> BoardRank {
        match self {
            Self::White => BoardRank::R2,
            Self::Black => BoardRank::R7,
        }
    }

    pub const fn up(self) -> i32 {
        match self {
            Self::White => 1,
            Self::Black => -1,
        }
    }
}

impl Not for Color {
    type Output = Color;

    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

use std::ops::Not;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub const fn from_is_white(is_white: bool) -> Self {
        if is_white { Self::White } else { Self::Black }
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

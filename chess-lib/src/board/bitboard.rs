use derive_more::{BitAnd, BitOr, BitXor};

use super::square::Square;

#[derive(Clone, Copy, Debug, Eq, PartialEq, BitAnd, BitOr, BitXor)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Self = Bitboard(0);

    /// Returns the bitboard with only `sq` set
    pub fn single(sq: Square) -> Self {
        Self(1 << sq.as_index() as u64)
    }

    /// Sets the bit corresponding to `sq`
    pub fn set(&mut self, sq: Square) {
        *self = self.with_set(sq)
    }

    /// Unsets the bit corresponding to `sq`
    pub fn unset(&mut self, sq: Square) {
        *self = self.with_unset(sq)
    }

    /// Returns a copy of this bitboard with `sq` set
    pub fn with_set(self, sq: Square) -> Self {
        Self(self.0 | Self::single(sq).0)
    }

    /// Returns a copy of this bitboard with `sq` not set
    pub fn with_unset(self, sq: Square) -> Self {
        Self(self.0 & !Self::single(sq).0)
    }

    /// Returns an iterator over the squares set in this bitboard
    pub fn iter(self) -> BitboardIterator {
        BitboardIterator(self.0)
    }
}

pub struct BitboardIterator(u64);

impl Iterator for BitboardIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        let trailing_zeros = self.0.trailing_zeros();

        if trailing_zeros < 64 {
            self.0 &= !(1 << trailing_zeros as u64);
            Some(Square::from_index(trailing_zeros as usize))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_bitboard_iterator() {
        let squares = [Square::A1, Square::B2, Square::H1, Square::H8];

        let mut bitboard = Bitboard::EMPTY;
        for square in squares {
            bitboard.set(square);
        }

        let squares_set: HashSet<Square> = squares.into_iter().collect();
        let recovered_squares_set: HashSet<Square> = bitboard.iter().collect();

        assert_eq!(squares_set, recovered_squares_set);
    }
}

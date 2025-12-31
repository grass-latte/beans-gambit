use std::fmt::{Debug, Write};

use crate::board::{BoardFile, BoardRank};
use derive_more::{BitAnd, BitOr, BitXor};
use strum::IntoEnumIterator;

use super::square::Square;

#[derive(Clone, Copy, Eq, PartialEq, BitAnd, BitOr, BitXor)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Returns the bitboard with only `sq` set
    pub const fn single(sq: Square) -> Self {
        Self(1 << sq.as_u8() as u64)
    }

    /// Mostly used for writing tests.
    pub fn from_ranks(ranks: [u8; 8]) -> Self {
        let mut result = 0;

        for (rank_index, rank) in ranks.into_iter().enumerate() {
            result |= (rank as u64) << ((rank_index as u64) * 8);
        }

        Self(result)
    }

    /// True if the bit corresponding to `sq` is set.
    pub const fn contains(self, sq: Square) -> bool {
        self.0 & (1 << sq.as_u8() as u64) != 0
    }

    /// Sets the bit corresponding to `sq` to true.
    pub const fn insert(&mut self, sq: Square) {
        *self = self.with_inserted(sq)
    }

    /// Sets the bit corresponding to `sq` to false.
    pub const fn remove(&mut self, sq: Square) {
        *self = self.with_removed(sq)
    }

    pub const fn insert_if(&mut self, sq: Square, condition: bool) {
        self.0 = self.0 | (Self::single(sq).0 * (condition as u64));
    }

    /// Returns a copy of this bitboard with `sq` set
    pub const fn with_inserted(self, sq: Square) -> Self {
        Self(self.0 | Self::single(sq).0)
    }

    /// Returns a copy of this bitboard with `sq` not set
    pub const fn with_removed(self, sq: Square) -> Self {
        Self(self.0 & !Self::single(sq).0)
    }

    /// Returns an iterator over the squares set in this bitboard
    pub const fn iter(self) -> BitboardIterator {
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

            debug_assert!(trailing_zeros < 64);
            // SAFETY: `trailing_zeros` is less than 64.
            Some(unsafe { Square::from_u8_unchecked(trailing_zeros as u8) })
        } else {
            None
        }
    }
}

impl Debug for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in BoardRank::iter().rev() {
            for file in BoardFile::iter() {
                if self.contains(Square::at(file, rank)) {
                    f.write_char('#')?;
                } else {
                    f.write_char('.')?;
                }
            }
            if rank != BoardRank::R1 {
                f.write_char('\n')?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_bitboard_iterator() {
        let squares = [Square::A1, Square::B2, Square::H1, Square::H8];

        let mut bitboard = Bitboard::empty();
        for square in squares {
            bitboard.insert(square);
        }

        let squares_set: HashSet<Square> = squares.into_iter().collect();
        let recovered_squares_set: HashSet<Square> = bitboard.iter().collect();

        assert_eq!(squares_set, recovered_squares_set);
    }

    #[test]
    fn test_bitboard_debug() {
        let bitboard = Bitboard::empty()
            .with_inserted(Square::A1)
            .with_inserted(Square::B1)
            .with_inserted(Square::H8);

        let expected = "\
            .......#\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ........\n\
            ##......";

        let formatted = format!("{:?}", bitboard);

        assert_eq!(formatted, expected.to_string());
    }
}

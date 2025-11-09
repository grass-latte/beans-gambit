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
        Self(self.0 & Self::single(sq).0)
    }

    /// Returns a copy of this bitboard with `sq` not set
    pub fn with_unset(self, sq: Square) -> Self {
        Self(self.0 & !Self::single(sq).0)
    }
}

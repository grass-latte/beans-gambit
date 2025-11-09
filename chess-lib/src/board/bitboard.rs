use derive_more::{BitAnd, BitOr, BitXor};

#[derive(Clone, Copy, Debug, Eq, PartialEq, BitAnd, BitOr, BitXor)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Self = Bitboard(0);
}

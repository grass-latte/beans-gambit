/*
 * Terminology:
 *  - Attack set: bitboard of squares attacked by a piece on a given square.
 *  - Pseudolegal: move thats are possible if king safety is ignored.
 */

mod precomputed_bitboards;
mod sliding;

use crate::{
    board::{Bitboard, Move, Square},
    movegen::sliding::SlidingAttackTable,
};

/// The maximum number of legal moves in a reachable chess position.
pub const MAXIMUM_LEGAL_MOVES: usize = 218;

/// Responsible for calculating the legal moves on a `Board`.
#[derive(Clone, Debug)]
pub struct MoveGenerator {
    legal_moves: Vec<Move>,
    bishop_attack_table: SlidingAttackTable,
    rook_attack_table: SlidingAttackTable,
}

impl MoveGenerator {
    pub fn new() -> Self {
        Self {
            legal_moves: Vec::with_capacity(MAXIMUM_LEGAL_MOVES),
            bishop_attack_table: SlidingAttackTable::compute_for_bishop(),
            rook_attack_table: SlidingAttackTable::compute_for_rook(),
        }
    }
}

impl Default for MoveGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the bitboard of squares attacked by a white pawn on the given square.
pub fn white_pawn_attacks(sq: Square) -> Bitboard {
    Bitboard(precomputed_bitboards::WHITE_PAWN_ATTACK_SETS[sq.as_u8() as usize])
}

/// Returns the bitboard of squares attacked by a black pawn on the given square.
pub fn black_pawn_attacks(sq: Square) -> Bitboard {
    Bitboard(precomputed_bitboards::BLACK_PAWN_ATTACK_SETS[sq.as_u8() as usize])
}

/// Returns the bitboard of squares attacked by a knight on the given square.
pub fn knight_attacks(sq: Square) -> Bitboard {
    Bitboard(precomputed_bitboards::KNIGHT_ATTACK_SETS[sq.as_u8() as usize])
}

/// Returns the bitboard of squares attacked by a king on the given square.
pub fn king_attacks(sq: Square) -> Bitboard {
    Bitboard(precomputed_bitboards::KING_ATTACK_SETS[sq.as_u8() as usize])
}

/// Returns the bitboard of squares attacked by an unobstructed rook on the given square.
pub fn unobstructed_rook_attacks(sq: Square) -> Bitboard {
    Bitboard(precomputed_bitboards::UNOBSTRUCTED_ROOK_ATTACK_SETS[sq.as_u8() as usize])
}

/// Returns the bitboard of squares attacked by an unobstructed bishop on the given square.
pub fn unobstructed_bishop_attacks(sq: Square) -> Bitboard {
    Bitboard(precomputed_bitboards::UNOBSTRUCTED_BISHOP_ATTACK_SETS[sq.as_u8() as usize])
}

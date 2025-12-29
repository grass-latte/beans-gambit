/*
 * Terminology:
 *  - Attack set: bitboard of squares attacked by a piece on a given square.
 *  - Pseudolegal: moves that are possible if king safety is ignored.
 */

mod precomputed_bitboards;
mod sliding;

use smallvec::SmallVec;

use crate::{
    board::{Bitboard, Board, Color, Move, Piece, Square},
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

/// Information important for move generation that doesn't change during movegen (but does change
/// each turn).
struct MoveGenContext {
    side_to_move: Color,
    friendly_pieces: SmallVec<[(Square, Piece); 16]>,
    enemy_pieces: SmallVec<[(Square, Piece); 16]>,
    friendly_pieces_bitboard: Bitboard,
    enemy_pieces_bitboard: Bitboard,
    king_square: Square,
    is_check: bool,
    /// Bitboard of squares on which the king would be placed in check.
    king_danger_mask: Bitboard,
    /// List of enemy pieces that are currently checking the king.
    checking_pieces: SmallVec<[(Square, Piece); 16]>,
    /// Bitboard of squares containing pieces pinned to the king.
    pin_mask: Bitboard,
}

impl MoveGenContext {
    fn compute(board: &Board) -> Self {
        todo!()
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

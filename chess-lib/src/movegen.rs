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

/// Returns the bitboard of squares attacked by an unobstructed queen on the given square.
pub fn unobstructed_queen_attacks(sq: Square) -> Bitboard {
    unobstructed_rook_attacks(sq) | unobstructed_bishop_attacks(sq)
}

impl MoveGenerator {
    pub fn new() -> Self {
        Self {
            legal_moves: Vec::with_capacity(MAXIMUM_LEGAL_MOVES),
            bishop_attack_table: SlidingAttackTable::compute_for_bishop(),
            rook_attack_table: SlidingAttackTable::compute_for_rook(),
        }
    }

    /// Returns the bitboard of squares attacked by the piece on the given square,
    /// if king safety is ignored.
    pub fn get_pseudolegal_attacks_bitboard(
        &self,
        piece: Piece,
        sq: Square,
        all_pieces_bitboard: Bitboard,
    ) -> Bitboard {
        match piece {
            Piece::WhitePawn => white_pawn_attacks(sq),
            Piece::BlackPawn => black_pawn_attacks(sq),
            Piece::WhiteKnight | Piece::BlackKnight => knight_attacks(sq),
            Piece::WhiteKing | Piece::BlackKing => king_attacks(sq),
            Piece::WhiteBishop | Piece::BlackBishop => self
                .bishop_attack_table
                .get_attack_set(sq, all_pieces_bitboard),
            Piece::WhiteRook | Piece::BlackRook => self
                .rook_attack_table
                .get_attack_set(sq, all_pieces_bitboard),
            Piece::WhiteQueen | Piece::BlackQueen => {
                let rook_attacks = self
                    .rook_attack_table
                    .get_attack_set(sq, all_pieces_bitboard);
                let bishop_attacks = self
                    .bishop_attack_table
                    .get_attack_set(sq, all_pieces_bitboard);
                rook_attacks | bishop_attacks
            }
        }
    }

    fn compute_movegen_context(&self, board: &Board) -> MoveGenContext {
        let friendly_pieces = board
            .pieces()
            .iter_single_color(board.color_to_move())
            .collect::<PieceVec>();

        let enemy_pieces = board
            .pieces()
            .iter_single_color(!board.color_to_move())
            .collect::<PieceVec>();

        let friendly_pieces_bitboard = friendly_pieces
            .iter()
            .copied()
            .fold(Bitboard::empty(), |bitboard, (sq, _)| {
                bitboard | Bitboard::single(sq)
            });

        let enemy_pieces_bitboard = enemy_pieces
            .iter()
            .copied()
            .fold(Bitboard::empty(), |bitboard, (sq, _)| {
                bitboard | Bitboard::single(sq)
            });

        let king_square = friendly_pieces
            .iter()
            .copied()
            .find(|&(_, piece)| piece == Piece::WhiteKing || piece == Piece::BlackKing)
            .map(|(sq, _)| sq)
            .expect("There must be a king.");

        let (king_danger_mask, checking_pieces) = self.get_king_danger_mask_and_checking_pieces(
            &enemy_pieces,
            friendly_pieces_bitboard | enemy_pieces_bitboard,
            king_square,
        );

        MoveGenContext {
            color_to_move: board.color_to_move(),
            friendly_pieces,
            enemy_pieces,
            friendly_pieces_bitboard,
            enemy_pieces_bitboard,
            king_square,
            king_danger_mask,
            checking_pieces,
            pin_mask: todo!(),
        }
    }

    /// Returns:
    ///  - The bitboard of squares on which the king would be placed in check.
    ///    This is the set of squares attacked by enemy pieces, with the king excluded as
    ///    a blocker.
    ///  - The bitboard of squares containing pieces that are checking the king.
    fn get_king_danger_mask_and_checking_pieces(
        &self,
        enemy_pieces: &[(Square, Piece)],
        all_pieces_bitboard: Bitboard,
        king_square: Square,
    ) -> (Bitboard, Bitboard) {
        let all_pieces_except_king_bitboard = all_pieces_bitboard.with_removed(king_square);
        let mut king_danger_mask = Bitboard::empty();
        let mut checking_pieces = Bitboard::empty();

        for &(sq, enemy_piece) in enemy_pieces {
            let attack_set = self.get_pseudolegal_attacks_bitboard(
                enemy_piece,
                sq,
                all_pieces_except_king_bitboard,
            );
            king_danger_mask = king_danger_mask | attack_set;
            checking_pieces.insert_if(sq, attack_set.contains(king_square));
        }

        todo!()
    }
}

impl Default for MoveGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Efficiently holds pieces - as each side has 16 pieces, this should never heap-allocate.
type PieceVec = SmallVec<[(Square, Piece); 16]>;

/// Information important for move generation that doesn't change during movegen (but does change
/// each turn).
struct MoveGenContext {
    color_to_move: Color,
    friendly_pieces: PieceVec,
    enemy_pieces: PieceVec,
    friendly_pieces_bitboard: Bitboard,
    enemy_pieces_bitboard: Bitboard,
    king_square: Square,
    /// Bitboard of squares on which the king would be placed in check.
    king_danger_mask: Bitboard,
    /// Bitboard of enemy pieces that are currently checking the king.
    checking_pieces: Bitboard,
    /// Bitboard of squares containing pieces pinned to the king.
    pin_mask: Bitboard,
}

impl MoveGenContext {}

/*
 * Terminology:
 *  - Attack set: bitboard of squares attacked by a piece on a given square.
 *  - Pseudolegal: moves that are possible if king safety is ignored.
 */

mod precomputed_bitboards;
mod sliding;

use smallvec::SmallVec;

use crate::{
    board::{Bitboard, Board, Color, Move, Piece, PieceKind, Square},
    movegen::sliding::SlidingAttackTable,
};

/// The maximum number of legal moves in a reachable chess position.
pub const MAXIMUM_LEGAL_MOVES: usize = 218;

/// Responsible for calculating the legal moves on a `Board`.
#[derive(Clone, Debug)]
pub struct MoveGenerator {
    moves: Vec<Move>,
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

/// Return the bitboard of squares a pawn can be pushed to.
fn get_pawn_push_bitboard(color: Color, sq: Square, all_pieces_bitboard: Bitboard) -> Bitboard {
    let up = color.up();
    let mut result = Bitboard::empty();

    if let Some(push_sq) = sq.translated_by((0, up)) {
        result |= Bitboard::single(push_sq) & !all_pieces_bitboard;
    }

    // Double push from starting rank.
    // We check count_ones because if we can't push, we can't double push.
    if sq.rank() == color.pawn_starting_rank() && result.0.count_ones() == 1 {
        let thrust_sq = {
            let sq = sq.translated_by((0, 2 * up));
            debug_assert!(sq.is_some());
            // SAFETY: 2 up from starting rank is necessarily a valid rank (R4 or R5).
            unsafe { sq.unwrap_unchecked() }
        };

        result |= Bitboard::single(thrust_sq) & !all_pieces_bitboard;
    }

    result
}

impl MoveGenerator {
    pub fn new() -> Self {
        Self {
            moves: Vec::with_capacity(MAXIMUM_LEGAL_MOVES),
            bishop_attack_table: SlidingAttackTable::compute_for_bishop(),
            rook_attack_table: SlidingAttackTable::compute_for_rook(),
        }
    }

    /// Returns a reference to a move list stored within the move generator, to avoid allocating
    /// between turns.
    pub fn compute_legal_moves(&mut self, board: &Board) -> &[Move] {
        // TODO: Handle castling, en passant.

        self.moves.clear();
        let cx = self.compute_movegen_context(board);
        let all_pieces_bitboard = cx.friendly_pieces_bitboard | cx.enemy_pieces_bitboard;

        for (sq, piece) in board.pieces().iter_single_color(board.color_to_move()) {
            let piece_kind = piece.kind();

            // If we are in double check, filter only for king moves.
            if piece_kind == PieceKind::King
                && cx.checks_analysis.checking_pieces_mask.0.count_ones() > 1
            {
                continue;
            }

            // Bitboard of pseudolegal moves - will be refined to legal moves.
            let mut move_set = if piece_kind == PieceKind::Pawn {
                let attacks = match board.color_to_move() {
                    Color::White => white_pawn_attacks(sq),
                    Color::Black => black_pawn_attacks(sq),
                };
                let pushes = get_pawn_push_bitboard(board.color_to_move(), sq, all_pieces_bitboard);
                pushes | (attacks & cx.enemy_pieces_bitboard)
            } else {
                self.get_pseudolegal_attacks_bitboard(piece, sq, all_pieces_bitboard)
            };
            move_set &= !cx.friendly_pieces_bitboard;

            // Prevent the king from stepping into check.
            if piece_kind == PieceKind::King {
                move_set &= cx.checks_analysis.king_danger_mask;
            }

            // Handle pins.
            if cx.checks_analysis.pinned_pieces_mask.contains(sq) {
                let dx = sq.file().as_u8() as i32 - cx.king_square.file().as_u8() as i32;
                let dy = sq.rank().as_u8() as i32 - cx.king_square.rank().as_u8() as i32;
                let pin_ray_bitboard = ray_bitboard_empty(cx.king_square, (dx, dy));
                move_set &= pin_ray_bitboard;
            }

            // If we are in single check, filter only for:
            // - King moves
            // - Moves that capture the checking piece
            // - Interpositions (moves that block the check)
            if cx.checks_analysis.checking_pieces_mask != Bitboard::empty()
                && piece_kind != PieceKind::King
            {
                let mut valid_moves_mask = Bitboard::empty();

                let checking_piece_sq = cx
                    .checks_analysis
                    .checking_pieces_mask
                    .iter()
                    .next()
                    .expect("There should be one checking piece.");
                let checking_piece = board
                    .pieces()
                    .get(checking_piece_sq)
                    .expect("There should be a piece on this square.");

                // Capturing the checking piece.
                valid_moves_mask.insert(checking_piece_sq);

                // Interpositions.
                let kind = checking_piece.kind();
                if kind == PieceKind::Bishop || kind == PieceKind::Rook || kind == PieceKind::Queen
                {
                    let dx = checking_piece_sq.file().as_u8() as i32
                        - cx.king_square.file().as_u8() as i32;
                    let dy = checking_piece_sq.rank().as_u8() as i32
                        - cx.king_square.rank().as_u8() as i32;
                    let ray_bitboard = ray_bitboard_empty(cx.king_square, (dx, dy));
                    valid_moves_mask |= ray_bitboard;
                }

                move_set &= valid_moves_mask;
            }

            // Add the moves in the move set to the move list.
            for destination in move_set.iter() {
                // Handle promotions.
                if piece_kind == PieceKind::Pawn
                    && sq.rank() == board.color_to_move().promotion_rank()
                {
                    self.moves.extend(
                        [
                            PieceKind::Queen,
                            PieceKind::Rook,
                            PieceKind::Knight,
                            PieceKind::Bishop,
                        ]
                        .into_iter()
                        .map(|promotion| Move {
                            source: sq,
                            destination,
                            promotion: Some(promotion),
                        }),
                    );
                } else {
                    self.moves.push(Move {
                        source: sq,
                        destination,
                        promotion: None,
                    });
                }
            }
        }

        &self.moves
    }

    /// Returns the bitboard of squares attacked by the piece on the given square,
    /// if king safety is ignored.
    fn get_pseudolegal_attacks_bitboard(
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

        let checks_analysis = self.analyse_checks(
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
            checks_analysis,
        }
    }

    /// Calculates:
    ///  - The bitboard of squares on which the king would be placed in check.
    ///    This is the set of squares attacked by enemy pieces, with the king excluded as
    ///    a blocker.
    ///  - The bitboard of squares containing pieces that are checking the king.
    ///  - The bitboard of squares containing pieces that are pinned to the king.
    fn analyse_checks(
        &self,
        enemy_pieces: &[(Square, Piece)],
        all_pieces_bitboard: Bitboard,
        king_square: Square,
    ) -> ChecksAnalysis {
        let all_pieces_except_king_bitboard = all_pieces_bitboard.with_removed(king_square);
        let mut king_danger_mask = Bitboard::empty();
        let mut checking_pieces_mask = Bitboard::empty();

        // Approach for detecting pinned piece locations:
        // A piece is pinned if it is attacked by an enemy rook/bishop and both the
        // pinned piece and the pinning piece would be attacked by our king if it were
        // an enemy rook/bishop.
        let mut orthogonal_pin_rays = Bitboard::empty();
        let mut diagonal_pin_rays = Bitboard::empty();

        let mut update_diagonal_pin_rays =
            |enemy_attack_set: Bitboard, enemy_square: Square, king_square: Square| {
                if enemy_attack_set.contains(king_square)
                    && unobstructed_bishop_attacks(king_square).contains(enemy_square)
                {
                    diagonal_pin_rays |= enemy_attack_set;
                }
            };

        let mut update_orthogonal_pin_rays =
            |enemy_attacks: Bitboard, enemy_square: Square, king_square: Square| {
                if enemy_attacks.contains(king_square)
                    && unobstructed_bishop_attacks(king_square).contains(enemy_square)
                {
                    orthogonal_pin_rays |= enemy_attacks;
                }
            };

        for &(enemy_square, enemy_piece) in enemy_pieces {
            let attack_set = match enemy_piece {
                Piece::WhiteRook | Piece::BlackRook => {
                    let attacks = self
                        .rook_attack_table
                        .get_attack_set(enemy_square, all_pieces_except_king_bitboard);

                    update_orthogonal_pin_rays(attacks, enemy_square, king_square);
                    attacks
                }
                Piece::WhiteBishop | Piece::BlackBishop => {
                    let attacks = self
                        .bishop_attack_table
                        .get_attack_set(enemy_square, all_pieces_except_king_bitboard);

                    update_diagonal_pin_rays(attacks, enemy_square, king_square);
                    attacks
                }
                Piece::WhiteQueen | Piece::BlackQueen => {
                    let orthogonal_attacks = self
                        .rook_attack_table
                        .get_attack_set(enemy_square, all_pieces_except_king_bitboard);
                    let diagonal_attacks = self
                        .bishop_attack_table
                        .get_attack_set(enemy_square, all_pieces_except_king_bitboard);

                    update_diagonal_pin_rays(diagonal_attacks, enemy_square, king_square);
                    update_orthogonal_pin_rays(orthogonal_attacks, enemy_square, king_square);

                    orthogonal_attacks | diagonal_attacks
                }
                _ => self.get_pseudolegal_attacks_bitboard(
                    enemy_piece,
                    enemy_square,
                    all_pieces_except_king_bitboard,
                ),
            };

            king_danger_mask |= attack_set;
            checking_pieces_mask.insert_if(enemy_square, attack_set.contains(king_square));
        }

        let orthogonal_pin_mask = self
            .rook_attack_table
            .get_attack_set(king_square, all_pieces_bitboard)
            & orthogonal_pin_rays;
        let diagonal_pin_mask = self
            .bishop_attack_table
            .get_attack_set(king_square, all_pieces_bitboard)
            & diagonal_pin_rays;

        ChecksAnalysis {
            king_danger_mask,
            checking_pieces_mask,
            pinned_pieces_mask: orthogonal_pin_mask | diagonal_pin_mask,
        }
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
#[derive(Debug)]
struct MoveGenContext {
    color_to_move: Color,
    friendly_pieces: PieceVec,
    enemy_pieces: PieceVec,
    friendly_pieces_bitboard: Bitboard,
    enemy_pieces_bitboard: Bitboard,
    king_square: Square,
    checks_analysis: ChecksAnalysis,
}

/// Information about pieces attacking/pinned to the king.
#[derive(Clone, Copy, Debug)]
struct ChecksAnalysis {
    /// Bitboard of squares on which the king would be placed in check if it moved there.
    king_danger_mask: Bitboard,
    /// Bitboard of enemy pieces that are currently checking the king.
    checking_pieces_mask: Bitboard,
    /// Bitboard of friendly pieces that are pinned to the king.
    pinned_pieces_mask: Bitboard,
}

fn ray_bitboard(origin: Square, occupancy_bitboard: Bitboard, offset: (i32, i32)) -> Bitboard {
    // TODO: This can be optimized by calculating the number of squares to the closest edge of the board.
    // - This way we can loop a fixed number of times knowing the translated square is valid.
    let mut current_sq = origin;
    let mut result = Bitboard::empty();
    loop {
        let Some(new_sq) = current_sq.translated_by(offset) else {
            break;
        };

        current_sq = new_sq;
        result.insert(current_sq);
        if occupancy_bitboard.contains(current_sq) {
            break;
        }
    }
    result
}
fn ray_bitboard_empty(origin: Square, offset: (i32, i32)) -> Bitboard {
    // TODO: Same optimization as ray_bitboard
    let mut current_sq = origin;
    let mut result = Bitboard::empty();
    loop {
        let Some(new_sq) = current_sq.translated_by(offset) else {
            break;
        };
        current_sq = new_sq;
        result.insert(current_sq);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    #[test]
    fn stupid_test() {
        let board = Board::starting();
        let mut movegen = MoveGenerator::new();
        let moves = movegen.compute_legal_moves(&board);

        assert_eq!(moves.len(), 20);
    }
}

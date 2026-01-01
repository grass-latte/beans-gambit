/*
 * Terminology:
 *  - Attack set: bitboard of squares attacked by a piece on a given square.
 *  - Pseudolegal: moves that are possible if king safety is ignored.
 */

mod precomputed_bitboards;
mod sliding;

use smallvec::SmallVec;

use crate::{
    board::{Bitboard, Board, BoardFile, Color, Move, Piece, PieceKind, Square},
    movegen::sliding::SlidingAttackTable,
};

/// The maximum number of legal moves in a reachable chess position.
pub const MAXIMUM_LEGAL_MOVES: usize = 218;

pub type MoveList = SmallVec<[Move; MAXIMUM_LEGAL_MOVES]>;

/// Responsible for calculating the legal moves on a `Board`.
#[derive(Clone, Debug)]
pub struct MoveGenerator {
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
            bishop_attack_table: SlidingAttackTable::compute_for_bishop(),
            rook_attack_table: SlidingAttackTable::compute_for_rook(),
        }
    }

    pub fn compute_legal_moves(&self, moves: &mut MoveList, board: &Board) {
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

        let all_pieces_bitboard = friendly_pieces_bitboard | enemy_pieces_bitboard;

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
        let is_check = checks_analysis.checking_pieces_mask != Bitboard::empty();

        for (sq, piece) in board.pieces().iter_single_color(board.color_to_move()) {
            let piece_kind = piece.kind();

            // If we are in double check, filter only for king moves.
            if piece_kind != PieceKind::King
                && checks_analysis.checking_pieces_mask.0.count_ones() > 1
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

                // Enemy pieces and en passant target.
                let valid_capture_destinations = enemy_pieces_bitboard
                    | board
                        .en_passant_destination()
                        .filter(|&en_passant_destination| {
                            !self.detect_en_passant_pin(
                                board,
                                all_pieces_bitboard,
                                sq,
                                king_square,
                                en_passant_destination,
                            )
                        })
                        .map(Bitboard::single)
                        .unwrap_or(Bitboard::empty());

                pushes | (attacks & valid_capture_destinations)
            } else {
                self.get_pseudolegal_attacks_bitboard(piece, sq, all_pieces_bitboard)
            };
            move_set &= !friendly_pieces_bitboard;

            // Prevent the king from stepping into check.
            if piece_kind == PieceKind::King {
                move_set &= !checks_analysis.king_danger_mask;
            }

            // Handle pins.
            if checks_analysis.pinned_pieces_mask.contains(sq) {
                let dx = (sq.file().as_u8() as i32 - king_square.file().as_u8() as i32).signum();
                let dy = (sq.rank().as_u8() as i32 - king_square.rank().as_u8() as i32).signum();
                let pin_ray_bitboard = ray_bitboard_empty(king_square, (dx, dy));
                move_set = move_set & pin_ray_bitboard;
            }

            // If we are in single check, filter only for:
            // - King moves
            // - Moves that capture the checking piece
            // - Interpositions (moves that block the check)
            if is_check && piece_kind != PieceKind::King {
                let mut valid_moves_mask = Bitboard::empty();

                let checking_piece_sq = checks_analysis
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
                    let dx = (checking_piece_sq.file().as_u8() as i32
                        - king_square.file().as_u8() as i32)
                        .signum();
                    let dy = (checking_piece_sq.rank().as_u8() as i32
                        - king_square.rank().as_u8() as i32)
                        .signum();
                    let ray_bitboard = ray_bitboard(king_square, all_pieces_bitboard, (dx, dy));
                    valid_moves_mask |= ray_bitboard;
                }

                move_set &= valid_moves_mask;
            }

            // Add the moves in the move set to the move list.
            for destination in move_set.iter() {
                // Handle promotions.
                if piece_kind == PieceKind::Pawn
                    && destination.rank() == board.color_to_move().promotion_rank()
                {
                    moves.extend(
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
                    moves.push(Move {
                        source: sq,
                        destination,
                        promotion: None,
                    });
                }
            }
        }

        // Consider castling.
        let castling_rights = board.castling_rights_for_color(board.color_to_move());
        let back_rank = board.color_to_move().back_rank();

        // Squares that must be empty and safe in order to kingside castle.
        let kingside_castle_clear_mask = Bitboard::empty()
            .with_inserted(Square::at(BoardFile::F, back_rank))
            .with_inserted(Square::at(BoardFile::G, back_rank));
        // Squares that must be empty in order to queenside castle.
        let queenside_castle_clear_mask = Bitboard::empty()
            .with_inserted(Square::at(BoardFile::B, back_rank))
            .with_inserted(Square::at(BoardFile::C, back_rank))
            .with_inserted(Square::at(BoardFile::D, back_rank));
        // Squares that must be safe in order to queenside castle.
        let queenside_castle_danger_mask = Bitboard::empty()
            .with_inserted(Square::at(BoardFile::C, back_rank))
            .with_inserted(Square::at(BoardFile::D, back_rank));

        if castling_rights.kingside
            && !kingside_castle_clear_mask
                .intersects(all_pieces_bitboard | checks_analysis.king_danger_mask)
            && !is_check
            && board.pieces().get(Square::at(BoardFile::H, back_rank))
                == Some(Piece::new(PieceKind::Rook, board.color_to_move()))
        {
            moves.push(Move {
                source: Square::at(BoardFile::E, back_rank),
                destination: Square::at(BoardFile::G, back_rank),
                promotion: None,
            })
        }

        if castling_rights.queenside
            && !queenside_castle_clear_mask.intersects(all_pieces_bitboard)
            && !queenside_castle_danger_mask.intersects(checks_analysis.king_danger_mask)
            && !is_check
            && board.pieces().get(Square::at(BoardFile::A, back_rank))
                == Some(Piece::new(PieceKind::Rook, board.color_to_move()))
        {
            moves.push(Move {
                source: Square::at(BoardFile::E, back_rank),
                destination: Square::at(BoardFile::C, back_rank),
                promotion: None,
            })
        }
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
                if unobstructed_bishop_attacks(king_square).contains(enemy_square) {
                    diagonal_pin_rays |= enemy_attack_set;
                }
            };

        let mut update_orthogonal_pin_rays =
            |enemy_attacks: Bitboard, enemy_square: Square, king_square: Square| {
                if unobstructed_rook_attacks(king_square).contains(enemy_square) {
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

    /// Detect the annoying en passant pin, when the capturing pawn and the captured pawn are the
    /// only pieces blocking a horizontal check by a rook.
    fn detect_en_passant_pin(
        &self,
        board: &Board,
        all_pieces_bitboard: Bitboard,
        capturing_pawn_square: Square,
        king_square: Square,
        en_passant_destination: Square,
    ) -> bool {
        if king_square.rank() != capturing_pawn_square.rank() {
            return false;
        }

        // Signed dist from king to capturing pawn.
        let dx = capturing_pawn_square.as_u8() as i32 - king_square.file().as_u8() as i32;
        let sx = dx.signum();

        // March from king square to the end of the board or until we encounter an enemy rook or
        // another piece to block the check.
        let mut sq = king_square;
        let enemy_rook_bitboard = board
            .pieces()
            .piece_bitboard(Piece::new(PieceKind::Rook, !board.color_to_move()));
        let enemy_pawn_bitboard = board
            .pieces()
            .piece_bitboard(Piece::new(PieceKind::Pawn, !board.color_to_move()));

        while let Some(new_sq) = sq.translated_by((sx, 0)) {
            sq = new_sq;

            if all_pieces_bitboard.contains(sq) {
                if enemy_rook_bitboard.contains(sq) {
                    // Pinned by this rook.
                    return true;
                } else if (sq == capturing_pawn_square)
                    || (enemy_pawn_bitboard.contains(sq)
                        && sq.file() == en_passant_destination.file())
                {
                    // This is the capturing pawn or the captured pawn.
                    continue;
                } else {
                    // Found another piece blocking the check.
                    return false;
                }
            }
        }

        false
    }
}

impl Default for MoveGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Efficiently holds pieces - as each side has 16 pieces, this should never heap-allocate.
type PieceVec = SmallVec<[(Square, Piece); 16]>;

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
    use std::{collections::HashSet, ptr::hash};

    use itertools::Itertools;

    use super::*;
    use crate::board::Board;

    fn check_includes_moves(board_fen: &str, moves_uci: &[&str]) {
        let board = Board::from_fen(board_fen).unwrap();
        let mg = MoveGenerator::new();
        let mut moves = MoveList::new();
        mg.compute_legal_moves(&mut moves, &board);

        for mv in moves_uci {
            if !moves.contains(&Move::from_uci(mv).unwrap()) {
                panic!(
                    "missing expected move {}. got moves {:?}",
                    mv,
                    moves.iter().map(Move::as_uci).collect_vec()
                );
            }
        }
    }

    fn check_excludes_moves(board_fen: &str, moves_uci: &[&str]) {
        let board = Board::from_fen(board_fen).unwrap();
        let mg = MoveGenerator::new();
        let mut moves = MoveList::new();
        mg.compute_legal_moves(&mut moves, &board);

        for mv in moves_uci {
            if moves.contains(&Move::from_uci(mv).unwrap()) {
                panic!(
                    "unexpected move {}. got moves {:?}",
                    mv,
                    moves.iter().map(Move::as_uci).collect_vec()
                );
            }
        }
    }

    fn check_exact_moves(board_fen: &str, moves_uci: &[&str]) {
        let board = Board::from_fen(board_fen).unwrap();
        let mg = MoveGenerator::new();
        let mut moves = MoveList::new();
        mg.compute_legal_moves(&mut moves, &board);

        let expected = moves_uci
            .iter()
            .map(|s| s.to_string())
            .collect::<HashSet<_>>();
        let got = moves.iter().map(Move::as_uci).collect::<HashSet<_>>();

        let unexpected = got.difference(&expected).collect_vec();
        let missing = expected.difference(&got).collect_vec();

        if expected != got {
            panic!("incorrect moves - unexpected {unexpected:?}, missing {missing:?}");
        }
    }

    #[test]
    fn test_simple_capture() {
        check_includes_moves("8/8/8/8/3Kp2k/8/8/8 w - - 0 1", &["d4e4"]);
    }

    #[test]
    fn test_no_self_capture() {
        check_excludes_moves("8/8/8/8/3KP2k/8/8/8 w - - 0 1", &["d4e4"]);
    }

    #[test]
    fn test_simple_bishop_pin() {
        check_excludes_moves("K7/8/2P5/8/4b3/8/8/k7 w - - 0 1", &["c6c7"]);
    }

    #[test]
    fn test_simple_rook_pin() {
        check_excludes_moves("8/8/K1P4r/8/8/8/8/k7 w - - 0 1", &["c6c7"]);
    }

    #[test]
    fn test_en_passant() {
        check_includes_moves(
            "rnbqkbnr/ppp1ppp1/7p/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3",
            &["e5d6"],
        );
    }

    #[test]
    fn test_legal_castling() {
        // White castling.
        check_includes_moves("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", &["e1g1", "e1c1"]);

        // Black castling.
        check_includes_moves("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1", &["e8g8", "e8c8"]);
    }

    #[test]
    fn test_no_castling_through_check() {
        check_excludes_moves("3rkr2/8/8/8/8/8/8/R3K2R w KQ - 0 1", &["e1g1", "e1c1"]);
    }

    #[test]
    fn test_no_castling_without_rook() {
        // Can't castle if rooks are missing.
        check_excludes_moves("4k3/8/8/8/8/8/8/4K3 w KQ - 0 1", &["e1g1", "e1c1"]);
    }

    #[test]
    fn test_blocked_castling() {
        check_excludes_moves("r2bk1Br/8/8/8/8/8/8/R2BK1bR w KQ - 0 1", &["e1g1", "e1c1"]);
        check_excludes_moves("r2bk1Br/8/8/8/8/8/8/R2BK1bR b KQ - 0 1", &["e8g8", "e8c8"]);
    }

    #[test]
    fn test_en_passant_pin() {
        // Tricky situation where a pawn and the pawn it can capture e.p. are the only pieces
        // blocking a horizontal check.
        check_excludes_moves("8/8/8/K2pP2r/8/8/8/7k w - d6 0 1", &["e5d6"]);
    }

    #[test]
    fn test_interpositions() {
        // Check from bishop.
        check_exact_moves(
            "rnbqkbnr/ppp1pppp/8/1B1p4/4P3/8/PPPP1PPP/RNBQK1NR b KQkq - 1 2",
            &["c7c6", "d8d7", "c8d7", "b8c6", "b8d7"],
        );

        // Check from rook.
        check_exact_moves("3R4/k7/8/8/8/8/5PPP/r5K1 w - - 0 1", &["d8d1"]);

        // A false interposition that was being included.
        check_excludes_moves(
            "rnbqk1nr/pppp1ppp/8/4p3/Pb1P4/8/1PP1PPPP/RNBQKBNR w KQkq - 0 1",
            &["a4a5"],
        );
    }

    #[test]
    fn test_promotions() {
        check_includes_moves(
            "8/P7/8/8/8/8/8/K6k w - - 0 1",
            &["a7a8q", "a7a8r", "a7a8b", "a7a8n"],
        );
        check_excludes_moves("8/P7/8/8/8/8/8/K6k w - - 0 1", &["a7a8"]);
    }
}

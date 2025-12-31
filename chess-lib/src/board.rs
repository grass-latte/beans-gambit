mod bitboard;
mod color;
mod mv;
mod piece;
mod piece_storage;
mod square;

pub use bitboard::*;
pub use color::*;
pub use mv::*;
pub use piece::*;
pub use piece_storage::*;
pub use square::*;
use std::collections::HashSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Board {
    pieces: PieceStorage,
    color_to_move: Color,
    en_passant_destination: Option<Square>,
    white_castling_rights: CastlingRights,
    black_castling_rights: CastlingRights,
    halfmoves_since_event: usize, // Since last capture or pawn move
    fullmoves: usize,
}

impl Board {
    pub fn new(
        pieces: &[Option<Piece>; 64],
        color_to_move: Color,
        en_passant_destination: Option<Square>,
        white_castling_rights: CastlingRights,
        black_castling_rights: CastlingRights,
        halfmoves_since_event: usize,
        fullmoves: usize,
    ) -> Board {
        let mut piece_storage = PieceStorage::new();
        for (i, piece) in pieces.iter().enumerate() {
            if let Some(piece) = piece {
                debug_assert!(i < 64);
                // SAFETY: pieces length is 64 so the u8 is correct
                unsafe {
                    piece_storage.set(Square::from_u8_unchecked(i as u8), Some(*piece));
                }
            }
        }

        Self {
            pieces: piece_storage,
            color_to_move,
            en_passant_destination,
            white_castling_rights,
            black_castling_rights,
            halfmoves_since_event,
            fullmoves,
        }
    }

    pub fn empty() -> Board {
        Board::new(
            &[None; 64],
            Color::White,
            None,
            Default::default(),
            Default::default(),
            0,
            0,
        )
    }

    pub fn starting() -> Board {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    pub fn from_fen(fen: &str) -> Result<Board, String> {
        let invalid_fen_err = |err: String| Err(format!("Invalid fen: {} - {}", fen, err));

        let sections = fen.split(" ").collect::<Vec<&str>>();
        if sections.len() != 6 {
            return invalid_fen_err("must have 6 fields".to_string());
        }

        // * Board
        let mut pieces = [None; 64];
        let ranks = sections[0].split("/").collect::<Vec<&str>>();
        if ranks.len() != 8 {
            return invalid_fen_err("must have 8 ranks".to_string());
        }
        for (r, rank) in ranks.into_iter().enumerate() {
            let mut f: usize = 0;
            for c in rank.chars() {
                if c.is_ascii_digit() {
                    f += c.to_string().parse::<usize>().unwrap();
                    continue;
                }

                if f >= 8 {
                    return invalid_fen_err(format!("file {} out of range", f));
                }

                let Some(piece) = Piece::from_char(c) else {
                    return invalid_fen_err(format!("unrecognised piece '{c}'"));
                };

                pieces[(7 - r) * 8 + f] = Some(piece);

                f += 1;
            }
        }

        // * To move
        let to_move = match sections[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return invalid_fen_err("invalid fen".to_string()),
        };

        // * Castling availability
        let castling_chars: HashSet<char> = if sections[2] == "-" {
            HashSet::new()
        } else {
            HashSet::from_iter(sections[2].chars())
        };

        for c in castling_chars.iter() {
            match c {
                'K' | 'Q' | 'k' | 'q' => {}
                _ => return invalid_fen_err(format!("unrecognised castling character '{c}'")),
            }
        }

        let white_castling_rights = CastlingRights {
            kingside: castling_chars.contains(&'K'),
            queenside: castling_chars.contains(&'Q'),
        };
        let black_castling_rights = CastlingRights {
            kingside: castling_chars.contains(&'k'),
            queenside: castling_chars.contains(&'q'),
        };

        // * En Passant
        let en_passant = if sections[3] == "-" {
            None
        } else {
            if sections[3].len() != 2 {
                return invalid_fen_err("en passant must be '-' or 2 character".to_string());
            }

            let Some(square) = Square::from_name(sections[3]) else {
                return invalid_fen_err("invalid en passant square".to_string());
            };

            Some(square)
        };

        // * Halfmoves since last capture or pawn move
        let Ok(halfmoves_since_event): Result<usize, _> = sections[4].parse() else {
            return invalid_fen_err("halfmoves not a number".to_string());
        };

        // * Fullmoves
        let Ok(fullmoves): Result<usize, _> = sections[5].parse() else {
            return invalid_fen_err("fullmoves not a number".to_string());
        };

        Ok(Board::new(
            &pieces,
            to_move,
            en_passant,
            white_castling_rights,
            black_castling_rights,
            halfmoves_since_event,
            fullmoves,
        ))
    }

    pub fn to_fen(&self) -> String {
        let mut output = String::new();

        let mut cur_x = 0;

        for y in (0u8..8).rev() {
            for x in 0u8..8 {
                debug_assert!(x < 8);
                debug_assert!(y < 8);
                // SAFETY: Range limited
                unsafe {
                    if let Some(piece) = self.pieces.get(Square::at_xy_unchecked(x, y)) {
                        if cur_x != x {
                            output += &format!("{}", x - cur_x);
                            cur_x = x;
                        }

                        output.push(piece.as_char());

                        cur_x += 1;
                    }
                }
            }

            if cur_x != 8 {
                output += &format!("{}", 8 - cur_x);
            }
            cur_x = 0;

            if y != 0 {
                output.push('/');
            }
        }

        output += &format!(" {} ", self.color_to_move.as_char());

        let mut castling_string = String::new();
        if self.white_castling_rights.kingside {
            castling_string.push('K');
        }
        if self.white_castling_rights.queenside {
            castling_string.push('Q');
        }
        if self.black_castling_rights.kingside {
            castling_string.push('k');
        }
        if self.black_castling_rights.queenside {
            castling_string.push('q');
        }
        if castling_string.is_empty() {
            castling_string.push('-');
        }

        output += &castling_string;

        if let Some(square) = self.en_passant_destination {
            output += &format!(" {}", square.name())
        } else {
            output += " -";
        }

        output += &format!(" {} {}", self.halfmoves_since_event, self.fullmoves);

        output
    }

    fn castling_rights_of(&mut self, color: Color) -> &mut CastlingRights {
        if color == Color::White {
            &mut self.white_castling_rights
        } else {
            &mut self.black_castling_rights
        }
    }

    pub fn make_move(&mut self, mv: Move) -> UnmakeInfo {
        let source_piece = self.pieces.get(mv.source).unwrap();
        let destination_piece = self.pieces.get(mv.destination).map(|p| p.kind());
        let dx = mv.destination.file().as_u8() as i32 - mv.source.file().as_u8() as i32;
        let dy = mv.destination.rank().as_u8() as i32 - mv.source.rank().as_u8() as i32;

        let mut unmake_info = UnmakeInfo {
            piece: source_piece.kind(),
            source: mv.source,
            destination: mv.destination,
            captured: destination_piece, // Changed for en passant
            old_en_passant_destination: self.en_passant_destination,
            old_castling_rights: *self.castling_rights_of(self.color_to_move),
            old_halfmoves_since_event: self.halfmoves_since_event,
        };

        self.en_passant_destination = None;

        match (
            source_piece.kind(),
            destination_piece,
            (dx, dy),
            mv.promotion,
        ) {
            (PieceKind::Pawn, None, (dx, _), _) if dx != 0 => {
                // En passant
                self.pieces.set(mv.source, None);
                self.pieces.set(mv.destination, Some(source_piece));
                self.pieces
                    .set(Square::at(mv.destination.file(), mv.source.rank()), None);
                unmake_info.captured = Some(PieceKind::Pawn);
            }
            (_, _, (_, _), Some(piece)) => {
                // Promotion
                self.pieces.set(mv.source, None);
                self.pieces.set(
                    mv.destination,
                    Some(Piece::new(piece, source_piece.color())),
                );
            }
            (PieceKind::King, _, (-2, _), _) => {
                // Long castle
                *self.castling_rights_of(source_piece.color()) = CastlingRights::none();
                self.pieces.set(mv.source, None);
                self.pieces.set(mv.destination, Some(source_piece));
                self.pieces
                    .set(Square::at(BoardFile::A, mv.source.rank()), None);
                self.pieces.set(
                    Square::at(BoardFile::D, mv.source.rank()),
                    Some(Piece::new(PieceKind::Rook, source_piece.color())),
                );
            }
            (PieceKind::King, _, (2, _), _) => {
                // Short
                *self.castling_rights_of(source_piece.color()) = CastlingRights::none();
                self.castling_rights_of(source_piece.color()).queenside = false;
                self.castling_rights_of(source_piece.color()).kingside = false;
                self.pieces.set(mv.source, None);
                self.pieces.set(mv.destination, Some(source_piece));
                self.pieces
                    .set(Square::at(BoardFile::H, mv.source.rank()), None);
                self.pieces.set(
                    Square::at(BoardFile::F, mv.source.rank()),
                    Some(Piece::new(PieceKind::Rook, source_piece.color())),
                );
            }
            (PieceKind::Pawn, _, (_, 2 | -2), _) => {
                debug_assert!(((mv.destination.rank().as_u8() as i32 - (dy / 2)) as u8) < 8);
                // SAFETY: if dy.abs() == 2, then there is a rank between the source and destination rank.
                //     Subtracting dy / 2 from the destination rank gets that rank.
                unsafe {
                    self.en_passant_destination = Some(Square::at(
                        mv.destination.file(),
                        BoardRank::from_u8_unchecked(
                            (mv.destination.rank().as_u8() as i32 - (dy / 2)) as u8,
                        ),
                    ));
                }

                self.pieces.set(mv.source, None);
                self.pieces.set(mv.destination, Some(source_piece));
            }
            _ => {
                self.pieces.set(mv.source, None);
                self.pieces.set(mv.destination, Some(source_piece));
            }
        };

        if unmake_info.captured.is_some() {
            self.halfmoves_since_event = 0;
        }

        self.color_to_move = !self.color_to_move;
        if self.color_to_move == Color::White {
            self.fullmoves += 1;
        }

        unmake_info
    }

    pub fn unmake_last_move(&mut self, um: UnmakeInfo) {
        let dx = um.destination.file().as_u8() as i32 - um.source.file().as_u8() as i32;

        self.color_to_move = !self.color_to_move;
        self.halfmoves_since_event = um.old_halfmoves_since_event;
        if self.color_to_move == Color::Black {
            self.fullmoves -= 1;
        }
        self.en_passant_destination = um.old_en_passant_destination;
        *self.castling_rights_of(self.color_to_move) = um.old_castling_rights;

        // Don't need to handle promotion or leap
        match (um.piece, um.destination.rank(), dx) {
            (PieceKind::Pawn, _, _) if Some(um.destination) == um.old_en_passant_destination => {
                // En Passant
                self.pieces.set(um.destination, None);
                self.pieces
                    .set(um.source, Some(Piece::new(um.piece, self.color_to_move)));

                self.pieces.set(
                    Square::at(um.destination.file(), um.source.rank()),
                    Some(Piece::new(PieceKind::Pawn, !self.color_to_move)),
                );
            }
            (PieceKind::King, _, -2) => {
                // Long castle
                self.pieces.set(um.destination, None);
                self.pieces
                    .set(um.source, Some(Piece::new(um.piece, self.color_to_move)));

                self.pieces
                    .set(Square::at(BoardFile::D, um.source.rank()), None);
                self.pieces.set(
                    Square::at(BoardFile::A, um.source.rank()),
                    Some(Piece::new(PieceKind::Rook, self.color_to_move)),
                );
            }
            (PieceKind::King, _, 2) => {
                // Short castle
                self.pieces.set(um.destination, None);
                self.pieces
                    .set(um.source, Some(Piece::new(um.piece, self.color_to_move)));

                self.pieces
                    .set(Square::at(BoardFile::F, um.source.rank()), None);
                self.pieces.set(
                    Square::at(BoardFile::H, um.source.rank()),
                    Some(Piece::new(PieceKind::Rook, self.color_to_move)),
                );
            }
            _ => {
                self.pieces.set(
                    um.destination,
                    um.captured.map(|k| Piece::new(k, !self.color_to_move)),
                );
                self.pieces
                    .set(um.source, Some(Piece::new(um.piece, self.color_to_move)));
            }
        };
    }

    pub fn pieces(&self) -> &PieceStorage {
        &self.pieces
    }

    pub fn color_to_move(&self) -> Color {
        self.color_to_move
    }

    pub fn en_passant_destination(&self) -> Option<Square> {
        self.en_passant_destination
    }

    pub fn white_castling_rights(&self) -> CastlingRights {
        self.white_castling_rights
    }

    pub fn black_castling_rights(&self) -> CastlingRights {
        self.black_castling_rights
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CastlingRights {
    pub kingside: bool,
    pub queenside: bool,
}

impl CastlingRights {
    pub fn none() -> CastlingRights {
        CastlingRights {
            kingside: false,
            queenside: false,
        }
    }
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self {
            kingside: true,
            queenside: true,
        }
    }
}

/// Information necessary to unmake the last move
#[derive(Clone, Copy, Debug)]
pub struct UnmakeInfo {
    /// original piece kind, in case of promotion
    pub piece: PieceKind,
    pub source: Square,
    pub destination: Square,
    pub captured: Option<PieceKind>,
    pub old_en_passant_destination: Option<Square>,
    pub old_castling_rights: CastlingRights,
    pub old_halfmoves_since_event: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fen_forward_back() {
        const FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        assert_eq!(Board::from_fen(FEN).unwrap().to_fen(), FEN);
    }

    fn test_move_and_unmake(fen_before: &str, mv: Move, fen_after: &str) {
        let mut board = Board::from_fen(fen_before).unwrap();

        let unmake = board.make_move(mv);
        assert_eq!(board.to_fen(), fen_after, "Making the move");

        board.unmake_last_move(unmake);
        assert_eq!(board.to_fen(), fen_before, "Unmaking the move");
    }

    #[test]
    fn test_basic_unmake() {
        test_move_and_unmake(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            Move {
                source: Square::B2,
                destination: Square::B4,
                promotion: None,
            },
            "rnbqkbnr/pppppppp/8/8/1P6/8/P1PPPPPP/RNBQKBNR b KQkq b3 0 1",
        )
    }

    #[test]
    fn test_unmake_en_passant_white() {
        test_move_and_unmake(
            // Black just played d7–d5, ep square is d6
            "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3",
            Move {
                source: Square::E5,
                destination: Square::D6,
                promotion: None,
            },
            "rnbqkbnr/ppp1pppp/3P4/8/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 3",
        )
    }

    #[test]
    fn test_unmake_en_passant_black() {
        test_move_and_unmake(
            // White just played e2–e4, ep square is e3
            "rnbqkbnr/pppppppp/8/8/3Pp3/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 2",
            Move {
                source: Square::E4,
                destination: Square::D3,
                promotion: None,
            },
            "rnbqkbnr/pppppppp/8/8/8/3p4/PPP1PPPP/RNBQKBNR w KQkq - 0 3",
        )
    }

    #[test]
    fn test_unmake_white_short_castle() {
        test_move_and_unmake(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1",
            Move {
                source: Square::E1,
                destination: Square::G1,
                promotion: None,
            },
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQ1RK1 b kq - 0 1",
        )
    }

    #[test]
    fn test_unmake_black_short_castle() {
        test_move_and_unmake(
            "rnbqk2r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1",
            Move {
                source: Square::E8,
                destination: Square::G8,
                promotion: None,
            },
            "rnbq1rk1/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 2",
        )
    }

    #[test]
    fn test_unmake_white_long_castle() {
        test_move_and_unmake(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R3KBNR w KQkq - 0 1",
            Move {
                source: Square::E1,
                destination: Square::C1,
                promotion: None,
            },
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/2KR1BNR b kq - 0 1",
        )
    }

    #[test]
    fn test_unmake_black_long_castle() {
        test_move_and_unmake(
            "r3kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1",
            Move {
                source: Square::E8,
                destination: Square::C8,
                promotion: None,
            },
            "2kr1bnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 2",
        )
    }

    #[test]
    fn test_unmake_white_promotion() {
        test_move_and_unmake(
            "8/P7/8/8/8/8/8/k6K w - - 0 1",
            Move {
                source: Square::A7,
                destination: Square::A8,
                promotion: Some(PieceKind::Queen),
            },
            "Q7/8/8/8/8/8/8/k6K b - - 0 1",
        )
    }

    #[test]
    fn test_unmake_black_promotion() {
        test_move_and_unmake(
            "k6K/8/8/8/8/8/p7/8 b - - 0 1",
            Move {
                source: Square::A2,
                destination: Square::A1,
                promotion: Some(PieceKind::Queen),
            },
            "k6K/8/8/8/8/8/8/q7 w - - 0 2",
        )
    }
}

mod bitboard;
mod color;
mod mv;
mod piece;
mod piece_storage;
mod square;

pub use bitboard::*;
pub use color::*;
use derive_getters::Getters;
pub use mv::*;
pub use piece::*;
pub use piece_storage::*;
pub use square::*;
use std::collections::HashSet;

#[derive(Clone, Debug, Getters, Eq, PartialEq)]
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

    pub fn make_move(&mut self, mv: Move) -> Result<(), String> {
        todo!();
    }

    pub fn make_null_move(&mut self) -> UnmakeInfo {
        self.color_to_move = !self.color_to_move;
        UnmakeInfo::NullMove
    }

    pub fn unmake_last_move(&mut self) {
        todo!();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CastlingRights {
    kingside: bool,
    queenside: bool,
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
pub enum UnmakeInfo {
    Move {
        /// original piece kind, in case of promotion
        piece: PieceKind,
        source: Square,
        destination: Square,
        captured: Option<PieceKind>,
        old_castling_rights: CastlingRights,
        old_en_passant_destination: Option<Square>,
    },
    NullMove,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fen_forward_back() {
        const FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        assert_eq!(Board::from_fen(FEN).unwrap().to_fen(), FEN);
    }
}

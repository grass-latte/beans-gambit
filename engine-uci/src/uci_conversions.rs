use chess_lib::board::{BoardFile, BoardRank, Move, PieceKind, Square};
use std::str::FromStr;
use vampirc_uci::{UciMove, UciPiece, UciSquare};

pub fn to_uci_piece(piece_kind: PieceKind) -> UciPiece {
    UciPiece::from_str(&piece_kind.as_char().to_string()).unwrap()
}

pub fn from_uci_piece(piece: UciPiece) -> PieceKind {
    PieceKind::from_char(piece.as_char().unwrap_or('p')).unwrap()
}

pub fn to_uci_square(square: Square) -> UciSquare {
    UciSquare::from(square.file().as_char(), square.rank().as_u8() + 1)
}

pub fn from_uci_square(square: UciSquare) -> Square {
    Square::at(
        BoardFile::from_char(square.file).unwrap(),
        BoardRank::from_u8(square.rank - 1).unwrap(),
    )
}

pub fn from_uci_move(mv: UciMove) -> Move {
    Move {
        source: from_uci_square(mv.from),
        destination: from_uci_square(mv.to),
        promotion: mv.promotion.map(from_uci_piece),
    }
}

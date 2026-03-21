use crate::constant_heuristics::heatmaps::{
    BISHOP_HEATMAP, KING_HEATMAP, KNIGHT_HEATMAP, PAWN_HEATMAP, QUEEN_HEATMAP, ROOK_HEATMAP,
};
use crate::constant_heuristics::pieces::{
    BISHOP_VALUE, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE,
};
use chess_lib::board::{Board, PieceKind, Square};

pub fn eval(board: &Board) -> f32 {
    let color_to_move = board.color_to_move();

    let mut score = 0f32;

    for (position, piece) in board.pieces().iter() {
        let position = if piece.color().is_white() {
            position.as_u8()
        } else {
            Square::at(position.file(), position.rank().flipped()).as_u8()
        };

        let mut s = match piece.kind() {
            PieceKind::Pawn => PAWN_VALUE + PAWN_HEATMAP[position as usize],
            PieceKind::Knight => KNIGHT_VALUE + KNIGHT_HEATMAP[position as usize],
            PieceKind::Bishop => BISHOP_VALUE + BISHOP_HEATMAP[position as usize],
            PieceKind::Rook => ROOK_VALUE + ROOK_HEATMAP[position as usize],
            PieceKind::Queen => QUEEN_VALUE + QUEEN_HEATMAP[position as usize],
            PieceKind::King => 0.0 + KING_HEATMAP[position as usize],
        };

        if piece.color() != color_to_move {
            s *= -1.0;
        }
        score += s;
    }

    score
}

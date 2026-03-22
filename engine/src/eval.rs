use crate::constant_heuristics::heatmaps::{
    BISHOP_HEATMAP, BISHOP_HEATMAP_SCALE, KING_HEATMAP, KING_HEATMAP_SCALE, KNIGHT_HEATMAP,
    KNIGHT_HEATMAP_SCALE, PAWN_HEATMAP, PAWN_HEATMAP_SCALE, QUEEN_HEATMAP, QUEEN_HEATMAP_SCALE,
    ROOK_HEATMAP, ROOK_HEATMAP_SCALE,
};
use crate::constant_heuristics::pieces::{
    BISHOP_VALUE, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE,
};
use crate::results::Score;
use chess_lib::board::{Board, PieceKind, Square};

pub fn eval(board: &Board) -> Score {
    let color_to_move = board.color_to_move();

    let mut score = 0f32;

    for (position, piece) in board.pieces().iter() {
        let position = if piece.color().is_white() {
            position.as_u8()
        } else {
            Square::at(position.file(), position.rank().flipped()).as_u8()
        };

        #[rustfmt::skip]
        let mut s = match piece.kind() {
            PieceKind::Pawn => PAWN_VALUE + PAWN_HEATMAP[position as usize] / PAWN_HEATMAP_SCALE,
            PieceKind::Knight => KNIGHT_VALUE + KNIGHT_HEATMAP[position as usize] / KNIGHT_HEATMAP_SCALE,
            PieceKind::Bishop => BISHOP_VALUE + BISHOP_HEATMAP[position as usize] / BISHOP_HEATMAP_SCALE,
            PieceKind::Rook => ROOK_VALUE + ROOK_HEATMAP[position as usize] / ROOK_HEATMAP_SCALE,
            PieceKind::Queen => QUEEN_VALUE + QUEEN_HEATMAP[position as usize] / QUEEN_HEATMAP_SCALE,
            PieceKind::King => 0.0 + KING_HEATMAP[position as usize] / KING_HEATMAP_SCALE,
        };

        if piece.color() != color_to_move {
            s *= -1.0;
        }
        score += s;
    }

    Score::Score(score)
}

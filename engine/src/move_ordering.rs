use crate::eval::{piece_value_at_position, simple_piece_value};
use chess_lib::board::{Board, Move};

pub fn order_moves(mvs: &mut [Move], board: &Board) {
    mvs.sort_by_cached_key(|mv| {
        let mut score = 0f32;

        // Most valuable victim, least valuable attacker
        // TODO: En-passant
        if let Some(source) = board.pieces().get(mv.source)
            && let Some(dest) = board.pieces().get(mv.destination)
            && dest.color() != source.color()
        {
            score +=
                piece_value_at_position(dest, mv.destination) - (simple_piece_value(source) / 2f32);
        }

        (score * 1000f32) as u32 // Janky
    })
}

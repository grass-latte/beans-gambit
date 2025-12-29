use crate::board::{Board, Move};

/// The maximum number of legal moves in a reachable chess position.
pub const MAXIMUM_LEGAL_MOVES: usize = 218;

/// Responsible for calculating the legal moves on a `Board`.
/// `MoveGenerators` are slow to create, so you should create and reuse one for the whole search
/// rather than creating one each time you generate moves.
#[derive(Clone, Debug)]
pub struct MoveGenerator {
    legal_moves: Vec<Move>,
}

impl MoveGenerator {
    pub fn new() -> Self {
        Self {
            legal_moves: Vec::with_capacity(MAXIMUM_LEGAL_MOVES),
        }
    }

    /// Calculate and return the list of legal moves in the chess position.
    /// Designed not to allocate in normal cases.
    pub fn get_legal_moves<'s>(&'s mut self, board: &Board) -> &'s [Move] {
        &self.legal_moves
    }
}

impl Default for MoveGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[rustfmt::skip]
mod rook_attack_sets;
#[rustfmt::skip]
mod bishop_attack_sets;

use super::{precomputed_bitboards, ray_bitboard};
use crate::board::{Bitboard, Square};
use crate::movegen::sliding::bishop_attack_sets::BISHOP_ATTACK_SETS;
use crate::movegen::sliding::rook_attack_sets::ROOK_ATTACK_SETS;

/// Implements the "magic bitboards" approach to sliding piece movegen.
#[derive(Clone, Debug)]
pub struct SlidingAttackTable {
    attack_sets: [&'static [Bitboard]; 64],
    magics: &'static [u64; 64],
    relevant_bits: &'static [u64; 64],
    relevant_occupancy_masks: &'static [u64; 64],
}

impl SlidingAttackTable {
    const fn rook() -> Self {
        Self {
            attack_sets: ROOK_ATTACK_SETS,
            magics: &precomputed_bitboards::ROOK_MAGIC_NUMBERS,
            relevant_bits: &precomputed_bitboards::ROOK_RELEVANT_BITS,
            relevant_occupancy_masks: &precomputed_bitboards::ROOK_RELEVANT_OCCUPANCY_MASKS,
        }
    }

    const fn bishop() -> Self {
        Self {
            attack_sets: BISHOP_ATTACK_SETS,
            magics: &precomputed_bitboards::BISHOP_MAGIC_NUMBERS,
            relevant_bits: &precomputed_bitboards::BISHOP_RELEVANT_BITS,
            relevant_occupancy_masks: &precomputed_bitboards::BISHOP_RELEVANT_OCCUPANCY_MASKS,
        }
    }

    // fn compute_for_rook() -> Self {
    //     Self::compute(
    //         generate_rook_attack_set,
    //         &precomputed_bitboards::ROOK_MAGIC_NUMBERS,
    //         &precomputed_bitboards::ROOK_RELEVANT_BITS,
    //         &precomputed_bitboards::ROOK_RELEVANT_OCCUPANCY_MASKS,
    //     )
    // }
    //
    // fn compute_for_bishop() -> Self {
    //     Self::compute(
    //         generate_bishop_attack_set,
    //         &precomputed_bitboards::BISHOP_MAGIC_NUMBERS,
    //         &precomputed_bitboards::BISHOP_RELEVANT_BITS,
    //         &precomputed_bitboards::BISHOP_RELEVANT_OCCUPANCY_MASKS,
    //     )
    // }

    pub fn get_attack_set(&self, sq: Square, all_pieces_bitboard: Bitboard) -> Bitboard {
        let sq_index = sq.as_u8() as usize;
        let relevant_occupancy_bitboard =
            all_pieces_bitboard.0 & self.relevant_occupancy_masks[sq_index];
        let key = (u64::wrapping_mul(relevant_occupancy_bitboard, self.magics[sq_index]))
            >> (64 - self.relevant_bits[sq_index]);
        self.attack_sets[sq_index][key as usize]
    }

    // fn compute(
    //     attack_set_generator: impl Fn(Square, Bitboard) -> Bitboard,
    //     magics: &'static [u64; 64],
    //     relevant_bits: &'static [u64; 64],
    //     relevant_occupancy_masks: &'static [u64; 64],
    // ) -> Self {
    //     let attack_sets = Square::iter()
    //         .enumerate()
    //         .map(|(sq_index, sq)| {
    //             let table_size = 1 << relevant_bits[sq_index];
    //             let mut attack_sets = vec![Bitboard::empty(); table_size];
    //
    //             for relevant_occupancy_bitboard in Self::iter_all_relevant_occupancy_bitboards(
    //                 Bitboard(relevant_occupancy_masks[sq_index]),
    //             ) {
    //                 let key = (u64::wrapping_mul(relevant_occupancy_bitboard.0, magics[sq_index]))
    //                     >> (64 - relevant_bits[sq_index]);
    //
    //                 attack_sets[key as usize] =
    //                     attack_set_generator(sq, relevant_occupancy_bitboard);
    //             }
    //             attack_sets
    //         })
    //         .collect_array::<64>()
    //         .expect("Should have exactly 64.");
    //
    //     Self {
    //         attack_sets,
    //         magics,
    //         relevant_bits,
    //         relevant_occupancy_masks,
    //     }
    // }

    // Given a bitboard, returns an iterator over the bitboards for all combinations of
    // pieces occupying the marked squares.
    fn iter_all_relevant_occupancy_bitboards(bitboard: Bitboard) -> impl Iterator<Item = Bitboard> {
        // Find the total number of combinations.
        let num_set = bitboard.0.count_ones();
        let num_combinations = 1 << num_set;

        // For each combination, project its bits onto those at the marked square indices
        (0..num_combinations).map(move |combination| {
            let mut combination_bitboard = Bitboard::empty();

            for (i, sq) in bitboard.iter().enumerate() {
                if (combination & (1 << i)) != 0 {
                    combination_bitboard.insert(sq);
                }
            }

            combination_bitboard
        })
    }
}

fn generate_rook_attack_set(origin: Square, occupancy_bitboard: Bitboard) -> Bitboard {
    const OFFSETS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    OFFSETS
        .into_iter()
        .map(|offset| ray_bitboard(origin, occupancy_bitboard, offset))
        .fold(Bitboard::empty(), std::ops::BitOr::bitor)
}

fn generate_bishop_attack_set(origin: Square, occupancy_bitboard: Bitboard) -> Bitboard {
    const OFFSETS: [(i32, i32); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
    OFFSETS
        .into_iter()
        .map(|offset| ray_bitboard(origin, occupancy_bitboard, offset))
        .fold(Bitboard::empty(), std::ops::BitOr::bitor)
}

pub const ROOK_ATTACK_TABLE: SlidingAttackTable = SlidingAttackTable::rook();
pub const BISHOP_ATTACK_TABLE: SlidingAttackTable = SlidingAttackTable::bishop();

#[cfg(test)]
mod tests {
    use crate::board::{BoardFile, BoardRank};

    use super::*;

    // #[test]
    // fn generate() {
    //     for (name, sat) in [
    //         ("rook_attack_set", SlidingAttackTable::compute_for_rook()),
    //         (
    //             "bishop_attack_set",
    //             SlidingAttackTable::compute_for_bishop(),
    //         ),
    //     ] {
    //         let name_caps = name.to_uppercase();
    //         let name_plural = name.to_string() + "s";
    //         let name_plural_caps = name_plural.to_uppercase();
    //         let mut s = String::new();
    //         s += "use crate::board::Bitboard;\n\n";
    //
    //         for (i, vb) in sat.attack_sets.iter().enumerate() {
    //             s += &format!(
    //                 "#[rustfmt::skip]\nconst {name_caps}_{i}: [u64; {}] = [\n",
    //                 vb.len()
    //             );
    //             for (j, b) in vb.iter().enumerate() {
    //                 if j % 8 == 0 {
    //                     s += "        ";
    //                 }
    //                 s += &format!("{}, ", b.0);
    //                 if j % 8 == 7 {
    //                     s += "\n";
    //                 }
    //             }
    //             s += "];\n";
    //             s += &format!("const {name_caps}_{i}_LEN: usize = {name_caps}_{i}.len();\n\n");
    //         }
    //
    //         s += "// SAFETY: Bitboard is internally represented as u64\n";
    //         s += &format!(
    //             "pub const {name_plural_caps}: [&[Bitboard]; {}] = [\n",
    //             sat.attack_sets.len()
    //         );
    //         for i in 0..sat.attack_sets.len() {
    //             s += &format!(
    //                 "    unsafe {{ &std::mem::transmute::<[u64; {name_caps}_{i}_LEN], [Bitboard; {name_caps}_{i}_LEN]>({name_caps}_{i}) }},\n"
    //             );
    //         }
    //         s += "];\n";
    //         fs::write(format!("src/movegen/sliding/{name_plural}.rs"), s).unwrap();
    //     }
    // }

    #[test]
    fn test_generate_rook_attacks() {
        #[rustfmt::skip]
        let occupancy_bitboard = Bitboard::from_ranks([
            0b00000000,
            0b00001000,
            0b00000000,
            0b00000000,
            0b10000010,
            0b00000000,
            0b00001000,
            0b00000000,
        ]);
        #[rustfmt::skip]
        let expected = Bitboard::from_ranks([
            0b00000000,
            0b00001000,
            0b00001000,
            0b00001000,
            0b11110110,
            0b00001000,
            0b00001000,
            0b00000000,
        ]);
        let origin = Square::D5;
        assert_eq!(
            generate_rook_attack_set(origin, occupancy_bitboard),
            expected
        );
    }

    #[test]
    fn test_generate_bishop_attacks() {
        #[rustfmt::skip]
        let occupancy_bitboard = Bitboard::from_ranks([
            0b00000000,
            0b00000000,
            0b00100000,
            0b00000100,
            0b00000000,
            0b00000000,
            0b00100000,
            0b00000001,
        ]);
        #[rustfmt::skip]
        let expected = Bitboard::from_ranks([
            0b00000000,
            0b00000000,
            0b00100000,
            0b00010100,
            0b00000000,
            0b00010100,
            0b00100010,
            0b00000001,
        ]);
        let origin = Square::D5;
        assert_eq!(
            generate_bishop_attack_set(origin, occupancy_bitboard),
            expected
        );
    }

    /// Try 1000 random squares and random occupancy bitboards.
    /// Check that the results from the sliding attack table match the results by naively
    /// generating attacks.
    fn test_attack_table(
        table: SlidingAttackTable,
        ground_truth: impl Fn(Square, Bitboard) -> Bitboard,
    ) {
        let rng = rand::rng();
        const TIMES: usize = 1000;

        for _ in 0..TIMES {
            // Random square to search from.
            let origin = {
                let file = BoardFile::from_u8(rand::random_range(0..8)).unwrap();
                let rank = BoardRank::from_u8(rand::random_range(0..8)).unwrap();
                Square::at(file, rank)
            };

            // Number of pieces on the test board.
            let num_pieces = rand::random_range(0..32);

            // Randomly place `num_pieces` pieces on the bitboard.
            let all_pieces_bitboard = {
                let mut result = Bitboard::empty();

                for _ in 0..num_pieces {
                    let file = BoardFile::from_u8(rand::random_range(0..8)).unwrap();
                    let rank = BoardRank::from_u8(rand::random_range(0..8)).unwrap();

                    result.insert(Square::at(file, rank));
                }

                result
            };

            assert_eq!(
                ground_truth(origin, all_pieces_bitboard),
                table.get_attack_set(origin, all_pieces_bitboard)
            );
        }
    }

    #[test]
    fn test_rook_attack_table() {
        test_attack_table(ROOK_ATTACK_TABLE, generate_rook_attack_set);
    }

    #[test]
    fn test_bishop_attack_table() {
        test_attack_table(BISHOP_ATTACK_TABLE, generate_bishop_attack_set);
    }
}

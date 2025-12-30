use crate::board::{Bitboard, Square};
use itertools::Itertools;
use strum::IntoEnumIterator;

/// Implements the "magic bitboards" approach to sliding piece movegen.
pub struct SlidingAttackTable {
    attack_sets: [Vec<Bitboard>; 64],
    magics: &'static [u64; 64],
    relevant_bits: &'static [u64; 64],
    relevant_occupancy_masks: &'static [u64; 64],
}

impl SlidingAttackTable {
    pub fn compute_for_rook() -> Self {
        Self::compute(
            generate_rook_attack_set,
            &precomputed::ROOK_MAGIC_NUMBERS,
            &precomputed::ROOK_RELEVANT_BITS,
            &precomputed::ROOK_RELEVANT_OCCUPANCY_MASKS,
        )
    }

    pub fn compute_for_bishop() -> Self {
        Self::compute(
            generate_bishop_attack_set,
            &precomputed::BISHOP_MAGIC_NUMBERS,
            &precomputed::BISHOP_RELEVANT_BITS,
            &precomputed::BISHOP_RELEVANT_OCCUPANCY_MASKS,
        )
    }

    pub fn get_attack_set(&self, sq: Square, all_pieces_bitboard: Bitboard) -> Bitboard {
        let sq_index = sq.as_u8() as usize;
        let relevant_occupancy_bitboard =
            all_pieces_bitboard.0 & self.relevant_occupancy_masks[sq_index];
        let key = (relevant_occupancy_bitboard - self.magics[sq_index])
            >> (64 - self.relevant_bits[sq_index]);
        self.attack_sets[sq_index][key as usize]
    }

    fn compute(
        attack_set_generator: impl Fn(Square, Bitboard) -> Bitboard,
        magics: &'static [u64; 64],
        relevant_bits: &'static [u64; 64],
        relevant_occupancy_masks: &'static [u64; 64],
    ) -> Self {
        let attack_sets = Square::iter()
            .enumerate()
            .map(|(sq_index, sq)| {
                let table_size = 1 << relevant_bits[sq_index];
                let mut attack_sets = vec![Bitboard::empty(); table_size];

                for relevant_occupancy_bitboard in Self::iter_all_relevant_occupancy_bitboards(
                    Bitboard(relevant_occupancy_masks[sq_index]),
                ) {
                    let key = (relevant_occupancy_bitboard.0 - magics[sq_index])
                        >> (64 - relevant_bits[sq_index]);

                    attack_sets[key as usize] =
                        attack_set_generator(sq, relevant_occupancy_bitboard);
                }
                attack_sets
            })
            .collect_array::<64>()
            .expect("Should have exactly 64.");

        Self {
            attack_sets,
            magics,
            relevant_bits,
            relevant_occupancy_masks,
        }
    }

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

fn ray_bitboard(origin: Square, occupancy_bitboard: Bitboard, offset: (i32, i32)) -> Bitboard {
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

#[rustfmt::skip]
mod precomputed {
    pub const ROOK_MAGIC_NUMBERS: [u64; 64] = [9259400972386469971, 378302682768609280, 432363709392882176, 792669819509149696, 72066390400761862, 3170696865660274184, 1297037800784303112, 4647724711070220416, 4621115431220971552, 9223935263835750912, 36451421809283073, 288371182435059713, 18155170357837952, 4630263409842586640, 577023710847092737, 45317473469333539, 4611827305877078112, 2904830830706688580, 1152992973133185794, 342560544483450920, 108228778483778560, 282574823891480, 2891324155045679234, 1126037350074400, 18155146737369096, 18049583955431424, 1153308569208094848, 72066392286826496, 1776900986372352, 9223409422399963264, 5764609739238410520, 9224515531128766592, 5875016085073297442, 1585302321930175552, 704374661713920, 5084146078588940, 11745466994378413313, 5512828164964881408, 9225711969732920610, 9232942392284283008, 72239288342839296, 306315145569697824, 576479445611774016, 6953593077768454216, 287006911430672, 180781701872517248, 180706952246067208, 142010875916, 337840929260044800, 9403691945961718400, 2328362177773175424, 140771849142400, 5630049374437760, 10450040056571232768, 4611967510617523456, 585471421900161088, 72077798662996233, 882706631853350929, 4900479379968131474, 1450198702706655489, 1234550484871155714, 36310289176725537, 8804951458436, 9224570659152134278];
    pub const ROOK_RELEVANT_BITS: [u64; 64] = [12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 12, 11, 10, 10, 10, 10, 10, 10, 12, 11, 10, 10, 10, 10, 10, 10, 12, 11, 10, 10, 10, 10, 10, 10, 12, 11, 10, 10, 10, 10, 10, 10, 12, 11, 10, 10, 10, 10, 10, 10, 12, 12, 11, 11, 11, 11, 11, 11, 12];
    pub const ROOK_RELEVANT_OCCUPANCY_MASKS: [u64; 64] = [282578800148862, 565157600297596, 1130315200595066, 2260630401190006, 4521260802379886, 9042521604759646, 18085043209519166, 36170086419038334, 282578800180736, 565157600328704, 1130315200625152, 2260630401218048, 4521260802403840, 9042521604775424, 18085043209518592, 36170086419037696, 282578808340736, 565157608292864, 1130315208328192, 2260630408398848, 4521260808540160, 9042521608822784, 18085043209388032, 36170086418907136, 282580897300736, 565159647117824, 1130317180306432, 2260632246683648, 4521262379438080, 9042522644946944, 18085043175964672, 36170086385483776, 283115671060736, 565681586307584, 1130822006735872, 2261102847592448, 4521664529305600, 9042787892731904, 18085034619584512, 36170077829103616, 420017753620736, 699298018886144, 1260057572672512, 2381576680245248, 4624614895390720, 9110691325681664, 18082844186263552, 36167887395782656, 35466950888980736, 34905104758997504, 34344362452452352, 33222877839362048, 30979908613181440, 26493970160820224, 17522093256097792, 35607136465616896, 9079539427579068672, 8935706818303361536, 8792156787827803136, 8505056726876686336, 7930856604974452736, 6782456361169985536, 4485655873561051136, 9115426935197958144];

    pub const BISHOP_MAGIC_NUMBERS: [u64; 64] = [1197960816612343840, 4617882883208794114, 653093423084995074, 4617386739290521616, 14989110974533862472, 572914420221952, 4908998412701533200, 2306124760412063760, 585476782154056016, 18017998161904128, 72392550023438368, 3458804182147399905, 9259420651123378193, 2306970077755607104, 435029482951155712, 648887928310865920, 1162529866810458368, 633971801333888, 650770163337013762, 423690235822100, 577587822632894465, 10376857041717168152, 4648841011882624096, 141841941139536, 38316060780400640, 13537191561528131, 288318389024809024, 9800396290612297738, 216317917724155908, 290764750756774016, 6923163202255728640, 1688987399946496, 1171222463469592576, 577596583250494016, 1165332809042758657, 157637533978067456, 2323859615337422976, 4503883095228672, 293583075781183497, 1209219799029448834, 4693331422714283008, 79184970449920, 13792415643078688, 1105014558976, 450364374104806400, 2314859073283040320, 9008307524117124, 4756366364089260544, 3458909666573025296, 4622140733466347521, 3479059935378539538, 18157336128520192, 2449958335278022658, 35872145809408, 2287265841741824, 2308097025236406560, 9264187854190821376, 577950457856, 288230453478983712, 7512461579728978944, 292734285302071809, 220711703620370826, 1152925937684251661, 666541566856396826];
    pub const BISHOP_RELEVANT_BITS: [u64; 64] = [6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 6];
    pub const BISHOP_RELEVANT_OCCUPANCY_MASKS: [u64; 64]= [18049651735527936, 70506452091904, 275415828992, 1075975168, 38021120, 8657588224, 2216338399232, 567382630219776, 9024825867763712, 18049651735527424, 70506452221952, 275449643008, 9733406720, 2216342585344, 567382630203392, 1134765260406784, 4512412933816832, 9024825867633664, 18049651768822272, 70515108615168, 2491752130560, 567383701868544, 1134765256220672, 2269530512441344, 2256206450263040, 4512412900526080, 9024834391117824, 18051867805491712, 637888545440768, 1135039602493440, 2269529440784384, 4539058881568768, 1128098963916800, 2256197927833600, 4514594912477184, 9592139778506752, 19184279556981248, 2339762086609920, 4538784537380864, 9077569074761728, 562958610993152, 1125917221986304, 2814792987328512, 5629586008178688, 11259172008099840, 22518341868716544, 9007336962655232, 18014673925310464, 2216338399232, 4432676798464, 11064376819712, 22137335185408, 44272556441600, 87995357200384, 35253226045952, 70506452091904, 567382630219776, 1134765260406784, 2832480465846272, 5667157807464448, 11333774449049600, 22526811443298304, 9024825867763712, 18049651735527936];
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Try 1000 random squares and random occupancy bitboards.
    /// Check that the results from the sliding attack table match the results by naively
    /// generateing attacks
    fn test_attack_table(
        table: SlidingAttackTable,
        ground_truth: impl Fn(Square, Bitboard) -> Bitboard,
    ) {
        let mut rng = rand::rng();
        const TIMES: usize = 1000;

        for _ in 0..TIMES {
            // Number of pieces on the test board.
            let num_pieces = rand::random_range(0..32);

            // Randomly place `num_pieces` pieces on the bitboard.
            let all_pieces_bitboard = {
                let mut result = Bitboard::empty();
            };
        }
    }

    /*
    #[test]
    fn test_rook_attack_table() {
        test_attack_table(
            SlidingAttackTable::compute_for_rook(),
            generate_rook_attack_set,
        );
    }

    #[test]
    fn test_bishop_attack_table() {
        test_attack_table(
            SlidingAttackTable::compute_for_bishop(),
            generate_bishop_attack_set,
        );
    }
    */
}

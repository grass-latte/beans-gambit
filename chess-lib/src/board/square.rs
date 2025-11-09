type SquareIndex = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Square(pub SquareIndex);

impl Square {
    pub fn at(file: BoardFile, rank: BoardRank) -> Self {
        Self(file.as_index() | (rank.as_index() << 3))
    }

    /// Returns the square with the given name in algebraic notation, or None
    /// if the name is invalid
    /// Remainder of the string is ignored
    pub fn from_name(name: &str) -> Option<Self> {
        let mut chars = name.chars();

        Some(Self::at(
            BoardFile::from_char(chars.next()?)?,
            BoardRank::from_char(chars.next()?)?,
        ))
    }

    pub fn file(&self) -> BoardFile {
        BoardFile(self.as_index() & 7)
    }

    pub fn rank(&self) -> BoardRank {
        BoardRank(self.as_index() >> 3)
    }

    pub fn as_index(&self) -> SquareIndex {
        self.0
    }

    /// Returns the name of this square in algebraic notation
    pub fn name(&self) -> String {
        [self.file().as_char(), self.rank().as_char()]
            .iter()
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardFile(pub SquareIndex);

impl BoardFile {
    pub fn from_char(c: char) -> Option<Self> {
        let index = c as i64 - 'a' as i64;
        if (0..8).contains(&index) {
            Some(Self(index as usize))
        } else {
            None
        }
    }

    pub fn as_index(&self) -> SquareIndex {
        self.0
    }

    pub fn as_char(&self) -> char {
        const FILES: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        FILES[self.as_index()]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardRank(pub SquareIndex);

impl BoardRank {
    pub fn from_char(c: char) -> Option<Self> {
        let index = c as i64 - '1' as i64;
        if (0..8).contains(&index) {
            Some(Self(index as usize))
        } else {
            None
        }
    }

    pub fn as_index(&self) -> SquareIndex {
        self.0
    }

    pub fn as_char(&self) -> char {
        const RANKS: [char; 8] = ['1', '2', '3', '4', '5', '6', '7', '8'];
        RANKS[self.as_index()]
    }
}

pub const A1: Square = Square(0);
pub const B1: Square = Square(1);
pub const C1: Square = Square(2);
pub const D1: Square = Square(3);
pub const E1: Square = Square(4);
pub const F1: Square = Square(5);
pub const G1: Square = Square(6);
pub const H1: Square = Square(7);
pub const A2: Square = Square(8);
pub const B2: Square = Square(9);
pub const C2: Square = Square(10);
pub const D2: Square = Square(11);
pub const E2: Square = Square(12);
pub const F2: Square = Square(13);
pub const G2: Square = Square(14);
pub const H2: Square = Square(15);
pub const A3: Square = Square(16);
pub const B3: Square = Square(17);
pub const C3: Square = Square(18);
pub const D3: Square = Square(19);
pub const E3: Square = Square(20);
pub const F3: Square = Square(21);
pub const G3: Square = Square(22);
pub const H3: Square = Square(23);
pub const A4: Square = Square(24);
pub const B4: Square = Square(25);
pub const C4: Square = Square(26);
pub const D4: Square = Square(27);
pub const E4: Square = Square(28);
pub const F4: Square = Square(29);
pub const G4: Square = Square(30);
pub const H4: Square = Square(31);
pub const A5: Square = Square(32);
pub const B5: Square = Square(33);
pub const C5: Square = Square(34);
pub const D5: Square = Square(35);
pub const E5: Square = Square(36);
pub const F5: Square = Square(37);
pub const G5: Square = Square(38);
pub const H5: Square = Square(39);
pub const A6: Square = Square(40);
pub const B6: Square = Square(41);
pub const C6: Square = Square(42);
pub const D6: Square = Square(43);
pub const E6: Square = Square(44);
pub const F6: Square = Square(45);
pub const G6: Square = Square(46);
pub const H6: Square = Square(47);
pub const A7: Square = Square(48);
pub const B7: Square = Square(49);
pub const C7: Square = Square(50);
pub const D7: Square = Square(51);
pub const E7: Square = Square(52);
pub const F7: Square = Square(53);
pub const G7: Square = Square(54);
pub const H7: Square = Square(55);
pub const A8: Square = Square(56);
pub const B8: Square = Square(57);
pub const C8: Square = Square(58);
pub const D8: Square = Square(59);
pub const E8: Square = Square(60);
pub const F8: Square = Square(61);
pub const G8: Square = Square(62);
pub const H8: Square = Square(63);

pub const FILE_A: BoardFile = BoardFile(0);
pub const FILE_B: BoardFile = BoardFile(1);
pub const FILE_C: BoardFile = BoardFile(2);
pub const FILE_D: BoardFile = BoardFile(3);
pub const FILE_E: BoardFile = BoardFile(4);
pub const FILE_F: BoardFile = BoardFile(5);
pub const FILE_G: BoardFile = BoardFile(6);
pub const FILE_H: BoardFile = BoardFile(7);

pub const RANK_1: BoardRank = BoardRank(0);
pub const RANK_2: BoardRank = BoardRank(1);
pub const RANK_3: BoardRank = BoardRank(2);
pub const RANK_4: BoardRank = BoardRank(3);
pub const RANK_5: BoardRank = BoardRank(4);
pub const RANK_6: BoardRank = BoardRank(5);
pub const RANK_7: BoardRank = BoardRank(6);
pub const RANK_8: BoardRank = BoardRank(7);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_at() {
        assert_eq!(Square::at(FILE_A, RANK_1), A1);
        assert_eq!(Square::at(FILE_B, RANK_2), B2);
        assert_eq!(Square::at(FILE_H, RANK_1), H1);
    }

    #[test]
    fn test_square_file() {
        assert_eq!(A1.file(), FILE_A);
        assert_eq!(B2.file(), FILE_B);
        assert_eq!(H1.file(), FILE_H);
    }

    #[test]
    fn test_square_rank() {
        assert_eq!(A1.rank(), RANK_1);
        assert_eq!(B2.rank(), RANK_2);
        assert_eq!(H1.rank(), RANK_1);
    }

    #[test]
    fn test_square_name() {
        assert_eq!(A1.name(), "a1");
        assert_eq!(B2.name(), "b2");
        assert_eq!(H1.name(), "h1");
    }

    #[test]
    fn test_square_from_name() {
        assert_eq!(Square::from_name("a1"), Some(A1));
        assert_eq!(Square::from_name("b2"), Some(B2));
        assert_eq!(Square::from_name("h1"), Some(H1));
        assert_eq!(Square::from_name("i9"), None);
        assert_eq!(Square::from_name(""), None);
        assert_eq!(Square::from_name("a111"), Some(A1));
    }
}

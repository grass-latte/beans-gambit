#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Square {
    A1,
    B1,
    C1,
    D1,
    E1,
    F1,
    G1,
    H1,
    A2,
    B2,
    C2,
    D2,
    E2,
    F2,
    G2,
    H2,
    A3,
    B3,
    C3,
    D3,
    E3,
    F3,
    G3,
    H3,
    A4,
    B4,
    C4,
    D4,
    E4,
    F4,
    G4,
    H4,
    A5,
    B5,
    C5,
    D5,
    E5,
    F5,
    G5,
    H5,
    A6,
    B6,
    C6,
    D6,
    E6,
    F6,
    G6,
    H6,
    A7,
    B7,
    C7,
    D7,
    E7,
    F7,
    G7,
    H7,
    A8,
    B8,
    C8,
    D8,
    E8,
    F8,
    G8,
    H8,
}

impl Square {
    pub fn get(index: usize) -> Self {
        debug_assert!(index < 64);
        unsafe { std::mem::transmute(index) }
    }

    pub fn at(file: BoardFile, rank: BoardRank) -> Self {
        Self::get(file.as_index() | (rank.as_index() << 3))
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
        BoardFile::get(self.as_index() & 7)
    }

    pub fn rank(&self) -> BoardRank {
        BoardRank::get(self.as_index() >> 3)
    }

    pub fn as_index(&self) -> usize {
        (*self) as usize
    }

    /// Returns the name of this square in algebraic notation
    pub fn name(&self) -> String {
        [self.file().as_char(), self.rank().as_char()]
            .iter()
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum BoardFile {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl BoardFile {
    pub fn get(index: usize) -> Self {
        debug_assert!(index < 8);
        unsafe { std::mem::transmute(index) }
    }

    pub fn from_char(c: char) -> Option<Self> {
        let index = c as i64 - 'a' as i64;
        if (0..8).contains(&index) {
            Some(Self::get(index as usize))
        } else {
            None
        }
    }

    pub fn as_index(&self) -> usize {
        (*self) as usize
    }

    pub fn as_char(&self) -> char {
        const FILES: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        FILES[self.as_index()]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum BoardRank {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
}

impl BoardRank {
    pub fn get(index: usize) -> Self {
        // ! SAFETY: `BoardRank` has `repr(usize)` and `index` is <8, so this is valid
        debug_assert!(index < 8);
        unsafe { std::mem::transmute(index) }
    }

    pub fn from_char(c: char) -> Option<Self> {
        let index = c as i64 - '1' as i64;
        if (0..8).contains(&index) {
            Some(Self::get(index as usize))
        } else {
            None
        }
    }

    pub fn as_index(&self) -> usize {
        (*self) as usize
    }

    pub fn as_char(&self) -> char {
        const RANKS: [char; 8] = ['1', '2', '3', '4', '5', '6', '7', '8'];
        RANKS[self.as_index()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_at() {
        assert_eq!(Square::at(BoardFile::A, BoardRank::R1), Square::A1);
        assert_eq!(Square::at(BoardFile::B, BoardRank::R2), Square::B2);
        assert_eq!(Square::at(BoardFile::H, BoardRank::R1), Square::H1);
    }

    #[test]
    fn test_square_file() {
        assert_eq!(Square::A1.file(), BoardFile::A);
        assert_eq!(Square::B2.file(), BoardFile::B);
        assert_eq!(Square::H1.file(), BoardFile::H);
    }

    #[test]
    fn test_square_rank() {
        assert_eq!(Square::A1.rank(), BoardRank::R1);
        assert_eq!(Square::B2.rank(), BoardRank::R2);
        assert_eq!(Square::H1.rank(), BoardRank::R1);
    }

    #[test]
    fn test_square_name() {
        assert_eq!(Square::A1.name(), "a1");
        assert_eq!(Square::B2.name(), "b2");
        assert_eq!(Square::H1.name(), "h1");
    }

    #[test]
    fn test_square_from_name() {
        assert_eq!(Square::from_name("a1"), Some(Square::A1));
        assert_eq!(Square::from_name("b2"), Some(Square::B2));
        assert_eq!(Square::from_name("h1"), Some(Square::H1));
        assert_eq!(Square::from_name("i9"), None);
        assert_eq!(Square::from_name(""), None);
        assert_eq!(Square::from_name("a111"), Some(Square::A1));
    }
}

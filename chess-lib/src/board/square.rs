use strum_macros::EnumIter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter)]
#[repr(u8)]
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
    pub const fn at(file: BoardFile, rank: BoardRank) -> Self {
        debug_assert!(file.as_u8() | (rank.as_u8() << 3) < 64);
        // SAFETY: Guaranteed to be < 64
        unsafe { Self::from_u8_unchecked(file.as_u8() | (rank.as_u8() << 3)) }
    }

    /// # Safety
    /// `file` and `rank` must be less than 8
    pub const unsafe fn at_xy_unchecked(file: u8, rank: u8) -> Self {
        debug_assert!(file < 8 && rank < 8);
        // SAFETY: Rank and file are less than 8
        unsafe {
            Self::at(
                BoardFile::from_u8_unchecked(file),
                BoardRank::from_u8_unchecked(rank),
            )
        }
    }

    pub const fn from_u8(index: u8) -> Option<Self> {
        if index < 64 {
            // SAFETY: `from_u8_unchecked` requires that index < 64.
            Some(unsafe { Self::from_u8_unchecked(index) })
        } else {
            None
        }
    }

    /// # Safety
    /// `index` must be less than 64.
    pub const unsafe fn from_u8_unchecked(index: u8) -> Self {
        debug_assert!(index < 64);
        // SAFETY: `Self` is repr(u8) and index < 64.
        unsafe { std::mem::transmute(index) }
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

    pub const fn file(self) -> BoardFile {
        // SAFETY: Guaranteed to be < 8
        unsafe { BoardFile::from_u8_unchecked(self.as_u8() & 7) }
    }

    pub const fn rank(self) -> BoardRank {
        debug_assert!(self.as_u8() >> 3 < 8);
        // SAFETY: Guaranteed to be < 8
        unsafe { BoardRank::from_u8_unchecked(self.as_u8() >> 3) }
    }

    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Returns the name of this square in algebraic notation
    pub fn name(self) -> String {
        [self.file().as_char(), self.rank().as_char()]
            .iter()
            .collect()
    }

    /// Returns the square given by translating this square by the first offset
    /// horizontally (-1 is towards the A file) and the second offset vertically (-1 is towards rank 1),
    /// or None if this square would be off the board.
    pub fn translated_by(self, (offset_x, offset_y): (i32, i32)) -> Option<Square> {
        let new_x = self.file().as_u8() as i32 + offset_x;
        let new_y = self.rank().as_u8() as i32 + offset_y;

        if (0..8).contains(&new_x) && (0..8).contains(&new_y) {
            // SAFETY: Verified that new_x and new_y are in the right range.
            unsafe {
                Some(Self::at(
                    BoardFile::from_u8_unchecked(new_x as u8),
                    BoardRank::from_u8_unchecked(new_y as u8),
                ))
            }
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, EnumIter)]
#[repr(u8)]
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
    pub const fn from_u8(index: u8) -> Option<Self> {
        if index < 8 {
            // SAFETY: `from_u8_unchecked` requires that index < 8.
            Some(unsafe { Self::from_u8_unchecked(index) })
        } else {
            None
        }
    }

    /// # Safety
    /// `index` must be less than 8.
    pub const unsafe fn from_u8_unchecked(index: u8) -> Self {
        debug_assert!(index < 8);
        // SAFETY: Index is a valid variant representation
        unsafe { std::mem::transmute(index) }
    }

    pub const fn from_char(c: char) -> Option<Self> {
        let index = c as i64 - 'a' as i64;
        if index >= 0 && index < 8 {
            // SAFETY: Index is a valid variant representation
            Some(unsafe { Self::from_u8_unchecked(index as u8) })
        } else {
            None
        }
    }

    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub const fn as_char(self) -> char {
        const FILES: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
        FILES[self.as_u8() as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, EnumIter)]
#[repr(u8)]
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
    pub const fn from_u8(index: u8) -> Option<Self> {
        if index < 8 {
            // SAFETY: `from_u8_unchecked` requires that index < 8.
            Some(unsafe { Self::from_u8_unchecked(index) })
        } else {
            None
        }
    }

    /// # Safety
    /// `index` must be less than 8.
    pub const unsafe fn from_u8_unchecked(index: u8) -> Self {
        debug_assert!(index < 8);
        // SAFETY: index is less than 8
        unsafe { std::mem::transmute(index) }
    }

    pub const fn from_char(c: char) -> Option<Self> {
        let index = c as i64 - '1' as i64;
        if index >= 0 && index < 8 {
            // SAFETY: index is 0 - 7
            Some(unsafe { Self::from_u8_unchecked(index as u8) })
        } else {
            None
        }
    }

    pub const fn flipped(self) -> BoardRank {
        debug_assert!(7 - self.as_u8() < 8);
        // SAFETY: 7 - (0..=7) results in 0..=7
        unsafe { BoardRank::from_u8_unchecked(7 - self.as_u8()) }
    }

    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub const fn as_char(self) -> char {
        const RANKS: [char; 8] = ['1', '2', '3', '4', '5', '6', '7', '8'];
        RANKS[self.as_u8() as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_from_u8() {
        assert_eq!(Square::from_u8(0), Some(Square::A1));
        assert_eq!(Square::from_u8(1), Some(Square::B1));
        assert_eq!(Square::from_u8(63), Some(Square::H8));
        assert_eq!(Square::from_u8(64), None);
    }

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

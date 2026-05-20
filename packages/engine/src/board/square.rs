/// Represents one of 64 squares on the chessboard.
/// Squares are indexed a1=0, b1=1, ..., h1=7, a2=8, ..., h8=63.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
    pub const NUM: usize = 64;

    /// Create a Square from a raw index (0–63). Panics if out of range.
    #[inline(always)]
    pub fn from_index(index: u8) -> Square {
        debug_assert!(index < 64, "Square index out of range: {}", index);
        // SAFETY: Square is repr(u8) with variants 0..63
        unsafe { std::mem::transmute(index) }
    }

    /// Create a Square from file (0–7) and rank (0–7).
    #[inline(always)]
    pub fn from_file_rank(file: u8, rank: u8) -> Square {
        Square::from_index(rank * 8 + file)
    }

    /// Returns the square index (0–63).
    #[inline(always)]
    pub fn index(self) -> usize {
        self as usize
    }

    /// Returns the file (0=a, 7=h).
    #[inline(always)]
    pub fn file(self) -> u8 {
        (self as u8) & 7
    }

    /// Returns the rank (0=rank1, 7=rank8).
    #[inline(always)]
    pub fn rank(self) -> u8 {
        (self as u8) >> 3
    }

    /// Returns the file as a char ('a'–'h').
    #[inline(always)]
    pub fn file_char(self) -> char {
        (b'a' + self.file()) as char
    }

    /// Returns the rank as a char ('1'–'8').
    #[inline(always)]
    pub fn rank_char(self) -> char {
        (b'1' + self.rank()) as char
    }

    /// Returns the bitboard mask for this square (1u64 << index).
    #[inline(always)]
    pub fn bb(self) -> u64 {
        1u64 << (self as u8)
    }

    /// Flips the square vertically (mirrors rank).
    #[inline(always)]
    pub fn flip(self) -> Square {
        Square::from_index(self as u8 ^ 56)
    }

    /// Manhattan distance between two squares.
    pub fn distance(self, other: Square) -> u8 {
        let file_diff = (self.file() as i8 - other.file() as i8).unsigned_abs();
        let rank_diff = (self.rank() as i8 - other.rank() as i8).unsigned_abs();
        file_diff.max(rank_diff)
    }

    /// Returns the square shifted by delta, or None if off the board.
    pub fn shift(self, delta: i8) -> Option<Square> {
        let idx = self as i8 + delta;
        if (0..64).contains(&idx) {
            Some(Square::from_index(idx as u8))
        } else {
            None
        }
    }

    /// Parse a square from algebraic notation (e.g. "e4").
    pub fn from_str(s: &str) -> Option<Square> {
        let mut chars = s.chars();
        let file = chars.next()? as u8;
        let rank = chars.next()? as u8;
        if (b'a'..=b'h').contains(&file) && (b'1'..=b'8').contains(&rank) {
            Some(Square::from_file_rank(file - b'a', rank - b'1'))
        } else {
            None
        }
    }

    /// Iterator over all 64 squares.
    pub fn all() -> impl Iterator<Item = Square> {
        (0u8..64).map(Square::from_index)
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file_char(), self.rank_char())
    }
}

// Well-known squares as constants for readability
impl Square {
    pub const E1: Square = Square::E1;
    pub const E8: Square = Square::E8;
    pub const A1: Square = Square::A1;
    pub const H1: Square = Square::H1;
    pub const A8: Square = Square::A8;
    pub const H8: Square = Square::H8;
}

/// Represents the side to move or piece ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub const NUM: usize = 2;

    /// Returns the opposite color.
    #[inline(always)]
    pub fn flip(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// Returns the color as a usize index (White=0, Black=1).
    #[inline(always)]
    pub fn index(self) -> usize {
        self as usize
    }

    /// Returns +1 for White, -1 for Black. Used in eval sign.
    #[inline(always)]
    pub fn sign(self) -> i32 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }

    /// Pawn push direction: White pushes up (+8), Black pushes down (-8).
    #[inline(always)]
    pub fn pawn_push(self) -> i32 {
        match self {
            Color::White => 8,
            Color::Black => -8,
        }
    }

    /// Rank that pawns start on (rank index 0-7).
    #[inline(always)]
    pub fn pawn_start_rank(self) -> u32 {
        match self {
            Color::White => 1,
            Color::Black => 6,
        }
    }

    /// Rank that pawns promote on (rank index 0-7).
    #[inline(always)]
    pub fn promotion_rank(self) -> u32 {
        match self {
            Color::White => 7,
            Color::Black => 0,
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::White => write!(f, "w"),
            Color::Black => write!(f, "b"),
        }
    }
}

impl TryFrom<char> for Color {
    type Error = String;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'w' => Ok(Color::White),
            'b' => Ok(Color::Black),
            _ => Err(format!("Invalid color char: {}", c)),
        }
    }
}

use crate::board::color::Color;

/// The type of a chess piece, without color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PieceType {
    Pawn   = 0,
    Knight = 1,
    Bishop = 2,
    Rook   = 3,
    Queen  = 4,
    King   = 5,
}

impl PieceType {
    pub const NUM: usize = 6;

    #[inline(always)]
    pub fn index(self) -> usize {
        self as usize
    }

    /// Standard centipawn material value.
    pub fn value(self) -> i32 {
        match self {
            PieceType::Pawn   => 100,
            PieceType::Knight => 320,
            PieceType::Bishop => 330,
            PieceType::Rook   => 500,
            PieceType::Queen  => 900,
            PieceType::King   => 20000,
        }
    }

    /// Is this piece a slider (bishop, rook, queen)?
    #[inline(always)]
    pub fn is_slider(self) -> bool {
        matches!(self, PieceType::Bishop | PieceType::Rook | PieceType::Queen)
    }

    /// Is this piece a diagonal slider (bishop, queen)?
    #[inline(always)]
    pub fn is_diagonal_slider(self) -> bool {
        matches!(self, PieceType::Bishop | PieceType::Queen)
    }

    /// Is this piece an orthogonal slider (rook, queen)?
    #[inline(always)]
    pub fn is_orthogonal_slider(self) -> bool {
        matches!(self, PieceType::Rook | PieceType::Queen)
    }

    pub fn from_index(i: usize) -> Option<PieceType> {
        match i {
            0 => Some(PieceType::Pawn),
            1 => Some(PieceType::Knight),
            2 => Some(PieceType::Bishop),
            3 => Some(PieceType::Rook),
            4 => Some(PieceType::Queen),
            5 => Some(PieceType::King),
            _ => None,
        }
    }

    /// Parse from FEN character (lowercase).
    pub fn from_char(c: char) -> Option<PieceType> {
        match c.to_ascii_lowercase() {
            'p' => Some(PieceType::Pawn),
            'n' => Some(PieceType::Knight),
            'b' => Some(PieceType::Bishop),
            'r' => Some(PieceType::Rook),
            'q' => Some(PieceType::Queen),
            'k' => Some(PieceType::King),
            _   => None,
        }
    }

    pub fn to_char(self) -> char {
        match self {
            PieceType::Pawn   => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook   => 'r',
            PieceType::Queen  => 'q',
            PieceType::King   => 'k',
        }
    }

    /// All piece types in order.
    pub fn all() -> [PieceType; 6] {
        [
            PieceType::Pawn,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Queen,
            PieceType::King,
        ]
    }
}

impl std::fmt::Display for PieceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

/// A piece with both type and color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

impl Piece {
    pub const fn new(piece_type: PieceType, color: Color) -> Piece {
        Piece { piece_type, color }
    }

    /// Returns the index 0–11 (used for bitboard arrays and Zobrist).
    /// Layout: [WhitePawn, WhiteKnight, ..., WhiteKing, BlackPawn, ..., BlackKing]
    #[inline(always)]
    pub fn index(self) -> usize {
        self.color.index() * PieceType::NUM + self.piece_type.index()
    }

    pub fn from_char(c: char) -> Option<Piece> {
        let piece_type = PieceType::from_char(c)?;
        let color = if c.is_uppercase() { Color::White } else { Color::Black };
        Some(Piece::new(piece_type, color))
    }

    pub fn to_char(self) -> char {
        let c = self.piece_type.to_char();
        if self.color == Color::White { c.to_ascii_uppercase() } else { c }
    }

    pub fn value(self) -> i32 {
        self.piece_type.value()
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

// All 12 piece constants
pub const WP: Piece = Piece::new(PieceType::Pawn,   Color::White);
pub const WN: Piece = Piece::new(PieceType::Knight, Color::White);
pub const WB: Piece = Piece::new(PieceType::Bishop, Color::White);
pub const WR: Piece = Piece::new(PieceType::Rook,   Color::White);
pub const WQ: Piece = Piece::new(PieceType::Queen,  Color::White);
pub const WK: Piece = Piece::new(PieceType::King,   Color::White);
pub const BP: Piece = Piece::new(PieceType::Pawn,   Color::Black);
pub const BN: Piece = Piece::new(PieceType::Knight, Color::Black);
pub const BB: Piece = Piece::new(PieceType::Bishop, Color::Black);
pub const BR: Piece = Piece::new(PieceType::Rook,   Color::Black);
pub const BQ: Piece = Piece::new(PieceType::Queen,  Color::Black);
pub const BK: Piece = Piece::new(PieceType::King,   Color::Black);

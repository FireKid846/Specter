use crate::board::bitboard::*;
use crate::board::color::Color;
use crate::board::piece::{Piece, PieceType};
use crate::board::square::Square;
use crate::board::zobrist::{
    CASTLE_WK, CASTLE_WQ, CASTLE_BK, CASTLE_BQ,
    CASTLING_KEYS, EN_PASSANT_KEYS, PIECE_KEYS, SIDE_KEY,
};

pub const STARTPOS_FEN: &str =
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// ─── Move representation ──────────────────────────────────────────────────────

/// A compact move encoded in a u32.
///
/// Bits:
///  0– 5: from square (0–63)
///  6–11: to square   (0–63)
/// 12–14: promotion piece type (0=none, 1=N, 2=B, 3=R, 4=Q)
/// 15–18: move flags (see MoveFlag)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move(pub u32);

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveFlag {
    Normal       = 0,
    DoublePush   = 1,
    CastleKing   = 2,
    CastleQueen  = 3,
    EnPassant    = 4,
    PromoteKnight= 5,
    PromoteBishop= 6,
    PromoteRook  = 7,
    PromoteQueen = 8,
}

impl Move {
    pub const NULL: Move = Move(0);

    pub fn new(from: Square, to: Square, flag: MoveFlag) -> Move {
        Move((from as u32) | ((to as u32) << 6) | ((flag as u32) << 12))
    }

    #[inline(always)] pub fn from(self) -> Square { Square::from_index((self.0 & 0x3F) as u8) }
    #[inline(always)] pub fn to(self)   -> Square { Square::from_index(((self.0 >> 6) & 0x3F) as u8) }
    #[inline(always)] pub fn flag(self) -> u32    { (self.0 >> 12) & 0xF }

    pub fn is_null(self) -> bool { self == Move::NULL }

    pub fn is_capture(self, pos: &Position) -> bool {
        pos.piece_on(self.to()).is_some() || self.flag() == MoveFlag::EnPassant as u32
    }

    pub fn is_promotion(self) -> bool {
        self.flag() >= MoveFlag::PromoteKnight as u32
    }

    pub fn promotion_piece(self) -> Option<PieceType> {
        match self.flag() {
            5 => Some(PieceType::Knight),
            6 => Some(PieceType::Bishop),
            7 => Some(PieceType::Rook),
            8 => Some(PieceType::Queen),
            _ => None,
        }
    }

    /// UCI string representation (e.g. "e2e4", "e7e8q").
    pub fn to_uci(self) -> String {
        let promo = match self.flag() {
            5 => "n", 6 => "b", 7 => "r", 8 => "q",
            _ => "",
        };
        format!("{}{}{}", self.from(), self.to(), promo)
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_uci())
    }
}

// ─── Position ─────────────────────────────────────────────────────────────────

/// Stores the complete game state.
#[derive(Clone)]
pub struct Position {
    /// Bitboard for each of the 12 piece types: [WP, WN, WB, WR, WQ, WK, BP, BN, BB, BR, BQ, BK]
    pub pieces:          [u64; 12],
    /// Bitboard of all pieces for each color.
    pub colors:          [u64; 2],
    /// Combined occupancy (colors[0] | colors[1]).
    pub occupancy:       u64,
    /// Side to move.
    pub side:            Color,
    /// Castling rights bitmask (bits: WK=1, WQ=2, BK=4, BQ=8).
    pub castling:        u8,
    /// En passant target square (if any).
    pub en_passant:      Option<Square>,
    /// Halfmove clock for 50-move rule.
    pub halfmove_clock:  u8,
    /// Fullmove number.
    pub fullmove:        u16,
    /// Zobrist hash of this position.
    pub hash:            u64,
    /// History of hashes for repetition detection.
    pub hash_history:    Vec<u64>,
    /// Check counter for Three-Check variant.
    pub checks_given:    [u8; 2],
}

impl Position {
    // ─── Accessors ────────────────────────────────────────────────────────────

    /// Returns bitboard of a given piece type for a given color.
    #[inline(always)]
    pub fn bb(&self, color: Color, pt: PieceType) -> u64 {
        self.pieces[color.index() * PieceType::NUM + pt.index()]
    }

    /// Returns bitboard of all pieces of a given color.
    #[inline(always)]
    pub fn color_bb(&self, color: Color) -> u64 {
        self.colors[color.index()]
    }

    /// Returns the piece on a square, if any.
    pub fn piece_on(&self, sq: Square) -> Option<Piece> {
        let mask = sq.bb();
        for color in [Color::White, Color::Black] {
            for pt in PieceType::all() {
                if self.bb(color, pt) & mask != 0 {
                    return Some(Piece::new(pt, color));
                }
            }
        }
        None
    }

    /// Returns the piece type on a square for the given color.
    pub fn piece_type_on(&self, sq: Square, color: Color) -> Option<PieceType> {
        let mask = sq.bb();
        for pt in PieceType::all() {
            if self.bb(color, pt) & mask != 0 {
                return Some(pt);
            }
        }
        None
    }

    /// King square for a given color.
    #[inline(always)]
    pub fn king_sq(&self, color: Color) -> Square {
        let bb = self.bb(color, PieceType::King);
        Square::from_index(lsb(bb) as u8)
    }

    // ─── Internal mutation ────────────────────────────────────────────────────

    #[inline(always)]
    fn put_piece(&mut self, color: Color, pt: PieceType, sq: Square) {
        let idx = color.index() * PieceType::NUM + pt.index();
        let mask = sq.bb();
        self.pieces[idx] |= mask;
        self.colors[color.index()] |= mask;
        self.occupancy |= mask;
        self.hash ^= PIECE_KEYS[idx][sq.index()];
    }

    #[inline(always)]
    fn remove_piece(&mut self, color: Color, pt: PieceType, sq: Square) {
        let idx = color.index() * PieceType::NUM + pt.index();
        let mask = sq.bb();
        self.pieces[idx] &= !mask;
        self.colors[color.index()] &= !mask;
        self.occupancy &= !mask;
        self.hash ^= PIECE_KEYS[idx][sq.index()];
    }

    #[inline(always)]
    fn move_piece(&mut self, color: Color, pt: PieceType, from: Square, to: Square) {
        self.remove_piece(color, pt, from);
        self.put_piece(color, pt, to);
    }

    // ─── FEN parsing ──────────────────────────────────────────────────────────

    pub fn from_fen(fen: &str) -> Result<Position, String> {
        let mut pos = Position {
            pieces:         [0u64; 12],
            colors:         [0u64; 2],
            occupancy:      0,
            side:           Color::White,
            castling:       0,
            en_passant:     None,
            halfmove_clock: 0,
            fullmove:       1,
            hash:           0,
            hash_history:   Vec::new(),
            checks_given:   [0; 2],
        };

        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() < 4 {
            return Err("FEN too short".into());
        }

        // 1. Piece placement
        let mut rank: i8 = 7;
        let mut file: i8 = 0;
        for ch in parts[0].chars() {
            match ch {
                '/' => { rank -= 1; file = 0; }
                '1'..='8' => { file += (ch as u8 - b'0') as i8; }
                _ => {
                    let piece = Piece::from_char(ch)
                        .ok_or_else(|| format!("Invalid piece char: {}", ch))?;
                    let sq = Square::from_file_rank(file as u8, rank as u8);
                    pos.put_piece(piece.color, piece.piece_type, sq);
                    file += 1;
                }
            }
        }

        // 2. Side to move
        pos.side = Color::try_from(parts[1].chars().next().unwrap_or('w'))
            .map_err(|e| e)?;
        if pos.side == Color::Black {
            pos.hash ^= SIDE_KEY;
        }

        // 3. Castling rights
        pos.hash ^= CASTLING_KEYS[pos.castling as usize]; // remove old (0)
        for ch in parts[2].chars() {
            match ch {
                'K' => pos.castling |= CASTLE_WK,
                'Q' => pos.castling |= CASTLE_WQ,
                'k' => pos.castling |= CASTLE_BK,
                'q' => pos.castling |= CASTLE_BQ,
                '-' => {}
                _   => return Err(format!("Invalid castling char: {}", ch)),
            }
        }
        pos.hash ^= CASTLING_KEYS[pos.castling as usize];

        // 4. En passant
        if parts[3] != "-" {
            if let Some(sq) = Square::from_str(parts[3]) {
                pos.hash ^= EN_PASSANT_KEYS[sq.file() as usize];
                pos.en_passant = Some(sq);
            }
        }

        // 5. Halfmove clock
        if parts.len() > 4 {
            pos.halfmove_clock = parts[4].parse().unwrap_or(0);
        }

        // 6. Fullmove number
        if parts.len() > 5 {
            pos.fullmove = parts[5].parse().unwrap_or(1);
        }

        pos.hash_history.push(pos.hash);
        Ok(pos)
    }

    /// Load the standard starting position.
    pub fn startpos() -> Position {
        Position::from_fen(STARTPOS_FEN).expect("startpos FEN must be valid")
    }

    // ─── FEN export ───────────────────────────────────────────────────────────

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        // Piece placement
        for rank in (0..8).rev() {
            let mut empty = 0u8;
            for file in 0..8u8 {
                let sq = Square::from_file_rank(file, rank);
                if let Some(piece) = self.piece_on(sq) {
                    if empty > 0 { fen.push((b'0' + empty) as char); empty = 0; }
                    fen.push(piece.to_char());
                } else {
                    empty += 1;
                }
            }
            if empty > 0 { fen.push((b'0' + empty) as char); }
            if rank > 0 { fen.push('/'); }
        }

        // Side to move
        fen.push(' ');
        fen.push_str(&self.side.to_string());

        // Castling
        fen.push(' ');
        if self.castling == 0 {
            fen.push('-');
        } else {
            if self.castling & CASTLE_WK != 0 { fen.push('K'); }
            if self.castling & CASTLE_WQ != 0 { fen.push('Q'); }
            if self.castling & CASTLE_BK != 0 { fen.push('k'); }
            if self.castling & CASTLE_BQ != 0 { fen.push('q'); }
        }

        // En passant
        fen.push(' ');
        match self.en_passant {
            Some(sq) => fen.push_str(&sq.to_string()),
            None     => fen.push('-'),
        }

        // Clocks
        fen.push_str(&format!(" {} {}", self.halfmove_clock, self.fullmove));
        fen
    }

    // ─── Make / Unmake move ───────────────────────────────────────────────────

    /// Applies a move and returns the state needed to unmake it.
    pub fn make_move(&mut self, mv: Move) -> UnmakeState {
        let state = UnmakeState {
            castling:       self.castling,
            en_passant:     self.en_passant,
            halfmove_clock: self.halfmove_clock,
            hash:           self.hash,
            captured:       None,
        };

        let us   = self.side;
        let them = us.flip();
        let from = mv.from();
        let to   = mv.to();
        let flag = mv.flag();

        // Remove en passant from hash
        if let Some(ep) = self.en_passant {
            self.hash ^= EN_PASSANT_KEYS[ep.file() as usize];
            self.en_passant = None;
        }

        // Remove old castling rights from hash
        self.hash ^= CASTLING_KEYS[self.castling as usize];

        let moving_pt = self.piece_type_on(from, us)
            .expect("No piece on from square");

        // Handle captures
        let mut captured_pt = self.piece_type_on(to, them);

        if let Some(cpt) = captured_pt {
            self.remove_piece(them, cpt, to);
            self.halfmove_clock = 0;
        }

        // Move the piece
        match flag {
            // Normal move or double pawn push
            0 | 1 => {
                self.move_piece(us, moving_pt, from, to);
                if moving_pt == PieceType::Pawn {
                    self.halfmove_clock = 0;
                    if flag == 1 {
                        // Set en passant square
                        let ep_sq = Square::from_index((to as i32 - us.pawn_push()) as u8);
                        self.en_passant = Some(ep_sq);
                        self.hash ^= EN_PASSANT_KEYS[ep_sq.file() as usize];
                    }
                } else {
                    self.halfmove_clock += 1;
                }
            }
            // Castling
            2 => {
                // Kingside
                self.move_piece(us, PieceType::King, from, to);
                let (rook_from, rook_to) = if us == Color::White {
                    (Square::H1, Square::F1)
                } else {
                    (Square::H8, Square::F8)
                };
                self.move_piece(us, PieceType::Rook, rook_from, rook_to);
                self.halfmove_clock += 1;
            }
            3 => {
                // Queenside
                self.move_piece(us, PieceType::King, from, to);
                let (rook_from, rook_to) = if us == Color::White {
                    (Square::A1, Square::D1)
                } else {
                    (Square::A8, Square::D8)
                };
                self.move_piece(us, PieceType::Rook, rook_from, rook_to);
                self.halfmove_clock += 1;
            }
            // En passant
            4 => {
                self.move_piece(us, PieceType::Pawn, from, to);
                let cap_sq = Square::from_index((to as i32 - us.pawn_push()) as u8);
                self.remove_piece(them, PieceType::Pawn, cap_sq);
                captured_pt = Some(PieceType::Pawn);
                self.halfmove_clock = 0;
            }
            // Promotions (5=N, 6=B, 7=R, 8=Q)
            5..=8 => {
                let promo_pt = mv.promotion_piece().unwrap();
                self.remove_piece(us, PieceType::Pawn, from);
                self.put_piece(us, promo_pt, to);
                self.halfmove_clock = 0;
            }
            _ => panic!("Unknown move flag: {}", flag),
        }

        // Update castling rights
        self.castling &= castling_mask(from) & castling_mask(to);
        self.hash ^= CASTLING_KEYS[self.castling as usize];

        // Switch side
        self.side = them;
        self.hash ^= SIDE_KEY;

        // Fullmove counter
        if us == Color::Black {
            self.fullmove += 1;
        }

        self.hash_history.push(self.hash);

        UnmakeState { captured: captured_pt, ..state }
    }

    /// Reverts the last move using saved state.
    pub fn unmake_move(&mut self, mv: Move, state: UnmakeState) {
        self.hash_history.pop();

        let us   = self.side.flip(); // who made the move
        let them = self.side;
        let from = mv.from();
        let to   = mv.to();
        let flag = mv.flag();

        // Restore state
        self.castling       = state.castling;
        self.en_passant     = state.en_passant;
        self.halfmove_clock = state.halfmove_clock;
        self.hash           = state.hash;
        self.side           = us;
        if us == Color::Black { self.fullmove -= 1; }

        match flag {
            0 | 1 => {
                let pt = self.piece_type_on(to, us).expect("No piece on to square");
                self.move_piece(us, pt, to, from);
                if let Some(cpt) = state.captured {
                    self.put_piece(them, cpt, to);
                }
            }
            2 => {
                self.move_piece(us, PieceType::King, to, from);
                let (rook_from, rook_to) = if us == Color::White {
                    (Square::H1, Square::F1)
                } else {
                    (Square::H8, Square::F8)
                };
                self.move_piece(us, PieceType::Rook, rook_to, rook_from);
            }
            3 => {
                self.move_piece(us, PieceType::King, to, from);
                let (rook_from, rook_to) = if us == Color::White {
                    (Square::A1, Square::D1)
                } else {
                    (Square::A8, Square::D8)
                };
                self.move_piece(us, PieceType::Rook, rook_to, rook_from);
            }
            4 => {
                self.move_piece(us, PieceType::Pawn, to, from);
                let cap_sq = Square::from_index((to as i32 - us.pawn_push()) as u8);
                self.put_piece(them, PieceType::Pawn, cap_sq);
            }
            5..=8 => {
                let promo_pt = mv.promotion_piece().unwrap();
                self.remove_piece(us, promo_pt, to);
                self.put_piece(us, PieceType::Pawn, from);
                if let Some(cpt) = state.captured {
                    self.put_piece(them, cpt, to);
                }
            }
            _ => panic!("Unknown move flag: {}", flag),
        }
    }

    /// Make a null move (switch side, clear en passant). Used in null move pruning.
    pub fn make_null_move(&mut self) -> NullMoveState {
        let state = NullMoveState {
            en_passant:     self.en_passant,
            halfmove_clock: self.halfmove_clock,
            hash:           self.hash,
        };
        if let Some(ep) = self.en_passant {
            self.hash ^= EN_PASSANT_KEYS[ep.file() as usize];
            self.en_passant = None;
        }
        self.side = self.side.flip();
        self.hash ^= SIDE_KEY;
        self.hash_history.push(self.hash);
        state
    }

    pub fn unmake_null_move(&mut self, state: NullMoveState) {
        self.hash_history.pop();
        self.en_passant     = state.en_passant;
        self.halfmove_clock = state.halfmove_clock;
        self.hash           = state.hash;
        self.side           = self.side.flip();
    }

    // ─── Game state queries ───────────────────────────────────────────────────

    /// Is the current position a draw by 50-move rule?
    pub fn is_fifty_move_draw(&self) -> bool {
        self.halfmove_clock >= 100
    }

    /// Is the current position a draw by threefold repetition?
    pub fn is_repetition(&self) -> bool {
        let current = self.hash;
        let len = self.hash_history.len();
        if len < 5 { return false; }
        let mut count = 0;
        for &h in self.hash_history[..len - 1].iter().rev().step_by(2) {
            if h == current {
                count += 1;
                if count >= 2 { return true; }
            }
        }
        false
    }

    /// Is there insufficient material for checkmate?
    pub fn is_insufficient_material(&self) -> bool {
        // Only kings
        if self.occupancy == self.colors[0] | self.colors[1]
            && popcount(self.occupancy) == 2 {
            return true;
        }
        // King + minor piece vs King
        let total = popcount(self.occupancy);
        if total == 3 {
            let minor_w = popcount(self.bb(Color::White, PieceType::Knight)
                | self.bb(Color::White, PieceType::Bishop));
            let minor_b = popcount(self.bb(Color::Black, PieceType::Knight)
                | self.bb(Color::Black, PieceType::Bishop));
            if minor_w == 1 || minor_b == 1 { return true; }
        }
        false
    }

    // ─── Debug ───────────────────────────────────────────────────────────────

    pub fn print(&self) {
        println!();
        println!("  +---+---+---+---+---+---+---+---+");
        for rank in (0..8).rev() {
            print!("{} |", rank + 1);
            for file in 0..8u8 {
                let sq = Square::from_file_rank(file, rank);
                let ch = self.piece_on(sq).map(|p| p.to_char()).unwrap_or('.');
                print!(" {} |", ch);
            }
            println!();
            println!("  +---+---+---+---+---+---+---+---+");
        }
        println!("    a   b   c   d   e   f   g   h");
        println!();
        println!("  FEN:      {}", self.to_fen());
        println!("  Hash:     {:#018x}", self.hash);
        println!("  Side:     {}", self.side);
        println!("  Castling: {:04b}", self.castling);
        if let Some(ep) = self.en_passant {
            println!("  En pass:  {}", ep);
        }
        println!();
    }
}

/// State needed to unmake a move.
#[derive(Clone, Copy)]
pub struct UnmakeState {
    pub castling:       u8,
    pub en_passant:     Option<Square>,
    pub halfmove_clock: u8,
    pub hash:           u64,
    pub captured:       Option<PieceType>,
}

/// State needed to unmake a null move.
#[derive(Clone, Copy)]
pub struct NullMoveState {
    pub en_passant:     Option<Square>,
    pub halfmove_clock: u8,
    pub hash:           u64,
}

/// Castling rights mask — if a rook or king moves from a square,
/// the corresponding castling right is revoked.
const CASTLING_MASKS: [u8; 64] = {
    let mut masks = [0xFFu8; 64];
    masks[Square::E1 as usize] = !(CASTLE_WK | CASTLE_WQ);
    masks[Square::A1 as usize] = !CASTLE_WQ;
    masks[Square::H1 as usize] = !CASTLE_WK;
    masks[Square::E8 as usize] = !(CASTLE_BK | CASTLE_BQ);
    masks[Square::A8 as usize] = !CASTLE_BQ;
    masks[Square::H8 as usize] = !CASTLE_BK;
    masks
};

#[inline(always)]
fn castling_mask(sq: Square) -> u8 {
    CASTLING_MASKS[sq.index()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_startpos_fen() {
        let pos = Position::startpos();
        assert_eq!(pos.to_fen(), STARTPOS_FEN);
    }

    #[test]
    fn test_fen_roundtrip() {
        let fens = [
            STARTPOS_FEN,
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        ];
        for fen in &fens {
            let pos = Position::from_fen(fen).unwrap();
            assert_eq!(&pos.to_fen(), fen, "FEN roundtrip failed for: {}", fen);
        }
    }

    #[test]
    fn test_piece_count_startpos() {
        let pos = Position::startpos();
        assert_eq!(popcount(pos.bb(Color::White, PieceType::Pawn)), 8);
        assert_eq!(popcount(pos.bb(Color::Black, PieceType::Pawn)), 8);
        assert_eq!(popcount(pos.occupancy), 32);
    }
}

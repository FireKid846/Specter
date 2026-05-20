/// Pseudo-legal move generation.
/// Generates all moves that are legal ignoring checks (king may be left in check).
/// Legal move generation in legal.rs filters these with in-check validation.

use crate::board::bitboard::*;
use crate::board::color::Color;
use crate::board::piece::PieceType;
use crate::board::position::{Move, MoveFlag, Position};
use crate::board::square::Square;
use crate::board::zobrist::{CASTLE_WK, CASTLE_WQ, CASTLE_BK, CASTLE_BQ};
use crate::movegen::attacks::*;

pub struct MoveList {
    pub moves: [Move; 256],
    pub count: usize,
}

impl MoveList {
    pub fn new() -> Self {
        MoveList {
            moves: [Move::NULL; 256],
            count: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, mv: Move) {
        self.moves[self.count] = mv;
        self.count += 1;
    }

    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.count]
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate all pseudo-legal moves for the side to move.
pub fn generate_all(pos: &Position) -> MoveList {
    let mut list = MoveList::new();
    let us   = pos.side;
    let them = us.flip();
    let occ  = pos.occupancy;
    let our  = pos.color_bb(us);
    let their = pos.color_bb(them);

    gen_pawns(pos, us, occ, their, &mut list);
    gen_knights(pos, us, our, &mut list);
    gen_bishops(pos, us, our, occ, &mut list);
    gen_rooks(pos, us, our, occ, &mut list);
    gen_queens(pos, us, our, occ, &mut list);
    gen_king(pos, us, our, &mut list);
    gen_castling(pos, us, occ, &mut list);

    list
}

/// Generate only captures and promotions (for quiescence search).
pub fn generate_captures(pos: &Position) -> MoveList {
    let mut list = MoveList::new();
    let us    = pos.side;
    let them  = us.flip();
    let occ   = pos.occupancy;
    let our   = pos.color_bb(us);
    let their = pos.color_bb(them);

    gen_pawn_captures(pos, us, occ, their, &mut list);
    gen_knight_captures(pos, us, their, &mut list);
    gen_bishop_captures(pos, us, our, occ, their, &mut list);
    gen_rook_captures(pos, us, our, occ, their, &mut list);
    gen_queen_captures(pos, us, our, occ, their, &mut list);
    gen_king_captures(pos, us, our, their, &mut list);

    list
}

// ─── Pawn generation ──────────────────────────────────────────────────────────

fn gen_pawns(pos: &Position, us: Color, occ: u64, their: u64, list: &mut MoveList) {
    gen_pawn_pushes(pos, us, occ, list);
    gen_pawn_captures(pos, us, occ, their, list);
}

fn gen_pawn_pushes(pos: &Position, us: Color, occ: u64, list: &mut MoveList) {
    let pawns = pos.bb(us, PieceType::Pawn);
    let empty = !occ;

    let (single, double_start, promo_rank, push): (u64, u64, u64, fn(u64) -> u64) = match us {
        Color::White => (north(pawns) & empty, RANK_2, RANK_8, north),
        Color::Black => (south(pawns) & empty, RANK_7, RANK_1, south),
    };

    // Single pushes (non-promotion)
    let mut singles = single & !promo_rank;
    while singles != 0 {
        let to = pop_lsb(&mut singles);
        let from_sq = if us == Color::White { to - 8 } else { to + 8 };
        list.push(Move::new(
            Square::from_index(from_sq as u8),
            Square::from_index(to as u8),
            MoveFlag::Normal,
        ));
    }

    // Double pushes
    let mut doubles = push(single & if us == Color::White { RANK_3 } else { RANK_6 }) & empty;
    while doubles != 0 {
        let to = pop_lsb(&mut doubles);
        let from_sq = if us == Color::White { to - 16 } else { to + 16 };
        list.push(Move::new(
            Square::from_index(from_sq as u8),
            Square::from_index(to as u8),
            MoveFlag::DoublePush,
        ));
    }

    // Promotions
    let mut promos = single & promo_rank;
    while promos != 0 {
        let to = pop_lsb(&mut promos);
        let from_sq = if us == Color::White { to - 8 } else { to + 8 };
        let from = Square::from_index(from_sq as u8);
        let to_sq = Square::from_index(to as u8);
        list.push(Move::new(from, to_sq, MoveFlag::PromoteQueen));
        list.push(Move::new(from, to_sq, MoveFlag::PromoteRook));
        list.push(Move::new(from, to_sq, MoveFlag::PromoteBishop));
        list.push(Move::new(from, to_sq, MoveFlag::PromoteKnight));
    }
}

fn gen_pawn_captures(pos: &Position, us: Color, occ: u64, their: u64, list: &mut MoveList) {
    let pawns = pos.bb(us, PieceType::Pawn);
    let promo_rank = if us == Color::White { RANK_8 } else { RANK_1 };

    for sq in BitIter(pawns) {
        let attacks = pawn_attacks(us, sq) & their;
        let mut caps = attacks & !promo_rank;
        while caps != 0 {
            let to = pop_lsb(&mut caps);
            list.push(Move::new(
                Square::from_index(sq as u8),
                Square::from_index(to as u8),
                MoveFlag::Normal,
            ));
        }
        // Promotion captures
        let mut promo_caps = attacks & promo_rank;
        while promo_caps != 0 {
            let to = pop_lsb(&mut promo_caps);
            let from = Square::from_index(sq as u8);
            let to_sq = Square::from_index(to as u8);
            list.push(Move::new(from, to_sq, MoveFlag::PromoteQueen));
            list.push(Move::new(from, to_sq, MoveFlag::PromoteRook));
            list.push(Move::new(from, to_sq, MoveFlag::PromoteBishop));
            list.push(Move::new(from, to_sq, MoveFlag::PromoteKnight));
        }
    }

    // En passant
    if let Some(ep) = pos.en_passant {
        let ep_attackers = pawn_attacks(us.flip(), ep as u32) & pawns;
        for sq in BitIter(ep_attackers) {
            list.push(Move::new(
                Square::from_index(sq as u8),
                ep,
                MoveFlag::EnPassant,
            ));
        }
    }
}

// ─── Knight generation ────────────────────────────────────────────────────────

fn gen_knights(pos: &Position, us: Color, our: u64, list: &mut MoveList) {
    let mut knights = pos.bb(us, PieceType::Knight);
    while knights != 0 {
        let sq = pop_lsb(&mut knights);
        let mut attacks = knight_attacks_sq(sq) & !our;
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            list.push(Move::new(
                Square::from_index(sq as u8),
                Square::from_index(to as u8),
                MoveFlag::Normal,
            ));
        }
    }
}

fn gen_knight_captures(pos: &Position, us: Color, their: u64, list: &mut MoveList) {
    let mut knights = pos.bb(us, PieceType::Knight);
    while knights != 0 {
        let sq = pop_lsb(&mut knights);
        let mut attacks = knight_attacks_sq(sq) & their;
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            list.push(Move::new(
                Square::from_index(sq as u8),
                Square::from_index(to as u8),
                MoveFlag::Normal,
            ));
        }
    }
}

// ─── Bishop generation ────────────────────────────────────────────────────────

fn gen_bishops(pos: &Position, us: Color, our: u64, occ: u64, list: &mut MoveList) {
    let mut bishops = pos.bb(us, PieceType::Bishop);
    while bishops != 0 {
        let sq = pop_lsb(&mut bishops);
        let mut attacks = bishop_attacks(sq, occ) & !our;
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            list.push(Move::new(
                Square::from_index(sq as u8),
                Square::from_index(to as u8),
                MoveFlag::Normal,
            ));
        }
    }
}

fn gen_bishop_captures(pos: &Position, us: Color, our: u64, occ: u64, their: u64, list: &mut MoveList) {
    let mut bishops = pos.bb(us, PieceType::Bishop);
    while bishops != 0 {
        let sq = pop_lsb(&mut bishops);
        let mut attacks = bishop_attacks(sq, occ) & their;
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            list.push(Move::new(
                Square::from_index(sq as u8),
                Square::from_index(to as u8),
                MoveFlag::Normal,
            ));
        }
    }
}

// ─── Rook generation ──────────────────────────────────────────────────────────

fn gen_rooks(pos: &Position, us: Color, our: u64, occ: u64, list: &mut MoveList) {
    let mut rooks = pos.bb(us, PieceType::Rook);
    while rooks != 0 {
        let sq = pop_lsb(&mut rooks);
        let mut attacks = rook_attacks(sq, occ) & !our;
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            list.push(Move::new(
                Square::from_index(sq as u8),
                Square::from_index(to as u8),
                MoveFlag::Normal,
            ));
        }
    }
}

fn gen_rook_captures(pos: &Position, us: Color, our: u64, occ: u64, their: u64, list: &mut MoveList) {
    let mut rooks = pos.bb(us, PieceType::Rook);
    while rooks != 0 {
        let sq = pop_lsb(&mut rooks);
        let mut attacks = rook_attacks(sq, occ) & their;
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            list.push(Move::new(
                Square::from_index(sq as u8),
                Square::from_index(to as u8),
                MoveFlag::Normal,
            ));
        }
    }
}

// ─── Queen generation ─────────────────────────────────────────────────────────

fn gen_queens(pos: &Position, us: Color, our: u64, occ: u64, list: &mut MoveList) {
    let mut queens = pos.bb(us, PieceType::Queen);
    while queens != 0 {
        let sq = pop_lsb(&mut queens);
        let mut attacks = queen_attacks(sq, occ) & !our;
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            list.push(Move::new(
                Square::from_index(sq as u8),
                Square::from_index(to as u8),
                MoveFlag::Normal,
            ));
        }
    }
}

fn gen_queen_captures(pos: &Position, us: Color, our: u64, occ: u64, their: u64, list: &mut MoveList) {
    let mut queens = pos.bb(us, PieceType::Queen);
    while queens != 0 {
        let sq = pop_lsb(&mut queens);
        let mut attacks = queen_attacks(sq, occ) & their;
        while attacks != 0 {
            let to = pop_lsb(&mut attacks);
            list.push(Move::new(
                Square::from_index(sq as u8),
                Square::from_index(to as u8),
                MoveFlag::Normal,
            ));
        }
    }
}

// ─── King generation ──────────────────────────────────────────────────────────

fn gen_king(pos: &Position, us: Color, our: u64, list: &mut MoveList) {
    let king_sq = pos.king_sq(us);
    let mut attacks = king_attacks_sq(king_sq as u32) & !our;
    while attacks != 0 {
        let to = pop_lsb(&mut attacks);
        list.push(Move::new(
            king_sq,
            Square::from_index(to as u8),
            MoveFlag::Normal,
        ));
    }
}

fn gen_king_captures(pos: &Position, us: Color, our: u64, their: u64, list: &mut MoveList) {
    let king_sq = pos.king_sq(us);
    let mut attacks = king_attacks_sq(king_sq as u32) & their;
    while attacks != 0 {
        let to = pop_lsb(&mut attacks);
        list.push(Move::new(
            king_sq,
            Square::from_index(to as u8),
            MoveFlag::Normal,
        ));
    }
}

// ─── Castling ─────────────────────────────────────────────────────────────────

fn gen_castling(pos: &Position, us: Color, occ: u64, list: &mut MoveList) {
    let (king_sq, ks_right, qs_right, ks_path, qs_path, ks_to, qs_to) = match us {
        Color::White => (
            Square::E1, CASTLE_WK, CASTLE_WQ,
            Square::F1.bb() | Square::G1.bb(),
            Square::B1.bb() | Square::C1.bb() | Square::D1.bb(),
            Square::G1, Square::C1,
        ),
        Color::Black => (
            Square::E8, CASTLE_BK, CASTLE_BQ,
            Square::F8.bb() | Square::G8.bb(),
            Square::B8.bb() | Square::C8.bb() | Square::D8.bb(),
            Square::G8, Square::C8,
        ),
    };

    if pos.castling & ks_right != 0 && occ & ks_path == 0 {
        list.push(Move::new(king_sq, ks_to, MoveFlag::CastleKing));
    }
    if pos.castling & qs_right != 0 && occ & qs_path == 0 {
        list.push(Move::new(king_sq, qs_to, MoveFlag::CastleQueen));
    }
}

/// Threats evaluation — rewards attacking enemy pieces and penalizes hanging pieces.

use crate::board::bitboard::*;
use crate::board::color::Color;
use crate::board::piece::PieceType;
use crate::board::position::Position;
use crate::eval::make_score;
use crate::movegen::attacks::*;

const HANGING_PENALTY:          i32 = make_score(-69, -50);
const PAWN_ATTACKS_MINOR:       i32 = make_score( 59,  30);
const PAWN_ATTACKS_ROOK:        i32 = make_score( 83,  43);
const PAWN_ATTACKS_QUEEN:       i32 = make_score( 89,  53);
const MINOR_ATTACKS_ROOK:       i32 = make_score( 46,  26);
const MINOR_ATTACKS_QUEEN:      i32 = make_score( 49,  28);
const ROOK_ATTACKS_QUEEN:       i32 = make_score( 24,  15);
const SAFE_PAWN_THREAT:         i32 = make_score( 20,  25);
const KING_THREAT_MINOR:        i32 = make_score( 17,  13);
const KING_THREAT_ROOK:         i32 = make_score( 17,  13);

pub fn evaluate_threats(pos: &Position) -> i32 {
    let mut score = 0i32;
    score += eval_threats_for(pos, Color::White);
    score -= eval_threats_for(pos, Color::Black);
    score
}

fn eval_threats_for(pos: &Position, us: Color) -> i32 {
    let them  = us.flip();
    let occ   = pos.occupancy;
    let _our   = pos.color_bb(us);
    let their = pos.color_bb(them);

    let our_pawns   = pos.bb(us, PieceType::Pawn);
    let their_pawns = pos.bb(them, PieceType::Pawn);

    // Our pawn attacks
    let our_pawn_attacks = match us {
        Color::White => white_pawn_attacks(our_pawns),
        Color::Black => black_pawn_attacks(our_pawns),
    };

    let their_pawn_attacks = match them {
        Color::White => white_pawn_attacks(their_pawns),
        Color::Black => black_pawn_attacks(their_pawns),
    };

    let mut score = 0i32;

    // ── Hanging pieces ────────────────────────────────────────────────────
    // A piece is hanging if it's attacked and not defended.
    for pt in [PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen] {
        let mut pieces = pos.bb(them, pt);
        while pieces != 0 {
            let sq = pop_lsb(&mut pieces) as u32;
            let attacked = is_attacked_by(pos, sq, us, occ);
            let defended = is_attacked_by(pos, sq, them, occ);
            if attacked && !defended {
                score += HANGING_PENALTY;
            }
        }
    }

    // ── Pawn threats ──────────────────────────────────────────────────────
    let pawn_attacked = our_pawn_attacks & their;
    let minor_attacked_by_pawn = pawn_attacked
        & (pos.bb(them, PieceType::Knight) | pos.bb(them, PieceType::Bishop));
    let rook_attacked_by_pawn  = pawn_attacked & pos.bb(them, PieceType::Rook);
    let queen_attacked_by_pawn = pawn_attacked & pos.bb(them, PieceType::Queen);

    score += popcount(minor_attacked_by_pawn) as i32 * PAWN_ATTACKS_MINOR;
    score += popcount(rook_attacked_by_pawn)  as i32 * PAWN_ATTACKS_ROOK;
    score += popcount(queen_attacked_by_pawn) as i32 * PAWN_ATTACKS_QUEEN;

    // ── Minor piece threats ───────────────────────────────────────────────
    let mut minor_attacks = 0u64;
    let mut knights = pos.bb(us, PieceType::Knight);
    while knights != 0 { let sq = pop_lsb(&mut knights); minor_attacks |= knight_attacks_sq(sq); }
    let mut bishops = pos.bb(us, PieceType::Bishop);
    while bishops != 0 { let sq = pop_lsb(&mut bishops); minor_attacks |= bishop_attacks(sq, occ); }

    let rook_attacked_by_minor  = minor_attacks & pos.bb(them, PieceType::Rook);
    let queen_attacked_by_minor = minor_attacks & pos.bb(them, PieceType::Queen);
    score += popcount(rook_attacked_by_minor)  as i32 * MINOR_ATTACKS_ROOK;
    score += popcount(queen_attacked_by_minor) as i32 * MINOR_ATTACKS_QUEEN;

    // ── Rook threats ──────────────────────────────────────────────────────
    let mut rook_attacks_bb = 0u64;
    let mut rooks = pos.bb(us, PieceType::Rook);
    while rooks != 0 { let sq = pop_lsb(&mut rooks); rook_attacks_bb |= rook_attacks(sq, occ); }
    let queen_attacked_by_rook = rook_attacks_bb & pos.bb(them, PieceType::Queen);
    score += popcount(queen_attacked_by_rook) as i32 * ROOK_ATTACKS_QUEEN;

    // ── Safe pawn push threats ────────────────────────────────────────────
    let empty = !occ;
    let safe_pushes = match us {
        Color::White => north(our_pawns) & empty & !their_pawn_attacks,
        Color::Black => south(our_pawns) & empty & !their_pawn_attacks,
    };
    let safe_push_attacks = match us {
        Color::White => white_pawn_attacks(safe_pushes),
        Color::Black => black_pawn_attacks(safe_pushes),
    };
    score += popcount(safe_push_attacks & their) as i32 * SAFE_PAWN_THREAT;

    score
}

fn is_attacked_by(pos: &Position, sq: u32, attacker: Color, occ: u64) -> bool {
    let pawns = pos.bb(attacker, PieceType::Pawn);
    if pawn_attacks(attacker.flip(), sq) & pawns != 0 { return true; }
    if knight_attacks_sq(sq) & pos.bb(attacker, PieceType::Knight) != 0 { return true; }
    if bishop_attacks(sq, occ) & (pos.bb(attacker, PieceType::Bishop) | pos.bb(attacker, PieceType::Queen)) != 0 { return true; }
    if rook_attacks(sq, occ) & (pos.bb(attacker, PieceType::Rook) | pos.bb(attacker, PieceType::Queen)) != 0 { return true; }
    if king_attacks_sq(sq) & pos.bb(attacker, PieceType::King) != 0 { return true; }
    false
}

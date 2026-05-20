/// Mobility evaluation — rewards pieces that have more legal moves available.
/// More mobility = more options = better position.

use crate::board::bitboard::*;
use crate::board::color::Color;
use crate::board::piece::PieceType;
use crate::board::position::Position;
use crate::eval::make_score;
use crate::movegen::attacks::*;

// ─── Mobility bonuses per move count ─────────────────────────────────────────

/// Bonus per mobility square for knights (0–8 moves possible).
const KNIGHT_MOBILITY: [i32; 9] = [
    make_score(-62,-81), make_score(-53,-56), make_score(-12,-30), make_score(-4,-14),
    make_score(  3,  8), make_score( 13, 15), make_score( 22, 23), make_score( 28, 27),
    make_score( 33, 33),
];

/// Bonus per mobility square for bishops (0–13 moves possible).
const BISHOP_MOBILITY: [i32; 14] = [
    make_score(-48,-59), make_score(-20,-23), make_score( 16,-3), make_score( 26, 13),
    make_score( 38, 24), make_score( 51, 42), make_score( 55, 54), make_score( 63, 57),
    make_score( 63, 65), make_score( 68, 73), make_score( 81, 78), make_score( 81, 86),
    make_score( 91, 88), make_score( 98, 97),
];

/// Bonus per mobility square for rooks (0–14 moves possible).
const ROOK_MOBILITY: [i32; 15] = [
    make_score(-60,-78), make_score(-20,-17), make_score(  2, 23), make_score(  3, 39),
    make_score(  3, 70), make_score( 11, 99), make_score( 22,103), make_score( 31,121),
    make_score( 40,134), make_score( 40,139), make_score( 41,158), make_score( 48,164),
    make_score( 57,168), make_score( 57,169), make_score( 62,172),
];

/// Bonus per mobility square for queens (0–27 moves possible).
const QUEEN_MOBILITY: [i32; 28] = [
    make_score(-30,-48), make_score(-12,-30), make_score( -8, -7), make_score( -9, 19),
    make_score( 20, 40), make_score( 23, 55), make_score( 23, 59), make_score( 35, 75),
    make_score( 38, 78), make_score( 53, 96), make_score( 64, 96), make_score( 65,100),
    make_score( 65,121), make_score( 66,127), make_score( 67,131), make_score( 67,133),
    make_score( 72,136), make_score( 72,141), make_score( 77,147), make_score( 79,150),
    make_score( 93,151), make_score(108,168), make_score(108,168), make_score(108,171),
    make_score(110,182), make_score(114,182), make_score(114,192), make_score(116,219),
];

// ─── Bonus for rook on open/semi-open file ────────────────────────────────────

const ROOK_OPEN_FILE:      i32 = make_score(47, 26);
const ROOK_SEMI_OPEN_FILE: i32 = make_score(19, 10);
const ROOK_ON_SEVENTH:     i32 = make_score(11, 48);

// ─── Main mobility evaluation ─────────────────────────────────────────────────

pub fn evaluate_mobility(pos: &Position) -> i32 {
    let mut score = 0i32;
    score += eval_mobility_for(pos, Color::White);
    score -= eval_mobility_for(pos, Color::Black);
    score
}

fn eval_mobility_for(pos: &Position, us: Color) -> i32 {
    let them = us.flip();
    let occ  = pos.occupancy;
    let our  = pos.color_bb(us);

    // Exclude squares attacked by enemy pawns from mobility area
    let their_pawns = pos.bb(them, PieceType::Pawn);
    let pawn_attacks_them = match them {
        Color::White => white_pawn_attacks(their_pawns),
        Color::Black => black_pawn_attacks(their_pawns),
    };
    // Mobility area: squares not occupied by our pieces and not attacked by enemy pawns
    let mobility_area = !our & !pawn_attacks_them;

    let mut score = 0i32;

    // ── Knights ───────────────────────────────────────────────────────────
    let mut knights = pos.bb(us, PieceType::Knight);
    while knights != 0 {
        let sq = pop_lsb(&mut knights);
        let mob = (knight_attacks_sq(sq) & mobility_area).count_ones() as usize;
        score += KNIGHT_MOBILITY[mob.min(8)];
    }

    // ── Bishops ───────────────────────────────────────────────────────────
    let mut bishops = pos.bb(us, PieceType::Bishop);
    while bishops != 0 {
        let sq = pop_lsb(&mut bishops);
        let mob = (bishop_attacks(sq, occ) & mobility_area).count_ones() as usize;
        score += BISHOP_MOBILITY[mob.min(13)];
    }

    // ── Rooks ─────────────────────────────────────────────────────────────
    let our_pawns   = pos.bb(us, PieceType::Pawn);
    let their_pawns_bb = pos.bb(them, PieceType::Pawn);
    let seventh_rank = if us == Color::White { RANK_7 } else { RANK_2 };

    let mut rooks = pos.bb(us, PieceType::Rook);
    while rooks != 0 {
        let sq = pop_lsb(&mut rooks);
        let attacks = rook_attacks(sq, occ);
        let mob = (attacks & mobility_area).count_ones() as usize;
        score += ROOK_MOBILITY[mob.min(14)];

        // Rook on open/semi-open file
        let file_mask = FILE_A << (sq % 8);
        if our_pawns & file_mask == 0 {
            if their_pawns_bb & file_mask == 0 {
                score += ROOK_OPEN_FILE;
            } else {
                score += ROOK_SEMI_OPEN_FILE;
            }
        }
        // Rook on 7th rank (trapping enemy king)
        if (1u64 << sq) & seventh_rank != 0 {
            score += ROOK_ON_SEVENTH;
        }
    }

    // ── Queens ────────────────────────────────────────────────────────────
    let mut queens = pos.bb(us, PieceType::Queen);
    while queens != 0 {
        let sq = pop_lsb(&mut queens);
        let mob = (queen_attacks(sq, occ) & mobility_area).count_ones() as usize;
        score += QUEEN_MOBILITY[mob.min(27)];
    }

    score
}

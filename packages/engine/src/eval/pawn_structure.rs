/// Pawn structure evaluation.
/// Covers: passed pawns, isolated pawns, doubled pawns, backward pawns.

use crate::board::bitboard::*;
use crate::board::color::Color;
use crate::board::piece::PieceType;
use crate::board::position::Position;
use crate::eval::make_score;

// ─── Bonuses / Penalties ─────────────────────────────────────────────────────

/// Passed pawn bonus by rank (rank 1–8, index 0–7). White perspective.
const PASSED_PAWN_BONUS: [i32; 8] = [
    make_score(  0,   0),
    make_score(  5,  10),
    make_score( 10,  20),
    make_score( 20,  40),
    make_score( 40,  70),
    make_score( 70, 110),
    make_score(110, 170),
    make_score(  0,   0),
];

const ISOLATED_PAWN_PENALTY: i32 = make_score(-15, -20);
const DOUBLED_PAWN_PENALTY:  i32 = make_score(-10, -20);
const BACKWARD_PAWN_PENALTY: i32 = make_score(-12, -10);

/// Phalanx bonus (pawns side by side on the same rank).
const PHALANX_BONUS: i32 = make_score(8, 5);

/// Connected pawn bonus (supported by another pawn diagonally behind).
const CONNECTED_BONUS: i32 = make_score(7, 8);

// ─── File masks ───────────────────────────────────────────────────────────────

/// Mask of adjacent files to a given file (0–7).
const fn adjacent_files(file: u8) -> u64 {
    let f = file as u64;
    let mut mask = 0u64;
    if file > 0 { mask |= FILE_A << (f - 1); }
    if file < 7 { mask |= FILE_A << (f + 1); }
    mask
}

/// Precomputed adjacent file masks.
const ADJACENT_FILE_MASKS: [u64; 8] = [
    adjacent_files(0), adjacent_files(1), adjacent_files(2), adjacent_files(3),
    adjacent_files(4), adjacent_files(5), adjacent_files(6), adjacent_files(7),
];

/// Mask of all squares in front of a pawn (for passed pawn detection).
/// For White on file f, rank r: all squares north of the pawn on files f-1, f, f+1.
fn passed_pawn_mask_white(sq: u32) -> u64 {
    let file = sq % 8;
    let rank = sq / 8;
    let mut mask = north_fill(1u64 << sq) & !( 1u64 << sq); // squares north on same file
    if file > 0 { mask |= north_fill(1u64 << (sq - 1 + 8)) }
    if file < 7 { mask |= north_fill(1u64 << (sq + 1 + 8)) }
    // Only ranks above the pawn
    let _rank_mask = !((1u64 << ((rank + 1) * 8)) - 1) | ((1u64 << (rank * 8)) - 1);
    mask
}

fn passed_pawn_mask_black(sq: u32) -> u64 {
    let file = sq % 8;
    let rank = sq / 8;
    let mut mask = south_fill(1u64 << sq) & !(1u64 << sq);
    if file > 0 && rank > 0 { mask |= south_fill(1u64 << (sq - 1 - 8)) }
    if file < 7 && rank > 0 { mask |= south_fill(1u64 << (sq + 1 - 8)) }
    mask
}

// ─── Main pawn evaluation ─────────────────────────────────────────────────────

pub fn evaluate_pawns(pos: &Position) -> i32 {
    let mut score = 0i32;

    let wp = pos.bb(Color::White, PieceType::Pawn);
    let bp = pos.bb(Color::Black, PieceType::Pawn);

    score += eval_pawns_for_color(pos, Color::White, wp, bp);
    score -= eval_pawns_for_color(pos, Color::Black, bp, wp);

    score
}

fn eval_pawns_for_color(_pos: &Position, us: Color, our_pawns: u64, their_pawns: u64) -> i32 {
    let mut score = 0i32;
    let mut pawns = our_pawns;

    while pawns != 0 {
        let sq = pop_lsb(&mut pawns);
        let file = (sq % 8) as usize;
        let rank = (sq / 8) as usize;
        let display_rank = if us == Color::White { rank } else { 7 - rank };

        // ── Passed pawn ────────────────────────────────────────────────────
        let pass_mask = if us == Color::White {
            passed_pawn_mask_white(sq)
        } else {
            passed_pawn_mask_black(sq)
        };
        let is_passed = their_pawns & pass_mask == 0;
        if is_passed {
            score += PASSED_PAWN_BONUS[display_rank];
        }

        // ── Isolated pawn ──────────────────────────────────────────────────
        let adj_files = ADJACENT_FILE_MASKS[file];
        let is_isolated = our_pawns & adj_files == 0;
        if is_isolated {
            score += ISOLATED_PAWN_PENALTY;
        }

        // ── Doubled pawn ───────────────────────────────────────────────────
        let file_mask = FILE_A << file;
        let doubled = (our_pawns & file_mask).count_ones() > 1;
        if doubled {
            score += DOUBLED_PAWN_PENALTY;
        }

        // ── Backward pawn ──────────────────────────────────────────────────
        if !is_isolated {
            let is_backward = is_backward_pawn(us, sq, our_pawns, their_pawns);
            if is_backward {
                score += BACKWARD_PAWN_PENALTY;
            }
        }

        // ── Phalanx (side-by-side pawns) ───────────────────────────────────
        let phalanx_mask = east(1u64 << sq) | west(1u64 << sq);
        if our_pawns & phalanx_mask != 0 {
            score += PHALANX_BONUS;
        }

        // ── Connected (supported diagonally from behind) ────────────────────
        let support_mask = if us == Color::White {
            south_west(1u64 << sq) | south_east(1u64 << sq)
        } else {
            north_west(1u64 << sq) | north_east(1u64 << sq)
        };
        if our_pawns & support_mask != 0 {
            score += CONNECTED_BONUS;
        }
    }

    score
}

fn is_backward_pawn(us: Color, sq: u32, our_pawns: u64, their_pawns: u64) -> bool {
    let file = sq % 8;
    let adj = ADJACENT_FILE_MASKS[file as usize];

    // A pawn is backward if it cannot safely advance because:
    // 1. No friendly pawns on adjacent files that are behind it
    // 2. The square in front is attacked by enemy pawns
    let (behind_mask, front_sq): (u64, u64) = if us == Color::White {
        let behind = south_fill(1u64 << sq) & adj;
        let front = 1u64 << (sq + 8);
        (behind, front)
    } else {
        if sq < 8 { return false; }
        let behind = north_fill(1u64 << sq) & adj;
        let front = 1u64 << (sq - 8);
        (behind, front)
    };

    let no_support = our_pawns & behind_mask == 0;
    let front_attacked = if us == Color::White {
        black_pawn_attacks(their_pawns) & front_sq != 0
    } else {
        white_pawn_attacks(their_pawns) & front_sq != 0
    };

    no_support && front_attacked
}

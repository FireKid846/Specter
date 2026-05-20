/// King safety evaluation.
/// Covers: pawn shelter, king attack zone, open files near king.

use crate::board::bitboard::*;
use crate::board::color::Color;
use crate::board::piece::PieceType;
use crate::board::position::Position;
use crate::eval::make_score;
use crate::movegen::attacks::*;

// ─── Attack weights by piece type ─────────────────────────────────────────────

/// How dangerous each attacker type is near the king.
const ATTACK_WEIGHT: [i32; 6] = [
    0,   // Pawn   — handled separately via pawn shelter
    20,  // Knight
    20,  // Bishop
    40,  // Rook
    80,  // Queen
    0,   // King
];

/// Bonus per attacker piece targeting the king zone.
const ATTACK_COUNT_BONUS: [i32; 8] = [0, 0, 50, 75, 88, 94, 97, 99];

// ─── Pawn shelter values ─────────────────────────────────────────────────────

/// Shelter bonus based on pawn position relative to king (rank distance).
/// Index: rank distance (0 = same rank, 3+ = far away).
const SHELTER_BONUS: [i32; 4] = [
    make_score(26, 0),
    make_score(18, 0),
    make_score(10, 0),
    make_score( 0, 0),
];

const OPEN_FILE_PENALTY:  i32 = make_score(-20, -10);
const SEMI_OPEN_PENALTY:  i32 = make_score(-10,  -5);
const STORM_PENALTY:      i32 = make_score(-15,   0);

// ─── Main king safety ─────────────────────────────────────────────────────────

pub fn evaluate_king_safety(pos: &Position, phase: i32) -> i32 {
    let mut score = 0i32;
    score += eval_king_side(pos, Color::White, phase);
    score -= eval_king_side(pos, Color::Black, phase);
    score
}

fn eval_king_side(pos: &Position, us: Color, phase: i32) -> i32 {
    let them = us.flip();
    let king_sq = pos.king_sq(us) as u32;
    let occ = pos.occupancy;

    // ── King attack zone ───────────────────────────────────────────────────
    // Zone = king square + adjacent squares + one rank forward
    let king_zone = king_attacks_sq(king_sq) | (1u64 << king_sq);

    let mut attacker_count = 0i32;
    let mut attack_value   = 0i32;

    for pt in [PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen] {
        let mut attackers = pos.bb(them, pt);
        while attackers != 0 {
            let sq = pop_lsb(&mut attackers);
            let attacks = match pt {
                PieceType::Knight => knight_attacks_sq(sq),
                PieceType::Bishop => bishop_attacks(sq, occ),
                PieceType::Rook   => rook_attacks(sq, occ),
                PieceType::Queen  => queen_attacks(sq, occ),
                _ => 0,
            };
            if attacks & king_zone != 0 {
                attacker_count += 1;
                attack_value += ATTACK_WEIGHT[pt.index()];
            }
        }
    }

    // Scale by number of attackers
    let attack_score = attack_value
        * ATTACK_COUNT_BONUS[attacker_count.min(7) as usize]
        / 100;

    // ── Pawn shelter ───────────────────────────────────────────────────────
    let shelter_score = evaluate_shelter(pos, us, king_sq);

    // ── Open files near king ───────────────────────────────────────────────
    let open_score = evaluate_open_files_near_king(pos, us, king_sq);

    // King safety matters more in the middlegame — scale with phase
    let safety = make_score(attack_score, attack_score / 4);

    shelter_score + open_score - safety
}

fn evaluate_shelter(pos: &Position, us: Color, king_sq: u32) -> i32 {
    let mut score = 0i32;
    let king_file = king_sq % 8;
    let king_rank = king_sq / 8;
    let our_pawns = pos.bb(us, PieceType::Pawn);

    // Check 3 files around the king
    for df in 0i32..=2 {
        let f = king_file as i32 - 1 + df;
        if f < 0 || f > 7 { continue; }
        let file_mask = FILE_A << f;
        let pawns_on_file = our_pawns & file_mask;

        if pawns_on_file == 0 {
            // No shelter pawn on this file
            score += make_score(-15, 0);
            continue;
        }

        // Find closest pawn to king
        let closest = if us == Color::White {
            // Closest pawn is the one with the smallest rank above king
            let above = pawns_on_file >> ((king_rank + 1) * 8);
            if above != 0 { lsb(above) } else { msb(pawns_on_file) }
        } else {
            let below = pawns_on_file & ((1u64 << (king_rank * 8)) - 1);
            if below != 0 { msb(below) } else { lsb(pawns_on_file) }
        };

        let pawn_rank = closest / 8;
        let dist = (king_rank as i32 - pawn_rank as i32).unsigned_abs() as usize;
        score += SHELTER_BONUS[dist.min(3)];
    }

    score
}

fn evaluate_open_files_near_king(pos: &Position, us: Color, king_sq: u32) -> i32 {
    let mut score = 0i32;
    let king_file = king_sq % 8;
    let our_pawns   = pos.bb(us, PieceType::Pawn);
    let their_pawns = pos.bb(us.flip(), PieceType::Pawn);

    for df in 0i32..=2 {
        let f = king_file as i32 - 1 + df;
        if f < 0 || f > 7 { continue; }
        let file_mask = FILE_A << f;
        let our_on_file   = our_pawns   & file_mask;
        let their_on_file = their_pawns & file_mask;

        if our_on_file == 0 && their_on_file == 0 {
            score += OPEN_FILE_PENALTY;
        } else if our_on_file == 0 {
            score += SEMI_OPEN_PENALTY;
        }

        // Pawn storm: enemy pawn close to our king
        if their_on_file != 0 {
            let pawn_sq = if us == Color::White { lsb(their_on_file) } else { msb(their_on_file) };
            let pawn_rank = pawn_sq / 8;
            let king_rank = king_sq / 8;
            let dist = (king_rank as i32 - pawn_rank as i32).unsigned_abs();
            if dist <= 2 {
                score += STORM_PENALTY;
            }
        }
    }

    score
}

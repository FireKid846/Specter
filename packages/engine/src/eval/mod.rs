/// Specter Engine — Evaluation
///
/// Uses NNUE evaluation when a trained network is embedded (feature = "nnue").
/// Falls back to tapered hand-crafted evaluation when no network is present.
///
/// Hand-crafted eval uses tapered evaluation: scores are blended between
/// middlegame (MG) and endgame (EG) based on remaining material (game phase).
///
/// Score packing: a single i32 packs both MG and EG values:
///   bits  0–15: middlegame score (i16)
///   bits 16–31: endgame score   (i16)

pub mod king_safety;
pub mod material;
pub mod mobility;
pub mod nnue;
pub mod pawn_structure;
pub mod pst;
pub mod threats;

use crate::board::color::Color;
use crate::board::piece::PieceType;
use crate::board::position::Position;
use crate::eval::king_safety::evaluate_king_safety;
use crate::eval::material::evaluate_material;
use crate::eval::mobility::evaluate_mobility;
use crate::eval::pawn_structure::evaluate_pawns;
use crate::eval::pst::evaluate_pst;
use crate::eval::threats::evaluate_threats;

// ─── Score packing ────────────────────────────────────────────────────────────

#[inline(always)]
pub const fn make_score(mg: i32, eg: i32) -> i32 {
    (eg << 16) + mg
}

#[inline(always)]
pub fn mg(s: i32) -> i32 { (s as i16) as i32 }

#[inline(always)]
pub fn eg(s: i32) -> i32 { (s + 0x8000) >> 16 }

// ─── Game phase ───────────────────────────────────────────────────────────────

const PHASE_KNIGHT: i32 = 1;
const PHASE_BISHOP: i32 = 1;
const PHASE_ROOK:   i32 = 2;
const PHASE_QUEEN:  i32 = 4;
pub const MAX_PHASE: i32 = 4 + 4 + 8 + 8; // 24

pub fn game_phase(pos: &Position) -> i32 {
    let mut phase = 0i32;
    for color in [Color::White, Color::Black] {
        phase += pos.bb(color, PieceType::Knight).count_ones() as i32 * PHASE_KNIGHT;
        phase += pos.bb(color, PieceType::Bishop).count_ones() as i32 * PHASE_BISHOP;
        phase += pos.bb(color, PieceType::Rook  ).count_ones() as i32 * PHASE_ROOK;
        phase += pos.bb(color, PieceType::Queen ).count_ones() as i32 * PHASE_QUEEN;
    }
    phase.min(MAX_PHASE)
}

#[inline(always)]
pub fn taper(score: i32, phase: i32) -> i32 {
    (mg(score) * phase + eg(score) * (MAX_PHASE - phase)) / MAX_PHASE
}

// ─── Score constants ──────────────────────────────────────────────────────────

pub const SCORE_INFINITE: i32 =  32000;
pub const SCORE_MATE:     i32 =  31000;
pub const SCORE_MATED:    i32 = -31000;
pub const SCORE_DRAW:     i32 =  0;
pub const SCORE_NONE:     i32 = -32001;

pub fn mate_in(score: i32) -> i32 { (SCORE_MATE - score.abs() + 1) / 2 }
pub fn is_mate_score(score: i32) -> bool { score.abs() >= SCORE_MATE - 200 }

// ─── Hand-crafted evaluation ──────────────────────────────────────────────────

const TEMPO: i32 = 15;

/// Full hand-crafted evaluation. Used as fallback when NNUE is not active.
/// Returns score in centipawns from side-to-move perspective.
pub fn evaluate_hce(pos: &Position) -> i32 {
    let phase = game_phase(pos);
    let mut score = 0i32;
    score += evaluate_material(pos);
    score += evaluate_pst(pos, phase);
    score += evaluate_pawns(pos);
    score += evaluate_king_safety(pos, phase);
    score += evaluate_mobility(pos);
    score += evaluate_threats(pos);
    let mut result = taper(score, phase);
    result += TEMPO;
    result * pos.side.sign()
}

// ─── Main evaluation entry point ─────────────────────────────────────────────

/// Evaluate the current position.
///
/// When `nnue_eval` is Some and active, uses NNUE.
/// Otherwise falls back to hand-crafted eval.
///
/// Returns score in centipawns from side-to-move perspective.
pub fn evaluate(pos: &Position) -> i32 {
    // Hand-crafted fallback — used when no NnueEval is threaded through search.
    // The search passes NnueEval through SearchState when available.
    evaluate_hce(pos)
}

/// Evaluate using NNUE if available, hand-crafted otherwise.
/// This is the preferred call site from inside the search.
pub fn evaluate_with_nnue(
    pos:  &Position,
    nnue: &crate::eval::nnue::NnueEval,
) -> i32 {
    if nnue.is_active() {
        nnue.evaluate(pos).unwrap_or_else(|| evaluate_hce(pos))
    } else {
        evaluate_hce(pos)
    }
}

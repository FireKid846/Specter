/// Specter Engine — Evaluation
///
/// Uses tapered evaluation: scores are blended between middlegame (MG)
/// and endgame (EG) based on remaining material (game phase).
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

// ─── Main evaluation ──────────────────────────────────────────────────────────

const TEMPO: i32 = 15;

pub fn evaluate(pos: &Position) -> i32 {
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
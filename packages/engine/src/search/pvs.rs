/// Principal Variation Search (PVS) — the core search algorithm.
///
/// PVS is a refinement of alpha-beta: after searching the first move with a
/// full window, subsequent moves are searched with a null window [alpha, alpha+1].
/// If they fail high, we re-search with a full window.
/// This is much faster than plain alpha-beta because most moves will fail low
/// with the null window and never need re-searching.

use crate::board::position::{Move, Position};
use crate::eval::{evaluate, is_mate_score, SCORE_DRAW, SCORE_MATED, SCORE_NONE};
use crate::history::butterfly::ButterflyHistory;
use crate::history::capture::CaptureHistory;
use crate::history::continuation::ContinuationHistory;
use crate::history::correction::CorrectionHistory;
use crate::history::killer::KillerTable;
use crate::movegen::legal::{is_in_check, legal_moves};
use crate::search::nmp::try_null_move;
use crate::search::quiescence::quiescence;
use crate::search::singular::try_singular_extension;
use crate::search::lmr::lmr_reduction;
use crate::search::timeman::TimeManager;
use crate::tt::entry::Bound;
use crate::tt::table::TranspositionTable;

pub const MAX_PLY: usize = 246;

/// Per-search state shared across all recursive calls.
pub struct SearchState<'a> {
    pub tt:           &'a mut TranspositionTable,
    pub killers:      &'a mut KillerTable,
    pub butterfly:    &'a mut ButterflyHistory,
    pub capture_hist: &'a mut CaptureHistory,
    pub cont_hist:    &'a mut ContinuationHistory,
    pub corr_hist:    &'a mut CorrectionHistory,
    pub time:         &'a TimeManager,
    pub nodes:        u64,
    pub seldepth:     u32,
    pub pv:           [[Move; MAX_PLY]; MAX_PLY],
    pub pv_length:    [usize; MAX_PLY],
    /// Stack of moves made at each ply (for continuation history).
    pub move_stack:   [Move; MAX_PLY],
}

impl<'a> SearchState<'a> {
    pub fn update_pv(&mut self, ply: usize, mv: Move) {
        self.pv[ply][0] = mv;
        let next_len = self.pv_length[ply + 1];
        for i in 0..next_len {
            self.pv[ply][i + 1] = self.pv[ply + 1][i];
        }
        self.pv_length[ply] = next_len + 1;
    }

    pub fn get_pv(&self) -> Vec<Move> {
        self.pv[0][..self.pv_length[0]].to_vec()
    }
}

/// Main PVS search function.
/// Returns a score in centipawns from the current side's perspective.
pub fn pvs(
    pos:   &mut Position,
    state: &mut SearchState,
    mut alpha: i32,
    beta:  i32,
    depth: i32,
    ply:   usize,
    is_pv: bool,
    cut_node: bool,
) -> i32 {
    // ── Terminal conditions ────────────────────────────────────────────────
    if ply >= MAX_PLY - 1 { return evaluate(pos); }

    state.pv_length[ply] = 0;

    // Draw detection
    if ply > 0 && (pos.is_repetition() || pos.is_fifty_move_draw() || pos.is_insufficient_material()) {
        return SCORE_DRAW;
    }

    // Drop into quiescence search at depth 0
    if depth <= 0 {
        return quiescence(pos, state, alpha, beta, ply);
    }

    state.nodes += 1;
    if ply > state.seldepth as usize { state.seldepth = ply as u32; }

    // Time check
    if state.time.should_check(state.nodes) && state.time.is_hard_expired() {
        return alpha;
    }

    let in_check = is_in_check(pos, pos.side);
    let is_root  = ply == 0;

    // ── Transposition table probe ──────────────────────────────────────────
    let tt_hit = state.tt.probe(pos.hash);
    let mut tt_move = Move::NULL;

    if let Some(entry) = tt_hit {
        tt_move = entry.mv;
        // Use TT score if not PV node and search was deep enough
        if !is_pv && entry.depth as i32 >= depth {
            let score = entry.score;
            match entry.bound {
                Bound::Exact                     => return score,
                Bound::Lower if score >= beta    => return score,
                Bound::Upper if score <= alpha   => return score,
                _ => {}
            }
        }
    }

    // ── Static evaluation ─────────────────────────────────────────────────
    let static_eval = if in_check {
        SCORE_NONE // Can't evaluate in check
    } else {
        let raw = evaluate(pos);
        // Apply correction history
        let corr = state.corr_hist.get(pos.side.index(), pos.hash);
        raw + corr / 128
    };

    let improving = !in_check && ply >= 2 && static_eval > evaluate(pos);

    // ── Pruning (skip in check, PV, or root) ──────────────────────────────
    if !in_check && !is_pv {
        // ── Reverse Futility Pruning (RFP) ────────────────────────────────
        // If static eval is well above beta, we can prune.
        let rfp_margin = 120 * depth;
        if depth < 9 && static_eval - rfp_margin >= beta {
            return static_eval;
        }

        // ── Null Move Pruning (NMP) ────────────────────────────────────────
        if let Some(score) = try_null_move(pos, state, beta, depth, ply, static_eval) {
            return score;
        }

        // ── Razoring ──────────────────────────────────────────────────────
        if depth <= 2 && static_eval + 300 * depth < alpha {
            let q = quiescence(pos, state, alpha, beta, ply);
            if q < alpha { return q; }
        }
    }

    // ── Move loop ─────────────────────────────────────────────────────────
    let moves = legal_moves(pos);
    if moves.is_empty() {
        return if in_check { SCORE_MATED + ply as i32 } else { SCORE_DRAW };
    }

    // Sort moves (move picker handles ordering — simplified here for initial version)
    let mut scored_moves: Vec<(Move, i32)> = moves.as_slice()
        .iter()
        .map(|&mv| {
            let score = score_move(pos, state, mv, tt_move, ply);
            (mv, score)
        })
        .collect();
    scored_moves.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    let mut best_score = SCORE_MATED + ply as i32;
    let mut best_move  = Move::NULL;
    let mut moves_searched = 0;

    for (mv, _) in &scored_moves {
        let mv = *mv;
        let is_capture   = pos.piece_on(mv.to()).is_some();
        let is_promotion = mv.is_promotion();
        let gives_check  = {
            let state_bak = pos.make_move(mv);
            let ch = is_in_check(pos, pos.side);
            pos.unmake_move(mv, state_bak);
            ch
        };

        // ── Extensions ────────────────────────────────────────────────────
        let mut extension = 0;
        if in_check { extension = 1; }  // Check extension

        // Singular extension
        if !is_root
            && depth >= 8
            && mv == tt_move
            && !is_mate_score(tt_hit.map_or(SCORE_NONE, |e| e.score))
        {
            if let Some(ext) = try_singular_extension(pos, state, mv, alpha, beta, depth, ply) {
                extension = ext;
            }
        }

        let new_depth = depth - 1 + extension;

        // ── Futility pruning ──────────────────────────────────────────────
        if !is_pv && !in_check && !is_capture && !gives_check && moves_searched > 0 {
            let futility_margin = 80 + 60 * new_depth;
            if depth <= 8 && static_eval + futility_margin <= alpha {
                continue;
            }
            // Late move pruning — skip late quiet moves at low depths
            let lmp_threshold = if improving { 4 + depth * depth } else { 2 + depth * depth / 2 };
            if depth <= 5 && moves_searched >= lmp_threshold { continue; }
        }

        // ── Make move ─────────────────────────────────────────────────────
        state.move_stack[ply] = mv;
        let unmake = pos.make_move(mv);
        moves_searched += 1;

        // ── LMR — Late Move Reductions ────────────────────────────────────
        let score = if moves_searched == 1 {
            // First move: full window search
            -pvs(pos, state, -beta, -alpha, new_depth, ply + 1, is_pv, false)
        } else {
            // Subsequent moves: try null window with possible reduction
            let reduction = if !is_capture && !gives_check && !in_check && moves_searched > 2 {
                lmr_reduction(depth, moves_searched as i32)
            } else {
                0
            };

            let reduced_depth = (new_depth - reduction).max(1);
            let null_score = -pvs(pos, state, -alpha - 1, -alpha, reduced_depth, ply + 1, false, true);

            // Re-search if promising
            if null_score > alpha && (reduction > 0 || !is_pv) {
                -pvs(pos, state, -alpha - 1, -alpha, new_depth, ply + 1, false, !cut_node)
            } else if is_pv && null_score > alpha {
                -pvs(pos, state, -beta, -alpha, new_depth, ply + 1, true, false)
            } else {
                null_score
            }
        };

        pos.unmake_move(mv, unmake);

        // Stop if time expired
        if state.time.should_check(state.nodes) && state.time.is_hard_expired() {
            return alpha;
        }

        // ── Update best ───────────────────────────────────────────────────
        if score > best_score {
            best_score = score;
            best_move  = mv;

            if score > alpha {
                alpha = score;
                if is_pv { state.update_pv(ply, mv); }

                if score >= beta {
                    // Beta cutoff — update histories
                    if !is_capture {
                        state.killers.store(ply, mv);
                        let bonus = history_bonus(depth);
                        state.butterfly.update(pos.side.index(), mv.from().index(), mv.to().index(), bonus);
                        // Penalize quiets that didn't cause cutoff
                        for &(other_mv, _) in scored_moves.iter().take((moves_searched - 1) as usize) {
                            if pos.piece_on(other_mv.to()).is_none() {
                                state.butterfly.update(pos.side.index(), other_mv.from().index(), other_mv.to().index(), -bonus);
                            }
                        }
                    }
                    break;
                }
            }
        }
    }

    // ── Store in TT ───────────────────────────────────────────────────────
    let bound = if best_score >= beta {
        Bound::Lower
    } else if is_pv && best_move != Move::NULL {
        Bound::Exact
    } else {
        Bound::Upper
    };

    state.tt.store(pos.hash, best_move, best_score, depth as u8, bound);

    best_score
}

/// Score a move for ordering (higher = try first).
fn score_move(
    pos:     &Position,
    state:   &SearchState,
    mv:      Move,
    tt_move: Move,
    ply:     usize,
) -> i32 {
    use crate::eval::material::piece_value_simple;
    use crate::board::piece::PieceType;

    if mv == tt_move { return 2_000_000; }

    let captured = pos.piece_on(mv.to());

    if let Some(cap) = captured {
        // MVV-LVA: Most Valuable Victim - Least Valuable Aggressor
        let victim_val   = piece_value_simple(cap.piece_type);
        let _aggressor_pt = pos.piece_type_on(mv.from(), pos.side).unwrap_or(PieceType::Pawn);
        let aggressor_val = piece_value_simple(aggressor_pt);
        return 1_000_000 + victim_val * 10 - aggressor_val;
    }

    if mv.is_promotion() { return 900_000; }

    // Killer moves
    let killers = state.killers.get(ply);
    if mv == killers[0] { return 800_000; }
    if mv == killers[1] { return 700_000; }

    // Butterfly history
    let _aggressor_pt = pos.piece_type_on(mv.from(), pos.side).unwrap_or(PieceType::Pawn);
    state.butterfly.get(pos.side.index(), mv.from().index(), mv.to().index())
}

/// History bonus/malus scaled by depth.
fn history_bonus(depth: i32) -> i32 {
    (300 * depth - 300).min(2800)
}

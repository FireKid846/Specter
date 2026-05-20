/// Quiescence search — extends search through captures and promotions
/// to avoid the horizon effect (stopping in a tactically unstable position).

use crate::board::position::Position;
use crate::eval::{evaluate, SCORE_MATED};
use crate::movegen::legal::{is_in_check, legal_captures, legal_moves};
use crate::search::pvs::SearchState;
use crate::eval::material::piece_value_simple;

/// Delta pruning margin (don't bother if even queen can't save us).
const DELTA_MARGIN: i32 = 975;

pub fn quiescence(
    pos:   &mut Position,
    state: &mut SearchState,
    mut alpha: i32,
    beta:  i32,
    ply:   usize,
) -> i32 {
    state.nodes += 1;

    if state.time.should_check(state.nodes) && state.time.is_hard_expired() {
        return alpha;
    }

    // Draw check
    if pos.is_repetition() || pos.is_fifty_move_draw() || pos.is_insufficient_material() {
        return 0;
    }

    let in_check = is_in_check(pos, pos.side);

    // Stand-pat: the current position without any capture might be good enough
    let stand_pat = evaluate(pos);

    if !in_check {
        if stand_pat >= beta { return stand_pat; } // Beta cutoff
        if stand_pat > alpha { alpha = stand_pat; }
    }

    // Generate captures (and quiet evasions when in check)
    let captures = legal_captures(pos);

    if captures.is_empty() {
        if in_check {
            // No captures available — check for quiet evasions before declaring mate.
            // Quiet moves (king steps, blocks) may still escape check.
            let all_moves = legal_moves(pos);
            if all_moves.is_empty() {
                return SCORE_MATED + ply as i32; // genuine checkmate
            }
            // Has quiet evasion — return stand_pat (we won't search quiet moves in qsearch)
            return stand_pat;
        }
        return stand_pat;
    }

    let mut scored: Vec<_> = captures.as_slice().iter().map(|&mv| {
        let cap = pos.piece_on(mv.to());
        let val = cap.map(|p| piece_value_simple(p.piece_type)).unwrap_or(0);
        (mv, val)
    }).collect();
    scored.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    let mut best = stand_pat;

    for (mv, cap_val) in &scored {
        let mv = *mv;

        // Delta pruning: skip captures that can't raise alpha
        if !in_check && stand_pat + cap_val + DELTA_MARGIN < alpha {
            continue;
        }

        let unmake = pos.make_move(mv);
        let score  = -quiescence(pos, state, -beta, -alpha, ply + 1);
        pos.unmake_move(mv, unmake);

        if score > best {
            best = score;
            if score > alpha {
                alpha = score;
                if score >= beta { break; }
            }
        }
    }

    best
}

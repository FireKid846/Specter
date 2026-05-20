/// Null Move Pruning (NMP) — if we can skip our move and still beat beta,
/// the position is so good that we can prune this branch.

use crate::board::position::Position;
use crate::eval::material::non_pawn_material;
use crate::search::pvs::{pvs, SearchState};

/// Returns Some(score) if null move pruning applies, None otherwise.
pub fn try_null_move(
    pos:         &mut Position,
    state:       &mut SearchState,
    beta:        i32,
    depth:       i32,
    ply:         usize,
    static_eval: i32,
) -> Option<i32> {
    // Don't try NMP if: in check, no major pieces, or at low depth
    if depth < 3 { return None; }
    if non_pawn_material(pos, pos.side) == 0 { return None; }
    if static_eval < beta { return None; }

    let reduction = 3 + depth / 4 + ((static_eval - beta) / 200).min(3);
    let new_depth = (depth - reduction).max(1);

    let null_state = pos.make_null_move();
    let score = -pvs(pos, state, -beta, -beta + 1, new_depth, ply + 1, false, false);
    pos.unmake_null_move(null_state);

    if score >= beta {
        Some(score.min(30000)) // Don't return mate scores from NMP
    } else {
        None
    }
}

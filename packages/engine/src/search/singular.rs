/// Singular Extension — if a move appears to be "singularly" better than all others,
/// extend the search depth for that move.

use crate::board::position::{Move, Position};
use crate::search::pvs::{pvs, SearchState};
use crate::tt::entry::Bound;

/// Returns Some(extension) if singular extension applies, None otherwise.
pub fn try_singular_extension(
    pos:   &mut Position,
    state: &mut SearchState,
    mv:    Move,
    _alpha: i32,
    beta:  i32,
    depth: i32,
    ply:   usize,
) -> Option<i32> {
    let tt_entry = state.tt.probe(pos.hash)?;
    if tt_entry.bound == Bound::Upper { return None; }
    if (tt_entry.depth as i32) < depth - 3 { return None; }

    let s_beta  = (tt_entry.score - depth * 2).max(-30000);
    let s_depth = (depth - 1) / 2;

    // Exclude the TT move so the sub-search cannot use it.
    // This lets us check: is this move singularly better than everything else?
    state.excl_move = mv;
    let excl_score = pvs(pos, state, s_beta - 1, s_beta, s_depth, ply, false, false);
    state.excl_move = Move::NULL;

    if excl_score < s_beta {
        // TT move is singular — extend it
        Some(1 + (excl_score < s_beta - 50) as i32)
    } else if s_beta >= beta {
        // Multi-cut — prune
        Some(-1) // Negative extension = signal to prune
    } else {
        None
    }
}

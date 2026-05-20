/// Iterative deepening — searches increasing depths (1, 2, 3, ...) until
/// time or depth limit is reached. Uses the previous iteration's result to
/// order moves better in the next iteration.
///
/// Also handles aspiration windows: search with a narrow window first.
/// If it fails, widen and re-search.

use crate::board::position::{Move, Position};
use crate::eval::{SCORE_INFINITE, is_mate_score, mate_in};
use crate::history::butterfly::ButterflyHistory;
use crate::history::capture::CaptureHistory;
use crate::history::continuation::ContinuationHistory;
use crate::history::correction::CorrectionHistory;
use crate::history::killer::KillerTable;
use crate::movegen::attacks::init_all;
use crate::search::lmr::init_lmr;
use crate::search::pvs::{pvs, SearchState, MAX_PLY};
use crate::search::timeman::TimeManager;
use crate::tt::table::TranspositionTable;

/// The result of a completed search.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move:  Move,
    pub score:      i32,
    pub depth:      u32,
    pub seldepth:   u32,
    pub nodes:      u64,
    pub time_ms:    u64,
    pub pv:         Vec<Move>,
    pub nps:        u64,
    pub hashfull:   u32,
}

/// A callback invoked after each depth iteration with search info.
pub type InfoCallback = Box<dyn Fn(&SearchResult)>;

/// Main search entry point. Performs iterative deepening with aspiration windows.
pub fn search(
    pos:      &mut Position,
    tt:       &mut TranspositionTable,
    time:     TimeManager,
    callback: Option<InfoCallback>,
) -> SearchResult {
    // Initialize attack tables and LMR table
    init_all();
    init_lmr();

    let mut killers   = KillerTable::new();
    let mut butterfly = ButterflyHistory::new();
    let mut cap_hist  = CaptureHistory::new();
    let mut cont_hist = ContinuationHistory::new();
    let mut corr_hist = CorrectionHistory::new();

    tt.new_search();

    let mut state = SearchState {
        tt,
        killers:      &mut killers,
        butterfly:    &mut butterfly,
        capture_hist: &mut cap_hist,
        cont_hist:    &mut cont_hist,
        corr_hist:    &mut corr_hist,
        time:         &time,
        nodes:        0,
        seldepth:     0,
        pv:           [[Move::NULL; MAX_PLY]; MAX_PLY],
        pv_length:    [0; MAX_PLY],
        move_stack:   [Move::NULL; MAX_PLY],
    };

    let mut best_move  = Move::NULL;
    let mut best_score = -SCORE_INFINITE;
    let mut result     = SearchResult {
        best_move:  Move::NULL,
        score:      0,
        depth:      0,
        seldepth:   0,
        nodes:      0,
        time_ms:    0,
        pv:         vec![],
        nps:        0,
        hashfull:   0,
    };

    let max_depth = if time.max_depth > 0 { time.max_depth } else { 100 };

    // ── Aspiration window parameters ──────────────────────────────────────
    let mut asp_delta = 25i32;

    for depth in 1..=max_depth {
        if time.depth_limit_reached(depth) { break; }
        if depth > 1 && time.is_soft_expired() { break; }

        state.seldepth = 0;

        // ── Aspiration windows ────────────────────────────────────────────
        let (mut alpha, mut beta) = if depth >= 4 {
            (best_score - asp_delta, best_score + asp_delta)
        } else {
            (-SCORE_INFINITE, SCORE_INFINITE)
        };

        let score = loop {
            let s = pvs(pos, &mut state, alpha, beta, depth as i32, 0, true, false);

            if time.is_hard_expired() { break s; }

            if s <= alpha {
                // Failed low — widen lower bound
                alpha = (s - asp_delta).max(-SCORE_INFINITE);
                asp_delta *= 2;
            } else if s >= beta {
                // Failed high — widen upper bound
                beta = (s + asp_delta).min(SCORE_INFINITE);
                asp_delta *= 2;
            } else {
                asp_delta = 25; // Reset delta
                break s;
            }
        };

        if time.is_hard_expired() { break; }

        best_score = score;
        let pv = state.get_pv();
        if !pv.is_empty() {
            best_move = pv[0];
        }

        let elapsed_ms = time.elapsed_ms();
        let nps = if elapsed_ms > 0 { state.nodes * 1000 / elapsed_ms } else { 0 };

        result = SearchResult {
            best_move,
            score:     best_score,
            depth,
            seldepth:  state.seldepth,
            nodes:     state.nodes,
            time_ms:   elapsed_ms,
            pv:        pv.clone(),
            nps,
            hashfull:  state.tt.hashfull(),
        };

        if let Some(ref cb) = callback {
            cb(&result);
        }

        // If we found mate, no need to search deeper
        if is_mate_score(best_score) { break; }
    }

    result
}

/// Formats a search result as a UCI info string.
pub fn format_info(r: &SearchResult) -> String {
    let score_str = if is_mate_score(r.score) {
        format!("mate {}", mate_in(r.score))
    } else {
        format!("cp {}", r.score)
    };

    let pv_str: Vec<String> = r.pv.iter().map(|m| m.to_uci()).collect();

    format!(
        "info depth {} seldepth {} score {} nodes {} time {} nps {} hashfull {} pv {}",
        r.depth, r.seldepth, score_str, r.nodes, r.time_ms, r.nps, r.hashfull,
        pv_str.join(" ")
    )
}

/// Syzygy probe interface.
/// Full implementation uses the Fathom C library via FFI (Phase 2).
/// This module provides the interface — FFI bindings added when Fathom is linked.

use crate::board::position::Position;
use crate::syzygy::{SyzygyResult, WdlScore};

/// Initialize Syzygy tablebases from a directory path.
/// Returns the maximum number of pieces supported.
pub fn init_syzygy(path: &str) -> Result<u32, String> {
    // Phase 2: call tb_init() from Fathom via FFI
    // For now, return 0 (disabled)
    Ok(0)
}

/// Probe Win/Draw/Loss for a position.
pub fn probe_wdl(pos: &Position) -> Option<WdlScore> {
    // Phase 2: call tb_probe_wdl() from Fathom
    None
}

/// Probe Distance-To-Zero for a position.
pub fn probe_dtz(pos: &Position) -> Option<WdlScore> {
    // Phase 2: call tb_probe_root() from Fathom
    None
}

/// Convert a Fathom WDL value to SyzygyResult.
fn fathom_to_result(wdl: i32) -> SyzygyResult {
    match wdl {
        2  => SyzygyResult::Win,
        0  => SyzygyResult::Draw,
        -2 => SyzygyResult::Loss,
        _  => SyzygyResult::Unknown,
    }
}

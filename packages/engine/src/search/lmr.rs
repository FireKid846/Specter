/// Late Move Reductions (LMR) — reduce depth for moves unlikely to be best.
/// Later moves in the sorted list are probably not as good, so we search them less deeply.

/// Precomputed LMR reduction table: [depth][move_number]
static mut LMR_TABLE: [[i32; 64]; 64] = [[0; 64]; 64];
static mut LMR_INITIALIZED: bool = false;

pub fn init_lmr() {
    unsafe {
        if LMR_INITIALIZED { return; }
        for depth in 1..64usize {
            for moves in 1..64usize {
                let r = (0.75 + (depth as f64).ln() * (moves as f64).ln() / 2.25) as i32;
                LMR_TABLE[depth][moves] = r.max(0);
            }
        }
        LMR_INITIALIZED = true;
    }
}

/// Returns the reduction amount for a given depth and move index.
#[inline(always)]
pub fn lmr_reduction(depth: i32, moves: i32) -> i32 {
    unsafe {
        init_lmr();
        let d = (depth as usize).min(63);
        let m = (moves as usize).min(63);
        LMR_TABLE[d][m]
    }
}

/// Lazy SMP — parallel search using multiple threads.
/// All threads share the transposition table and search the same tree independently.
/// The thread with the most nodes reports the best move.
/// Note: WASM build is single-threaded. Lazy SMP only active in CLI binary.
///
/// Phase 2 implementation — placeholder for now.
pub struct LazySmp {
    pub num_threads: usize,
}

impl LazySmp {
    pub fn new(num_threads: usize) -> Self {
        LazySmp { num_threads: num_threads.max(1) }
    }
}

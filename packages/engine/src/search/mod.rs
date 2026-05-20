pub mod iterative_deepening;
pub mod lazy_smp;
pub mod lmr;
pub mod nmp;
pub mod pvs;
pub mod quiescence;
pub mod singular;
pub mod timeman;

pub use iterative_deepening::search;
pub use timeman::TimeManager;

/// Maximum search depth.
pub const MAX_DEPTH: usize = 128;
/// Maximum ply (search tree depth including extensions).
pub const MAX_PLY: usize = 246;

/// Score returned for a draw.
pub const DRAW_SCORE: i32 = 0;

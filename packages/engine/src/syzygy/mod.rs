pub mod probe;
pub mod tbprobe;

use crate::board::position::Position;

/// Result of a Syzygy tablebase probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyzygyResult {
    Win,
    Loss,
    Draw,
    Unknown,
}

/// WDL (Win/Draw/Loss) probe result with distance to zero.
#[derive(Debug, Clone, Copy)]
pub struct WdlScore {
    pub result: SyzygyResult,
    pub dtm:    Option<i32>, // Distance to mate (if known)
    pub dtz:    Option<i32>, // Distance to zeroing move (50-move rule)
}

pub struct SyzygyProber {
    pub enabled:    bool,
    pub max_pieces: u32,    // Max pieces supported by loaded TBs
    pub path:       String,
}

impl SyzygyProber {
    pub fn new() -> Self {
        SyzygyProber { enabled: false, max_pieces: 0, path: String::new() }
    }

    /// Initialize Syzygy with a path to .rtbw and .rtbz files.
    pub fn init(&mut self, path: &str) -> Result<(), String> {
        self.path       = path.to_string();
        self.max_pieces = probe::init_syzygy(path)?;
        self.enabled    = self.max_pieces > 0;
        Ok(())
    }

    /// Probe WDL (Win/Draw/Loss) for a position.
    /// Only valid when piece count <= max_pieces.
    pub fn probe_wdl(&self, pos: &Position) -> Option<WdlScore> {
        if !self.enabled { return None; }
        let piece_count = pos.occupancy.count_ones();
        if piece_count > self.max_pieces { return None; }
        probe::probe_wdl(pos)
    }

    /// Probe DTZ (Distance To Zeroing move) for best move selection.
    pub fn probe_dtz(&self, pos: &Position) -> Option<WdlScore> {
        if !self.enabled { return None; }
        let piece_count = pos.occupancy.count_ones();
        if piece_count > self.max_pieces { return None; }
        probe::probe_dtz(pos)
    }
}

impl Default for SyzygyProber {
    fn default() -> Self { Self::new() }
}

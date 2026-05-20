/// Time management — decides how long Specter should think.
/// Supports fixed depth, fixed time, and tournament time controls.

use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct TimeManager {
    pub start:       Instant,
    pub hard_limit:  Duration,   // absolute stop — never exceed this
    pub soft_limit:  Duration,   // preferred stop — stop here if move is stable
    pub max_depth:   u32,        // depth cap (0 = unlimited)
    pub nodes_limit: u64,        // node cap (0 = unlimited)
    stopped:         bool,
}

impl TimeManager {
    /// Create a time manager for a fixed time per move (milliseconds).
    pub fn fixed_time(ms: u64) -> Self {
        let dur = Duration::from_millis(ms);
        TimeManager {
            start:       Instant::now(),
            hard_limit:  dur,
            soft_limit:  Duration::from_millis((ms as f64 * 0.6) as u64),
            max_depth:   0,
            nodes_limit: 0,
            stopped:     false,
        }
    }

    /// Create a time manager for tournament time controls.
    /// time_left: remaining time in ms, increment: per-move increment in ms.
    pub fn tournament(time_left: u64, increment: u64, moves_to_go: Option<u32>) -> Self {
        let mtg = moves_to_go.unwrap_or(40) as u64;
        // Allocate roughly time_left / moves_to_go + increment * 0.8
        let alloc = (time_left / mtg + increment * 4 / 5).min(time_left / 2);
        let hard  = (alloc * 5).min(time_left - 100); // never use all time
        TimeManager {
            start:       Instant::now(),
            hard_limit:  Duration::from_millis(hard),
            soft_limit:  Duration::from_millis(alloc),
            max_depth:   0,
            nodes_limit: 0,
            stopped:     false,
        }
    }

    /// Create a time manager for fixed depth search.
    pub fn fixed_depth(depth: u32) -> Self {
        TimeManager {
            start:       Instant::now(),
            hard_limit:  Duration::from_secs(3600),
            soft_limit:  Duration::from_secs(3600),
            max_depth:   depth,
            nodes_limit: 0,
            stopped:     false,
        }
    }

    /// Create a time manager for infinite search (UCI "go infinite").
    pub fn infinite() -> Self {
        TimeManager {
            start:       Instant::now(),
            hard_limit:  Duration::from_secs(3600),
            soft_limit:  Duration::from_secs(3600),
            max_depth:   0,
            nodes_limit: 0,
            stopped:     false,
        }
    }

    /// Elapsed time since search started.
    #[inline(always)]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Elapsed time in milliseconds.
    #[inline(always)]
    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed().as_millis() as u64
    }

    /// True if we've exceeded the hard time limit (must stop immediately).
    #[inline(always)]
    pub fn is_hard_expired(&self) -> bool {
        self.stopped || self.elapsed() >= self.hard_limit
    }

    /// True if we've exceeded the soft time limit (should stop between iterations).
    #[inline(always)]
    pub fn is_soft_expired(&self) -> bool {
        self.stopped || self.elapsed() >= self.soft_limit
    }

    /// True if we've hit the depth limit.
    #[inline(always)]
    pub fn depth_limit_reached(&self, depth: u32) -> bool {
        self.max_depth > 0 && depth > self.max_depth
    }

    /// True if we've hit the node limit.
    #[inline(always)]
    pub fn nodes_limit_reached(&self, nodes: u64) -> bool {
        self.nodes_limit > 0 && nodes >= self.nodes_limit
    }

    /// Signal the search to stop immediately (e.g., UCI "stop" command).
    pub fn stop(&mut self) {
        self.stopped = true;
    }

    /// Should we check time? Avoid checking every node — expensive.
    /// Check every 4096 nodes.
    #[inline(always)]
    pub fn should_check(&self, nodes: u64) -> bool {
        nodes & 4095 == 0
    }
}

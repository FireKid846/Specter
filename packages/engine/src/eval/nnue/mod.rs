/// NNUE evaluation module.
///
/// Exposes `NnueEval` — the stateful evaluator that owns the accumulator
/// and network weights. One instance lives inside `SpectorEngine` (WASM)
/// or the search state (CLI).
///
/// Usage:
///   let mut nnue = NnueEval::new();          // loads weights
///   nnue.refresh(&pos);                       // full board scan
///   let score = nnue.evaluate(&pos);          // run forward pass
///
///   // Around make/unmake:
///   nnue.push(&pos, mv);                      // before make_move
///   pos.make_move(mv);
///   if nnue.needs_refresh() { nnue.refresh(&pos); }
///   // ... search ...
///   pos.unmake_move(mv, state);
///   nnue.pop();                               // after unmake_move

pub mod accumulator;
pub mod network;
pub mod weights;

use crate::board::position::{Move, Position};
use crate::eval::nnue::accumulator::Accumulator;
use crate::eval::nnue::network::forward;
use crate::eval::nnue::weights::{NetworkWeights};

/// Whether a king move was the last push (needs refresh after make_move).
/// We detect this by checking if the moved piece is a king.
use crate::board::piece::PieceType;

/// Stateful NNUE evaluator.
/// Holds the network weights (loaded once) and the incremental accumulator.
pub struct NnueEval {
    weights:      Option<NetworkWeights>,
    accumulator:  Accumulator,
    /// True if the last push was a king move and a refresh is pending.
    needs_refresh: bool,
}

impl NnueEval {
    /// Create a new evaluator and attempt to load embedded network weights.
    /// If no weights are embedded, `is_active()` returns false and the
    /// engine falls back to hand-crafted eval.
    pub fn new() -> Self {
        NnueEval {
            weights:      NetworkWeights::load(),
            accumulator:  Accumulator::new(),
            needs_refresh: false,
        }
    }

    /// Returns true if a valid network is loaded and NNUE eval is active.
    #[inline(always)]
    pub fn is_active(&self) -> bool {
        self.weights.is_some()
    }

    /// Full refresh — scan all pieces and rebuild the accumulator from scratch.
    /// Call this at the start of a new position (setPosition, reset, new game).
    pub fn refresh(&mut self, pos: &Position) {
        if let Some(ref w) = self.weights {
            self.accumulator.refresh(pos, w);
            self.needs_refresh = false;
        }
    }

    /// Returns true if the last push was a king move and a refresh is needed.
    #[inline(always)]
    pub fn needs_refresh(&self) -> bool {
        self.needs_refresh
    }

    /// Push accumulator state before making a move.
    /// Call this BEFORE `pos.make_move(mv)`.
    ///
    /// If the moved piece is a king, marks `needs_refresh = true`.
    /// Caller must call `refresh()` after `pos.make_move()` in that case.
    pub fn push(&mut self, pos: &Position, mv: Move) {
        if let Some(ref w) = self.weights {
            // Check if it's a king move before updating
            let is_king_move = pos
                .piece_type_on(mv.from(), pos.side)
                .map(|pt| pt == PieceType::King)
                .unwrap_or(false);

            self.accumulator.push_move(pos, mv, w);
            self.needs_refresh = is_king_move;
        }
    }

    /// Pop accumulator state after unmaking a move.
    /// Call this AFTER `pos.unmake_move(mv, state)`.
    pub fn pop(&mut self) {
        if self.weights.is_some() {
            self.accumulator.pop_move();
            self.needs_refresh = false;
        }
    }

    /// Run the NNUE forward pass and return a score in centipawns
    /// from the side-to-move's perspective.
    ///
    /// Returns None if no network is loaded.
    pub fn evaluate(&self, pos: &Position) -> Option<i32> {
        let w = self.weights.as_ref()?;
        let state = self.accumulator.current();
        Some(forward(state, pos.side, w))
    }
}

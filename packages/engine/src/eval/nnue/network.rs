/// NNUE network forward pass.
///
/// Architecture: (INPUT_SIZE → L1_SIZE)×2 → 1
///
/// Forward pass steps:
///   1. Take both perspective accumulators (already computed incrementally)
///   2. Apply ClippedReLU: clamp to [0, QA]
///   3. Concatenate both perspectives: [white | black] (size L1_SIZE * 2)
///      — but ordered by side to move first (stm perspective first)
///   4. Dot product with output weights (i16 × i16 → i32 accumulation)
///   5. Add output bias
///   6. Scale to centipawns: output / (QA * QB) * NETWORK_SCALE
///
/// Quantization:
///   L1 activations are i16, clamped to [0, QA=255].
///   Output weights are i16.
///   Dot product accumulates into i32.
///   Final: (dot_product / QAB) * NETWORK_SCALE / 1024

use crate::board::color::Color;
use crate::eval::nnue::accumulator::AccumulatorState;
use crate::eval::nnue::weights::{NetworkWeights, QA, QAB, NETWORK_SCALE, L1_SIZE};

/// Run the NNUE forward pass given the current accumulator state and side to move.
///
/// Returns a score in centipawns from the side-to-move's perspective.
pub fn forward(
    state:      &AccumulatorState,
    stm:        Color,
    weights:    &NetworkWeights,
) -> i32 {
    // ── Step 1: ClippedReLU on both perspectives ───────────────────────────
    // Side-to-move perspective goes first, then opponent.
    let (stm_acc, opp_acc) = match stm {
        Color::White => (&state.white, &state.black),
        Color::Black => (&state.black, &state.white),
    };

    // ── Step 2: Dot product with output weights ────────────────────────────
    // Output weights layout: [stm_weights: L1_SIZE | opp_weights: L1_SIZE]
    let mut sum: i32 = 0;

    // STM perspective (first half of output weights)
    for n in 0..L1_SIZE {
        let activation = clipped_relu(stm_acc[n]);
        sum += (activation as i32) * (weights.output_weights[n] as i32);
    }

    // OPP perspective (second half of output weights)
    for n in 0..L1_SIZE {
        let activation = clipped_relu(opp_acc[n]);
        sum += (activation as i32) * (weights.output_weights[L1_SIZE + n] as i32);
    }

    // Add output bias (already scaled)
    sum += weights.output_bias as i32;

    // ── Step 3: Scale to centipawns ────────────────────────────────────────
    // sum is in units of QA * QB = QAB
    // Divide by QAB and multiply by NETWORK_SCALE to get centipawns.
    sum * NETWORK_SCALE / QAB
}

/// ClippedReLU: clamp i16 value to [0, QA].
#[inline(always)]
fn clipped_relu(x: i16) -> i16 {
    x.clamp(0, QA as i16)
}

/// Incremental accumulator for NNUE.
///
/// The accumulator stores the hidden layer (L1) activation for both perspectives
/// (White and Black) after all pieces on the board have been added.
///
/// On each make_move, instead of recomputing from scratch, we:
///   1. Copy the current accumulator to a stack entry
///   2. Subtract features for pieces that moved/were captured
///   3. Add features for pieces that appeared (destinations, promotions)
///
/// On unmake_move, we pop the stack and restore the previous accumulator.
///
/// King moves require a full refresh because the king bucket may change —
/// all features are king-relative so every index changes.

use crate::board::color::Color;
use crate::board::piece::PieceType;
use crate::board::position::{Move, Position};
use crate::eval::nnue::weights::{
    feature_index, NetworkWeights, L1_SIZE,
};

/// One accumulator state — L1 activations for both perspectives.
/// Values are pre-bias i16 sums (biases added once at forward pass time).
#[derive(Clone)]
pub struct AccumulatorState {
    /// White's perspective hidden layer.
    pub white: [i16; L1_SIZE],
    /// Black's perspective hidden layer.
    pub black: [i16; L1_SIZE],
}

impl AccumulatorState {
    pub fn zeroed() -> Self {
        AccumulatorState {
            white: [0i16; L1_SIZE],
            black: [0i16; L1_SIZE],
        }
    }
}

/// The live accumulator — tracks current position and maintains a stack
/// for make/unmake.
pub struct Accumulator {
    /// Stack of saved states. Current state is stack[top].
    stack: Vec<AccumulatorState>,
}

impl Accumulator {
    pub fn new() -> Self {
        Accumulator {
            stack: Vec::with_capacity(256),
        }
    }

    /// Returns the current accumulator state.
    #[inline(always)]
    pub fn current(&self) -> &AccumulatorState {
        self.stack.last().expect("accumulator stack empty")
    }

    /// Full refresh from the current position.
    /// Called at the root (new game, setPosition) and after king moves.
    pub fn refresh(&mut self, pos: &Position, weights: &NetworkWeights) {
        let mut state = AccumulatorState::zeroed();

        // Initialize with biases
        for n in 0..L1_SIZE {
            state.white[n] = weights.feature_biases[n];
            state.black[n] = weights.feature_biases[n];
        }

        let white_king_sq = pos.king_sq(Color::White) as usize;
        let black_king_sq = pos.king_sq(Color::Black) as usize;

        // Add every piece on the board
        for color_idx in 0..2usize {
            let color = if color_idx == 0 { Color::White } else { Color::Black };
            for pt_idx in 0..6usize {
                let pt = PieceType::from_index(pt_idx).unwrap();
                let mut bb = pos.bb(color, pt);
                while bb != 0 {
                    let sq = (bb.trailing_zeros()) as usize;
                    bb &= bb - 1;

                    // White perspective
                    let wi = feature_index(white_king_sq, color_idx, pt_idx, sq, 0);
                    add_feature(&mut state.white, wi, &weights.feature_weights);

                    // Black perspective (flip sq and color)
                    let bi = feature_index(black_king_sq, color_idx, pt_idx, sq, 1);
                    add_feature(&mut state.black, bi, &weights.feature_weights);
                }
            }
        }

        // Push fresh state onto stack (replace if stack non-empty at root)
        if self.stack.is_empty() {
            self.stack.push(state);
        } else {
            *self.stack.last_mut().unwrap() = state;
        }
    }

    /// Push a copy of the current state onto the stack before making a move.
    /// Then update the accumulator for the move.
    ///
    /// For king moves, performs a full refresh instead of incremental update.
    pub fn push_move(&mut self, pos: &Position, mv: Move, weights: &NetworkWeights) {
        // Clone current state
        let new_state = self.current().clone();
        self.stack.push(new_state);
        let new_state = self.stack.last_mut().unwrap();

        let us       = pos.side;
        let them     = us.flip();
        let from     = mv.from() as usize;
        let to       = mv.to() as usize;
        let flag     = mv.flag();
        let us_idx   = us.index();
        let them_idx = them.index();

        let moving_pt = pos.piece_type_on(mv.from(), us)
            .expect("no piece on from square") as usize;

        let white_king_sq = pos.king_sq(Color::White) as usize;
        let black_king_sq = pos.king_sq(Color::Black) as usize;

        // Detect king move — requires full refresh after make_move
        // We mark this by leaving the state as-is and letting the caller
        // do a refresh. We still push so unmake works correctly.
        if moving_pt == PieceType::King as usize {
            // Will be refreshed by the caller after make_move
            return;
        }

        // ── Remove moving piece from source ───────────────────────────────
        let wi_from = feature_index(white_king_sq, us_idx, moving_pt, from, 0);
        let bi_from = feature_index(black_king_sq, us_idx, moving_pt, from, 1);
        sub_feature(&mut new_state.white, wi_from, &weights.feature_weights);
        sub_feature(&mut new_state.black, bi_from, &weights.feature_weights);

        // ── Handle captures ───────────────────────────────────────────────
        if let Some(cap_pt) = pos.piece_on(mv.to()).map(|p| p.piece_type as usize) {
            let wi_cap = feature_index(white_king_sq, them_idx, cap_pt, to, 0);
            let bi_cap = feature_index(black_king_sq, them_idx, cap_pt, to, 1);
            sub_feature(&mut new_state.white, wi_cap, &weights.feature_weights);
            sub_feature(&mut new_state.black, bi_cap, &weights.feature_weights);
        }

        match flag {
            // Normal move or double pawn push
            0 | 1 => {
                let wi_to = feature_index(white_king_sq, us_idx, moving_pt, to, 0);
                let bi_to = feature_index(black_king_sq, us_idx, moving_pt, to, 1);
                add_feature(&mut new_state.white, wi_to, &weights.feature_weights);
                add_feature(&mut new_state.black, bi_to, &weights.feature_weights);
            }

            // Castling kingside (2) or queenside (3)
            2 | 3 => {
                // King already handled above (king move → will refresh)
                // Rook moves
                let (rook_from, rook_to) = if flag == 2 {
                    if us == Color::White { (7usize, 5usize) } else { (63usize, 61usize) }
                } else {
                    if us == Color::White { (0usize, 3usize) } else { (56usize, 59usize) }
                };
                let rook = PieceType::Rook as usize;
                let wi_rf = feature_index(white_king_sq, us_idx, rook, rook_from, 0);
                let bi_rf = feature_index(black_king_sq, us_idx, rook, rook_from, 1);
                sub_feature(&mut new_state.white, wi_rf, &weights.feature_weights);
                sub_feature(&mut new_state.black, bi_rf, &weights.feature_weights);
                let wi_rt = feature_index(white_king_sq, us_idx, rook, rook_to, 0);
                let bi_rt = feature_index(black_king_sq, us_idx, rook, rook_to, 1);
                add_feature(&mut new_state.white, wi_rt, &weights.feature_weights);
                add_feature(&mut new_state.black, bi_rt, &weights.feature_weights);
            }

            // En passant (4)
            4 => {
                // Add pawn to destination
                let wi_to = feature_index(white_king_sq, us_idx, PieceType::Pawn as usize, to, 0);
                let bi_to = feature_index(black_king_sq, us_idx, PieceType::Pawn as usize, to, 1);
                add_feature(&mut new_state.white, wi_to, &weights.feature_weights);
                add_feature(&mut new_state.black, bi_to, &weights.feature_weights);

                // Remove captured pawn (not on `to` square — one rank behind)
                let cap_sq = if us == Color::White { to - 8 } else { to + 8 };
                let wi_ep = feature_index(white_king_sq, them_idx, PieceType::Pawn as usize, cap_sq, 0);
                let bi_ep = feature_index(black_king_sq, them_idx, PieceType::Pawn as usize, cap_sq, 1);
                sub_feature(&mut new_state.white, wi_ep, &weights.feature_weights);
                sub_feature(&mut new_state.black, bi_ep, &weights.feature_weights);
            }

            // Promotions (5=N, 6=B, 7=R, 8=Q)
            5..=8 => {
                // Source pawn already removed above.
                // Add the promoted piece.
                let promo_pt = (flag - 4) as usize; // 1=N, 2=B, 3=R, 4=Q
                let wi_to = feature_index(white_king_sq, us_idx, promo_pt, to, 0);
                let bi_to = feature_index(black_king_sq, us_idx, promo_pt, to, 1);
                add_feature(&mut new_state.white, wi_to, &weights.feature_weights);
                add_feature(&mut new_state.black, bi_to, &weights.feature_weights);
            }

            _ => {}
        }
    }

    /// Pop the top state, restoring the accumulator to before the last push.
    #[inline(always)]
    pub fn pop_move(&mut self) {
        self.stack.pop();
    }
}

/// Add a feature's column of weights to the accumulator.
#[inline(always)]
fn add_feature(acc: &mut [i16; L1_SIZE], feature_idx: usize, weights: &[i16]) {
    let offset = feature_idx * L1_SIZE;
    for n in 0..L1_SIZE {
        acc[n] = acc[n].saturating_add(weights[offset + n]);
    }
}

/// Subtract a feature's column of weights from the accumulator.
#[inline(always)]
fn sub_feature(acc: &mut [i16; L1_SIZE], feature_idx: usize, weights: &[i16]) {
    let offset = feature_idx * L1_SIZE;
    for n in 0..L1_SIZE {
        acc[n] = acc[n].saturating_sub(weights[offset + n]);
    }
}

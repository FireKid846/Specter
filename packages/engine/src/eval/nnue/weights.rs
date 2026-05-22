/// NNUE weight loading and feature indexing.
///
/// Architecture: (HalfKA → L1)×2 → 1
///
/// HalfKA input features:
///   For each perspective (us/them), for each piece on the board:
///     index = king_bucket * 768 + piece_color * 384 + piece_type * 64 + square
///
///   But we use the simpler 768-input variant (no king buckets in v1):
///     index = piece_index * 64 + square
///   where piece_index = color * 6 + piece_type (0-11), square 0-63.
///
///   Two perspectives: from White's POV and from Black's POV.
///   Black's POV mirrors squares vertically (sq ^ 56) and swaps colors.
///
/// King buckets (4 buckets, mirrored horizontally if king on a-d files):
///   Bucket 0: king on rank 1-2
///   Bucket 1: king on rank 3-4
///   Bucket 2: king on rank 5-6
///   Bucket 3: king on rank 7-8
///
/// Total input size per perspective: KING_BUCKETS * 768 = 3072
/// Hidden layer (L1): L1_SIZE neurons per perspective
/// Output: 1 scalar (centipawns before scaling)

pub const L1_SIZE: usize = 256;
pub const KING_BUCKETS: usize = 4;
pub const INPUT_SIZE: usize = KING_BUCKETS * 768;

/// King bucket lookup by square index (0-63).
/// Mirror king to e-h files (file >= 4 stays, file < 4 mirrors to 7-file).
/// Then split by rank into 4 buckets.
#[rustfmt::skip]
pub const KING_BUCKET_MAP: [usize; 64] = [
    // rank 1 (a1-h1) → bucket 0
    0, 0, 0, 0, 0, 0, 0, 0,
    // rank 2 (a2-h2) → bucket 0
    0, 0, 0, 0, 0, 0, 0, 0,
    // rank 3 → bucket 1
    1, 1, 1, 1, 1, 1, 1, 1,
    // rank 4 → bucket 1
    1, 1, 1, 1, 1, 1, 1, 1,
    // rank 5 → bucket 2
    2, 2, 2, 2, 2, 2, 2, 2,
    // rank 6 → bucket 2
    2, 2, 2, 2, 2, 2, 2, 2,
    // rank 7 → bucket 3
    3, 3, 3, 3, 3, 3, 3, 3,
    // rank 8 → bucket 3
    3, 3, 3, 3, 3, 3, 3, 3,
];

/// Compute the feature index for a piece from a given perspective.
///
/// # Arguments
/// * `king_sq` - King square of the perspective side (0-63)
/// * `piece_color` - Color of the piece (0=White, 1=Black)
/// * `piece_type` - Piece type (0=Pawn, 1=Knight, 2=Bishop, 3=Rook, 4=Queen, 5=King)
/// * `piece_sq` - Square the piece is on (0-63)
/// * `perspective` - Which side's perspective (0=White, 1=Black)
///
/// For Black's perspective, squares are flipped vertically (^ 56) and colors swapped.
#[inline(always)]
pub fn feature_index(
    king_sq:    usize,
    piece_color: usize,
    piece_type: usize,
    piece_sq:   usize,
    perspective: usize,
) -> usize {
    // Mirror king horizontally if on a-d files (file 0-3 → mirror to 7-file)
    let king_file = king_sq % 8;
    let king_sq_mirrored = if king_file < 4 {
        king_sq ^ 7  // mirror file: file 0→7, 1→6, 2→5, 3→4
    } else {
        king_sq
    };

    let bucket = KING_BUCKET_MAP[king_sq];

    // From Black's perspective: flip squares and swap colors
    let (sq, color) = if perspective == 1 {
        (piece_sq ^ 56, piece_color ^ 1)
    } else {
        (piece_sq, piece_color)
    };

    // Mirror piece square horizontally if king was mirrored
    let sq = if king_file < 4 { sq ^ 7 } else { sq };

    let _ = king_sq_mirrored; // used implicitly via bucket

    // Index: bucket * 768 + color * 384 + piece_type * 64 + square
    bucket * 768 + color * 384 + piece_type * 64 + sq
}

/// Quantization scale for the output layer.
/// Network output is divided by this to get centipawns.
pub const NETWORK_SCALE: i32 = 400;

/// Quantization scale for L1 weights (i8 range).
pub const QA: i32 = 255;

/// Quantization scale for output weights (i8 range).
pub const QB: i32 = 64;

/// Combined scale factor for the forward pass.
pub const QAB: i32 = QA * QB;

// ─── Weight storage ───────────────────────────────────────────────────────────
//
// Weights are loaded from an embedded .nnue binary file.
// Format (matches Bullet trainer output):
//   [feature_weights: i16; INPUT_SIZE * L1_SIZE]  — L1 weight matrix
//   [feature_biases:  i16; L1_SIZE]               — L1 biases
//   [output_weights:  i16; L1_SIZE * 2]            — output weights (2 perspectives)
//   [output_bias:     i16; 1]                      — output bias
//
// All weights are stored in row-major order.
// Feature weights are laid out as [feature_index][neuron_index].

/// The embedded network file. Will be `None` at compile time if no .nnue file
/// is present — the engine falls back to hand-crafted eval in that case.
///
/// To embed a trained network, place the .nnue file at:
///   packages/engine/src/eval/nnue/specter.nnue
/// and set the `nnue` feature in Cargo.toml.
#[cfg(feature = "nnue")]
pub static NETWORK_BYTES: &[u8] =
    include_bytes!("specter.nnue");

#[cfg(not(feature = "nnue"))]
pub static NETWORK_BYTES: &[u8] = &[];

/// Size checks — validated at startup.
pub const FEATURE_WEIGHT_COUNT: usize = INPUT_SIZE * L1_SIZE;
pub const FEATURE_BIAS_COUNT:   usize = L1_SIZE;
pub const OUTPUT_WEIGHT_COUNT:  usize = L1_SIZE * 2; // two perspectives
pub const OUTPUT_BIAS_COUNT:    usize = 1;

/// Total expected byte count in the .nnue file (all i16 = 2 bytes each).
pub const EXPECTED_BYTES: usize = (
    FEATURE_WEIGHT_COUNT +
    FEATURE_BIAS_COUNT   +
    OUTPUT_WEIGHT_COUNT  +
    OUTPUT_BIAS_COUNT
) * 2;

/// Parsed network weights — loaded once at startup from NETWORK_BYTES.
pub struct NetworkWeights {
    pub feature_weights: Vec<i16>,  // [INPUT_SIZE * L1_SIZE]
    pub feature_biases:  Vec<i16>,  // [L1_SIZE]
    pub output_weights:  Vec<i16>,  // [L1_SIZE * 2]
    pub output_bias:     i16,
}

impl NetworkWeights {
    /// Load weights from the embedded byte slice.
    /// Returns None if no network is embedded or the file is malformed.
    pub fn load() -> Option<NetworkWeights> {
        let bytes = NETWORK_BYTES;
        if bytes.len() != EXPECTED_BYTES {
            return None;
        }

        // Read i16 values from little-endian bytes
        let mut offset = 0;
        let read_i16s = |bytes: &[u8], offset: &mut usize, count: usize| -> Vec<i16> {
            let v = (0..count)
                .map(|_| {
                    let val = i16::from_le_bytes([bytes[*offset], bytes[*offset + 1]]);
                    *offset += 2;
                    val
                })
                .collect();
            v
        };

        let feature_weights = read_i16s(bytes, &mut offset, FEATURE_WEIGHT_COUNT);
        let feature_biases  = read_i16s(bytes, &mut offset, FEATURE_BIAS_COUNT);
        let output_weights  = read_i16s(bytes, &mut offset, OUTPUT_WEIGHT_COUNT);
        let output_bias_vec = read_i16s(bytes, &mut offset, OUTPUT_BIAS_COUNT);

        Some(NetworkWeights {
            feature_weights,
            feature_biases,
            output_weights,
            output_bias: output_bias_vec[0],
        })
    }
}

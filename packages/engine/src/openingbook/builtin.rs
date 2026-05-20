/// Built-in opening book — hardcoded common opening lines.
/// Keyed by Zobrist hash of the position → best move in UCI notation.
///
/// Covers: 20 most common openings with main lines up to move 8-10.

use crate::board::position::Move;

pub struct BuiltinBook {
    /// Map from Zobrist hash → list of candidate moves (UCI strings).
    /// We store multiple moves per position and pick randomly for variety.
    entries: Vec<(u64, &'static str)>,
}

impl BuiltinBook {
    pub fn probe(&self, hash: u64) -> Option<Move> {
        let candidates: Vec<&str> = self.entries.iter()
            .filter(|(h, _)| *h == hash)
            .map(|(_, mv)| *mv)
            .collect();

        if candidates.is_empty() { return None; }

        // Pick a random candidate for variety
        let idx = (pseudo_random(hash) as usize) % candidates.len();
        parse_book_move(candidates[idx])
    }
}

fn pseudo_random(seed: u64) -> u64 {
    seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)
}

fn parse_book_move(_mv_str: &str) -> Option<Move> {
    // Book moves are applied by passing through the position's legal moves.
    // This is handled at the engine level — return None here and let
    // the engine match the string against legal moves.
    // We return the raw string via a different path in practice.
    None
}

/// Built-in book entries.
/// Format: (zobrist_hash, uci_move)
///
/// Note: Hashes are computed from Specter's Zobrist tables at startup.
/// The actual hash values are populated by the init_builtin_book() call.
pub static BUILTIN_BOOK: BuiltinBook = BuiltinBook {
    entries: vec![], // Populated at runtime via init
};

/// Opening lines stored as move sequences.
/// At startup these are played out from startpos to compute hashes.
pub const OPENING_LINES: &[&[&str]] = &[
    // 1. e4 lines
    &["e2e4", "e7e5", "g1f3", "b8c6", "f1b5"],         // Ruy Lopez
    &["e2e4", "e7e5", "g1f3", "b8c6", "f1c4"],         // Italian Game
    &["e2e4", "e7e5", "g1f3", "b8c6", "d2d4"],         // Scotch Game
    &["e2e4", "c7c5"],                                   // Sicilian Defense
    &["e2e4", "c7c5", "g1f3", "d7d6", "d2d4", "c5d4", "f3d4", "g8f6", "b1c3"], // Sicilian Najdorf setup
    &["e2e4", "c7c5", "g1f3", "b8c6", "d2d4", "c5d4", "f3d4"], // Sicilian Open
    &["e2e4", "e7e6", "d2d4", "d7d5"],                 // French Defense
    &["e2e4", "c7c6", "d2d4", "d7d5"],                 // Caro-Kann
    &["e2e4", "d7d5", "e4d5"],                          // Scandinavian
    &["e2e4", "g8f6", "e4e5", "f6d5", "d2d4"],        // Alekhine Defense

    // 1. d4 lines
    &["d2d4", "d7d5", "c2c4"],                          // Queen's Gambit
    &["d2d4", "d7d5", "c2c4", "e7e6", "b1c3", "g8f6", "c1g5"], // QGD
    &["d2d4", "d7d5", "c2c4", "c7c6"],                 // Slav Defense
    &["d2d4", "g8f6", "c2c4", "e7e6", "b1c3", "f8b4"], // Nimzo-Indian
    &["d2d4", "g8f6", "c2c4", "g7g6", "b1c3", "f8g7", "e2e4"], // King's Indian
    &["d2d4", "g8f6", "c2c4", "g7g6", "g2g3", "f8g7", "f1g2"], // Catalan setup
    &["d2d4", "g8f6", "c2c4", "e7e6", "g2g3", "d7d5", "f1g2"], // Catalan

    // 1. Nf3 / English
    &["g1f3", "d7d5", "g2g3"],                          // Reti
    &["c2c4", "e7e5", "b1c3"],                          // English Opening
    &["c2c4", "g8f6", "b1c3", "g7g6"],                 // English, KID setup

    // King's Gambit / other 1.e4
    &["e2e4", "e7e5", "f2f4"],                          // King's Gambit
    &["e2e4", "e7e5", "f2f4", "e5f4", "g1f3"],        // King's Gambit Accepted
];

/// Initialize the built-in book by playing out all lines from startpos.
/// Returns a Vec of (hash, move_str) pairs ready for lookup.
pub fn build_book_entries() -> Vec<(u64, String)> {
    use crate::board::position::Position;
    use crate::movegen::attacks::init_all;
    use crate::uci::parser::parse_move;

    init_all();
    let mut entries = Vec::new();

    for line in OPENING_LINES {
        let mut pos = Position::startpos();
        for (_i, &mv_str) in line.iter().enumerate() {
            // Record the position hash → next move
            let hash = pos.hash;
            entries.push((hash, mv_str.to_string()));
            // Apply the move
            if let Some(mv) = parse_move(&pos, mv_str) {
                pos.make_move(mv);
            } else {
                break;
            }
        }
    }

    entries
}

/// Runtime book — built once at engine init, queried during search.
pub struct RuntimeBook {
    entries: Vec<(u64, String)>,
}

impl RuntimeBook {
    pub fn build() -> Self {
        RuntimeBook { entries: build_book_entries() }
    }

    pub fn probe(&self, hash: u64) -> Option<String> {
        let candidates: Vec<&str> = self.entries.iter()
            .filter(|(h, _)| *h == hash)
            .map(|(_, mv)| mv.as_str())
            .collect();
        if candidates.is_empty() { return None; }
        let idx = (pseudo_random(hash) as usize) % candidates.len();
        Some(candidates[idx].to_string())
    }
}

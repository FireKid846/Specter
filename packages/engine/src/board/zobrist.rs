/// Zobrist hashing — assigns random 64-bit keys to each piece/square
/// combination, side to move, castling rights, and en passant file.
/// XOR these together to get a unique hash for any position.

use crate::board::piece::PieceType;
use crate::board::color::Color;

pub const PIECE_KEYS:     [[u64; 64]; 12] = generate_piece_keys();
pub const SIDE_KEY:       u64             = generate_side_key();
pub const CASTLING_KEYS:  [u64; 16]       = generate_castling_keys();
pub const EN_PASSANT_KEYS:[u64; 8]        = generate_ep_keys();

/// A simple compile-time LCG PRNG seeded with a fixed value.
/// Used only during const evaluation to fill Zobrist tables.
const fn lcg(state: u64) -> u64 {
    state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407)
}

const fn generate_piece_keys() -> [[u64; 64]; 12] {
    let mut keys = [[0u64; 64]; 12];
    let mut state: u64 = 0xDEADBEEFCAFEBABE;
    let mut piece = 0;
    while piece < 12 {
        let mut sq = 0;
        while sq < 64 {
            state = lcg(state);
            keys[piece][sq] = state;
            sq += 1;
        }
        piece += 1;
    }
    keys
}

const fn generate_side_key() -> u64 {
    // One fixed key XORed in when Black is to move.
    0x94D273B5A0F8B6C1
}

const fn generate_castling_keys() -> [u64; 16] {
    let mut keys = [0u64; 16];
    let mut state: u64 = 0xFEEDFACEDEADC0DE;
    let mut i = 0;
    while i < 16 {
        state = lcg(state);
        keys[i] = state;
        i += 1;
    }
    keys
}

const fn generate_ep_keys() -> [u64; 8] {
    let mut keys = [0u64; 8];
    let mut state: u64 = 0xBAADF00DDEADBEEF;
    let mut i = 0;
    while i < 8 {
        state = lcg(state);
        keys[i] = state;
        i += 1;
    }
    keys
}

/// Castling rights bit flags.
pub const CASTLE_WK: u8 = 0b0001; // White kingside
pub const CASTLE_WQ: u8 = 0b0010; // White queenside
pub const CASTLE_BK: u8 = 0b0100; // Black kingside
pub const CASTLE_BQ: u8 = 0b1000; // Black queenside

/// Returns the piece index into PIECE_KEYS: color * 6 + piece_type.
#[inline(always)]
pub fn piece_key_index(color: Color, pt: PieceType) -> usize {
    color.index() * PieceType::NUM + pt.index()
}

/// Compute a full Zobrist hash from scratch (used when loading a FEN).
/// Incremental updates are done directly in position.rs.
pub struct ZobristHasher {
    pub hash: u64,
}

impl ZobristHasher {
    pub fn new() -> Self {
        ZobristHasher { hash: 0 }
    }

    #[inline(always)]
    pub fn toggle_piece(&mut self, color: Color, pt: PieceType, sq: usize) {
        self.hash ^= PIECE_KEYS[piece_key_index(color, pt)][sq];
    }

    #[inline(always)]
    pub fn toggle_side(&mut self) {
        self.hash ^= SIDE_KEY;
    }

    #[inline(always)]
    pub fn set_castling(&mut self, rights: u8) {
        self.hash ^= CASTLING_KEYS[rights as usize];
    }

    #[inline(always)]
    pub fn set_ep(&mut self, file: u8) {
        self.hash ^= EN_PASSANT_KEYS[file as usize];
    }
}

impl Default for ZobristHasher {
    fn default() -> Self {
        Self::new()
    }
}

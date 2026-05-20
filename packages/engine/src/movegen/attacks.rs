/// Magic bitboard attack generation.
/// Magic numbers are generated at runtime via PRNG trial-and-error.
/// This guarantees correctness — no hardcoded magic numbers that could be wrong.

use crate::board::bitboard::*;
use crate::board::color::Color;

// ─── Attack tables (populated once at init) ───────────────────────────────────

struct MagicEntry {
    mask:  u64,
    magic: u64,
    shift: u32,
}

static mut ROOK_MAGICS:   [MagicEntry; 64] = unsafe { std::mem::zeroed() };
static mut BISHOP_MAGICS: [MagicEntry; 64] = unsafe { std::mem::zeroed() };

static mut ROOK_ATTACKS:   [Vec<u64>; 64] = [const { Vec::new() }; 64];
static mut BISHOP_ATTACKS: [Vec<u64>; 64] = [const { Vec::new() }; 64];

static mut KNIGHT_TABLE: [u64; 64] = [0; 64];
static mut KING_TABLE:   [u64; 64] = [0; 64];
static mut PAWN_ATTACKS: [[u64; 64]; 2] = [[0; 64]; 2];

static mut ATTACKS_INITIALIZED: bool = false;

// ─── Public init ──────────────────────────────────────────────────────────────

pub fn init_all() {
    unsafe {
        if ATTACKS_INITIALIZED { return; }
        init_leapers();
        init_sliders();
        ATTACKS_INITIALIZED = true;
    }
}

fn init_leapers() {
    unsafe {
        for sq in 0u32..64 {
            let bb = 1u64 << sq;
            KNIGHT_TABLE[sq as usize] = knight_attacks(bb);
            KING_TABLE[sq as usize]   = king_attacks(bb);
            PAWN_ATTACKS[0][sq as usize] = white_pawn_attacks(bb);
            PAWN_ATTACKS[1][sq as usize] = black_pawn_attacks(bb);
        }
    }
}

fn init_sliders() {
    unsafe {
        for sq in 0u32..64 {
            // ── Bishop ────────────────────────────────────────────────────────
            let b_mask  = bishop_mask_slow(sq);
            let b_bits  = b_mask.count_ones();
            let b_size  = 1usize << b_bits;
            let b_shift = 64 - b_bits;

            let b_magic = find_magic(sq, b_mask, b_bits, false);
            BISHOP_MAGICS[sq as usize] = MagicEntry { mask: b_mask, magic: b_magic, shift: b_shift };

            let mut b_table = vec![0u64; b_size];
            let mut occ = 0u64;
            loop {
                let idx = magic_idx(occ, b_magic, b_shift);
                b_table[idx] = bishop_attacks_slow(sq, occ);
                occ = occ.wrapping_sub(b_mask) & b_mask;
                if occ == 0 { break; }
            }
            // Also fill index 0 (empty board)
            let idx0 = magic_idx(0, b_magic, b_shift);
            b_table[idx0] = bishop_attacks_slow(sq, 0);
            BISHOP_ATTACKS[sq as usize] = b_table;

            // ── Rook ──────────────────────────────────────────────────────────
            let r_mask  = rook_mask_slow(sq);
            let r_bits  = r_mask.count_ones();
            let r_size  = 1usize << r_bits;
            let r_shift = 64 - r_bits;

            let r_magic = find_magic(sq, r_mask, r_bits, true);
            ROOK_MAGICS[sq as usize] = MagicEntry { mask: r_mask, magic: r_magic, shift: r_shift };

            let mut r_table = vec![0u64; r_size];
            let mut occ = 0u64;
            loop {
                let idx = magic_idx(occ, r_magic, r_shift);
                r_table[idx] = rook_attacks_slow(sq, occ);
                occ = occ.wrapping_sub(r_mask) & r_mask;
                if occ == 0 { break; }
            }
            let idx0 = magic_idx(0, r_magic, r_shift);
            r_table[idx0] = rook_attacks_slow(sq, 0);
            ROOK_ATTACKS[sq as usize] = r_table;
        }
    }
}

// ─── Magic finder (PRNG trial-and-error) ─────────────────────────────────────

fn find_magic(sq: u32, mask: u64, bits: u32, _is_rook: bool) -> u64 {
    let size = 1usize << bits;
    let mut used = vec![u64::MAX; size];
    let mut rng  = Rng::new(sq as u64 + 1);

    // Precompute all occupancy subsets and their attack sets
    let mut occs    = vec![0u64; size];
    let mut attacks = vec![0u64; size];
    let mut occ = 0u64;
    for i in 0..size {
        occs[i]    = occ;
        attacks[i] = if _is_rook { rook_attacks_slow(sq, occ) } else { bishop_attacks_slow(sq, occ) };
        occ = occ.wrapping_sub(mask) & mask;
    }

    'outer: loop {
        // Candidate magic: sparse random with few set bits
        let magic = rng.next() & rng.next() & rng.next();
        if (mask.wrapping_mul(magic) >> 56).count_ones() < 6 { continue; }

        used.iter_mut().for_each(|x| *x = u64::MAX);

        for i in 0..size {
            let idx = magic_idx(occs[i], magic, 64 - bits);
            if used[idx] == u64::MAX {
                used[idx] = attacks[i];
            } else if used[idx] != attacks[i] {
                continue 'outer; // Destructive collision — try another magic
            }
        }
        return magic;
    }
}

#[inline(always)]
fn magic_idx(occ: u64, magic: u64, shift: u32) -> usize {
    ((occ.wrapping_mul(magic)) >> shift) as usize
}

// ─── Simple PRNG ─────────────────────────────────────────────────────────────

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self { Rng(seed ^ 0x9E3779B97F4A7C15) }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        self.0 = x; x
    }
}

// ─── Public attack getters ────────────────────────────────────────────────────

#[inline(always)]
pub fn bishop_attacks(sq: u32, occ: u64) -> u64 {
    unsafe {
        let e = &BISHOP_MAGICS[sq as usize];
        let idx = magic_idx(occ & e.mask, e.magic, e.shift);
        BISHOP_ATTACKS[sq as usize][idx]
    }
}

#[inline(always)]
pub fn rook_attacks(sq: u32, occ: u64) -> u64 {
    unsafe {
        let e = &ROOK_MAGICS[sq as usize];
        let idx = magic_idx(occ & e.mask, e.magic, e.shift);
        ROOK_ATTACKS[sq as usize][idx]
    }
}

#[inline(always)]
pub fn queen_attacks(sq: u32, occ: u64) -> u64 {
    rook_attacks(sq, occ) | bishop_attacks(sq, occ)
}

#[inline(always)]
pub fn knight_attacks_sq(sq: u32) -> u64 {
    unsafe { KNIGHT_TABLE[sq as usize] }
}

#[inline(always)]
pub fn king_attacks_sq(sq: u32) -> u64 {
    unsafe { KING_TABLE[sq as usize] }
}

#[inline(always)]
pub fn pawn_attacks(color: Color, sq: u32) -> u64 {
    unsafe { PAWN_ATTACKS[color.index()][sq as usize] }
}

// ─── Slow reference implementations ──────────────────────────────────────────

fn rook_mask_slow(sq: u32) -> u64 {
    let rank = sq / 8;
    let file = sq % 8;
    let mut mask = 0u64;
    for r in (rank + 1)..7 { mask |= 1u64 << (r * 8 + file); }
    for r in 1..rank        { mask |= 1u64 << (r * 8 + file); }
    for f in (file + 1)..7  { mask |= 1u64 << (rank * 8 + f); }
    for f in 1..file        { mask |= 1u64 << (rank * 8 + f); }
    mask
}

fn bishop_mask_slow(sq: u32) -> u64 {
    let rank = sq / 8;
    let file = sq % 8;
    let mut mask = 0u64;
    for (dr, df) in [(1i32,1i32),(1,-1),(-1,1),(-1,-1)] {
        let (mut r, mut f) = (rank as i32 + dr, file as i32 + df);
        while r > 0 && r < 7 && f > 0 && f < 7 {
            mask |= 1u64 << (r * 8 + f);
            r += dr; f += df;
        }
    }
    mask
}

pub fn rook_attacks_slow(sq: u32, occ: u64) -> u64 {
    let rank = sq / 8;
    let file = sq % 8;
    let mut attacks = 0u64;
    for (dr, df) in [(1i32,0i32),(-1,0),(0,1),(0,-1)] {
        let (mut r, mut f) = (rank as i32 + dr, file as i32 + df);
        while (0..8).contains(&r) && (0..8).contains(&f) {
            let bit = 1u64 << (r * 8 + f);
            attacks |= bit;
            if occ & bit != 0 { break; }
            r += dr; f += df;
        }
    }
    attacks
}

pub fn bishop_attacks_slow(sq: u32, occ: u64) -> u64 {
    let rank = sq / 8;
    let file = sq % 8;
    let mut attacks = 0u64;
    for (dr, df) in [(1i32,1i32),(1,-1),(-1,1),(-1,-1)] {
        let (mut r, mut f) = (rank as i32 + dr, file as i32 + df);
        while (0..8).contains(&r) && (0..8).contains(&f) {
            let bit = 1u64 << (r * 8 + f);
            attacks |= bit;
            if occ & bit != 0 { break; }
            r += dr; f += df;
        }
    }
    attacks
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rook_attacks_empty_board() {
        init_all();
        // Rook on e4 (sq=28): 7 north + 3 south + 4 east + 3 west... wait
        // e4: file=4, rank=3
        // N: e5,e6,e7,e8 = 4; S: e3,e2,e1 = 3; E: f4,g4,h4 = 3; W: d4,c4,b4,a4 = 4 → 14
        assert_eq!(popcount(rook_attacks(28, 0)), 14);
    }

    #[test]
    fn test_bishop_attacks_empty_board() {
        init_all();
        // Bishop on e4 (sq=28): NE=3, NW=4, SE=3, SW=3 → 13
        assert_eq!(popcount(bishop_attacks(28, 0)), 13);
    }

    #[test]
    fn test_queen_attacks_empty_board() {
        init_all();
        assert_eq!(popcount(queen_attacks(28, 0)), 27);
    }

    #[test]
    fn test_knight_attacks_center() {
        init_all();
        assert_eq!(popcount(knight_attacks_sq(28)), 8);
    }
}

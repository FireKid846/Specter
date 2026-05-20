/// Magic bitboard attack generation.
///
/// For each sliding piece (bishop, rook) and each square, we precompute
/// an attack table indexed by a "magic index" derived from the blocker
/// occupancy. This gives O(1) attack lookup at runtime.
///
/// Algorithm:
///   index = ((occupancy & mask[sq]) * magic[sq]) >> shift[sq]
///   attacks = attack_table[sq][index]

use crate::board::bitboard::*;

// ─── Rook magic numbers (pre-computed, well-known values) ─────────────────────
#[rustfmt::skip]
const ROOK_MAGICS: [u64; 64] = [
    0x0080001020400080, 0x0040001000200040, 0x0080081000200080, 0x0080040800100080,
    0x0080020400080080, 0x0080010200040080, 0x0080008001000200, 0x0080002040800100,
    0x0000800020400080, 0x0000400020005000, 0x0000801000200080, 0x0000800800100080,
    0x0000800400080080, 0x0000800200040080, 0x0000800100020080, 0x0000800040800100,
    0x0000208000400080, 0x0000404000201000, 0x0000808010002000, 0x0000808008001000,
    0x0000808004000800, 0x0000808002000400, 0x0000010100020004, 0x0000020000408104,
    0x0000208080004000, 0x0000200040005000, 0x0000100080200080, 0x0000080080100080,
    0x0000040080080080, 0x0000020080040080, 0x0000010080800200, 0x0000800080004100,
    0x0000204000800080, 0x0000200040401000, 0x0000100080802000, 0x0000080080801000,
    0x0000040080800800, 0x0000020080800400, 0x0000020001010004, 0x0000800040800100,
    0x0000204000808000, 0x0000200040008080, 0x0000100020008080, 0x0000080010008080,
    0x0000040008008080, 0x0000020004008080, 0x0000010002008080, 0x0000004081020004,
    0x0000204000800080, 0x0000200040005000, 0x0000100080200080, 0x0000080080100080,
    0x0000040080080080, 0x0000020080040080, 0x0000010080800200, 0x0000800080004100,
    0x0000102040800101, 0x0000102040008101, 0x0000081020004101, 0x0000040810002101,
    0x0000020408001001, 0x0000010204000801, 0x0000008102000401, 0x0000004081020001,
];

// ─── Bishop magic numbers ─────────────────────────────────────────────────────
#[rustfmt::skip]
const BISHOP_MAGICS: [u64; 64] = [
    0x0002020202020200, 0x0002020202020000, 0x0004010202000000, 0x0004040080000000,
    0x0001104000000000, 0x0000821040000000, 0x0000410410400000, 0x0000104104104000,
    0x0000040404040400, 0x0000020202020200, 0x0000040102020000, 0x0000040400800000,
    0x0000011040000000, 0x0000008210400000, 0x0000004104104000, 0x0000002082082000,
    0x0004000808080800, 0x0002000404040400, 0x0001000202020200, 0x0000800802004000,
    0x0000200201000000, 0x0000100100200000, 0x0000040040402000, 0x0000020020401000,
    0x0000400080808080, 0x0000200040404040, 0x0000100020202020, 0x0000080010101000,
    0x0000040008040400, 0x0000020004020200, 0x0000010002010100, 0x0000008001008080,
    0x0000802080808080, 0x0000401040404040, 0x0000200020202020, 0x0000100010101010,
    0x0000080008080808, 0x0000040004040404, 0x0000020002020202, 0x0000010001010101,
    0x0000804081020020, 0x0000402040810010, 0x0000201020408008, 0x0000100810204004,
    0x0000081020100020, 0x0000040810200010, 0x0000020408100008, 0x0000010204080004,
    0x0000810040820020, 0x0000408020410010, 0x0000204010208008, 0x0000102008104004,
    0x0000081004820002, 0x0000040810410001, 0x0000020408208010, 0x0000010204104008,
    0x0000410040820020, 0x0000208020410010, 0x0000104010208008, 0x0000082008104004,
    0x0000041004082002, 0x0000020810041001, 0x0000010408020802, 0x0000008204010400,
];

const ROOK_BITS:   [u32; 64] = [
    12,11,11,11,11,11,11,12,
    11,10,10,10,10,10,10,11,
    11,10,10,10,10,10,10,11,
    11,10,10,10,10,10,10,11,
    11,10,10,10,10,10,10,11,
    11,10,10,10,10,10,10,11,
    11,10,10,10,10,10,10,11,
    12,11,11,11,11,11,11,12,
];

const BISHOP_BITS: [u32; 64] = [
    6,5,5,5,5,5,5,6,
    5,5,5,5,5,5,5,5,
    5,5,7,7,7,7,5,5,
    5,5,7,9,9,7,5,5,
    5,5,7,9,9,7,5,5,
    5,5,7,7,7,7,5,5,
    5,5,5,5,5,5,5,5,
    6,5,5,5,5,5,5,6,
];

// ─── Attack tables (populated at init) ───────────────────────────────────────

static mut ROOK_ATTACKS:   [[u64; 4096]; 64] = [[0; 4096]; 64];
static mut BISHOP_ATTACKS: [[u64; 512];  64] = [[0; 512];  64];
static mut ROOK_MASKS:     [u64; 64]         = [0; 64];
static mut BISHOP_MASKS:   [u64; 64]         = [0; 64];

static mut ATTACKS_INITIALIZED: bool = false;

/// Must be called once before using any attack lookup functions.
pub fn init_attacks() {
    unsafe {
        if ATTACKS_INITIALIZED { return; }
        for sq in 0..64u32 {
            ROOK_MASKS[sq as usize]   = rook_mask(sq);
            BISHOP_MASKS[sq as usize] = bishop_mask(sq);

            let rmask = ROOK_MASKS[sq as usize];
            let bmask = BISHOP_MASKS[sq as usize];

            // Enumerate all subsets of the mask (Carry-Rippler)
            let mut occ = 0u64;
            loop {
                let idx = magic_index(occ, ROOK_MAGICS[sq as usize], ROOK_BITS[sq as usize]);
                ROOK_ATTACKS[sq as usize][idx] = rook_attacks_slow(sq, occ);
                occ = occ.wrapping_sub(rmask) & rmask;
                if occ == 0 { break; }
            }

            let mut occ = 0u64;
            loop {
                let idx = magic_index(occ, BISHOP_MAGICS[sq as usize], BISHOP_BITS[sq as usize]);
                BISHOP_ATTACKS[sq as usize][idx] = bishop_attacks_slow(sq, occ);
                occ = occ.wrapping_sub(bmask) & bmask;
                if occ == 0 { break; }
            }
        }
        ATTACKS_INITIALIZED = true;
    }
}

#[inline(always)]
fn magic_index(occ: u64, magic: u64, bits: u32) -> usize {
    ((occ.wrapping_mul(magic)) >> (64 - bits)) as usize
}

// ─── Public attack getters ────────────────────────────────────────────────────

#[inline(always)]
pub fn rook_attacks(sq: u32, occ: u64) -> u64 {
    unsafe {
        let masked = occ & ROOK_MASKS[sq as usize];
        let idx = magic_index(masked, ROOK_MAGICS[sq as usize], ROOK_BITS[sq as usize]);
        ROOK_ATTACKS[sq as usize][idx]
    }
}

#[inline(always)]
pub fn bishop_attacks(sq: u32, occ: u64) -> u64 {
    unsafe {
        let masked = occ & BISHOP_MASKS[sq as usize];
        let idx = magic_index(masked, BISHOP_MAGICS[sq as usize], BISHOP_BITS[sq as usize]);
        BISHOP_ATTACKS[sq as usize][idx]
    }
}

#[inline(always)]
pub fn queen_attacks(sq: u32, occ: u64) -> u64 {
    rook_attacks(sq, occ) | bishop_attacks(sq, occ)
}

// ─── Precomputed non-slider tables ───────────────────────────────────────────

static mut KNIGHT_TABLE: [u64; 64] = [0; 64];
static mut KING_TABLE:   [u64; 64] = [0; 64];
static mut PAWN_ATTACKS: [[u64; 64]; 2] = [[0; 64]; 2];

pub fn init_non_slider_tables() {
    unsafe {
        for sq in 0..64u32 {
            let bb = 1u64 << sq;
            KNIGHT_TABLE[sq as usize] = knight_attacks(bb);
            KING_TABLE[sq as usize]   = king_attacks(bb);
            PAWN_ATTACKS[0][sq as usize] = white_pawn_attacks(bb); // White
            PAWN_ATTACKS[1][sq as usize] = black_pawn_attacks(bb); // Black
        }
    }
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
pub fn pawn_attacks(color: crate::board::color::Color, sq: u32) -> u64 {
    unsafe { PAWN_ATTACKS[color.index()][sq as usize] }
}

/// Initialize all attack tables. Call once at engine startup.
pub fn init_all() {
    init_non_slider_tables();
    init_attacks();
}

// ─── Slow reference implementations (used during table initialization) ────────

fn rook_mask(sq: u32) -> u64 {
    let rank = sq / 8;
    let file = sq % 8;
    let mut mask = 0u64;
    for r in (rank + 1)..7 { mask |= 1u64 << (r * 8 + file); }
    for r in 1..rank        { mask |= 1u64 << (r * 8 + file); }
    for f in (file + 1)..7  { mask |= 1u64 << (rank * 8 + f); }
    for f in 1..file        { mask |= 1u64 << (rank * 8 + f); }
    mask
}

fn bishop_mask(sq: u32) -> u64 {
    let rank = sq / 8;
    let file = sq % 8;
    let mut mask = 0u64;
    let dirs: [(i32, i32); 4] = [(1,1),(1,-1),(-1,1),(-1,-1)];
    for (dr, df) in dirs {
        let mut r = rank as i32 + dr;
        let mut f = file as i32 + df;
        while r > 0 && r < 7 && f > 0 && f < 7 {
            mask |= 1u64 << (r * 8 + f);
            r += dr; f += df;
        }
    }
    mask
}

fn rook_attacks_slow(sq: u32, occ: u64) -> u64 {
    let rank = sq / 8;
    let file = sq % 8;
    let mut attacks = 0u64;
    let dirs: [(i32, i32); 4] = [(1,0),(-1,0),(0,1),(0,-1)];
    for (dr, df) in dirs {
        let mut r = rank as i32 + dr;
        let mut f = file as i32 + df;
        while (0..8).contains(&r) && (0..8).contains(&f) {
            let bit = 1u64 << (r * 8 + f);
            attacks |= bit;
            if occ & bit != 0 { break; }
            r += dr; f += df;
        }
    }
    attacks
}

fn bishop_attacks_slow(sq: u32, occ: u64) -> u64 {
    let rank = sq / 8;
    let file = sq % 8;
    let mut attacks = 0u64;
    let dirs: [(i32, i32); 4] = [(1,1),(1,-1),(-1,1),(-1,-1)];
    for (dr, df) in dirs {
        let mut r = rank as i32 + dr;
        let mut f = file as i32 + df;
        while (0..8).contains(&r) && (0..8).contains(&f) {
            let bit = 1u64 << (r * 8 + f);
            attacks |= bit;
            if occ & bit != 0 { break; }
            r += dr; f += df;
        }
    }
    attacks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rook_attacks_empty_board() {
        init_all();
        // Rook on e4 (square 28) on empty board
        let attacks = rook_attacks(28, 0);
        assert_eq!(popcount(attacks), 14);
    }

    #[test]
    fn test_bishop_attacks_empty_board() {
        init_all();
        // Bishop on e4 (square 28) on empty board
        let attacks = bishop_attacks(28, 0);
        assert_eq!(popcount(attacks), 13);
    }

    #[test]
    fn test_queen_attacks_empty_board() {
        init_all();
        // Queen on e4 (square 28) on empty board: 14 + 13 = 27
        let attacks = queen_attacks(28, 0);
        assert_eq!(popcount(attacks), 27);
    }

    #[test]
    fn test_knight_attacks_center() {
        init_all();
        let attacks = knight_attacks_sq(28); // e4
        assert_eq!(popcount(attacks), 8);
    }
}

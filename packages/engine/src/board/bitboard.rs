/// A bitboard is a u64 where each bit represents a square.
/// Bit 0 = A1, Bit 7 = H1, Bit 56 = A8, Bit 63 = H8.

// ─── File masks ───────────────────────────────────────────────────────────────
pub const FILE_A: u64 = 0x0101010101010101;
pub const FILE_B: u64 = FILE_A << 1;
pub const FILE_C: u64 = FILE_A << 2;
pub const FILE_D: u64 = FILE_A << 3;
pub const FILE_E: u64 = FILE_A << 4;
pub const FILE_F: u64 = FILE_A << 5;
pub const FILE_G: u64 = FILE_A << 6;
pub const FILE_H: u64 = FILE_A << 7;

pub const NOT_FILE_A: u64 = !FILE_A;
pub const NOT_FILE_H: u64 = !FILE_H;
pub const NOT_FILE_AB: u64 = !(FILE_A | FILE_B);
pub const NOT_FILE_GH: u64 = !(FILE_G | FILE_H);

// ─── Rank masks ───────────────────────────────────────────────────────────────
pub const RANK_1: u64 = 0xFF;
pub const RANK_2: u64 = RANK_1 << 8;
pub const RANK_3: u64 = RANK_1 << 16;
pub const RANK_4: u64 = RANK_1 << 24;
pub const RANK_5: u64 = RANK_1 << 32;
pub const RANK_6: u64 = RANK_1 << 40;
pub const RANK_7: u64 = RANK_1 << 48;
pub const RANK_8: u64 = RANK_1 << 56;

// ─── Diagonal masks ───────────────────────────────────────────────────────────
pub const LIGHT_SQUARES: u64 = 0x55AA55AA55AA55AA;
pub const DARK_SQUARES:  u64 = 0xAA55AA55AA55AA55;

// ─── Core bitboard operations ─────────────────────────────────────────────────

/// Returns the number of set bits.
#[inline(always)]
pub fn popcount(bb: u64) -> u32 {
    bb.count_ones()
}

/// Returns true if exactly one bit is set.
#[inline(always)]
pub fn is_single(bb: u64) -> bool {
    bb != 0 && (bb & bb.wrapping_sub(1)) == 0
}

/// Returns the index of the least significant bit (LSB).
/// Undefined behavior if bb == 0.
#[inline(always)]
pub fn lsb(bb: u64) -> u32 {
    bb.trailing_zeros()
}

/// Returns the index of the most significant bit (MSB).
/// Undefined behavior if bb == 0.
#[inline(always)]
pub fn msb(bb: u64) -> u32 {
    63 - bb.leading_zeros()
}

/// Removes and returns the LSB index. Mutates the bitboard in place.
#[inline(always)]
pub fn pop_lsb(bb: &mut u64) -> u32 {
    let sq = bb.trailing_zeros();
    *bb &= *bb - 1;
    sq
}

// ─── Shift operations ─────────────────────────────────────────────────────────

#[inline(always)] pub fn north(bb: u64) -> u64 { bb << 8 }
#[inline(always)] pub fn south(bb: u64) -> u64 { bb >> 8 }
#[inline(always)] pub fn east(bb: u64)  -> u64 { (bb & NOT_FILE_H) << 1 }
#[inline(always)] pub fn west(bb: u64)  -> u64 { (bb & NOT_FILE_A) >> 1 }

#[inline(always)] pub fn north_east(bb: u64) -> u64 { (bb & NOT_FILE_H) << 9 }
#[inline(always)] pub fn north_west(bb: u64) -> u64 { (bb & NOT_FILE_A) << 7 }
#[inline(always)] pub fn south_east(bb: u64) -> u64 { (bb & NOT_FILE_H) >> 7 }
#[inline(always)] pub fn south_west(bb: u64) -> u64 { (bb & NOT_FILE_A) >> 9 }

// ─── Fill operations ──────────────────────────────────────────────────────────

/// Kogge-Stone north fill — fills all squares north of each set bit.
pub fn north_fill(mut bb: u64) -> u64 {
    bb |= bb << 8;
    bb |= bb << 16;
    bb |= bb << 32;
    bb
}

/// Kogge-Stone south fill.
pub fn south_fill(mut bb: u64) -> u64 {
    bb |= bb >> 8;
    bb |= bb >> 16;
    bb |= bb >> 32;
    bb
}

// ─── Attack generation helpers ────────────────────────────────────────────────

/// Knight attacks from a single square bitboard.
#[inline(always)]
pub fn knight_attacks(sq_bb: u64) -> u64 {
    let l1 = (sq_bb >> 1) & NOT_FILE_H;
    let l2 = (sq_bb >> 2) & NOT_FILE_GH;
    let r1 = (sq_bb << 1) & NOT_FILE_A;
    let r2 = (sq_bb << 2) & NOT_FILE_AB;
    let h1 = l1 | r1;
    let h2 = l2 | r2;
    (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8)
}

/// King attacks from a single square bitboard.
#[inline(always)]
pub fn king_attacks(sq_bb: u64) -> u64 {
    let attacks = east(sq_bb) | west(sq_bb) | sq_bb;
    let attacks = north(attacks) | south(attacks) | attacks;
    attacks & !sq_bb
}

/// White pawn attacks (northeast + northwest).
#[inline(always)]
pub fn white_pawn_attacks(pawns: u64) -> u64 {
    north_east(pawns) | north_west(pawns)
}

/// Black pawn attacks (southeast + southwest).
#[inline(always)]
pub fn black_pawn_attacks(pawns: u64) -> u64 {
    south_east(pawns) | south_west(pawns)
}

// ─── Debug print ─────────────────────────────────────────────────────────────

/// Prints a bitboard as an 8x8 grid for debugging.
pub fn print_bb(bb: u64) {
    println!();
    for rank in (0..8).rev() {
        print!("  {} ", rank + 1);
        for file in 0..8 {
            let sq = rank * 8 + file;
            print!("{} ", if (bb >> sq) & 1 == 1 { '1' } else { '.' });
        }
        println!();
    }
    println!("    a b c d e f g h");
    println!("    Hex: {:#018x}", bb);
    println!();
}

// ─── Iterator over set bits ───────────────────────────────────────────────────

pub struct BitIter(pub u64);

impl Iterator for BitIter {
    type Item = u32;
    #[inline(always)]
    fn next(&mut self) -> Option<u32> {
        if self.0 == 0 {
            None
        } else {
            let sq = pop_lsb(&mut self.0);
            Some(sq)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popcount() {
        assert_eq!(popcount(0), 0);
        assert_eq!(popcount(u64::MAX), 64);
        assert_eq!(popcount(RANK_1), 8);
        assert_eq!(popcount(FILE_A), 8);
    }

    #[test]
    fn test_lsb_msb() {
        assert_eq!(lsb(1), 0);
        assert_eq!(lsb(8), 3);
        assert_eq!(msb(8), 3);
        assert_eq!(msb(u64::MAX), 63);
    }

    #[test]
    fn test_knight_attacks() {
        // Knight on e4 (square 28)
        let sq_bb = 1u64 << 28;
        let attacks = knight_attacks(sq_bb);
        assert_eq!(popcount(attacks), 8);
    }

    #[test]
    fn test_king_attacks() {
        // King on e4 (square 28)
        let sq_bb = 1u64 << 28;
        let attacks = king_attacks(sq_bb);
        assert_eq!(popcount(attacks), 8);
    }

    #[test]
    fn test_pawn_attacks() {
        // White pawns on rank 2
        let attacks = white_pawn_attacks(RANK_2);
        assert_eq!(popcount(attacks), 14); // 6 interior files × 2 + 2 corner files × 1 = 14
    }
}

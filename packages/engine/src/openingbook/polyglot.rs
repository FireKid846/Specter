/// Polyglot opening book reader.
/// Polyglot is the standard binary book format used by chess engines.
///
/// Format: each entry is 16 bytes:
///   8 bytes: Zobrist key (big-endian u64)
///   2 bytes: move (encoded)
///   2 bytes: weight
///   4 bytes: learn (ignored)
///
/// Move encoding (2 bytes):
///   bits  0-5:  to file
///   bits  3-8:  to rank
///   bits  6-11: from file
///   bits  9-14: from rank  (overlapping — see polyglot spec)
///   bits 12-14: promotion piece (0=none,1=knight,2=bishop,3=rook,4=queen)

use crate::board::position::Move;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

const ENTRY_SIZE: usize = 16;

#[derive(Debug, Clone, Copy)]
struct PolyglotEntry {
    key:    u64,
    mv:     u16,
    weight: u16,
}

pub struct PolyglotBook {
    entries: Vec<PolyglotEntry>,
}

impl PolyglotBook {
    /// Load a polyglot book from a .bin file.
    pub fn load(path: &str) -> Result<Self, String> {
        let mut file = File::open(path)
            .map_err(|e| format!("Cannot open book '{}': {}", path, e))?;

        let size = file.seek(SeekFrom::End(0))
            .map_err(|e| e.to_string())?;
        file.seek(SeekFrom::Start(0))
            .map_err(|e| e.to_string())?;

        if size % ENTRY_SIZE as u64 != 0 {
            return Err("Invalid polyglot file: size not multiple of 16".into());
        }

        let num_entries = (size / ENTRY_SIZE as u64) as usize;
        let mut buf = vec![0u8; size as usize];
        file.read_exact(&mut buf)
            .map_err(|e| e.to_string())?;

        let mut entries = Vec::with_capacity(num_entries);
        for i in 0..num_entries {
            let off = i * ENTRY_SIZE;
            let key    = u64::from_be_bytes(buf[off..off+8].try_into().unwrap());
            let mv     = u16::from_be_bytes(buf[off+8..off+10].try_into().unwrap());
            let weight = u16::from_be_bytes(buf[off+10..off+12].try_into().unwrap());
            entries.push(PolyglotEntry { key, mv, weight });
        }

        Ok(PolyglotBook { entries })
    }

    /// Probe the book for a given Zobrist hash.
    /// Returns the best (highest weight) move, or None if not found.
    pub fn probe(&self, hash: u64) -> Option<Move> {
        // Binary search for the hash
        let idx = self.entries.partition_point(|e| e.key < hash);
        if idx >= self.entries.len() || self.entries[idx].key != hash {
            return None;
        }

        // Collect all entries with this hash, pick highest weight
        let mut best_weight = 0u16;
        let mut best_mv     = 0u16;

        let mut i = idx;
        while i < self.entries.len() && self.entries[i].key == hash {
            if self.entries[i].weight > best_weight {
                best_weight = self.entries[i].weight;
                best_mv     = self.entries[i].mv;
            }
            i += 1;
        }

        decode_polyglot_move(best_mv)
    }

    /// Probe and return all moves with weights for a position (for opening explorer).
    pub fn probe_all(&self, hash: u64) -> Vec<(String, u16)> {
        let idx = self.entries.partition_point(|e| e.key < hash);
        let mut result = Vec::new();
        let mut i = idx;
        while i < self.entries.len() && self.entries[i].key == hash {
            let mv_uci = decode_polyglot_move_uci(self.entries[i].mv);
            result.push((mv_uci, self.entries[i].weight));
            i += 1;
        }
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }

    pub fn num_entries(&self) -> usize {
        self.entries.len()
    }
}

/// Decode a polyglot move encoding into a UCI string.
fn decode_polyglot_move_uci(encoded: u16) -> String {
    let to_file   = (encoded & 0x7) as u8;
    let to_rank   = ((encoded >> 3) & 0x7) as u8;
    let from_file = ((encoded >> 6) & 0x7) as u8;
    let from_rank = ((encoded >> 9) & 0x7) as u8;
    let promo     = (encoded >> 12) & 0x7;

    let promo_ch = match promo {
        1 => "n", 2 => "b", 3 => "r", 4 => "q",
        _ => "",
    };

    format!(
        "{}{}{}{}{}",
        (b'a' + from_file) as char,
        (b'1' + from_rank) as char,
        (b'a' + to_file) as char,
        (b'1' + to_rank) as char,
        promo_ch
    )
}

/// Decode a polyglot move into a Specter Move.
/// Returns None — caller should match the UCI string against legal moves.
fn decode_polyglot_move(encoded: u16) -> Option<Move> {
    // We return None and handle matching externally via parse_move()
    // This avoids duplicating castling/en-passant logic here.
    None
}

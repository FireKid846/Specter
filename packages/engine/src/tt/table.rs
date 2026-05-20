use crate::tt::entry::{Bound, TTEntry};
use crate::board::position::Move;

pub struct TranspositionTable {
    entries: Vec<TTEntry>,
    size:    usize,
    age:     u8,
}

impl TranspositionTable {
    pub fn new(mb: usize) -> Self {
        let size = (mb * 1024 * 1024) / std::mem::size_of::<TTEntry>();
        TranspositionTable {
            entries: vec![TTEntry::EMPTY; size],
            size,
            age: 0,
        }
    }

    pub fn new_auto() -> Self {
        // Default to 64MB
        Self::new(64)
    }

    #[inline(always)]
    fn index(&self, hash: u64) -> usize {
        (hash as usize) % self.size
    }

    pub fn probe(&self, hash: u64) -> Option<TTEntry> {
        let entry = self.entries[self.index(hash)];
        if entry.hash == hash { Some(entry) } else { None }
    }

    pub fn store(&mut self, hash: u64, mv: Move, score: i32, depth: u8, bound: Bound) {
        let idx = self.index(hash);
        let existing = &self.entries[idx];
        // Always replace if different position, or deeper search, or same age
        if existing.hash != hash || depth >= existing.depth || self.age != existing.age {
            self.entries[idx] = TTEntry { hash, mv, score, depth, bound, age: self.age };
        }
    }

    pub fn clear(&mut self) {
        self.entries.iter_mut().for_each(|e| *e = TTEntry::EMPTY);
    }

    pub fn new_search(&mut self) {
        self.age = self.age.wrapping_add(1);
    }

    pub fn hashfull(&self) -> u32 {
        let sample = self.entries.iter().take(1000).filter(|e| e.age == self.age).count();
        (sample as u32 * 1000) / 1000
    }
}

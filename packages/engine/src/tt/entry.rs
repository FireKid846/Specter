use crate::board::position::Move;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Bound { Exact = 0, Lower = 1, Upper = 2 }

#[derive(Clone, Copy)]
pub struct TTEntry {
    pub hash:  u64,
    pub mv:    Move,
    pub score: i32,
    pub depth: u8,
    pub bound: Bound,
    pub age:   u8,
}

impl TTEntry {
    pub const EMPTY: TTEntry = TTEntry {
        hash: 0, mv: Move::NULL, score: 0, depth: 0,
        bound: Bound::Exact, age: 0,
    };
}

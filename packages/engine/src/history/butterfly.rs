/// Butterfly history table — indexed by [color][from][to].
/// Tracks how often a move caused a beta cutoff.
pub struct ButterflyHistory([[[[i32; 64]; 64]; 2]; 1]);

impl ButterflyHistory {
    pub fn new() -> Self { ButterflyHistory([[[[0; 64]; 64]; 2]]); Self([[[[0; 64]; 64]; 2]]) }
    pub fn get(&self, color: usize, from: usize, to: usize) -> i32 { self.0[0][color][from][to] }
    pub fn update(&mut self, color: usize, from: usize, to: usize, bonus: i32) {
        let v = &mut self.0[0][color][from][to];
        *v += bonus - (*v * bonus.abs()) / 16384;
    }
    pub fn clear(&mut self) { self.0[0].iter_mut().for_each(|c| c.iter_mut().for_each(|f| f.iter_mut().for_each(|v| *v = 0))); }
}

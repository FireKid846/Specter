/// Continuation history — for move pairs (previous move, current move).
/// Used in countermove and follow-up heuristics.
pub struct ContinuationHistory(Box<[[[[i32; 64]; 6]; 64]; 6]>);
impl ContinuationHistory {
    pub fn new() -> Self { ContinuationHistory(Box::new([[[[0i32; 64]; 6]; 64]; 6])) }
    pub fn get(&self, pt1: usize, sq1: usize, pt2: usize, sq2: usize) -> i32 {
        self.0[pt1][sq1][pt2][sq2]
    }
    pub fn update(&mut self, pt1: usize, sq1: usize, pt2: usize, sq2: usize, bonus: i32) {
        let v = &mut self.0[pt1][sq1][pt2][sq2];
        *v += bonus - (*v * bonus.abs()) / 16384;
    }
}

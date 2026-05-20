/// Correction history — adjusts static eval based on prediction errors.
/// Indexed by [color][pawn_hash % size].
pub const CORRECTION_SIZE: usize = 16384;
pub struct CorrectionHistory([[i32; CORRECTION_SIZE]; 2]);
impl CorrectionHistory {
    pub fn new() -> Self { CorrectionHistory([[0; CORRECTION_SIZE]; 2]) }
    pub fn get(&self, color: usize, pawn_hash: u64) -> i32 {
        self.0[color][(pawn_hash as usize) % CORRECTION_SIZE]
    }
    pub fn update(&mut self, color: usize, pawn_hash: u64, bonus: i32) {
        let v = &mut self.0[color][(pawn_hash as usize) % CORRECTION_SIZE];
        *v += bonus - (*v * bonus.abs()) / 1024;
    }
    pub fn clear(&mut self) { self.0.iter_mut().for_each(|c| c.iter_mut().for_each(|v| *v = 0)); }
}

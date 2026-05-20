use crate::board::position::Move;
/// Killer moves — two quiet moves per ply that caused beta cutoffs.
pub struct KillerTable([[Move; 2]; 128]);
impl KillerTable {
    pub fn new() -> Self { KillerTable([[Move::NULL; 2]; 128]) }
    pub fn get(&self, ply: usize) -> [Move; 2] { self.0[ply.min(127)] }
    pub fn store(&mut self, ply: usize, mv: Move) {
        let ply = ply.min(127);
        if self.0[ply][0] != mv {
            self.0[ply][1] = self.0[ply][0];
            self.0[ply][0] = mv;
        }
    }
    pub fn clear(&mut self) { self.0.iter_mut().for_each(|k| { k[0] = Move::NULL; k[1] = Move::NULL; }); }
}

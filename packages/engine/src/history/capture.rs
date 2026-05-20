use crate::board::piece::PieceType;
/// Capture history — indexed by [piece_type][captured_type][to].
pub struct CaptureHistory([[[i32; 64]; 6]; 6]);
impl CaptureHistory {
    pub fn new() -> Self { CaptureHistory([[[0; 64]; 6]; 6]) }
    pub fn get(&self, pt: PieceType, cap: PieceType, to: usize) -> i32 { self.0[pt.index()][cap.index()][to] }
    pub fn update(&mut self, pt: PieceType, cap: PieceType, to: usize, bonus: i32) {
        let v = &mut self.0[pt.index()][cap.index()][to];
        *v += bonus - (*v * bonus.abs()) / 16384;
    }
}

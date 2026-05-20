/// Perft (PERFormance Test) — counts leaf nodes at a given depth.
/// Used to verify move generation correctness against known values.
///
/// Known perft values from the starting position:
///   depth 1: 20
///   depth 2: 400
///   depth 3: 8,902
///   depth 4: 197,281
///   depth 5: 4,865,609
///   depth 6: 119,060,324

use crate::board::position::Position;
use crate::movegen::legal::legal_moves;

/// Returns the number of leaf nodes at the given depth.
pub fn perft(pos: &mut Position, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = legal_moves(pos);
    let mut nodes = 0u64;

    if depth == 1 {
        return moves.count as u64;
    }

    for &mv in moves.as_slice() {
        let state = pos.make_move(mv);
        nodes += perft(pos, depth - 1);
        pos.unmake_move(mv, state);
    }

    nodes
}

/// Perft with divide — shows node count per root move for debugging.
pub fn perft_divide(pos: &mut Position, depth: u32) -> u64 {
    let moves = legal_moves(pos);
    let mut total = 0u64;

    for &mv in moves.as_slice() {
        let state = pos.make_move(mv);
        let nodes = perft(pos, depth - 1);
        pos.unmake_move(mv, state);
        println!("{}: {}", mv, nodes);
        total += nodes;
    }

    println!("\nTotal: {}", total);
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movegen::attacks::init_all;

    fn setup() {
        init_all();
    }

    #[test]
    fn perft_startpos_depth1() {
        setup();
        let mut pos = Position::startpos();
        assert_eq!(perft(&mut pos, 1), 20);
    }

    #[test]
    fn perft_startpos_depth2() {
        setup();
        let mut pos = Position::startpos();
        assert_eq!(perft(&mut pos, 2), 400);
    }

    #[test]
    fn perft_startpos_depth3() {
        setup();
        let mut pos = Position::startpos();
        assert_eq!(perft(&mut pos, 3), 8_902);
    }

    #[test]
    #[ignore] // slow — run with: cargo test -- --ignored
    fn perft_startpos_depth4() {
        setup();
        let mut pos = Position::startpos();
        assert_eq!(perft(&mut pos, 4), 197_281);
    }

    #[test]
    #[ignore]
    fn perft_startpos_depth5() {
        setup();
        let mut pos = Position::startpos();
        assert_eq!(perft(&mut pos, 5), 4_865_609);
    }

    // Kiwipete — a comprehensive test position
    #[test]
    fn perft_kiwipete_depth1() {
        setup();
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let mut pos = Position::from_fen(fen).unwrap();
        assert_eq!(perft(&mut pos, 1), 48);
    }

    #[test]
    fn perft_kiwipete_depth2() {
        setup();
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let mut pos = Position::from_fen(fen).unwrap();
        assert_eq!(perft(&mut pos, 2), 2_039);
    }
}

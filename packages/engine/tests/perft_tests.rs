use specter::board::position::Position;
use specter::movegen::attacks::init_all;
use specter::movegen::perft::perft;

fn setup() { init_all(); }

#[test]
fn perft_startpos_d1() { setup(); let mut p = Position::startpos(); assert_eq!(perft(&mut p, 1), 20); }
#[test]
fn perft_startpos_d2() { setup(); let mut p = Position::startpos(); assert_eq!(perft(&mut p, 2), 400); }
#[test]
fn perft_startpos_d3() { setup(); let mut p = Position::startpos(); assert_eq!(perft(&mut p, 3), 8_902); }

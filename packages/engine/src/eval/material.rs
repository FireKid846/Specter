use crate::board::color::Color;
use crate::board::piece::PieceType;
use crate::board::position::Position;
use crate::eval::make_score;

const PAWN_VALUE:   i32 = make_score(126, 208);
const KNIGHT_VALUE: i32 = make_score(781, 854);
const BISHOP_VALUE: i32 = make_score(825, 915);
const ROOK_VALUE:   i32 = make_score(1276, 1380);
const QUEEN_VALUE:  i32 = make_score(2538, 2682);

pub const PIECE_VALUES: [i32; 6] = [
    PAWN_VALUE, KNIGHT_VALUE, BISHOP_VALUE, ROOK_VALUE, QUEEN_VALUE, 0,
];

pub const SIMPLE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, 20000];

pub fn piece_value_simple(pt: PieceType) -> i32 { SIMPLE_VALUES[pt.index()] }

pub fn evaluate_material(pos: &Position) -> i32 {
    let mut score = 0i32;
    for color in [Color::White, Color::Black] {
        let sign = color.sign();
        for pt in PieceType::all() {
            score += sign * pos.bb(color, pt).count_ones() as i32 * PIECE_VALUES[pt.index()];
        }
        if pos.bb(color, PieceType::Bishop).count_ones() >= 2 {
            score += sign * make_score(23, 62);
        }
        if pos.bb(color, PieceType::Knight).count_ones() >= 2 {
            score += sign * make_score(-5, -15);
        }
        if pos.bb(color, PieceType::Rook).count_ones() >= 2 {
            score += sign * make_score(-15, -25);
        }
    }
    score
}

pub fn material_balance(pos: &Position) -> i32 {
    let mut balance = 0i32;
    for color in [Color::White, Color::Black] {
        let sign = color.sign();
        for pt in [PieceType::Pawn, PieceType::Knight, PieceType::Bishop,
                   PieceType::Rook, PieceType::Queen] {
            balance += sign * pos.bb(color, pt).count_ones() as i32 * SIMPLE_VALUES[pt.index()];
        }
    }
    balance
}

pub fn non_pawn_material(pos: &Position, color: Color) -> i32 {
    let mut npm = 0i32;
    for pt in [PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen] {
        npm += pos.bb(color, pt).count_ones() as i32 * SIMPLE_VALUES[pt.index()];
    }
    npm
}

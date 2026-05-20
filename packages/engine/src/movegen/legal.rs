/// Legal move generation and check detection.
/// Filters pseudo-legal moves by verifying the king is not left in check.

use crate::board::bitboard::*;
use crate::board::color::Color;
use crate::board::piece::PieceType;
use crate::board::position::{Move, Position};
use crate::movegen::attacks::*;
use crate::movegen::pseudo_legal::{generate_all, generate_captures, MoveList};

/// Returns all fully legal moves in the position.
pub fn legal_moves(pos: &mut Position) -> MoveList {
    let pseudo = generate_all(pos);
    let mut legal = MoveList::new();
    for &mv in pseudo.as_slice() {
        if is_legal(pos, mv) {
            legal.push(mv);
        }
    }
    legal
}

/// Returns all legal captures (for quiescence search).
pub fn legal_captures(pos: &mut Position) -> MoveList {
    let pseudo = generate_captures(pos);
    let mut legal = MoveList::new();
    for &mv in pseudo.as_slice() {
        if is_legal(pos, mv) {
            legal.push(mv);
        }
    }
    legal
}

/// Checks if a move is fully legal (does not leave king in check).
pub fn is_legal(pos: &mut Position, mv: Move) -> bool {
    let state = pos.make_move(mv);
    let legal = !is_in_check(pos, pos.side.flip()); // was it check after move?
    pos.unmake_move(mv, state);
    legal
}

/// Returns true if the given color's king is currently in check.
pub fn is_in_check(pos: &Position, color: Color) -> bool {
    let king_sq = pos.king_sq(color) as u32;
    is_square_attacked(pos, king_sq, color.flip())
}

/// Returns true if a square is attacked by the given color.
pub fn is_square_attacked(pos: &Position, sq: u32, attacker: Color) -> bool {
    let occ   = pos.occupancy;
    let _their = pos.color_bb(attacker);

    // Pawn attacks
    if pawn_attacks(attacker.flip(), sq) & pos.bb(attacker, PieceType::Pawn) != 0 {
        return true;
    }
    // Knight attacks
    if knight_attacks_sq(sq) & pos.bb(attacker, PieceType::Knight) != 0 {
        return true;
    }
    // Bishop / Queen diagonals
    if bishop_attacks(sq, occ) & (pos.bb(attacker, PieceType::Bishop) | pos.bb(attacker, PieceType::Queen)) != 0 {
        return true;
    }
    // Rook / Queen orthogonals
    if rook_attacks(sq, occ) & (pos.bb(attacker, PieceType::Rook) | pos.bb(attacker, PieceType::Queen)) != 0 {
        return true;
    }
    // King (for adjacent king detection)
    if king_attacks_sq(sq) & pos.bb(attacker, PieceType::King) != 0 {
        return true;
    }

    false
}

/// Returns a bitboard of all squares attacked by the given color.
pub fn attacked_squares(pos: &Position, attacker: Color) -> u64 {
    let occ = pos.occupancy;
    let mut attacks = 0u64;

    // Pawns
    let pawns = pos.bb(attacker, PieceType::Pawn);
    attacks |= match attacker {
        Color::White => white_pawn_attacks(pawns),
        Color::Black => black_pawn_attacks(pawns),
    };

    // Knights
    let mut knights = pos.bb(attacker, PieceType::Knight);
    while knights != 0 {
        let sq = pop_lsb(&mut knights);
        attacks |= knight_attacks_sq(sq);
    }

    // Bishops
    let mut bishops = pos.bb(attacker, PieceType::Bishop);
    while bishops != 0 {
        let sq = pop_lsb(&mut bishops);
        attacks |= bishop_attacks(sq, occ);
    }

    // Rooks
    let mut rooks = pos.bb(attacker, PieceType::Rook);
    while rooks != 0 {
        let sq = pop_lsb(&mut rooks);
        attacks |= rook_attacks(sq, occ);
    }

    // Queens
    let mut queens = pos.bb(attacker, PieceType::Queen);
    while queens != 0 {
        let sq = pop_lsb(&mut queens);
        attacks |= queen_attacks(sq, occ);
    }

    // King
    let king = pos.bb(attacker, PieceType::King);
    if king != 0 {
        attacks |= king_attacks_sq(lsb(king));
    }

    attacks
}

/// Returns true if the current side has no legal moves (checkmate or stalemate).
pub fn has_no_legal_moves(pos: &mut Position) -> bool {
    legal_moves(pos).is_empty()
}

/// Returns true if the position is checkmate.
pub fn is_checkmate(pos: &mut Position) -> bool {
    is_in_check(pos, pos.side) && has_no_legal_moves(pos)
}

/// Returns true if the position is stalemate.
pub fn is_stalemate(pos: &mut Position) -> bool {
    !is_in_check(pos, pos.side) && has_no_legal_moves(pos)
}

/// Count checkers — how many pieces are giving check to the king.
pub fn checker_count(pos: &Position) -> u32 {
    let king_sq = pos.king_sq(pos.side) as u32;
    let them = pos.side.flip();
    let occ  = pos.occupancy;

    let mut count = 0u32;
    if pawn_attacks(pos.side, king_sq) & pos.bb(them, PieceType::Pawn) != 0 { count += 1; }
    if knight_attacks_sq(king_sq) & pos.bb(them, PieceType::Knight) != 0 { count += 1; }
    if bishop_attacks(king_sq, occ) & (pos.bb(them, PieceType::Bishop) | pos.bb(them, PieceType::Queen)) != 0 { count += 1; }
    if rook_attacks(king_sq, occ) & (pos.bb(them, PieceType::Rook) | pos.bb(them, PieceType::Queen)) != 0 { count += 1; }
    count
}

// Specter Engine
// Phase 1: Foundation — board, movegen, search, eval

pub mod board;
pub mod eval;
pub mod history;
pub mod movegen;
pub mod movepick;
pub mod openingbook;
pub mod personality;
pub mod search;
pub mod syzygy;
pub mod tt;
pub mod uci;
pub mod variants;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use board::color::Color;
pub use board::piece::{Piece, PieceType};
pub use board::position::Position;
pub use board::square::Square;

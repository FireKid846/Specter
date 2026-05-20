pub mod atomic;
pub mod chess960;
pub mod four_player;
pub mod horde;
pub mod standard;
pub mod three_check;

/// Which variant is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Standard,
    Chess960,
    FourPlayer,
    ThreeCheck,
    Horde,
    Atomic,
}

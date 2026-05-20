/// Fathom FFI bindings — Phase 2.
/// When Fathom is added as a C dependency in Cargo.toml,
/// this file will contain the unsafe extern "C" blocks.
///
/// Example:
/// extern "C" {
///     fn tb_init(path: *const i8) -> bool;
///     fn tb_probe_wdl(white: u64, black: u64, kings: u64, queens: u64,
///                     rooks: u64, bishops: u64, knights: u64, pawns: u64,
///                     rule50: u32, castling: u32, ep: u32, turn: bool) -> u32;
/// }

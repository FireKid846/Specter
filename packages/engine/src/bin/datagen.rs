/// Specter NNUE data generation binary.
///
/// Runs Specter selfplay games at a fixed depth and writes positions
/// to a .binpack file for training with Bullet.
///
/// Usage:
///   datagen --depth 7 --positions 100000000 --threads 4 \
///           --output specter_train.binpack --random-plies 8
///
/// Output format: custom binary format compatible with Bullet's marlinformat reader.
/// Each record is 32 bytes:
///   [occupancy: u64][pieces: u64][score: i16][wdl: u8][stm: u8][padding: 16 bytes]
/// This matches bullet_lib's MarlinFormat / PackedBoard input format.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use specter::board::color::Color;
use specter::board::position::Position;
use specter::movegen::attacks::init_all;
use specter::movegen::legal::{is_in_check, legal_moves};
use specter::search::iterative_deepening::search;
use specter::search::timeman::TimeManager;
use specter::tt::table::TranspositionTable;

// ─── CLI args ─────────────────────────────────────────────────────────────────

struct Config {
    depth:          u32,
    target:         u64,
    threads:        usize,
    output:         String,
    random_plies:   u32,
    filter_draws:   bool,
}

impl Config {
    fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut cfg = Config {
            depth:        7,
            target:       100_000_000,
            threads:      4,
            output:       "specter_train.binpack".to_string(),
            random_plies: 8,
            filter_draws: false,
        };

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--depth"        => { cfg.depth        = args[i+1].parse().unwrap(); i += 2; }
                "--positions"    => { cfg.target        = args[i+1].parse().unwrap(); i += 2; }
                "--threads"      => { cfg.threads       = args[i+1].parse().unwrap(); i += 2; }
                "--output"       => { cfg.output        = args[i+1].clone();           i += 2; }
                "--random-plies" => { cfg.random_plies  = args[i+1].parse().unwrap(); i += 2; }
                "--filter-draws" => { cfg.filter_draws  = true;                        i += 1; }
                _                => { i += 1; }
            }
        }

        cfg
    }
}

// ─── Binpack record (32 bytes, MarlinFormat) ──────────────────────────────────

/// One training position record.
/// Compatible with Bullet's MarlinFormat binary input.
#[repr(C)]
struct Record {
    /// Occupancy bitboard: which squares have pieces.
    occupancy: u64,
    /// Piece encoding: 4 bits per piece, in order of occupancy LSB-first.
    /// bits 0-2: piece type (0=P,1=N,2=B,3=R,4=Q,5=K), bit 3: color (0=W,1=B)
    pieces:    u64,
    /// Evaluation score from white's perspective (centipawns).
    score:     i16,
    /// Game result from white's perspective: 0=loss, 1=draw, 2=win.
    wdl:       u8,
    /// Side to move at this position: 0=White, 1=Black.
    stm:       u8,
    /// Padding to 32 bytes.
    _pad:      [u8; 16],
}

impl Record {
    fn from_position(pos: &Position, score_stm: i32, wdl_white: u8) -> Option<Self> {
        // score_stm is from the side-to-move perspective; convert to white's POV
        let score_white = if pos.side == Color::White {
            score_stm
        } else {
            -score_stm
        };

        // Clamp to i16 range
        let score = score_white.clamp(-30000, 30000) as i16;

        // Build occupancy and piece nibbles
        let mut occupancy: u64 = 0;
        let mut pieces:    u64 = 0;
        let mut piece_idx: u32 = 0;

        for sq in 0u8..64 {
            let sq_struct = specter::board::square::Square::from_index(sq);
            if let Some(piece) = pos.piece_on(sq_struct) {
                occupancy |= 1u64 << sq;
                let pt_bits  = piece.piece_type as u64;
                let col_bit  = if piece.color == Color::Black { 8u64 } else { 0u64 };
                pieces |= (pt_bits | col_bit) << (piece_idx * 4);
                piece_idx += 1;
            }
        }

        // Reject positions where score is too large (likely mate/blunder noise)
        if score_white.abs() > 10000 { return None; }

        Some(Record {
            occupancy,
            pieces,
            score,
            wdl: wdl_white,
            stm: if pos.side == Color::White { 0 } else { 1 },
            _pad: [0u8; 16],
        })
    }

    fn as_bytes(&self) -> [u8; 32] {
        let mut buf = [0u8; 32];
        buf[0..8].copy_from_slice(&self.occupancy.to_le_bytes());
        buf[8..16].copy_from_slice(&self.pieces.to_le_bytes());
        buf[16..18].copy_from_slice(&self.score.to_le_bytes());
        buf[18] = self.wdl;
        buf[19] = self.stm;
        buf
    }
}

// ─── Selfplay game ────────────────────────────────────────────────────────────

/// Outcome from white's perspective.
#[derive(Clone, Copy, PartialEq)]
enum Outcome { WhiteWin, BlackWin, Draw }

impl Outcome {
    fn wdl_white(self) -> u8 {
        match self {
            Outcome::WhiteWin => 2,
            Outcome::Draw     => 1,
            Outcome::BlackWin => 0,
        }
    }
}

/// Play one selfplay game and return all training records from it.
fn play_game(
    depth:        u32,
    random_plies: u32,
    filter_draws: bool,
) -> Vec<Record> {
    let mut pos = Position::startpos();
    let mut tt  = TranspositionTable::new(16); // small TT per thread

    let mut positions_in_game: Vec<(Position, i32)> = Vec::with_capacity(100);

    // ── Random opening ──────────────────────────────────────────────────────
    for ply in 0..random_plies {
        let moves = legal_moves(&mut pos);
        if moves.is_empty() { return vec![]; }

        let idx = (pos.hash.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407)
            .wrapping_shr(33)) as usize % moves.as_slice().len();

        pos.make_move(moves.as_slice()[idx]);

        // Don't record random-ply positions — too noisy
        let _ = ply;
    }

    // ── Selfplay ────────────────────────────────────────────────────────────
    let mut outcome = None;

    for _ in 0..400 {
        let moves = legal_moves(&mut pos);
        if moves.is_empty() {
            let in_check = is_in_check(&pos, pos.side);
            outcome = Some(if in_check {
                if pos.side == Color::White { Outcome::BlackWin } else { Outcome::WhiteWin }
            } else {
                Outcome::Draw
            });
            break;
        }

        if pos.is_fifty_move_draw() || pos.is_repetition() || pos.is_insufficient_material() {
            outcome = Some(Outcome::Draw);
            break;
        }

        let time = TimeManager::fixed_depth(depth);
        let result = search(&mut pos, &mut tt, time, None);
        let score  = result.score; // from STM perspective

        // Skip positions: in check, or too close to game start
        if !is_in_check(&pos, pos.side) {
            positions_in_game.push((pos.clone(), score));
        }

        if result.best_move.is_null() { break; }
        pos.make_move(result.best_move);
    }

    // If game didn't end naturally, call it a draw
    let outcome = outcome.unwrap_or(Outcome::Draw);

    if filter_draws && outcome == Outcome::Draw { return vec![]; }

    let wdl = outcome.wdl_white();

    // ── Build records ───────────────────────────────────────────────────────
    positions_in_game
        .into_iter()
        .filter_map(|(p, score)| Record::from_position(&p, score, wdl))
        .collect()
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    let cfg = Config::from_args();
    init_all();

    let total_written = Arc::new(AtomicU64::new(0));
    let writer = Arc::new(Mutex::new(
        BufWriter::new(File::create(&cfg.output).expect("cannot create output file"))
    ));

    println!("Specter NNUE data generation");
    println!("  depth:        {}", cfg.depth);
    println!("  target:       {}M positions", cfg.target / 1_000_000);
    println!("  threads:      {}", cfg.threads);
    println!("  output:       {}", cfg.output);
    println!("  random_plies: {}", cfg.random_plies);
    println!();

    let target   = cfg.target;
    let depth    = cfg.depth;
    let rp       = cfg.random_plies;
    let fd       = cfg.filter_draws;

    let handles: Vec<_> = (0..cfg.threads)
        .map(|thread_id| {
            let tw = Arc::clone(&total_written);
            let wr = Arc::clone(&writer);

            thread::spawn(move || {
                let mut games = 0u64;

                loop {
                    let already = tw.load(Ordering::Relaxed);
                    if already >= target { break; }

                    let records = play_game(depth, rp, fd);
                    let count   = records.len() as u64;

                    if count == 0 { continue; }

                    {
                        let mut w = wr.lock().unwrap();
                        for r in &records {
                            w.write_all(&r.as_bytes()).unwrap();
                        }
                    }

                    let new_total = tw.fetch_add(count, Ordering::Relaxed) + count;
                    games += 1;

                    if thread_id == 0 && games % 100 == 0 {
                        println!(
                            "  [{:.1}M / {}M] games={} thread={}",
                            new_total as f64 / 1_000_000.0,
                            target / 1_000_000,
                            games,
                            thread_id,
                        );
                    }

                    if new_total >= target { break; }
                }
            })
        })
        .collect();

    for h in handles { h.join().unwrap(); }

    let mut w = writer.lock().unwrap();
    w.flush().unwrap();

    println!();
    println!("Done. {} positions written to {}", total_written.load(Ordering::Relaxed), cfg.output);
}

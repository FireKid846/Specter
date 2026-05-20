/// UCI protocol handler — main loop that reads stdin and writes to stdout.
use std::io::{self, BufRead, Write};
use crate::board::position::Position;
use crate::movegen::attacks::init_all;
use crate::search::iterative_deepening::{search, format_info};
use crate::search::timeman::TimeManager;
use crate::tt::table::TranspositionTable;
use crate::uci::options::UciOptions;
use crate::uci::parser::{parse, parse_move, UciCommand};
use crate::board::color::Color;

pub fn run_uci_loop() {
    init_all();

    let mut opts   = UciOptions::default();
    let mut tt     = TranspositionTable::new_auto();
    let mut pos    = Position::startpos();

    let stdin  = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line.unwrap_or_default();
        let line = line.trim();
        if line.is_empty() { continue; }

        match parse(line) {
            UciCommand::Uci => {
                println!("id name Specter");
                println!("id author Specter Engine");
                println!("{}", UciOptions::uci_string());
                println!("uciok");
            }
            UciCommand::IsReady => {
                println!("readyok");
            }
            UciCommand::UciNewGame => {
                tt.clear();
                pos = Position::startpos();
            }
            UciCommand::Position { fen, moves } => {
                pos = Position::from_fen(&fen).unwrap_or_else(|_| Position::startpos());
                for mv_str in &moves {
                    if let Some(mv) = parse_move(&pos, mv_str) {
                        pos.make_move(mv);
                    }
                }
            }
            UciCommand::Go(params) => {
                let time = build_time_manager(&params, &pos, &opts);
                let result = search(&mut pos, &mut tt, time, Some(Box::new(|r| {
                    println!("{}", format_info(r));
                    let _ = io::stdout().flush();
                })));
                println!("bestmove {}", result.best_move);
                let _ = io::stdout().flush();
            }
            UciCommand::Stop  => {}
            UciCommand::Quit  => break,
            UciCommand::SetOption { name, value } => { opts.apply(&name, &value); }
            UciCommand::D     => { pos.print(); }
            UciCommand::Perft(depth) => {
                use crate::movegen::perft::perft_divide;
                perft_divide(&mut pos, depth);
            }
            UciCommand::Unknown(cmd) => {
                eprintln!("Unknown command: {}", cmd);
            }
        }
        let _ = io::stdout().flush();
    }
}

fn build_time_manager(params: &crate::uci::parser::GoParams, pos: &Position, opts: &UciOptions) -> TimeManager {
    if params.infinite { return TimeManager::infinite(); }
    if let Some(d) = params.depth { return TimeManager::fixed_depth(d); }
    if let Some(ms) = params.movetime { return TimeManager::fixed_time(ms - opts.move_overhead); }

    let (time_left, increment) = match pos.side {
        Color::White => (params.wtime.unwrap_or(60_000), params.winc.unwrap_or(0)),
        Color::Black => (params.btime.unwrap_or(60_000), params.binc.unwrap_or(0)),
    };

    TimeManager::tournament(
        time_left.saturating_sub(opts.move_overhead),
        increment,
        params.movestogo,
    )
}

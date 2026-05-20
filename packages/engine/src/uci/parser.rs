/// UCI command parser.
use crate::board::position::{Move, MoveFlag, Position};
use crate::board::square::Square;

#[derive(Debug)]
pub enum UciCommand {
    Uci,
    IsReady,
    UciNewGame,
    Position { fen: String, moves: Vec<String> },
    Go(GoParams),
    Stop,
    Quit,
    SetOption { name: String, value: String },
    D, // debug print board
    Perft(u32),
    Unknown(String),
}

#[derive(Debug, Default)]
pub struct GoParams {
    pub wtime:     Option<u64>,
    pub btime:     Option<u64>,
    pub winc:      Option<u64>,
    pub binc:      Option<u64>,
    pub movestogo: Option<u32>,
    pub depth:     Option<u32>,
    pub nodes:     Option<u64>,
    pub movetime:  Option<u64>,
    pub infinite:  bool,
}

pub fn parse(line: &str) -> UciCommand {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() { return UciCommand::Unknown(line.to_string()); }

    match parts[0] {
        "uci"        => UciCommand::Uci,
        "isready"    => UciCommand::IsReady,
        "ucinewgame" => UciCommand::UciNewGame,
        "stop"       => UciCommand::Stop,
        "quit" | "q" => UciCommand::Quit,
        "d"          => UciCommand::D,
        "perft"      => UciCommand::Perft(parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1)),
        "setoption"  => parse_setoption(&parts),
        "position"   => parse_position(&parts),
        "go"         => parse_go(&parts),
        _            => UciCommand::Unknown(line.to_string()),
    }
}

fn parse_setoption(parts: &[&str]) -> UciCommand {
    let name_idx  = parts.iter().position(|&s| s == "name").map(|i| i + 1).unwrap_or(2);
    let value_idx = parts.iter().position(|&s| s == "value").map(|i| i + 1).unwrap_or(parts.len());
    let name  = parts[name_idx..value_idx.saturating_sub(1)].join(" ");
    let value = parts[value_idx..].join(" ");
    UciCommand::SetOption { name, value }
}

fn parse_position(parts: &[&str]) -> UciCommand {
    let fen = if parts.get(1) == Some(&"startpos") {
        crate::board::position::STARTPOS_FEN.to_string()
    } else {
        let fen_start = parts.iter().position(|&s| s == "fen").unwrap_or(1) + 1;
        let fen_end   = parts.iter().position(|&s| s == "moves").unwrap_or(parts.len());
        parts[fen_start..fen_end].join(" ")
    };
    let moves_start = parts.iter().position(|&s| s == "moves").map(|i| i + 1).unwrap_or(parts.len());
    let moves = parts[moves_start..].iter().map(|s| s.to_string()).collect();
    UciCommand::Position { fen, moves }
}

fn parse_go(parts: &[&str]) -> UciCommand {
    let mut p = GoParams::default();
    let mut i = 1;
    while i < parts.len() {
        match parts[i] {
            "wtime"     => { p.wtime     = parts.get(i+1).and_then(|s| s.parse().ok()); i += 1; }
            "btime"     => { p.btime     = parts.get(i+1).and_then(|s| s.parse().ok()); i += 1; }
            "winc"      => { p.winc      = parts.get(i+1).and_then(|s| s.parse().ok()); i += 1; }
            "binc"      => { p.binc      = parts.get(i+1).and_then(|s| s.parse().ok()); i += 1; }
            "movestogo" => { p.movestogo = parts.get(i+1).and_then(|s| s.parse().ok()); i += 1; }
            "depth"     => { p.depth     = parts.get(i+1).and_then(|s| s.parse().ok()); i += 1; }
            "nodes"     => { p.nodes     = parts.get(i+1).and_then(|s| s.parse().ok()); i += 1; }
            "movetime"  => { p.movetime  = parts.get(i+1).and_then(|s| s.parse().ok()); i += 1; }
            "infinite"  => { p.infinite  = true; }
            _ => {}
        }
        i += 1;
    }
    UciCommand::Go(p)
}

/// Parse a UCI move string (e.g. "e2e4", "e7e8q") into a Move.
pub fn parse_move(pos: &Position, mv_str: &str) -> Option<Move> {
    use crate::movegen::legal::legal_moves;
    if mv_str.len() < 4 { return None; }
    let from = Square::from_str(&mv_str[0..2])?;
    let to   = Square::from_str(&mv_str[2..4])?;
    let promo = mv_str.chars().nth(4);

    // Find the matching legal move
    let mut pos_clone = pos.clone();
    legal_moves(&mut pos_clone).as_slice().iter()
        .find(|&&m| {
            m.from() == from && m.to() == to && match promo {
                Some('q') => m.flag() == MoveFlag::PromoteQueen  as u32,
                Some('r') => m.flag() == MoveFlag::PromoteRook   as u32,
                Some('b') => m.flag() == MoveFlag::PromoteBishop as u32,
                Some('n') => m.flag() == MoveFlag::PromoteKnight as u32,
                None      => !m.is_promotion(),
                _         => false,
            }
        })
        .copied()
}

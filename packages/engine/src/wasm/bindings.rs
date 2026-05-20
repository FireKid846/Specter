/// WASM bindings — exposes Specter to JavaScript via wasm-bindgen.
/// Build with: wasm-pack build packages/engine --target web --features wasm

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "wasm")]
use crate::board::position::Position;
#[cfg(feature = "wasm")]
use crate::movegen::attacks::init_all;
#[cfg(feature = "wasm")]
use crate::search::iterative_deepening::search;
#[cfg(feature = "wasm")]
use crate::search::timeman::TimeManager;
#[cfg(feature = "wasm")]
use crate::tt::table::TranspositionTable;
#[cfg(feature = "wasm")]
use crate::personality::elo::elo_to_params;
#[cfg(feature = "wasm")]
use crate::personality::skill::SkillParams;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct SpectorEngine {
    pos:   Position,
    tt:    TranspositionTable,
    skill: SkillParams,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl SpectorEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> SpectorEngine {
        init_all();
        SpectorEngine {
            pos:   Position::startpos(),
            tt:    TranspositionTable::new(32),
            skill: SkillParams::full_strength(),
        }
    }

    /// Set position from FEN string.
    #[wasm_bindgen(js_name = setPosition)]
    pub fn set_position(&mut self, fen: &str) -> Result<(), JsValue> {
        self.pos = Position::from_fen(fen)
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(())
    }

    /// Apply a UCI move string (e.g. "e2e4") to the current position.
    #[wasm_bindgen(js_name = makeMove)]
    pub fn make_move(&mut self, mv_str: &str) -> bool {
        use crate::uci::parser::parse_move;
        if let Some(mv) = parse_move(&self.pos, mv_str) {
            self.pos.make_move(mv);
            true
        } else {
            false
        }
    }

    /// Get the best move for the current position.
    /// time_ms: time limit in milliseconds (used when depth == 0).
    /// depth:   fixed depth (0 = use time_ms instead).
    /// ELO skill level is applied automatically if set via setElo().
    #[wasm_bindgen(js_name = getBestMove)]
    pub fn get_best_move(&mut self, time_ms: u32, depth: u32) -> String {
        // Determine effective depth: lower of user request and skill cap.
        let skill_depth = self.skill.depth;
        let effective_depth = match (depth, skill_depth) {
            (0, 0) => 0,          // no cap — use time
            (0, s) => s,          // skill cap only
            (d, 0) => d,          // user depth only
            (d, s) => d.min(s),   // take the tighter limit
        };

        let time = if effective_depth > 0 {
            TimeManager::fixed_depth(effective_depth)
        } else {
            TimeManager::fixed_time(time_ms as u64)
        };

        let result = search(&mut self.pos, &mut self.tt, time, None);

        // Blunder / mistake injection: play a random legal move instead.
        if self.skill.should_play_blunder() || self.skill.should_play_mistake() {
            use crate::movegen::legal::legal_moves;
            let moves = legal_moves(&mut self.pos);
            let count = moves.count;
            if count > 0 {
                // Pseudo-random pick derived from node count (avoids SystemTime in WASM).
                let idx = (result.nodes as usize) % count;
                return moves.as_slice()[idx].to_uci();
            }
        }

        result.best_move.to_uci()
    }

    /// Evaluate the current position (centipawns, side-to-move perspective).
    #[wasm_bindgen(js_name = evaluate)]
    pub fn eval(&self) -> i32 {
        crate::eval::evaluate(&self.pos)
    }

    /// Get the current FEN string.
    #[wasm_bindgen(js_name = getFen)]
    pub fn get_fen(&self) -> String {
        self.pos.to_fen()
    }

    /// Get all legal moves in UCI notation (space-separated).
    #[wasm_bindgen(js_name = getLegalMoves)]
    pub fn get_legal_moves(&mut self) -> String {
        use crate::movegen::legal::legal_moves;
        let moves = legal_moves(&mut self.pos);
        moves.as_slice().iter()
            .map(|m| m.to_uci())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Check if current position is check/checkmate/stalemate.
    #[wasm_bindgen(js_name = getGameState)]
    pub fn get_game_state(&mut self) -> String {
        use crate::movegen::legal::{is_in_check, is_checkmate, is_stalemate};
        if is_checkmate(&mut self.pos)  { return "checkmate".to_string(); }
        if is_stalemate(&mut self.pos)  { return "stalemate".to_string(); }
        if self.pos.is_repetition()     { return "repetition".to_string(); }
        if self.pos.is_fifty_move_draw(){ return "fifty-move".to_string(); }
        if is_in_check(&self.pos, self.pos.side) { return "check".to_string(); }
        "playing".to_string()
    }

    /// Reset to starting position and clear the TT.
    #[wasm_bindgen(js_name = reset)]
    pub fn reset(&mut self) {
        self.pos = Position::startpos();
        self.tt.clear();
    }

    /// Set ELO strength level (100–3200).
    /// Caps search depth and enables blunder/mistake injection.
    #[wasm_bindgen(js_name = setElo)]
    pub fn set_elo(&mut self, elo: u32) {
        self.skill = elo_to_params(elo);
    }

    /// Reset to full engine strength.
    #[wasm_bindgen(js_name = setFullStrength)]
    pub fn set_full_strength(&mut self) {
        self.skill = SkillParams::full_strength();
    }
}

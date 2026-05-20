pub mod builtin;
pub mod polyglot;

use crate::board::position::Position;
use crate::board::position::Move;
use crate::openingbook::builtin::BUILTIN_BOOK;
use crate::openingbook::polyglot::PolyglotBook;

pub struct OpeningBook {
    polyglot: Option<PolyglotBook>,
    enabled:  bool,
}

impl OpeningBook {
    pub fn new() -> Self {
        OpeningBook { polyglot: None, enabled: true }
    }

    pub fn load_polyglot(&mut self, path: &str) -> Result<(), String> {
        self.polyglot = Some(PolyglotBook::load(path)?);
        Ok(())
    }

    pub fn disable(&mut self) { self.enabled = false; }
    pub fn enable(&mut self)  { self.enabled = true; }

    /// Returns a book move for the current position, if any.
    pub fn probe(&self, pos: &Position) -> Option<Move> {
        if !self.enabled { return None; }
        // Polyglot takes priority over built-in
        if let Some(ref pg) = self.polyglot {
            if let Some(mv) = pg.probe(pos.hash) { return Some(mv); }
        }
        BUILTIN_BOOK.probe(pos.hash)
    }
}

impl Default for OpeningBook {
    fn default() -> Self { Self::new() }
}

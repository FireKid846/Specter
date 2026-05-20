/// UCI options — configurable engine settings exposed via "setoption".
use crate::personality::style::Style;

#[derive(Debug, Clone)]
pub struct UciOptions {
    pub hash_mb:        usize,   // Hash table size in MB
    pub threads:        usize,   // Number of search threads
    pub move_overhead:  u64,     // Extra time buffer in ms
    pub skill_level:    i32,     // 0-20, maps to ELO
    pub style:          Style,   // Playing personality
    pub blunder_rate:   f32,     // 0.0-1.0 probability of blunder
    pub opening_book:   bool,    // Use built-in opening book
    pub syzygy_path:    String,  // Path to Syzygy tablebase files
}

impl Default for UciOptions {
    fn default() -> Self {
        UciOptions {
            hash_mb:       64,
            threads:       1,
            move_overhead: 50,
            skill_level:   20,
            style:         Style::Balanced,
            blunder_rate:  0.0,
            opening_book:  true,
            syzygy_path:   String::new(),
        }
    }
}

impl UciOptions {
    pub fn apply(&mut self, name: &str, value: &str) {
        match name.to_lowercase().as_str() {
            "hash"          => { self.hash_mb       = value.parse().unwrap_or(64); }
            "threads"       => { self.threads        = value.parse().unwrap_or(1); }
            "moveoverhead"  => { self.move_overhead  = value.parse().unwrap_or(50); }
            "skilllevel"    => { self.skill_level    = value.parse().unwrap_or(20); }
            "blunderrate"   => { self.blunder_rate   = value.parse().unwrap_or(0.0); }
            "openingbook"   => { self.opening_book   = value == "true"; }
            "syzygypath"    => { self.syzygy_path    = value.to_string(); }
            "style"         => { self.style          = Style::from_str(value); }
            _ => {}
        }
    }

    pub fn uci_string() -> String {
        [
            "option name Hash type spin default 64 min 1 max 65536",
            "option name Threads type spin default 1 min 1 max 256",
            "option name MoveOverhead type spin default 50 min 0 max 5000",
            "option name SkillLevel type spin default 20 min 0 max 20",
            "option name BlunderRate type string default 0.0",
            "option name OpeningBook type check default true",
            "option name SyzygyPath type string default <empty>",
            "option name Style type combo default Balanced var Aggressive var Solid var Tactical var Tricky var Chaotic var Balanced",
        ].join("\n")
    }
}

/// Playing style — affects both eval weights and search tree shaping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Aggressive,  // Prefer attacking moves, sacrifices, complex positions
    Solid,       // Prefer positional moves, avoid risks
    Tactical,    // Seek combinations, forks, pins
    Tricky,      // Set traps, avoid main lines
    Chaotic,     // Unpredictable, random elements
    Balanced,    // Default — neutral
}

impl Style {
    pub fn from_str(s: &str) -> Style {
        match s.to_lowercase().as_str() {
            "aggressive" => Style::Aggressive,
            "solid"      => Style::Solid,
            "tactical"   => Style::Tactical,
            "tricky"     => Style::Tricky,
            "chaotic"    => Style::Chaotic,
            _            => Style::Balanced,
        }
    }

    /// Eval weight modifier for king safety (positive = value safety more).
    pub fn king_safety_weight(self) -> f32 {
        match self {
            Style::Aggressive => 0.7,   // Attack-first, less careful
            Style::Solid      => 1.4,   // Very careful with king
            Style::Tactical   => 0.9,
            Style::Tricky     => 0.8,
            Style::Chaotic    => 0.5,   // Reckless
            Style::Balanced   => 1.0,
        }
    }

    /// Eval weight modifier for mobility (positive = value mobility more).
    pub fn mobility_weight(self) -> f32 {
        match self {
            Style::Aggressive => 1.2,
            Style::Solid      => 0.9,
            Style::Tactical   => 1.3,
            Style::Tricky     => 1.0,
            Style::Chaotic    => 0.8,
            Style::Balanced   => 1.0,
        }
    }

    /// Move ordering modifier: how much to boost tactical moves.
    pub fn tactical_bonus(self) -> i32 {
        match self {
            Style::Aggressive => 30_000,
            Style::Tactical   => 40_000,
            Style::Tricky     => 20_000,
            Style::Chaotic    => 10_000,
            Style::Solid      => 0,
            Style::Balanced   => 0,
        }
    }

    /// Complexity preference: how much to prefer complex positions.
    /// Used to bias search toward/away from sharp lines.
    pub fn complexity_bias(self) -> i32 {
        match self {
            Style::Aggressive => 25,
            Style::Tactical   => 30,
            Style::Tricky     => 15,
            Style::Chaotic    => 40,
            Style::Solid      => -20,
            Style::Balanced   => 0,
        }
    }
}

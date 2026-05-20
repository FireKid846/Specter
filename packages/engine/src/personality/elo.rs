/// Maps ELO rating to engine parameters.
/// Specter can simulate any strength from 100 to 3200+ ELO.

use crate::personality::skill::SkillParams;

pub fn elo_to_params(elo: u32) -> SkillParams {
    let elo = elo.clamp(100, 3200);
    match elo {
        100..=500  => SkillParams { depth: 1, blunder_rate: 0.6,  mistake_rate: 0.3 },
        501..=800  => SkillParams { depth: 2, blunder_rate: 0.35, mistake_rate: 0.2 },
        801..=1000 => SkillParams { depth: 3, blunder_rate: 0.2,  mistake_rate: 0.15 },
        1001..=1200=> SkillParams { depth: 4, blunder_rate: 0.12, mistake_rate: 0.1 },
        1201..=1400=> SkillParams { depth: 5, blunder_rate: 0.07, mistake_rate: 0.08 },
        1401..=1600=> SkillParams { depth: 6, blunder_rate: 0.04, mistake_rate: 0.05 },
        1601..=1800=> SkillParams { depth: 7, blunder_rate: 0.02, mistake_rate: 0.03 },
        1801..=2000=> SkillParams { depth: 8, blunder_rate: 0.01, mistake_rate: 0.02 },
        2001..=2200=> SkillParams { depth: 9, blunder_rate: 0.005,mistake_rate: 0.01 },
        2201..=2500=> SkillParams { depth: 11,blunder_rate: 0.002,mistake_rate: 0.005},
        2501..=2800=> SkillParams { depth: 14,blunder_rate: 0.001,mistake_rate: 0.002},
        _          => SkillParams { depth: 0, blunder_rate: 0.0,  mistake_rate: 0.0 }, // Full strength
    }
}

/// Named skill levels that map to ELO ranges.
pub fn level_to_elo(level: &str) -> u32 {
    match level.to_lowercase().as_str() {
        "beginner"     => 400,
        "novice"       => 700,
        "casual"       => 1000,
        "intermediate" => 1300,
        "club"         => 1600,
        "advanced"     => 1900,
        "expert"       => 2200,
        "master"       => 2500,
        "grandmaster"  => 2800,
        "maximum"      => 3200,
        _              => 3200,
    }
}

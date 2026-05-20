/// Skill parameters — controls how strong Specter plays.
#[derive(Debug, Clone)]
pub struct SkillParams {
    /// Max search depth (0 = unlimited).
    pub depth:        u32,
    /// Probability of making a blunder (0.0–1.0).
    pub blunder_rate: f32,
    /// Probability of making a mistake (0.0–1.0).
    pub mistake_rate: f32,
}

impl SkillParams {
    pub fn full_strength() -> Self {
        SkillParams { depth: 0, blunder_rate: 0.0, mistake_rate: 0.0 }
    }

    /// Given a candidate move and its score vs the best score,
    /// decide whether to play it (simulating human imperfection).
    pub fn should_play_blunder(&self) -> bool {
        if self.blunder_rate <= 0.0 { return false; }
        rand_float() < self.blunder_rate as f64
    }

    pub fn should_play_mistake(&self) -> bool {
        if self.mistake_rate <= 0.0 { return false; }
        rand_float() < self.mistake_rate as f64
    }
}

/// Simple inline PRNG for skill variation (no external deps).
fn rand_float() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH)
        .unwrap_or_default().subsec_nanos() as u64;
    let x = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (x >> 11) as f64 / (1u64 << 53) as f64
}

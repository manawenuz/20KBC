/// Deterministic LCG-based RNG. Same seed → same sequence across all platforms.
/// Used to keep replays and lockstep multiplayer bit-exact.
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.state = self.state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    /// Returns a float in [0, 1).
    pub fn next_f32(&mut self) -> f32 {
        // Use top 24 bits for mantissa precision.
        let bits = (self.next_u64() >> 40) as u32;
        bits as f32 / (1u32 << 24) as f32
    }

    /// Returns a float in [min, max).
    pub fn next_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

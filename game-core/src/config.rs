/// Top-level configuration passed to `CSimulation::new`.
/// All rates are expressed per-tick (tick rate = 20 Hz, 50ms per tick).
#[derive(Clone, Debug)]
pub struct SimConfig {
    /// Map width in tiles.
    pub map_width: u32,
    /// Map height in tiles.
    pub map_height: u32,
    /// World-units per tile (pathfinding cell size).
    pub tile_size: f32,
    /// Ticks per second (fixed: 20).
    pub tick_rate: f32,
    /// Supply drained per tick when in supply range (1 supply / 10 s = 1/200 ticks).
    pub supply_drain_per_tick: f32,
    /// HP drained per tick when outside supply range (3× drain rate).
    pub hp_drain_per_tick: f32,
    /// HP healed per tick when supplied (1 HP / 10 s = 1/200 ticks).
    pub auto_heal_per_tick: f32,
    /// Radius around supply depot within which units are considered "supplied".
    pub supply_range: f32,
    /// RNG seed — determines all random events (GAIA waypoints, etc.).
    pub seed: u64,
}

impl Default for SimConfig {
    fn default() -> Self {
        const TICK_RATE: f32 = 20.0;
        const DRAIN_PERIOD_TICKS: f32 = 10.0 * TICK_RATE; // 10 s × 20 tps = 200 ticks

        Self {
            map_width: 64,
            map_height: 64,
            tile_size: 2.0,
            tick_rate: TICK_RATE,
            supply_drain_per_tick: 1.0 / DRAIN_PERIOD_TICKS,
            hp_drain_per_tick: 3.0 / DRAIN_PERIOD_TICKS,
            auto_heal_per_tick: 1.0 / DRAIN_PERIOD_TICKS,
            supply_range: 20.0,
            seed: 12345,
        }
    }
}

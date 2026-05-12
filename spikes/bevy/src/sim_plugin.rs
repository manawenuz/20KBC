use bevy::prelude::*;
use game_core::{CSimulation, SimConfig};

/// Holds the authoritative game simulation.
#[derive(Resource)]
pub struct GameSim(pub CSimulation);

/// Fractional tick accumulator — carries leftover real time between frames.
#[derive(Resource)]
pub struct SimTickAccum(pub f32);

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        let sim = CSimulation::new(SimConfig::default());
        app.insert_resource(GameSim(sim))
            .insert_resource(SimTickAccum(0.0))
            .add_systems(Update, tick_sim);
    }
}

/// Advance the simulation at a fixed 20 Hz (50 ms per tick), decoupled from
/// the render frame rate via an accumulator.
fn tick_sim(mut sim: ResMut<GameSim>, mut accum: ResMut<SimTickAccum>, time: Res<Time>) {
    const TICK_DT: f32 = 1.0 / 20.0; // 50 ms
    accum.0 += time.delta_secs();
    while accum.0 >= TICK_DT {
        sim.0.tick();
        accum.0 -= TICK_DT;
    }
}

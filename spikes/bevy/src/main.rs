mod camera_plugin;
mod day_night;
mod gaia_plugin;
mod hud_plugin;
mod input_plugin;
mod sim_plugin;
mod terrain_plugin;
mod unit_plugin;

use bevy::prelude::*;
use camera_plugin::RtsCameraPlugin;
use day_night::DayNightPlugin;
use gaia_plugin::GaiaPlugin;
use hud_plugin::HudPlugin;
use input_plugin::InputPlugin;
use sim_plugin::SimPlugin;
use terrain_plugin::TerrainPlugin;
use unit_plugin::UnitPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "20KBC — Bevy Spike".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            SimPlugin,
            TerrainPlugin,
            RtsCameraPlugin,
            UnitPlugin,
            InputPlugin,
            GaiaPlugin,
            HudPlugin,
            DayNightPlugin,
        ))
        .run();
}

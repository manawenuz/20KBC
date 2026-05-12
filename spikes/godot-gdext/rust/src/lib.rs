use godot::prelude::*;

mod camera_controller;
mod gaia_node;
mod hud;
mod sim_bridge;
mod terrain_node;
mod unit_node;

/// Extension entry point — called by Godot when the .gdextension is loaded.
struct SpikGodotGdext;

#[gdextension]
unsafe impl ExtensionLibrary for SpikGodotGdext {}

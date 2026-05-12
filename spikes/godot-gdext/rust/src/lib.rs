use godot::prelude::*;

mod camera_controller;
mod frame_time_label;
mod gaia_node;
mod hud;
mod selection_manager;
mod sim_bridge;
mod terrain_node;
mod resource_node_visual;
mod unit_node;

/// Extension entry point — called by Godot when the .gdextension is loaded.
struct SpikGodotGdext;

#[gdextension]
unsafe impl ExtensionLibrary for SpikGodotGdext {}

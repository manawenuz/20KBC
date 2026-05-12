use godot::prelude::*;

mod box_selector;
mod camera_controller;
mod damage_number;
mod day_night;
mod frame_time_label;
mod gaia_node;
mod hud;
mod resource_node_visual;
mod selection_manager;
mod sim_bridge;
mod sound_fx;
mod terrain_node;
mod unit_node;
mod unit_portrait;

/// Extension entry point — called by Godot when the .gdextension is loaded.
struct SpikGodotGdext;

#[gdextension]
unsafe impl ExtensionLibrary for SpikGodotGdext {}

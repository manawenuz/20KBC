use godot::prelude::*;

mod asset_registry;
mod blp;
mod box_selector;
mod building_node;
mod camera_controller;
mod damage_number;
mod datasource;
mod day_night;
mod frame_time_label;
mod gaia_node;
mod hud;
mod material_tint;
mod mdx;
mod resource_node_visual;
mod selection_manager;
mod sim_bridge;
mod sound_fx;
mod terrain_node;
mod unit_node;
mod unit_portrait;
mod team_color;
mod wc3_material;

/// Extension entry point — called by Godot when the .gdextension is loaded.
struct SpikGodotGdext;

#[gdextension]
unsafe impl ExtensionLibrary for SpikGodotGdext {}

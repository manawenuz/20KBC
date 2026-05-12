use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

use crate::sim_plugin::GameSim;
use crate::unit_plugin::{Selected, SimUnitId};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        // EguiPlugin may already be added; guard with has_plugin check.
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
        app.add_systems(Update, hud_system);
    }
}

fn hud_system(
    mut contexts: EguiContexts,
    sim: Res<GameSim>,
    selected_q: Query<&SimUnitId, With<Selected>>,
) {
    let ctx = contexts.ctx_mut();

    egui::Window::new("Resources")
        .anchor(egui::Align2::LEFT_TOP, [8.0, 8.0])
        .resizable(false)
        .show(ctx, |ui| {
            let (wood, stone) = sim.0.player_resources(0);
            ui.label(format!("Wood:  {wood}"));
            ui.label(format!("Stone: {stone}"));
            ui.separator();
            ui.label(format!("Tick:  {}", sim.0.tick));

            // Show HP for the single selected unit, if any.
            let selected_ids: Vec<_> = selected_q.iter().map(|s| s.0).collect();
            if selected_ids.len() == 1 {
                let uid = selected_ids[0];
                if let Some(unit) = sim.0.get_unit(uid) {
                    ui.separator();
                    ui.label(format!("HP: {:.0} / {:.0}", unit.hp, unit.max_hp));
                    ui.label(format!("Supply: {:.1}", unit.supply));
                }
            }
        });
}

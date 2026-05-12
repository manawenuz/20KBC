use godot::prelude::*;
use godot::classes::{Control, IControl, Label};

use crate::sim_bridge::SimBridge;

/// Heads-up display overlay showing player 0's current resource counts.
///
/// Wire up in Godot editor (or via `.tscn`):
/// 1. Add `GameHud` as a child of a `CanvasLayer`.
/// 2. Set `sim_bridge` export property to point at the `SimBridge` node.
/// 3. Add two `Label` children named "WoodLabel" and "StoneLabel".
///
/// The HUD polls `SimBridge` every visual frame (`_process`).  Resource
/// values change at most once per simulation tick (50 ms), so polling on
/// every frame is negligible overhead for label updates.
#[derive(GodotClass)]
#[class(base = Control)]
pub struct GameHud {
    /// Reference to the `SimBridge` node — set in the editor via `@export`.
    #[export]
    sim_bridge: Option<Gd<SimBridge>>,
    base: Base<Control>,
}

#[godot_api]
impl IControl for GameHud {
    fn init(base: Base<Control>) -> Self {
        Self {
            sim_bridge: None,
            base,
        }
    }

    fn process(&mut self, _delta: f64) {
        // Borrow sim_bridge immutably first to read values, then update labels.
        let (wood, stone) = if let Some(bridge) = &self.sim_bridge {
            (bridge.bind().get_wood(), bridge.bind().get_stone())
        } else {
            return;
        };

        // Update wood label.
        if let Some(mut label) = self
            .base()
            .get_node_or_null("WoodLabel")
            .and_then(|n| n.try_cast::<Label>().ok())
        {
            label.set_text(&format!("Wood: {wood}"));
        }

        // Update stone label.
        if let Some(mut label) = self
            .base()
            .get_node_or_null("StoneLabel")
            .and_then(|n| n.try_cast::<Label>().ok())
        {
            label.set_text(&format!("Stone: {stone}"));
        }
    }
}

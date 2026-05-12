use godot::prelude::*;
use game_core::{CSimulation, SimConfig, Order};

/// `SimBridge` is the Godot-facing wrapper around the pure-Rust `CSimulation`.
///
/// It is intended to be added as a Node child of Main (or registered as an
/// AutoLoad) so that GDScript and other Extension nodes can call into it.
///
/// Physics ticks at 20 Hz (`physics_ticks_per_second = 20` in project.godot),
/// matching the simulation's own 50 ms fixed step, so `_physics_process` maps
/// 1-to-1 with `sim.tick()`.
#[derive(GodotClass)]
#[class(base = Node)]
pub struct SimBridge {
    sim: Option<CSimulation>,
    base: Base<Node>,
}

#[godot_api]
impl INode for SimBridge {
    fn init(base: Base<Node>) -> Self {
        Self { sim: None, base }
    }

    fn ready(&mut self) {
        let config = SimConfig::default();
        self.sim = Some(CSimulation::new(config));
        godot_print!("SimBridge: simulation initialized");
    }

    /// Called by Godot at 20 Hz (physics_ticks_per_second = 20).
    /// One Godot physics tick == one simulation tick == 50 ms of game time.
    fn physics_process(&mut self, _delta: f64) {
        if let Some(sim) = &mut self.sim {
            sim.tick();
        }
    }
}

#[godot_api]
impl SimBridge {
    /// Issue a move order to the given unit.
    ///
    /// Called from GDScript when the player right-clicks on terrain:
    /// ```gdscript
    /// sim.issue_move_order(unit_id, hit.x, hit.z)
    /// ```
    #[func]
    pub fn issue_move_order(&mut self, unit_id: u32, x: f32, z: f32) {
        if let Some(sim) = &mut self.sim {
            // UnitId is a type alias for u32, so pass directly.
            sim.issue_order(
                unit_id,
                Order::Move {
                    target: glam::Vec2::new(x, z),
                },
            );
        }
    }

    /// Returns unit positions as a flat `Array<f32>`: [x0, z0, x1, z1, …].
    ///
    /// Using `Vector2` to pair (x, z) per unit keeps GDScript side simple:
    ///   ```gdscript
    ///   var positions: Array[Vector2] = sim.get_unit_positions()
    ///   for v in positions:
    ///       unit_nodes[i].position = Vector3(v.x, 0, v.y)
    ///   ```
    #[func]
    pub fn get_unit_positions(&self) -> Array<Vector2> {
        let mut arr = Array::new();
        if let Some(sim) = &self.sim {
            for unit in sim.iter_units() {
                if !unit.is_dead {
                    arr.push(Vector2::new(unit.pos.x, unit.pos.y));
                }
            }
        }
        arr
    }

    /// Wood owned by player 0.
    #[func]
    pub fn get_wood(&self) -> u32 {
        self.sim
            .as_ref()
            .and_then(|s| s.players.get(0))
            .map(|p| p.wood)
            .unwrap_or(0)
    }

    /// Stone owned by player 0.
    #[func]
    pub fn get_stone(&self) -> u32 {
        self.sim
            .as_ref()
            .and_then(|s| s.players.get(0))
            .map(|p| p.stone)
            .unwrap_or(0)
    }

    /// Total living unit count — used by GDScript to drive unit spawning.
    #[func]
    pub fn get_unit_count(&self) -> i64 {
        self.sim
            .as_ref()
            .map(|s| s.iter_units().filter(|u| !u.is_dead).count() as i64)
            .unwrap_or(0)
    }
}

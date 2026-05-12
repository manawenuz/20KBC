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
                    target: game_core::Vec2::new(x, z),
                },
            );
        }
    }

    /// Returns unit positions as a flat `Array<Vector2>`: [(x, z), …].
    /// Order matches `get_unit_ids()`.
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

    /// Returns the stable UnitIds of all living units, aligned with
    /// `get_unit_positions()`. Use this to key UnitNode dictionaries on the
    /// GDScript side so dead units leave gaps cleanly.
    #[func]
    pub fn get_unit_ids(&self) -> Array<i64> {
        let mut arr = Array::new();
        if let Some(sim) = &self.sim {
            for unit in sim.iter_units() {
                if !unit.is_dead {
                    arr.push(unit.id as i64);
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

    /// Returns GAIA entity positions as an `Array<Vector2>`: [(x, z), …].
    ///
    /// Only entities with hp > 0.0 are included.
    #[func]
    pub fn get_gaia_positions(&self) -> Array<Vector2> {
        let mut arr = Array::new();
        if let Some(sim) = &self.sim {
            for g in &sim.gaia {
                if g.hp > 0.0 {
                    arr.push(Vector2::new(g.pos.x, g.pos.y));
                }
            }
        }
        arr
    }

    /// Returns a flat Array<Vector3> where each element packs (id as f32, x, z).
    /// GDScript decodes: id = int(v.x), pos = Vector3(v.y, 0, v.z).
    /// Only non-depleted nodes are returned (amount > 0).
    #[func]
    pub fn get_resource_nodes(&self) -> Array<Vector3> {
        let mut arr = Array::new();
        if let Some(sim) = &self.sim {
            for n in sim.iter_resources() {
                if !n.is_depleted() {
                    arr.push(Vector3::new(n.id as f32, n.pos.x, n.pos.y));
                }
            }
        }
        arr
    }

    /// Returns kinds aligned with get_resource_nodes order: 1=Wood, 2=Stone.
    #[func]
    pub fn get_resource_kinds(&self) -> Array<i64> {
        let mut arr = Array::new();
        if let Some(sim) = &self.sim {
            for n in sim.iter_resources() {
                if !n.is_depleted() {
                    let kind_val = match &n.kind {
                        game_core::ResourceKind::Wood => 1_i64,
                        game_core::ResourceKind::Stone => 2_i64,
                    };
                    arr.push(kind_val);
                }
            }
        }
        arr
    }

    /// Order `unit_id` to gather from `node_id`. Forwards to CSimulation::issue_order.
    #[func]
    pub fn issue_gather_order(&mut self, unit_id: u32, node_id: u32) {
        if let Some(sim) = &mut self.sim {
            sim.issue_order(unit_id, Order::Gather { node: node_id });
        }
    }

    /// Returns the UnitId of the closest living unit within `radius` of (world_x, world_z).
    /// Returns -1 if no unit is within range.
    #[func]
    pub fn get_unit_at(&self, world_x: f32, world_z: f32, radius: f32) -> i64 {
        let target = game_core::Vec2::new(world_x, world_z);
        let radius_sq = radius * radius;
        self.sim
            .as_ref()
            .and_then(|s| {
                s.iter_units()
                    .filter(|u| !u.is_dead)
                    .filter(|u| u.pos.distance_squared(target) <= radius_sq)
                    .min_by(|a, b| {
                        a.pos
                            .distance_squared(target)
                            .partial_cmp(&b.pos.distance_squared(target))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|u| u.id as i64)
            })
            .unwrap_or(-1)
    }

    /// Serialize the input log to `path`. Called from main.gd on quit so
    /// every match leaves a replay.bin alongside the executable.
    #[func]
    pub fn save_replay(&self, path: GString) {
        if let Some(sim) = &self.sim {
            let p = path.to_string();
            sim.write_replay(&p);
        }
    }
}

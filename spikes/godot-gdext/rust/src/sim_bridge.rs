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

    /// Returns the current resource amount for the given node id.
    /// Returns -1 if the node is not found or has been depleted.
    #[func]
    pub fn get_resource_amount(&self, node_id: u32) -> i64 {
        self.sim
            .as_ref()
            .and_then(|s| s.iter_resources().find(|n| n.id == node_id))
            .map(|n| n.amount as i64)
            .unwrap_or(-1)
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

    /// Returns unit stats as `[hp, max_hp, supply, max_supply]` (Array<f64>).
    /// Returns an empty array if the unit doesn't exist.
    /// (Flat array used instead of Dictionary to avoid gdext generic constraints.)
    #[func]
    pub fn get_unit_stats(&self, unit_id: u32) -> Array<f64> {
        let mut arr = Array::new();
        if let Some(sim) = &self.sim {
            if let Some(u) = sim.iter_units().find(|u| u.id == unit_id) {
                arr.push(u.hp as f64);
                arr.push(u.max_hp as f64);
                arr.push(u.supply as f64);
                arr.push(u.max_supply as f64);
            }
        }
        arr
    }

    /// Spawn `count` extra workers near the depot for stress testing.
    /// Used from a debug binding (orchestrator will wire to a hotkey).
    #[func]
    pub fn spawn_workers(&mut self, count: u32) {
        if let Some(sim) = &mut self.sim {
            let depot = sim.players.get(0).and_then(|p| p.supply_depot)
                .unwrap_or(game_core::Vec2::ZERO);
            for i in 0..count {
                let angle = i as f32 * 0.6;
                let r = 5.0 + (i as f32 * 0.15);
                let offset = game_core::Vec2::new(angle.cos() * r, angle.sin() * r);
                sim.spawn_unit(0, depot + offset);
            }
        }
    }

    /// Issue a formation move order to a group of units.
    ///
    /// Distributes the units in concentric rings around the target point
    /// so they don't all clump at the same coordinate.
    #[func]
    pub fn issue_formation_move(&mut self, unit_ids: Array<i64>, x: f32, z: f32) {
        use game_core::formation::formation_positions;
        let center = game_core::Vec2::new(x, z);
        let n = unit_ids.len();
        let slots = formation_positions(center, n, 2.0);
        if let Some(sim) = &mut self.sim {
            for (i, raw) in unit_ids.iter_shared().enumerate() {
                let uid = raw as u32;
                let pos = slots.get(i).copied().unwrap_or(center);
                sim.issue_order(uid, game_core::Order::Move { target: pos });
            }
        }
    }

    /// Issue an attack order to the given unit.
    #[func]
    pub fn issue_attack_order(&mut self, attacker: u32, target: u32) {
        if let Some(sim) = &mut self.sim {
            sim.issue_order(attacker, game_core::Order::Attack { target });
        }
    }

    /// Returns the UnitId of the closest living hostile unit within `radius` of (world_x, world_z).
    /// Hostile means `owner != player`. Returns -1 if no hostile unit is within range.
    #[func]
    pub fn get_hostile_at(&self, world_x: f32, world_z: f32, player: u8, radius: f32) -> i64 {
        let pos = game_core::Vec2::new(world_x, world_z);
        let r2 = radius * radius;
        self.sim.as_ref().and_then(|s| {
            s.iter_units()
                .filter(|u| !u.is_dead && u.owner != player)
                .filter(|u| u.pos.distance_squared(pos) <= r2)
                .min_by(|a, b| a.pos.distance_squared(pos)
                    .partial_cmp(&b.pos.distance_squared(pos))
                    .unwrap_or(std::cmp::Ordering::Equal))
                .map(|u| u.id as i64)
        }).unwrap_or(-1)
    }

    /// Returns UnitIds of all living units whose world XZ position is inside
    /// the axis-aligned rectangle [min_x, max_x] × [min_z, max_z].
    #[func]
    pub fn get_units_in_rect(&self, min_x: f32, min_z: f32, max_x: f32, max_z: f32) -> Array<i64> {
        let mut arr = Array::new();
        if let Some(sim) = &self.sim {
            for unit in sim.iter_units() {
                if !unit.is_dead
                    && unit.pos.x >= min_x
                    && unit.pos.x <= max_x
                    && unit.pos.y >= min_z
                    && unit.pos.y <= max_z
                {
                    arr.push(unit.id as i64);
                }
            }
        }
        arr
    }

    /// Returns the current HP of the unit with the given id.
    /// Returns -1.0 if the unit is not found.
    #[func]
    pub fn get_unit_hp(&self, unit_id: u32) -> f32 {
        self.sim.as_ref()
            .and_then(|s| s.iter_units().find(|u| u.id == unit_id))
            .map(|u| u.hp).unwrap_or(-1.0)
    }
}

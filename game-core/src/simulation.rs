use glam::Vec2;
use std::io::Write as IoWrite;

use crate::config::SimConfig;
use crate::gaia::{CGaiaEntity, update_gaia};
use crate::orders::{InputEntry, Order};
use crate::pathfinding::GridPathfinder;
use crate::player::{CPlayer, PlayerId};
use crate::resource_node::{CResourceNode, ResourceKind, ResourceNodeId};
use crate::rng::DeterministicRng;
use crate::supply::update_supply;
use crate::unit::{BehaviorState, CUnit, GatherPhase, UnitId};

/// The central game simulation. Pure Rust, zero engine dependencies.
/// Tick rate: 20 Hz (50 ms per tick).
pub struct CSimulation {
    pub tick: u64,
    pub units: Vec<CUnit>,
    pub resource_nodes: Vec<CResourceNode>,
    pub players: Vec<CPlayer>,
    pub gaia: Vec<CGaiaEntity>,
    pub pathfinder: GridPathfinder,
    pub rng: DeterministicRng,
    pub pending_orders: Vec<(UnitId, Order)>,
    pub input_log: Vec<InputEntry>,
    pub config: SimConfig,
    next_unit_id: UnitId,
    next_node_id: ResourceNodeId,
    next_gaia_id: u32,
}

impl CSimulation {
    pub fn new(config: SimConfig) -> Self {
        let pathfinder = GridPathfinder::new(config.map_width, config.map_height, config.tile_size);
        let rng = DeterministicRng::new(config.seed);

        let mut sim = Self {
            tick: 0,
            units: Vec::new(),
            resource_nodes: Vec::new(),
            players: Vec::new(),
            gaia: Vec::new(),
            pathfinder,
            rng,
            pending_orders: Vec::new(),
            input_log: Vec::new(),
            config,
            next_unit_id: 1,
            next_node_id: 1,
            next_gaia_id: 1,
        };

        // Player 0 with depot at map center.
        let mut p0 = CPlayer::new(0);
        let half_w = sim.config.map_width as f32 * sim.config.tile_size * 0.5;
        let half_h = sim.config.map_height as f32 * sim.config.tile_size * 0.5;
        let depot = Vec2::new(half_w, half_h);
        p0.supply_depot = Some(depot);
        sim.players.push(p0);

        // Spawn 10 starter workers in a circle around the depot, radius 4.0 wu.
        const STARTER_WORKERS: u32 = 10;
        const SPAWN_RADIUS: f32 = 4.0;
        for i in 0..STARTER_WORKERS {
            let angle = i as f32 / STARTER_WORKERS as f32 * std::f32::consts::TAU;
            let offset = Vec2::new(angle.cos(), angle.sin()) * SPAWN_RADIUS;
            sim.spawn_unit(0, depot + offset);
        }

        // Resource nodes.
        let nodes = [
            (Vec2::new(depot.x + 15.0, depot.y + 5.0), 500u32),
            (Vec2::new(depot.x - 15.0, depot.y - 5.0), 500u32),
            (Vec2::new(depot.x + 5.0, depot.y + 20.0), 300u32),
        ];
        for (pos, amt) in nodes {
            sim.spawn_resource_node(ResourceKind::Wood, pos, amt);
        }
        sim.spawn_resource_node(ResourceKind::Stone, Vec2::new(depot.x - 5.0, depot.y + 15.0), 400);

        // One GAIA wolf near the map corner.
        let wolf_center = Vec2::new(20.0, 20.0);
        sim.spawn_gaia(wolf_center, wolf_center, 50.0);

        sim
    }

    // ── Spawning helpers ─────────────────────────────────────────────────────

    pub fn spawn_unit(&mut self, owner: PlayerId, pos: Vec2) -> UnitId {
        let id = self.next_unit_id;
        self.next_unit_id += 1;
        self.units.push(CUnit::new(id, owner, pos));
        id
    }

    pub fn spawn_resource_node(
        &mut self,
        kind: ResourceKind,
        pos: Vec2,
        amount: u32,
    ) -> ResourceNodeId {
        let id = self.next_node_id;
        self.next_node_id += 1;
        self.resource_nodes
            .push(CResourceNode::new(id, kind, pos, amount));
        id
    }

    pub fn spawn_gaia(
        &mut self,
        pos: Vec2,
        territory_center: Vec2,
        territory_radius: f32,
    ) -> u32 {
        let id = self.next_gaia_id;
        self.next_gaia_id += 1;
        self.gaia
            .push(CGaiaEntity::new(id, pos, territory_center, territory_radius));
        id
    }

    // ── Public interface ─────────────────────────────────────────────────────

    /// Queue an order and record it in the input log for replay.
    pub fn issue_order(&mut self, unit_id: UnitId, order: Order) {
        let player = self
            .units
            .iter()
            .find(|u| u.id == unit_id)
            .map(|u| u.owner)
            .unwrap_or(0);
        self.input_log.push(InputEntry {
            tick: self.tick,
            player,
            unit: unit_id,
            order: order.clone(),
        });
        self.pending_orders.push((unit_id, order));
    }

    pub fn iter_units(&self) -> impl Iterator<Item = &CUnit> {
        self.units.iter()
    }

    pub fn iter_resources(&self) -> impl Iterator<Item = &CResourceNode> {
        self.resource_nodes.iter()
    }

    /// Look up a single unit by id.
    pub fn get_unit(&self, id: UnitId) -> Option<&CUnit> {
        self.units.iter().find(|u| u.id == id)
    }

    /// Return (wood, stone) for a player.
    pub fn player_resources(&self, player_id: PlayerId) -> (u32, u32) {
        self.players
            .iter()
            .find(|p| p.id == player_id)
            .map_or((0, 0), |p| (p.wood, p.stone))
    }

    // ── Main tick ────────────────────────────────────────────────────────────

    /// Advance by one 50ms step.
    pub fn tick(&mut self) {
        self.tick += 1;
        self.apply_pending_orders();
        self.update_units();
        self.update_gaia_entities();
        self.update_supply_all();
        self.units.retain(|u| !u.is_dead);
    }

    // ── Private phases ───────────────────────────────────────────────────────

    fn apply_pending_orders(&mut self) {
        let orders: Vec<(UnitId, Order)> = std::mem::take(&mut self.pending_orders);
        for (uid, order) in orders {
            if let Some(unit) = self.units.iter_mut().find(|u| u.id == uid && !u.is_dead) {
                match order {
                    Order::Stop => unit.behavior = BehaviorState::Idle,
                    Order::Move { target } => {
                        let path = self.pathfinder.find_path(unit.pos, target);
                        unit.behavior =
                            BehaviorState::MovingTo { target, path, path_idx: 0 };
                    }
                    Order::Gather { node } => {
                        unit.behavior = BehaviorState::Gathering {
                            node,
                            phase: GatherPhase::MovingToNode,
                        };
                    }
                    Order::Attack { target } => {
                        unit.behavior =
                            BehaviorState::Attacking { target, cooldown: 0.0 };
                    }
                    Order::AttackMove { target } => {
                        let path = self.pathfinder.find_path(unit.pos, target);
                        unit.behavior =
                            BehaviorState::MovingTo { target, path, path_idx: 0 };
                    }
                }
            }
        }
    }

    fn update_units(&mut self) {
        for i in 0..self.units.len() {
            if self.units[i].is_dead {
                continue;
            }
            let behavior = self.units[i].behavior.clone();
            let next = match behavior {
                BehaviorState::Idle | BehaviorState::Dead => None,
                BehaviorState::MovingTo { target, path, path_idx } => {
                    tick_move(&mut self.units[i], target, path, path_idx)
                }
                BehaviorState::Gathering { node, phase } => tick_gather(
                    &mut self.units[i],
                    node,
                    phase,
                    &mut self.resource_nodes,
                    &mut self.players,
                    &self.pathfinder,
                ),
                BehaviorState::Attacking { target, cooldown } => {
                    tick_attack(i, target, cooldown, &mut self.units)
                }
            };
            if let Some(nb) = next {
                self.units[i].behavior = nb;
            }
        }
    }

    fn update_gaia_entities(&mut self) {
        for i in 0..self.gaia.len() {
            let mut entity = self.gaia[i].clone();
            update_gaia(&mut entity, &mut self.units, &mut self.rng);
            self.gaia[i] = entity;
        }
    }

    fn update_supply_all(&mut self) {
        for i in 0..self.units.len() {
            let owner = self.units[i].owner as usize;
            if owner < self.players.len() {
                let player = self.players[owner].clone();
                update_supply(&mut self.units[i], &player, &self.config);
            }
        }
    }

    // ── Replay ───────────────────────────────────────────────────────────────

    /// Serialize the input log to a simple binary file.
    ///
    /// Layout: `[u32 entry_count]` then for each entry:
    /// `[u64 tick][u8 player][u32 unit][u8 order_tag][payload]`
    pub fn write_replay(&self, path: &str) {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&(self.input_log.len() as u32).to_le_bytes());

        for e in &self.input_log {
            buf.extend_from_slice(&e.tick.to_le_bytes());
            buf.push(e.player);
            buf.extend_from_slice(&e.unit.to_le_bytes());
            match &e.order {
                Order::Stop => buf.push(0),
                Order::Move { target } => {
                    buf.push(1);
                    buf.extend_from_slice(&target.x.to_le_bytes());
                    buf.extend_from_slice(&target.y.to_le_bytes());
                }
                Order::Gather { node } => {
                    buf.push(2);
                    buf.extend_from_slice(&node.to_le_bytes());
                }
                Order::Attack { target } => {
                    buf.push(3);
                    buf.extend_from_slice(&target.to_le_bytes());
                }
                Order::AttackMove { target } => {
                    buf.push(4);
                    buf.extend_from_slice(&target.x.to_le_bytes());
                    buf.extend_from_slice(&target.y.to_le_bytes());
                }
            }
        }

        if let Ok(mut file) = std::fs::File::create(path) {
            let _ = file.write_all(&buf);
        }
    }
}

// ── Behavior tick functions ───────────────────────────────────────────────────
// Free functions avoid simultaneous mutable borrows of self.units.

fn move_unit_toward(unit: &mut CUnit, target: Vec2) {
    let delta = target - unit.pos;
    let dist = delta.length();
    if dist > CUnit::MOVE_SPEED {
        let dir = delta / dist;
        unit.pos += dir * CUnit::MOVE_SPEED;
        unit.facing = dir.y.atan2(dir.x);
    } else {
        unit.pos = target;
    }
}

fn tick_move(
    unit: &mut CUnit,
    target: Vec2,
    path: Vec<Vec2>,
    mut path_idx: usize,
) -> Option<BehaviorState> {
    if path.is_empty() {
        move_unit_toward(unit, target);
        if unit.pos.distance(target) < CUnit::MOVE_SPEED {
            return Some(BehaviorState::Idle);
        }
        return Some(BehaviorState::MovingTo { target, path, path_idx });
    }

    let wp = path[path_idx];
    move_unit_toward(unit, wp);

    if unit.pos.distance(wp) < CUnit::MOVE_SPEED * 1.5 {
        path_idx += 1;
        if path_idx >= path.len() {
            return Some(BehaviorState::Idle);
        }
    }
    Some(BehaviorState::MovingTo { target, path, path_idx })
}

fn tick_gather(
    unit: &mut CUnit,
    node_id: ResourceNodeId,
    phase: GatherPhase,
    resource_nodes: &mut Vec<CResourceNode>,
    players: &mut Vec<CPlayer>,
    pathfinder: &GridPathfinder,
) -> Option<BehaviorState> {
    let node_exists = resource_nodes
        .iter()
        .find(|n| n.id == node_id)
        .map_or(false, |n| !n.is_depleted());
    if !node_exists {
        return Some(BehaviorState::Idle);
    }

    match phase {
        GatherPhase::MovingToNode => {
            let node_pos = resource_nodes.iter().find(|n| n.id == node_id).unwrap().pos;
            move_unit_toward(unit, node_pos);
            if unit.pos.distance(node_pos) < 2.0 {
                Some(BehaviorState::Gathering {
                    node: node_id,
                    phase: GatherPhase::Harvesting {
                        ticks_remaining: CUnit::HARVEST_TICKS,
                    },
                })
            } else {
                None
            }
        }

        GatherPhase::Harvesting { ticks_remaining } => {
            if ticks_remaining == 0 {
                let node = resource_nodes.iter_mut().find(|n| n.id == node_id).unwrap();
                let is_wood = matches!(node.kind, ResourceKind::Wood);
                let taken = node.harvest(CUnit::HARVEST_AMOUNT);
                let owner = unit.owner as usize;
                if owner < players.len() {
                    if is_wood {
                        players[owner].wood += taken;
                    } else {
                        players[owner].stone += taken;
                    }
                }
                Some(BehaviorState::Gathering {
                    node: node_id,
                    phase: GatherPhase::ReturningToBase,
                })
            } else {
                Some(BehaviorState::Gathering {
                    node: node_id,
                    phase: GatherPhase::Harvesting {
                        ticks_remaining: ticks_remaining - 1,
                    },
                })
            }
        }

        GatherPhase::ReturningToBase => {
            let depot = players
                .get(unit.owner as usize)
                .and_then(|p| p.supply_depot);
            match depot {
                None => Some(BehaviorState::Idle),
                Some(d) => {
                    move_unit_toward(unit, d);
                    if unit.pos.distance(d) < 2.0 {
                        let np = resource_nodes.iter().find(|n| n.id == node_id).map(|n| n.pos);
                        match np {
                            Some(node_pos) => {
                                let path = pathfinder.find_path(unit.pos, node_pos);
                                Some(BehaviorState::MovingTo {
                                    target: node_pos,
                                    path,
                                    path_idx: 0,
                                })
                            }
                            None => Some(BehaviorState::Idle),
                        }
                    } else {
                        None
                    }
                }
            }
        }
    }
}

/// Attack behavior tick. Takes the attacker's index to avoid aliasing when
/// applying damage to another element in the same slice.
fn tick_attack(
    attacker_idx: usize,
    target_id: UnitId,
    mut cooldown: f32,
    units: &mut Vec<CUnit>,
) -> Option<BehaviorState> {
    let target_alive = units
        .iter()
        .find(|u| u.id == target_id)
        .map_or(false, |u| !u.is_dead);

    if !target_alive {
        return Some(BehaviorState::Idle);
    }

    let target_pos = units.iter().find(|u| u.id == target_id).unwrap().pos;
    let dist = units[attacker_idx].pos.distance(target_pos);

    if dist > CUnit::ATTACK_RANGE {
        let attacker_pos = units[attacker_idx].pos;
        let delta = target_pos - attacker_pos;
        let dir = delta / dist;
        units[attacker_idx].pos += dir * CUnit::MOVE_SPEED;
        units[attacker_idx].facing = dir.y.atan2(dir.x);
        return Some(BehaviorState::Attacking { target: target_id, cooldown });
    }

    cooldown -= 1.0;
    if cooldown <= 0.0 {
        if let Some(t) = units.iter_mut().find(|u| u.id == target_id) {
            t.hp -= CUnit::ATTACK_DAMAGE;
            if t.hp <= 0.0 {
                t.hp = 0.0;
                t.is_dead = true;
            }
        }
        cooldown = CUnit::ATTACK_COOLDOWN_TICKS;
    }

    Some(BehaviorState::Attacking { target: target_id, cooldown })
}

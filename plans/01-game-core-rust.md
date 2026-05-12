# Plan 01 — game-core Rust Crate

## Purpose

`game-core` is the heart of 20KBC. It is a **pure Rust library** with zero engine dependencies.
All game state and logic live here. Every engine spike links against it.

Inspired directly by WC3/Warsmash `CSimulation` → renderer separation (see `docs/WC3Analysis/06_game_simulation.md`).

---

## Crate Layout

```
game-core/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── simulation.rs      # CSimulation — the world
    ├── unit.rs            # CUnit + CUnitType (stats)
    ├── behaviors/
    │   ├── mod.rs         # CBehavior trait
    │   ├── idle.rs        # auto-acquire targets
    │   ├── move_to.rs     # pathfind + walk
    │   ├── gather.rs      # move → harvest → return
    │   ├── attack.rs      # melee attack + cooldown
    │   └── death.rs       # death anim → remove
    ├── pathfinding.rs     # Grid A* (8-directional)
    ├── player.rs          # CPlayer (resources, food, alliances)
    ├── resource_node.rs   # CResourceNode (wood/stone depot)
    ├── supply.rs          # supply drain / auto-heal
    ├── gaia.rs            # predator roam + territory attack
    ├── rng.rs             # deterministic LCG
    └── orders.rs          # OrderType enum, pending order queue
```

---

## Core Types

### CSimulation

```rust
pub struct CSimulation {
    pub tick: u64,                          // game turns elapsed
    pub units: Vec<CUnit>,
    pub resource_nodes: Vec<CResourceNode>,
    pub players: Vec<CPlayer>,
    pub gaia_entities: Vec<CGaiaEntity>,
    pub pathfinder: GridPathfinder,
    pub rng: DeterministicRng,
    pub pending_orders: Vec<(UnitId, Order)>,
}

impl CSimulation {
    pub fn new(config: SimConfig) -> Self;
    pub fn tick(&mut self);                 // advance one step (50ms)
    pub fn issue_order(&mut self, unit: UnitId, order: Order);
    pub fn iter_units(&self) -> impl Iterator<Item = &CUnit>;
    pub fn iter_resources(&self) -> impl Iterator<Item = &CResourceNode>;
}
```

**Tick rate:** 20Hz (50ms). Engine calls `sim.tick()` from a fixed-step timer.

### CUnit

```rust
pub struct CUnit {
    pub id: UnitId,
    pub unit_type: UnitTypeId,
    pub owner: PlayerId,
    pub pos: Vec2,              // world position
    pub facing: f32,            // radians
    pub hp: f32,
    pub max_hp: f32,
    pub supply: f32,            // current supply reserve (0..10)
    pub behavior: Box<dyn CBehavior>,
    pub attack_cooldown: f32,
    pub is_dead: bool,
}
```

### CBehavior trait

```rust
pub trait CBehavior: Send {
    fn update(&mut self, unit: &mut CUnit, sim: &mut CSimulation) -> Option<Box<dyn CBehavior>>;
    fn is_interruptible(&self) -> bool { true }
    fn order_id(&self) -> OrderId;
}
```

Returns `Some(next)` to transition behavior, `None` to continue.

### GridPathfinder

```rust
pub struct GridPathfinder {
    pub grid: Vec<bool>,        // passable cells (cell = 2 world-units)
    pub width: u32,
    pub height: u32,
}

impl GridPathfinder {
    pub fn find_path(&self, from: Vec2, to: Vec2) -> Vec<Vec2>;
    pub fn is_passable(&self, x: f32, y: f32) -> bool;
    pub fn set_blocked(&mut self, x: f32, y: f32, blocked: bool);
}
```

A* with diagonal movement (8 directions). Cell size = 2 world-units.

---

## Supply System

```rust
// supply.rs — called per unit per tick
pub fn update_supply(unit: &mut CUnit, player: &CPlayer, sim_config: &SimConfig) {
    let in_range = player.supply_depot_pos.map_or(false, |depot| {
        unit.pos.distance(depot) <= sim_config.supply_range
    });

    if in_range && player.food >= 1 {
        // drain supply from player food
        unit.supply -= sim_config.supply_drain_rate;   // 1 / (10s × 20tps) per tick
        if unit.supply < 0.0 { unit.supply = 0.0; }
    } else {
        // no supply: health drains at 3× rate
        unit.hp -= sim_config.supply_drain_rate * 3.0;
    }

    // auto-heal (only when supplied)
    if in_range && unit.supply > 0.0 {
        unit.hp = (unit.hp + sim_config.auto_heal_rate).min(unit.max_hp);
    }

    if unit.hp <= 0.0 {
        unit.is_dead = true;
    }
}
```

Rates (from `docs/08-units.md`):
- Supply drain: 1 per 10s → **1 / 200 ticks** per tick
- Health drain (no supply): 3× drain rate
- Auto-heal: 1 HP per 10s → **1 / 200 ticks** per tick

---

## GAIA Predator (Wolf)

```rust
pub struct CGaiaEntity {
    pub id: GaiaId,
    pub pos: Vec2,
    pub territory_center: Vec2,
    pub territory_radius: f32,     // 50 world-units (MVP)
    pub hp: f32,
    pub behavior: GaiaBehavior,
}

pub enum GaiaBehavior {
    Roaming { waypoint: Vec2, idle_ticks: u32 },
    Chasing { target: UnitId },
    Attacking { target: UnitId, cooldown: f32 },
    Returning,
}
```

Logic per tick:
1. If roaming: move toward waypoint (speed 1.5/s). Pick new waypoint every 5s.
2. If any worker enters `territory_radius` from `territory_center`: switch to Chasing.
3. If within melee range of target: switch to Attacking.
4. If target dies or leaves `territory_radius * 2`: switch to Returning.

---

## Order Types

```rust
pub enum Order {
    Stop,
    Move { target: Vec2 },
    Gather { node: ResourceNodeId },
    Attack { target: UnitId },
    AttackMove { target: Vec2 },
}
```

Orders are queued in `pending_orders` and resolved at the end of each tick (same as Warsmash `processPendingOrders()`).

---

## Deterministic RNG

```rust
pub struct DeterministicRng {
    state: u64,
}
impl DeterministicRng {
    pub fn new(seed: u64) -> Self;
    pub fn next_f32(&mut self) -> f32;   // [0, 1)
    pub fn next_range(&mut self, min: f32, max: f32) -> f32;
}
```

LCG: `state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)`.

This makes replays and multiplayer lockstep deterministic.

---

## Input Log (Replay)

Every `issue_order()` call is appended to an in-memory log:

```rust
pub struct InputEntry {
    pub tick: u64,
    pub player: PlayerId,
    pub order: Order,
}
```

Written to `replay.bin` on exit. Can be replayed by feeding orders back into `CSimulation` with same seed.

---

## Cargo.toml

```toml
[package]
name = "game-core"
version = "0.1.0"
edition = "2021"

[dependencies]
glam = "0.29"          # Vec2, Vec3 math

[dev-dependencies]
criterion = "0.5"      # benchmarks
```

No engine deps. No async. No threads. Pure synchronous logic.

---

## Benchmarks (targets)

| Scenario | Target |
|----------|--------|
| 50 units, 1000 ticks | < 5ms |
| A* pathfind 64×64 grid | < 1ms per call |
| 500 units, 10000 ticks (stress) | < 50ms |

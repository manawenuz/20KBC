# PRD-26 — Buildings in `game-core` + SimBridge accessor

## Goal

Add a minimal `CBuilding` representation to the simulation and 3 hardcoded
building instances around the depot. Expose them via `SimBridge` so the
Godot renderer can place models. **Buildings are purely visual at this
stage** — no combat, no production, no occupancy. Players walk through
them; that's fine for the MVP slice.

## Files you MAY create

- `game-core/src/building.rs` — `CBuilding` struct + `BuildingKind` enum

## Files you MAY modify

- `game-core/src/lib.rs` — add `pub mod building;` and re-export
  `CBuilding, BuildingKind`.
- `game-core/src/simulation.rs` — add `pub buildings: Vec<CBuilding>` to
  `CSimulation`, spawn 3 instances inside `CSimulation::new`, expose
  `spawn_building` helper. Do NOT touch `tick()` or any behavior code.
- `spikes/godot-gdext/rust/src/sim_bridge.rs` — add ONE `#[func]`:
  `get_buildings()` returning `Array<Vector4>` packed as
  `(kind as f32, x, z, rotation_radians)` per building.

## Files you MUST NOT touch

- `game-core/src/unit.rs`, `gaia.rs`, `behaviors/`, etc.
- `main.gd`, `Main.tscn`, `project.godot`
- Other Rust source

## Interface contract

```rust
// game-core/src/building.rs
use glam::Vec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildingKind {
    TownHall = 1,
    Keep = 2,
    Castle = 3,
}

#[derive(Clone, Debug)]
pub struct CBuilding {
    pub kind: BuildingKind,
    pub pos: Vec2,
    pub rotation: f32,  // radians, around Y / world-up
}

impl CBuilding {
    pub fn new(kind: BuildingKind, pos: Vec2, rotation: f32) -> Self {
        Self { kind, pos, rotation }
    }
}
```

```rust
// game-core/src/simulation.rs — add field + init
pub struct CSimulation {
    // ... existing fields ...
    pub buildings: Vec<CBuilding>,
}

// in CSimulation::new, after depot is computed and before workers spawn:
sim.buildings = vec![
    CBuilding::new(BuildingKind::TownHall, depot, 0.0),
    CBuilding::new(BuildingKind::Keep,    depot + Vec2::new(-25.0, -8.0), 0.3),
    CBuilding::new(BuildingKind::Castle,  depot + Vec2::new(28.0, 12.0), -0.4),
];
```

```rust
// sim_bridge.rs — append
#[func]
pub fn get_buildings(&self) -> Array<Vector4> {
    let mut arr = Array::new();
    if let Some(sim) = &self.sim {
        for b in &sim.buildings {
            arr.push(Vector4::new(b.kind as u32 as f32, b.pos.x, b.pos.y, b.rotation));
        }
    }
    arr
}
```

## Acceptance criteria

```bash
cd game-core && cargo build && cargo test
cd ../spikes/godot-gdext/rust && cargo build
```

- [ ] All builds clean
- [ ] Unit test in `building.rs` covers `BuildingKind` discriminant values (1, 2, 3)
- [ ] `git diff --stat` shows ≤ 4 files changed (new building.rs + lib.rs + simulation.rs + sim_bridge.rs)

## Out of scope

- Building hit points, ownership, attackability
- Building production (no peasant training yet)
- Path-blocking by buildings (sim doesn't avoid them yet)
- Texture / material variation per building

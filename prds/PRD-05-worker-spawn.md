# PRD-05 — 10-Worker Starter Pack

## Goal

Bump the starter worker count from 3 to 10 and spawn them in a tidy ring
around the depot so they're visible all at once when the camera first
loads. Also add a stone resource node so the HUD stone counter has
something to update.

## Context

`CSimulation::new` in `game-core/src/simulation.rs` spawns the starter
units and resources. Currently it spawns 3 workers in a short row and
three Wood nodes near the depot. The MVP success criterion is "Spawn 10
workers + 1 wolf" (plans/00-mvp-overview.md). The wolf already exists.

## Files you MAY create

(none)

## Files you MAY modify

- `game-core/src/simulation.rs` — only inside `CSimulation::new`. Do not
  change any other function. Do not change `tick_gather` or behavior code.

## Files you MUST NOT touch

- `spikes/**` — purely a game-core change
- Any other file under `game-core/` (config.rs, unit.rs, etc.)

## Interface contract

Inside `CSimulation::new`, replace the existing worker loop with:

```rust
// Spawn 10 starter workers in a circle around the depot, radius 4.0 wu.
const STARTER_WORKERS: u32 = 10;
const SPAWN_RADIUS: f32 = 4.0;
for i in 0..STARTER_WORKERS {
    let angle = i as f32 / STARTER_WORKERS as f32 * std::f32::consts::TAU;
    let offset = Vec2::new(angle.cos(), angle.sin()) * SPAWN_RADIUS;
    sim.spawn_unit(0, depot + offset);
}
```

And add one stone node alongside the three wood nodes:

```rust
sim.spawn_resource_node(ResourceKind::Stone, Vec2::new(depot.x - 5.0, depot.y + 15.0), 400);
```

## Acceptance criteria

From repo root:

```bash
cd game-core && cargo build && cd ..
cd spikes/godot-gdext/rust && cargo build
```

Both must succeed cleanly. Additionally:

- [ ] `cargo test -p game-core` still passes (run from repo root)
- [ ] `git diff --stat` shows exactly 1 file changed: `game-core/src/simulation.rs`
- [ ] The diff is small (under 20 added/removed lines)

## Out of scope

- Changing supply, HP, or movement constants
- Adding new resource kinds
- Wolf count changes

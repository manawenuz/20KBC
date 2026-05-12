# Handover — Bevy Spike

## Status: Compiles clean, not yet visually confirmed ⚠️

The crate builds with 0 errors. It has not been run yet — this should be the
first thing you do. Run it with:

```bash
cd /Users/manwe/CascadeProjects/20KBC
cargo run -p spike-bevy
```

Expected on first launch: flat terrain grid (green/brown checkerboard), 3 sandy
capsules near map center (workers), 1 red capsule (wolf), an egui window
top-left showing Wood / Stone / Tick.

---

## Project layout

```
spikes/bevy/
├── Cargo.toml          bevy 0.15 + bevy_egui 0.31 + game-core path dep
└── src/
    ├── main.rs         App::new() + all plugins
    ├── sim_plugin.rs   SimPlugin — GameSim resource, 20Hz accumulator tick
    ├── terrain_plugin.rs  64×64 vertex-colored flat mesh (grass/dirt)
    ├── camera_plugin.rs   RtsCameraPlugin — WASD pan + scroll zoom
    ├── unit_plugin.rs     UnitPlugin — spawn/sync entity per CUnit
    ├── input_plugin.rs    InputPlugin — left-click select, right-click move
    ├── gaia_plugin.rs     GaiaPlugin — spawn/sync wolf entity
    ├── hud_plugin.rs      HudPlugin — bevy_egui resource panel
    └── day_night.rs       DayNightPlugin — directional light cycle
```

---

## What is implemented

### SimPlugin (`sim_plugin.rs`)
- `GameSim(CSimulation)` resource initialized with `SimConfig::default()`
- `SimTickAccum(f32)` accumulator resource
- `tick_sim` system: adds `delta_secs()` each frame, calls `sim.tick()` for
  each full 50ms chunk — correctly handles multiple ticks per slow frame

### TerrainPlugin (`terrain_plugin.rs`)
- Builds a 64×64 grid mesh (128×128 world-units) programmatically
- Per-vertex `ATTRIBUTE_COLOR`: grass (0.3,0.6,0.2) / dirt (0.5,0.35,0.2)
  alternating by `(x+y) % 2`
- Uses `StandardMaterial` — Bevy 0.15 picks up `ATTRIBUTE_COLOR` automatically
  without a `vertex_colors` flag (that field was removed)

### RtsCameraPlugin (`camera_plugin.rs`)
- `Camera3d` at (64, 50, 80) looking toward map center
- WASD pan (20 units/s), scroll zoom (clamp Y 8..60)

### UnitPlugin (`unit_plugin.rs`)
- `SimUnitId(UnitId)` component, `Selected` marker component
- `SpawnedUnits(HashSet<UnitId>)` resource tracks what's been spawned
- `sync_units` system: iterates `sim.iter_units()`, spawns `Capsule3d` entities
  for new units, updates `Transform` each frame, despawns dead units
- Unit color: sandy brown

### InputPlugin (`input_plugin.rs`)
- `Selection(Vec<UnitId>)` resource
- `handle_selection`: left-click → ray → plane intersect at y=0 → find nearest
  unit within 1.5 units → insert `Selected`, populate `Selection`
- `handle_orders`: right-click → ray → plane intersect → `Order::Move` to all
  selected units via `sim.issue_order()`
- Uses Bevy 0.15 `camera.viewport_to_world()` which returns `Result` (fixed
  from the original `Option` — this was a compile error we already resolved)

### GaiaPlugin (`gaia_plugin.rs`)
- Syncs wolf entities from `sim.0.gaia` — same pattern as UnitPlugin
- Wolf mesh: red `Capsule3d`

### HudPlugin (`hud_plugin.rs`)
- `bevy_egui` `EguiPlugin` added with guard (`is_plugin_added` check)
- Top-left egui window: Wood, Stone, Tick counter
- Selected unit HP bar if one unit selected

### DayNightPlugin (`day_night.rs`)
- `DirectionalLight` with a time-based rotation over 600s period
- Illuminance lerp 15 000 (noon) → 500 (midnight)

---

## Known issues / things to verify on first run

1. **Terrain visibility** — the camera starts at (64, 50, 80). If the terrain
   grid (0..128 on X and Z) isn't in view, adjust the camera position in
   `camera_plugin.rs` or move the terrain spawn transform.

2. **Unit scale** — `Capsule3d::new(0.4, 1.4)` in Bevy 0.15 syntax. If units
   appear as flat discs or invisible, check the Bevy 0.15 `Capsule3d` constructor
   signature (radius, depth — not radius, height).

3. **bevy_egui window** — if the egui HUD doesn't appear, check that
   `EguiPlugin` is added before any system that reads `EguiContexts`. The guard
   in `HudPlugin::build` handles this but double-check if there's a conflict.

4. **Input deadzone** — `viewport_to_world` can return `Err` if the cursor is
   outside the viewport. The `let Ok(ray) = ... else { return }` pattern handles
   this correctly.

5. **Dynamic linking on macOS** — the `dynamic_linking` feature is enabled for
   fast dev builds. If you see dylib load errors, try:
   ```bash
   DYLD_FALLBACK_LIBRARY_PATH=$(rustup show home)/toolchains/stable-aarch64-apple-darwin/lib cargo run -p spike-bevy
   ```
   Or remove `features = ["dynamic_linking"]` from `Cargo.toml` for a static build.

---

## Immediate next tasks

1. **Run it** — confirm the scene renders and simulation is running.

2. **Fix unit movement visual** — after `handle_orders` issues a `Move` order,
   the simulation pathfinds and moves `CUnit.pos`. Verify `sync_units` is
   correctly copying `unit.pos.x` → `transform.translation.x` and
   `unit.pos.y` → `transform.translation.z` (sim Y is world Z in 3D).

3. **Selection highlight** — units with `Selected` component should show a
   visual indicator. Options: change material color, spawn a child ring mesh,
   or use Bevy's outline plugin.

4. **Resource node rendering** — add a `ResourcePlugin` that iterates
   `sim.iter_resources()` and spawns box meshes (brown=wood, grey=stone).

5. **Supply drain visual** — when a unit's `hp` drops below `max_hp`, show
   a health bar. Can use `bevy_egui` world-space UI or a billboard `Mesh2d`.

6. **Networking stub** — when ready to evaluate lockstep, wire up `bevy_replicon`
   + `renet`. The `input_log` written by `sim.write_replay()` is already
   structured for deterministic replay.

---

## game-core interface

```rust
// Resources available in any Bevy system:
sim: ResMut<GameSim>   // sim.0 is CSimulation

// Key CSimulation methods:
sim.0.tick()
sim.0.issue_order(unit_id: UnitId, order: Order)
sim.0.iter_units() -> impl Iterator<Item=&CUnit>
sim.0.iter_resources() -> impl Iterator<Item=&CResourceNode>
sim.0.player_resources(player_id: u8) -> (u32, u32)  // (wood, stone)
sim.0.tick  // u64 game tick counter
sim.0.gaia  // Vec<CGaiaEntity>

// CUnit fields:
unit.id: UnitId
unit.pos: Vec2        // sim XZ — map to world (x, 0, z)
unit.hp: f32
unit.max_hp: f32
unit.is_dead: bool
unit.owner: PlayerId
unit.behavior: BehaviorState

// Order variants:
Order::Move { target: Vec2 }
Order::Gather { node: ResourceNodeId }
Order::Attack { target: UnitId }
Order::Stop
```

---

## Architecture note

Bevy's ECS maps cleanly onto the RTS domain:
- Each `CUnit` → one Bevy `Entity` with `SimUnitId` component
- Simulation state lives in `GameSim` resource (authoritative)
- Renderer reads from `GameSim` resource and writes to `Transform` components
- No game logic runs in Bevy systems — all logic is in `game-core`

This separation means you can swap the rendering engine later with minimal
changes to `game-core`.

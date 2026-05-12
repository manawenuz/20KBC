# Handover — Fyrox Spike

## Status: Compiles clean (6 warnings), not yet visually confirmed ⚠️

The crate builds with 0 errors. Run it with:

```bash
cd /Users/manwe/CascadeProjects/20KBC
cargo run -p spike-fyrox
```

Expected on first launch: a window titled "20KBC — Fyrox Spike" with a large
flat grey plane (terrain), unit boxes spawning from the simulation, and a HUD
showing Wood / Stone / Tick in the top-left corner.

---

## Project layout

```
spikes/fyrox/
├── Cargo.toml          fyrox = "0.33" + game-core path dep
└── src/
    ├── main.rs         GamePlugin + GamePluginConstructor + fn main()
    ├── terrain.rs      create_terrain_mesh() — flat quad via SurfaceData::make_quad
    ├── units.rs        sync_units() — Handle<Node> map, spawn/update/remove
    ├── camera.rs       RtsCamera — WASD pan + scroll zoom (KeyInput struct)
    ├── hud.rs          GameHud — Fyrox retained UI, TextBuilder labels
    └── day_night.rs    DayNightCycle — directional light color lerp
```

---

## What is implemented

### GamePlugin (`main.rs`)
- Implements `Plugin` trait (Fyrox 0.33 API)
- `sim: CSimulation` owned directly in the plugin struct
- `sim_timer: f32` accumulator → `sim.tick()` every 50ms
- `unit_handles: HashMap<UnitId, Handle<Node>>` for scene sync
- `key_w/a/s/d: bool` tracked from `on_os_event`
- `scroll_delta: f32` accumulated from `MouseWheel` events, consumed each frame
- HUD updated from `sim.player_resources(0)` and `sim.tick` each frame

### Terrain (`terrain.rs`)
- Single flat quad scaled to 128×128 world-units centred at (64,0,64)
- Uses `SurfaceData::make_quad(&transform)` — the simplest Fyrox mesh primitive
- **No vertex colors yet** — uses default grey material
- To get checkerboard: replace with a manual mesh or a tiled multi-surface

### Unit sync (`units.rs`)
- `sync_units(scene, sim_units, unit_handles)` called every frame
- Spawns a 0.8×1.8×0.8 box (`SurfaceData::make_cube`) for new units
- Updates `local_transform_mut().set_position(...)` for live units
- Removes nodes for dead units via `scene.graph.remove_node(handle)`
- No vertex color tinting (API changed in 0.33 — see Known Issues)

### Camera (`camera.rs`)
- `RtsCamera { node: Handle<Node>, pan_speed: 20.0, zoom_speed: 15.0 }`
- Position: (64, 50, 80) looking at map center via `UnitQuaternion::face_towards`
- `update(scene, dt, keys, scroll)` called from `GamePlugin::update`
- `**node.local_transform().position()` double-deref required because
  `InheritableVariable<Vector3>` is not Copy (fixed compile error)

### HUD (`hud.rs`)
- `GameHud { wood_label, stone_label, tick_label: Handle<UiNode> }`
- Built in `GamePlugin::new` via `ctx.user_interface.build_ctx()`
- `update(ui, wood, stone, tick)` sends `TextMessage::text` to each label

### Day/Night (`day_night.rs`)
- `DayNightCycle { time, period: 600s, light_handle: Handle<Node> }`
- Modulates `DirectionalLight` color (RGB channels) to simulate illuminance
  (Fyrox 0.33 has no float `illuminance` property — fixed compile issue)

---

## Known issues

### Keyboard input stub
The `on_os_event` handler catches `WindowEvent::KeyboardInput` but the `input`
field was renamed to `event` in winit 0.28 (used by Fyrox 0.33). The current
code does `let _ = event;` — camera WASD pan does nothing.

**Fix**: In `main.rs`, replace the keyboard stub with:
```rust
WindowEvent::KeyboardInput { event, .. } => {
    let pressed = event.state == ElementState::Pressed;
    use fyrox::event::keyboard::{KeyCode, PhysicalKey};
    if let PhysicalKey::Code(code) = &event.physical_key {
        match code {
            KeyCode::KeyW => self.key_w = pressed,
            KeyCode::KeyA => self.key_a = pressed,
            KeyCode::KeyS => self.key_s = pressed,
            KeyCode::KeyD => self.key_d = pressed,
            _ => {}
        }
    }
}
```
Also import `ElementState` from `fyrox::event`. Adjust import path if the
compiler can't find `keyboard` sub-module — check what Fyrox 0.33.1 re-exports.

### No vertex colors on units
The `spawn_unit_box` function uses `make_cube` but Fyrox 0.33's vertex buffer
API (`push_vertex`, `iter_mut`) doesn't match the spec. Units render with the
default grey material.

**Fix**: Use a `MaterialResource` with a custom albedo color instead:
```rust
// After building the surface, create a material with the desired color.
// Fyrox material API varies by version — check fyrox::material::Material.
```

### `sync_units` dead code warning
The `GamePlugin::sync_units` method is defined but `update` calls
`units::sync_units` directly. Either remove the method or use it.

### Unused import warnings (6 total)
Harmless but worth cleaning: `OrthographicProjection`, `Projection` in camera,
`Vector3` in day_night, `UiMessage` in hud, `Order` in main, `mut ctx` in new.

---

## API gotchas for Fyrox 0.33.1

These were discovered during initial compilation:

| Issue | Correct 0.33 API |
|-------|-----------------|
| `register(&self, ctx: &mut PluginRegistrationContext)` | Takes by value: `ctx: PluginRegistrationContext<'_>` |
| `on_ui_message(..., _ui: &mut UserInterface)` | Only 3 params: `(&mut self, ctx, msg)` |
| `SurfaceSharedData::new(data, bool)` | One arg: `SurfaceSharedData::new(data)` |
| `SurfaceData::make_cube(SurfaceSharedData::new())` | `make_cube` takes `Matrix4`: `make_cube(Matrix4::identity())` |
| `WindowAttributes { title, ..Default::default() }` | Private fields — mutate from default: `let mut a = WindowAttributes::default(); a.title = ...` |
| `EventLoop::new()` returns `EventLoop<T>` | Returns `Result` — need `.expect(...)` |
| `*node.local_transform().position()` | `**` (double deref): `**node.local_transform().position()` |
| `VirtualKeyCode` at `fyrox::event::VirtualKeyCode` | Moved/removed in winit 0.28 |

---

## Immediate next tasks

1. **Run it** — confirm the window opens and the scene renders.

2. **Fix WASD camera** — replace the keyboard stub (see Known Issues above).
   Without this, you can't navigate the scene at all.

3. **Terrain checkerboard** — replace `make_quad` with a tiled mesh.
   Simplest approach: build N×N quads with alternating materials:
   ```rust
   let grass_mat = /* green material resource */;
   let dirt_mat  = /* brown material resource */;
   // For each tile, make_quad with the right material
   ```

4. **Unit tinting** — use `fyrox::material::Material` with `albedo_color` set
   per unit owner instead of vertex colors.

5. **Resource node rendering** — iterate `sim.iter_resources()`, spawn small
   box meshes (brown/grey by kind).

6. **Networking** — Fyrox has no built-in networking. Evaluate adding `renet`
   (UDP transport) + a custom lockstep loop reading from `sim`'s input log.
   This is the biggest risk factor for choosing Fyrox.

---

## game-core interface

```rust
// GamePlugin.sim is CSimulation — call these in update():
self.sim.tick()
self.sim.issue_order(unit_id: UnitId, order: Order)
self.sim.iter_units()     // Iterator<Item=&CUnit>
self.sim.iter_resources() // Iterator<Item=&CResourceNode>
self.sim.player_resources(0)  // (wood: u32, stone: u32)
self.sim.tick             // u64

// CUnit fields:
unit.id: UnitId (u32)
unit.pos: Vec2            // map to Vector3(x, 0, y) in Fyrox
unit.hp / max_hp: f32
unit.is_dead: bool
unit.owner: PlayerId (u8)

// Order variants:
Order::Move { target: Vec2 }
Order::Gather { node: ResourceNodeId }
Order::Attack { target: UnitId }
```

---

## Fyrox vs other spikes — evaluation notes

**Advantage**: Built-in scene editor (Fyroxed). Run with:
```bash
cargo install fyrox-template
fyrox-template init --name spike-fyrox --style 3d
# or open the project in Fyroxed directly
```
The editor is Fyrox's main differentiator from Bevy. Evaluate it for building
the command card / minimap UI before deciding.

**Risk**: No networking library. Building lockstep from scratch on top of `renet`
is weeks of work. This is the primary reason Godot+gdext or Bevy may be
preferred for a 2-person team targeting 20-30 player survival mode.

# Handover — Godot 4 + gdext Spike

## Status: Running ✅

The spike launched successfully. Godot 4.6.2 renders the terrain and 3 worker
units driven by the live `game-core` simulation at 20 Hz.

### What works
- GDExtension loads (`Initialize godot-rust` + `SimBridge: simulation initialized` in log)
- Flat terrain quad renders (grey — vertex colors need a shader, see below)
- 3 worker units appear as brown capsules in the scene center
- `SimBridge._physics_process` ticks `CSimulation` at 20 Hz (Godot physics = 20 tps)
- HUD labels `Wood: 0 / Stone: 0` visible top-left
- Right-click on terrain issues a Move order to unit 0 via `sim.issue_move_order`

### What doesn't work yet
- **Terrain vertex colors**: The `ArrayMesh` carries color data in `ARRAY_COLOR`
  but the default `StandardMaterial3D` ignores vertex colors in Forward+.
  Fix: add a minimal shader or switch to `BaseMaterial3D` with
  `vertex_color_use_as_albedo = true`.
- **Unit position sync**: `main.gd` calls `sim.get_unit_positions()` and updates
  `Node3D.position` every physics tick. Units are not moving yet because the
  GDScript loop uses index `i` as the key, not `UnitId`. Wire up unit IDs properly.
- **Camera pan**: `RtsCameraController` reads `Input.is_action_pressed("move_*")`
  — the input actions need to be defined in Godot's Input Map (Project Settings →
  Input Map → add `move_forward`, `move_back`, `move_left`, `move_right`).
  Alternatively, switch to polling `Input.is_key_pressed(KEY_W)` etc. directly.
- **Wolf / GAIA**: No `GaiaNode` type exists yet — the wolf is in the simulation
  but has no visual representation.
- **Resource nodes**: Not yet rendered.
- **HUD live update**: `GameHud` Rust type is registered but `main.gd` doesn't
  call into it — update the Labels from GDScript using `sim.get_wood()` /
  `sim.get_stone()`.

---

## Project layout

```
spikes/godot-gdext/
├── project.godot                   Godot 4 project (config_version=5, physics=20Hz)
├── spike-godot-gdext.gdextension   Extension manifest — points to the compiled dylib
├── .godot/
│   └── extension_list.cfg          Tells Godot to load spike-godot-gdext.gdextension
├── scenes/
│   └── Main.tscn                   Root scene — uses GDExtension node types directly
├── scripts/
│   └── main.gd                     GDScript bootstrap: unit sync + right-click orders
├── rust/
│   ├── Cargo.toml                  cdylib crate; game-core path = ../../../game-core
│   └── src/
│       ├── lib.rs                  ExtensionLibrary entry point
│       ├── sim_bridge.rs           SimBridge: wraps CSimulation, exposes to GDScript
│       ├── terrain_node.rs         TerrainNode: builds ArrayMesh terrain in ready()
│       ├── unit_node.rs            UnitNode: capsule mesh, #[var] unit_id
│       ├── camera_controller.rs    RtsCameraController: WASD pan + scroll zoom
│       └── hud.rs                  GameHud: Control node with resource labels
└── target/                         (gitignored) compiled artifacts
```

---

## How to build and run

```bash
# 1. Build the Rust extension (from project root)
cd spikes/godot-gdext/rust && cargo build

# 2. Launch the game
godot --path /Users/manwe/CascadeProjects/20KBC/spikes/godot-gdext/
```

Godot 4.6+ is required (`brew install --cask godot`).
Rust ≥ 1.94 is required (gdext 0.5.2 master). Run `rustup update stable`.

The compiled dylib lives at:
`spikes/godot-gdext/target/debug/libspike_godot_gdext.dylib`
(workspace root = `spikes/godot-gdext/`, not `rust/` — that's why the path is `target/`
not `rust/target/`).

---

## Key design decisions

**SimBridge as a Node, not a singleton.**
`SimBridge` is placed in the scene tree and found via `$SimBridge` in GDScript.
If you want it as a true Godot autoload singleton, register it in `project.godot`
under `[autoload]` and expose a `get_singleton()` Rust function.

**Physics process at 20 Hz.**
`project.godot` sets `physics/common/physics_ticks_per_second = 20`.
`SimBridge._physics_process()` calls `sim.tick()` exactly once per call, so the
simulation and render are in sync at 20 Hz. If you want render at 60 Hz + sim at
20 Hz, switch to `_process` + a float accumulator (see how spike-bevy does it).

**`gdext_rust_init` entry point.**
The `#[gdextension]` macro on `SpikGodotGdext` generates this symbol.
It matches `entry_symbol = "gdext_rust_init"` in the `.gdextension` file.

---

## Immediate next tasks

1. **Terrain vertex colors** — in `terrain_node.rs` `build_terrain_mesh()`,
   add a material with vertex color support:
   ```rust
   // After building the ArrayMesh:
   let mut mat = StandardMaterial3D::new_gd();
   mat.set_flag(BaseMaterial3DFlags::USE_POINT_SIZE, false);
   // Godot 4: vertex_color_use_as_albedo
   mat.set_flag(BaseMaterial3DFlags::ALBEDO_FROM_VERTEX_COLOR, true);
   self.base_mut().set_surface_override_material(0, &mat);
   ```
   Or use a ShaderMaterial with a trivial vertex-color shader.

2. **Unit movement** — units are in the sim but positions don't update in the
   GDScript loop because unit IDs aren't tracked. Fix `main.gd` `_sync_units()`
   to key by `UnitId` from `sim.get_unit_ids()` (add that method to `SimBridge`).

3. **Input map** — add WASD actions in `project.godot`:
   ```ini
   [input]
   move_forward={ "deadzone": 0.2, "events": [{"type":"key","keycode":87}] }
   move_back={    "deadzone": 0.2, "events": [{"type":"key","keycode":83}] }
   move_left={    "deadzone": 0.2, "events": [{"type":"key","keycode":65}] }
   move_right={   "deadzone": 0.2, "events": [{"type":"key","keycode":68}] }
   ```
   Key codes: W=87, A=65, S=83, D=68.

4. **Wolf / GAIA rendering** — add `get_gaia_positions()` to `SimBridge`,
   spawn `GaiaNode` (red capsule) in GDScript the same way as `UnitNode`.

5. **Resource node rendering** — add `get_resource_positions()` to `SimBridge`,
   spawn box meshes from GDScript.

---

## game-core interface (what SimBridge exposes)

```rust
// Current SimBridge @func methods callable from GDScript:
fn issue_move_order(&mut self, unit_id: u32, x: f32, z: f32)
fn get_unit_positions(&self) -> Array<Vector2>   // [(x,z) per living unit]
fn get_wood(&self) -> u32
fn get_stone(&self) -> u32
```

`CSimulation` on the Rust side also has:
- `iter_units()` → iterator over `CUnit` (pos, hp, behavior, owner, is_dead)
- `iter_resources()` → iterator over `CResourceNode` (pos, kind, amount)
- `gaia` field → `Vec<CGaiaEntity>` (pos, territory, behavior)
- `tick` field → `u64` (current game tick)
- `player_resources(id)` → `(wood, stone)`

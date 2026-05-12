# Plan 02 — Godot 4 + gdext Spike

## Goal

Validate that Godot 4 + Rust (gdext) can serve as the rendering layer for 20KBC's game-core.

---

## Project Structure

```
spikes/godot-gdext/
├── Cargo.toml          # workspace, members: [rust/]
├── rust/
│   ├── Cargo.toml      # gdext extension crate
│   └── src/
│       ├── lib.rs
│       ├── sim_bridge.rs   # wraps CSimulation, exposes to Godot
│       ├── unit_node.rs    # Node3D GDExtension class for units
│       ├── terrain.rs      # terrain mesh generation
│       └── hud.rs          # Control node for HUD
├── project.godot
├── scenes/
│   ├── Main.tscn
│   ├── Terrain.tscn
│   ├── Unit.tscn
│   └── HUD.tscn
├── scripts/            # GDScript stubs (minimal, only for editor wiring)
└── assets/
    └── placeholder/    # colored material overrides
```

---

## Implementation Steps

### Step 1: Project Scaffold
- `godot4 --headless --quit` to verify Godot installation
- Create `project.godot` with rendering/3D settings
- Set up `rust/Cargo.toml` with `gdext` + `game-core` path dep

### Step 2: Terrain Node
- Generate a flat mesh: 64×64 quads, tile size 2.0 world-units
- Alternate grass/dirt colors via vertex colors (no textures in spike)
- Grid lines optional overlay

### Step 3: RTS Camera
- `Camera3D` node, 60° tilt, pointing northwest
- Pan: WASD + edge scroll (when cursor near viewport edge)
- Zoom: scroll wheel, clamp 10..80 units height

### Step 4: SimBridge + Unit Nodes
- `SimBridge` holds `CSimulation`
- Each frame: call `sim.tick()` once per 50ms (Godot `_physics_process` at 20Hz)
- Spawn `UnitNode3D` for each unit ID on first tick
- Update `UnitNode3D.position` from `sim.iter_units()` each tick
- Unit mesh: cylinder placeholder (radius 0.4, height 1.8)
- Selection circle: flat ring mesh under unit, visible when selected

### Step 5: Input → Orders
- Raycast mouse → terrain for move destination
- `InputEventMouseButton::LEFT` → select unit(s)
- Box selection: track drag rect, select units inside screen-space box
- `InputEventMouseButton::RIGHT` on terrain → `Order::Move`
- `InputEventMouseButton::RIGHT` on resource node → `Order::Gather`

### Step 6: Resource Nodes
- Stone/wood nodes: box mesh (0.8×0.8×0.8), colored brown/grey
- Depletion: shrink mesh scale as resource amount decreases

### Step 7: Supply + GAIA
- Wolf: capsule mesh, red color
- No-supply units: flash red, health bar drains
- Health bar: 3D billboard `MeshInstance3D` with `QuadMesh`, shader-driven fill

### Step 8: Day/Night
- `DirectionalLight3D` — rotate from east to west over 10 min
- Ambient energy: lerp 0.8 (day) → 0.2 (night) over cycle
- Sky color: lerp light blue → dark blue → dawn orange

### Step 9: HUD
- `CanvasLayer` with `Control` nodes
- Wood counter: `Label`, updates from `sim.player.wood`
- Stone counter: `Label`, updates from `sim.player.stone`
- Selected unit panel: name + HP bar

---

## gdext Integration Notes

- Extension class registration: `#[derive(GodotClass)] #[class(base=Node)]`
- Singletons: `SimBridge` registered as Godot autoload
- gdext v0.5 removed mutex from `Gd<T>` — thread access is safe from main thread
- Build: `cargo build --release`, copy `.dylib` to `project/`, register in `.gdextension` file

## Key Godot APIs Needed

| Feature | Godot API |
|---------|-----------|
| Fixed timestep | `_physics_process(delta)` at Engine.physics_ticks_per_second=20 |
| Terrain mesh | `ArrayMesh` built in Rust, added to `MeshInstance3D` |
| Raycast | `PhysicsDirectSpaceState3D.intersect_ray()` |
| Camera unproject | `Camera3D.unproject_position()` for box selection |
| Canvas HUD | `CanvasLayer` > `Control` > `Label` |

---

## Success Criteria

- [ ] Terrain renders (64×64 grid, colored tiles)
- [ ] Camera pans and zooms
- [ ] Workers spawn, move to clicked target
- [ ] Workers gather from resource node, increment counter
- [ ] Wolf patrols, attacks nearby workers
- [ ] Supply drain kills unsupplied workers
- [ ] Day/night cycle visible
- [ ] 50 units maintain 60fps (check Godot profiler)
- [ ] `replay.bin` written on exit

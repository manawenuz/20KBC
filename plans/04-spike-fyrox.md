# Plan 04 — Fyrox Spike

## Goal

Validate Fyrox 1.0 as the rendering layer. Fyrox has a built-in editor (Fyroxed) which could accelerate UI iteration — the main thing to evaluate vs Bevy's code-only approach.

---

## Project Structure

```
spikes/fyrox/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── sim_bridge.rs    # owns CSimulation, ticked from game loop
    ├── terrain.rs       # procedural terrain mesh
    ├── units.rs         # unit handle map, sync positions
    ├── camera.rs        # RTS camera controller
    ├── input.rs         # selection, orders
    ├── hud.rs           # Fyrox UI widgets
    └── day_night.rs     # directional light cycle
```

---

## Key Differences vs Bevy

| | Bevy | Fyrox |
|---|---|---|
| Architecture | ECS | Scene graph (handles) |
| Editor | None | Fyroxed (built-in) |
| Networking | Replicon + Renet | Must build from scratch |
| Scene setup | Code only | Editor + code |
| Plugin system | Bevy plugins | Game plugins (trait) |

---

## Implementation Steps

### Step 1: Game Loop Scaffold

```rust
// main.rs
fn main() {
    Framework::new()
        .unwrap()
        .title("20KBC — Fyrox Spike")
        .run(|framework| {
            Box::new(Game::new(framework))
        });
}

struct Game {
    sim: CSimulation,
    sim_timer: f32,
    unit_handles: HashMap<UnitId, Handle<Node>>,
    scene: Handle<Scene>,
}
```

### Step 2: Terrain

```rust
// Build SurfaceData for 64×64 tile grid
let mut surface_data = SurfaceData::new(SurfaceSharedData::new(
    GeometryBuffer::from_surface_data(&build_terrain_verts(64, 64, 2.0), ...),
), vec![]);
```

Vertex colors: grass=green, dirt=brown, stone=grey (deterministic by tile hash).

### Step 3: RTS Camera

- `PivotBuilder` + `CameraBuilder` in Fyrox scene
- Script component on camera node for pan/zoom input

### Step 4: Unit Sync

```rust
fn update_units(&mut self, scene: &mut Scene) {
    for unit in self.sim.iter_units() {
        let handle = self.unit_handles.entry(unit.id).or_insert_with(|| {
            spawn_unit_node(scene, unit.unit_type)
        });
        let node = &mut scene.graph[*handle];
        node.local_transform_mut().set_position(
            Vector3::new(unit.pos.x, 0.0, unit.pos.y)
        );
    }
}
```

Units: cylinder mesh, colored by player.

### Step 5: Input + Orders

- Fyrox pick ray: `scene.graph.ray_cast(ray, ...)`
- Selection drag rect via `UserInterface` canvas
- Orders: same as other spikes

### Step 6: HUD

- Fyrox built-in UI system (XML-like builder or code)
- `TextBuilder` for resource counters
- `ProgressBarBuilder` for health bars

```rust
// In Game::new, build UI widgets:
let wood_label = TextBuilder::new(WidgetBuilder::new().with_margin(...))
    .with_text("Wood: 0")
    .build(&mut ui);
```

### Step 7: Day/Night

- `DirectionalLightBuilder` in scene
- Update `color` + `intensity` each frame via time-based lerp

---

## Fyrox-Specific Notes

- **Fyroxed editor**: Launch with `cargo run --package editor` — evaluate how useful it is for scene layout vs Godot's editor
- **Networking**: Fyrox has NO networking crate. This is the biggest risk. Would need to wire `renet` or `laminar` manually.
- **Scripting**: Fyrox uses Rust scripts attached to nodes (like Unity MonoBehaviour but Rust). This is actually clean for our use case.
- **Resource system**: `ResourceManager` for async asset loading — not needed for spike (use procedural meshes).

---

## What We're Specifically Testing

1. **Editor quality**: Is Fyroxed useful for building the command card / HUD / scene layout?
2. **Scene graph friction**: How hard is it to sync game-core state into Fyrox's handle-based scene graph vs Bevy ECS vs Godot nodes?
3. **Compile times**: Fyrox is one crate; compile profile matters.

---

## Success Criteria

- [ ] Terrain renders (64×64 grid, vertex-colored)
- [ ] Camera pans and zooms
- [ ] Workers spawn, move to clicked target
- [ ] Workers gather from resource node, UI label increments
- [ ] Wolf patrols, attacks nearby workers
- [ ] Supply drain kills unsupplied workers
- [ ] Day/night visible
- [ ] 50 units, 60fps (Fyrox diagnostic)
- [ ] `replay.bin` written on exit
- [ ] Fyroxed editor evaluated for scene/UI building

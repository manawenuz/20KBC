# Plan 03 — Bevy Spike

## Goal

Validate that Bevy's ECS can wrap game-core's simulation for 20KBC rendering.

---

## Project Structure

```
spikes/bevy/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── sim_plugin.rs       # SimPlugin: owns CSimulation, ticks at 20Hz
    ├── terrain_plugin.rs   # TerrainPlugin: mesh generation
    ├── unit_plugin.rs      # UnitPlugin: spawn/sync entity per CUnit
    ├── camera_plugin.rs    # RtsCameraPlugin: pan, zoom, edge scroll
    ├── input_plugin.rs     # InputPlugin: selection, orders
    ├── gaia_plugin.rs      # GaiaPlugin: render wolf entity
    ├── hud_plugin.rs       # HudPlugin: bevy_egui resource panel
    ├── day_night.rs        # DayNightPlugin: ambient light cycle
    └── components.rs       # SimUnitId, Selected, HealthBar, etc.
```

---

## ECS Design

### Resources

```rust
#[derive(Resource)]
pub struct GameSim(pub CSimulation);

#[derive(Resource)]
pub struct SimTickTimer(pub Timer);  // 50ms fixed
```

### Components

```rust
#[derive(Component)] pub struct SimUnitId(pub UnitId);
#[derive(Component)] pub struct Selected;
#[derive(Component)] pub struct HealthBar;
#[derive(Component)] pub struct GaiaTag;
#[derive(Component)] pub struct ResourceNodeTag(pub ResourceNodeId);
```

### Entities per concept

| Game concept | Bevy entity | Components |
|---|---|---|
| Worker | Yes | `SimUnitId`, `Transform`, `PbrBundle` |
| Wolf | Yes | `GaiaTag`, `Transform`, `PbrBundle` |
| Resource node | Yes | `ResourceNodeTag`, `Transform`, `PbrBundle` |
| Selection circle | Yes, child of worker | `Mesh2dBundle`, `Selected` marker |
| Health bar | Yes, child of worker | `HealthBar`, billboard material |

---

## Implementation Steps

### Step 1: Scaffold + SimPlugin

```rust
// main.rs
App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins((SimPlugin, TerrainPlugin, UnitPlugin, RtsCameraPlugin,
                  InputPlugin, HudPlugin, DayNightPlugin))
    .run();

// sim_plugin.rs
fn tick_system(mut sim: ResMut<GameSim>, mut timer: ResMut<SimTickTimer>, time: Res<Time>) {
    timer.0.tick(time.delta());
    while timer.0.just_finished() {
        sim.0.tick();
    }
}
```

### Step 2: Terrain

```rust
fn spawn_terrain(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>,
                 mut materials: ResMut<Assets<StandardMaterial>>) {
    // Build 64×64 quad mesh with per-vertex color
    // alternating grass (green) / dirt (brown) / stone (grey) by position hash
    let mesh = build_terrain_mesh(64, 64, 2.0);
    commands.spawn(PbrBundle { mesh: meshes.add(mesh), material: ..., ..default() });
}
```

### Step 3: RTS Camera

- `Camera3D` positioned at (32, 40, 32) looking at (32, 0, 32)
- Pan: track WASD + edge scroll → translate camera XZ
- Zoom: scroll wheel → translate camera Y, clamp [8, 60]
- Use `bevy_panorbit_camera` or custom system

### Step 4: SimPlugin → Unit Sync

```rust
fn sync_units(
    sim: Res<GameSim>,
    mut query: Query<(&SimUnitId, &mut Transform)>,
    mut commands: Commands,
    spawned: Local<HashSet<UnitId>>,
) {
    for unit in sim.0.iter_units() {
        if !spawned.contains(&unit.id) {
            // spawn new entity
            commands.spawn((SimUnitId(unit.id), PbrBundle { ... }, Transform::from_xyz(...)));
            spawned.insert(unit.id);
        }
    }
    for (sim_id, mut transform) in &mut query {
        if let Some(unit) = sim.0.get_unit(sim_id.0) {
            transform.translation = Vec3::new(unit.pos.x, 0.0, unit.pos.y);
            transform.rotation = Quat::from_rotation_y(unit.facing);
        }
    }
}
```

### Step 5: Input + Orders

- Raycast mouse → world XZ plane via `bevy_mod_raycast` or manual plane intersection
- Selection:
  - Left-click: pick unit entity by ray vs unit bounding sphere
  - Drag: collect units whose screen position falls inside selection rect
- Orders:
  - Right-click terrain: `sim.issue_order(id, Order::Move { target })`
  - Right-click resource node entity: `sim.issue_order(id, Order::Gather { node })`

### Step 6: HUD (bevy_egui)

```rust
fn hud_system(mut contexts: EguiContexts, sim: Res<GameSim>) {
    egui::Window::new("Resources").anchor(Align2::LEFT_TOP, [8., 8.]).show(ctx, |ui| {
        let p = &sim.0.players[0];
        ui.label(format!("Wood: {}", p.wood));
        ui.label(format!("Stone: {}", p.stone));
    });
}
```

### Step 7: Day/Night

```rust
fn day_night_system(mut lights: Query<&mut DirectionalLight>, time: Res<Time>,
                    cycle: Res<DayNightCycle>) {
    let t = (time.elapsed_seconds() / cycle.period_secs) % 1.0;
    let illuminance = lerp(5000.0, 500.0, night_curve(t));
    for mut light in &mut lights { light.illuminance = illuminance; }
}
```

---

## Key Crate Dependencies

```toml
[dependencies]
bevy = { version = "0.15", features = ["dynamic_linking"] }  # fast compile in dev
game-core = { path = "../../game-core" }
bevy_egui = "0.31"
```

Note: Use `dynamic_linking` feature flag during dev — dramatically reduces recompile times.

---

## Bevy-Specific Notes

- **ECS fits RTS perfectly**: units = entities, stats = components, behaviors live in game-core
- **No visual editor**: all scene setup is code — this is a friction point to measure
- **bevy_egui** is the recommended UI path for data panels; for complex RTS HUD (command card, minimap) this needs more evaluation
- **Lockstep**: Bevy has no built-in lockstep. `bevy_replicon` + `renet` is the path. The input log written by game-core is already lockstep-ready.

---

## Success Criteria

- [ ] Terrain renders (64×64 grid, vertex-colored)
- [ ] Camera pans and zooms
- [ ] Workers spawn, move to clicked target
- [ ] Workers gather from resource node, EguiHUD shows increment
- [ ] Wolf patrols, attacks nearby workers
- [ ] Supply drain kills unsupplied workers
- [ ] Day/night directional light cycle visible
- [ ] 50 units maintain 60fps (Bevy diagnostic overlay)
- [ ] `replay.bin` written on exit

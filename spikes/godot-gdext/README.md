# Godot + gdext Spike

Validates that Godot 4 + Rust (gdext) can serve as the rendering layer for the
20KBC game-core.

## Requirements

- Rust stable (1.77+)
- Godot 4.3 or newer
- `game-core` crate at `../../game-core` (built by the game-core agent)

## Build

```bash
cd rust
cargo build          # debug build
# or
cargo build --release
```

The compiled library lands at:
- macOS: `rust/target/debug/libspike_godot_gdext.dylib`
- Linux: `rust/target/debug/libspike_godot_gdext.so`
- Windows: `rust/target/debug/spike_godot_gdext.dll`

These paths are already registered in `spike-godot-gdext.gdextension`.

## Run

Open Godot 4 and import this directory as a project.  Godot will load the
extension automatically via the `.gdextension` file.  Press **F5** (Run Project)
to launch `scenes/Main.tscn`.

## Architecture

```
Godot (scenes/scripts)
    └── SimBridge  (GDExtension Node)
            └── CSimulation  (pure-Rust, game-core crate)
```

`SimBridge` is ticked once per Godot physics frame (20 Hz = 50 ms), matching
the simulation's own fixed step rate.  Positions are read back as
`Array[Vector2]` and applied to `UnitNode` instances in `main.gd`.

## Controls

| Input | Action |
|-------|--------|
| WASD | Pan camera |
| Scroll wheel | Zoom in / out |
| Right-click terrain | Move unit 0 to clicked position |

## Physics tick rate

`project.godot` sets `physics/common/physics_ticks_per_second = 20`, which
makes Godot's `_physics_process` fire at exactly 20 Hz — one call per
simulation tick.

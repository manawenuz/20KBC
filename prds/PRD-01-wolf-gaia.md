# PRD-01 — Wolf/GAIA Rendering

## Goal

Render the GAIA wolf that already exists in the `CSimulation` as a visible red
capsule in Godot, with its position synced every physics tick. Add a Rust
GDExtension class `GaiaNode` and expose `get_gaia_positions()` on `SimBridge`.

## Context

`game-core` already simulates GAIA entities — see `game-core/src/gaia.rs`.
`CSimulation::gaia: Vec<CGaiaEntity>` contains them. A single wolf spawns at
(20, 20) with territory radius 50 (see `simulation.rs` line ~78).

The existing `UnitNode` at `spikes/godot-gdext/rust/src/unit_node.rs` is the
pattern to copy — a `Node3D` that builds a child `MeshInstance3D` with a capsule
mesh in its `ready()`.

## Files you MAY create

- `spikes/godot-gdext/rust/src/gaia_node.rs`

## Files you MAY modify

- `spikes/godot-gdext/rust/src/lib.rs` — add `mod gaia_node;` only. Nothing else.
- `spikes/godot-gdext/rust/src/sim_bridge.rs` — add ONE new `#[func]` method
  `get_gaia_positions(&self) -> Array<Vector2>`. Do not touch existing methods.

## Files you MUST NOT touch

- `spikes/godot-gdext/scenes/Main.tscn`
- `spikes/godot-gdext/scripts/main.gd`
- `spikes/godot-gdext/project.godot`
- `game-core/**` — simulation is already correct
- Any other Rust source

The orchestrator will wire `GaiaNode` instantiation into `main.gd` after merge.

## Interface contract

```rust
// gaia_node.rs
#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct GaiaNode {
    #[var] pub gaia_id: u32,
    base: Base<Node3D>,
}
// In ready(): build a red capsule (radius 0.5, height 1.2) as a child MeshInstance3D
// at local position (0, 0.6, 0). Color: Color { r: 0.75, g: 0.18, b: 0.15, a: 1.0 }.
// Use StandardMaterial3D with ShadingMode::PER_PIXEL (see unit_node.rs).
```

```rust
// sim_bridge.rs — append to the existing #[godot_api] impl SimBridge block
#[func]
pub fn get_gaia_positions(&self) -> Array<Vector2> {
    // Iterate self.sim.as_ref()?.gaia and push Vector2::new(g.pos.x, g.pos.y)
    // for every entity with hp > 0.0.
}
```

The `CGaiaEntity` struct exposes `pub pos: Vec2` and `pub hp: f32` — both public.
`Vec2.x` is the world x, `Vec2.y` is the world z (we treat the sim plane as XZ).

## Implementation hints

- Look at `unit_node.rs` for the exact pattern of building a capsule mesh inside
  a Node3D's `ready()`. Same imports, same approach.
- gdext 0.5.2 master is on this branch. The existing `unit_node.rs` already
  uses the right API surface (`CapsuleMesh::new_gd()`, `surface_set_material(0, &mat)`).
- The `#[var]` attribute on `gaia_id` lets GDScript set/get it after instantiation.

## Acceptance criteria

Run from `spikes/godot-gdext/rust/`:

```bash
cargo build
```

Must finish with no warnings beyond the existing baseline and no errors.

Additionally:

- [ ] `gaia_node.rs` exists and compiles
- [ ] `lib.rs` has exactly one new line: `mod gaia_node;` (alphabetical order preferred)
- [ ] `sim_bridge.rs` has the new `get_gaia_positions` `#[func]` method
- [ ] No other files modified
- [ ] `git diff --stat` shows at most 3 files changed

If `cargo build` fails: read the error, fix, retry. Do not work around errors
by deleting code from other files.

## Out of scope (do NOT do these)

- Wiring `GaiaNode` into `main.gd` — orchestrator handles it
- Changing `Main.tscn`
- Adding multiple wolves to the sim — sim is already correct
- Adding a wolf health bar — separate PRD later

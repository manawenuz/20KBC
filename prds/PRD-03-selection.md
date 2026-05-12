# PRD-03 — Selection System

## Goal

Add unit selection: left-click to select a single unit, click-and-drag to
box-select multiple. Selected units show a green selection ring decal at their
feet. Multi-unit right-click sends the same order to every selected unit.

## Context

Currently `main.gd` hard-codes orders to unit 0:
```gdscript
sim.issue_move_order(0, hit.x, hit.z)
```

We need to track a selection set in Rust (so the existing `UnitNode` can render
its own ring) and expose a Godot-callable API for adding/removing/clearing.

The existing `UnitNode` at `spikes/godot-gdext/rust/src/unit_node.rs` has a
`#[var] pub unit_id: u32`. Reuse this.

## Files you MAY create

- `spikes/godot-gdext/rust/src/selection_manager.rs`

## Files you MAY modify

- `spikes/godot-gdext/rust/src/lib.rs` — add `mod selection_manager;` only
- `spikes/godot-gdext/rust/src/unit_node.rs` — add an OPTIONAL `set_selected(bool)`
  `#[func]` that toggles a child ring MeshInstance3D's visibility. Do not change
  existing `init`/`ready` body beyond adding the ring as an extra child.
- `spikes/godot-gdext/rust/src/sim_bridge.rs` — add ONE new `#[func]`:
  `get_unit_at(&self, world_x: f32, world_z: f32, radius: f32) -> i64` returning
  the UnitId of the closest unit within `radius`, or -1 if none.

## Files you MUST NOT touch

- `spikes/godot-gdext/scenes/Main.tscn`
- `spikes/godot-gdext/scripts/main.gd`
- `spikes/godot-gdext/project.godot`
- `game-core/**`
- Any other Rust source

## Interface contract

```rust
// selection_manager.rs
use std::collections::HashSet;

#[derive(GodotClass)]
#[class(base = Node)]
pub struct SelectionManager {
    selected: HashSet<u32>,
    base: Base<Node>,
}

#[godot_api]
impl SelectionManager {
    #[func] pub fn clear(&mut self) { ... }
    #[func] pub fn add(&mut self, unit_id: u32) { ... }
    #[func] pub fn remove(&mut self, unit_id: u32) { ... }
    #[func] pub fn contains(&self, unit_id: u32) -> bool { ... }
    /// Returns the selection as an Array<i64> of unit ids (sorted ascending).
    #[func] pub fn get_all(&self) -> Array<i64> { ... }
    #[func] pub fn count(&self) -> i64 { ... }
}
```

```rust
// unit_node.rs — add this child to ready() AND add the toggle #[func]
// Build a green ring: TorusMesh with inner_radius=0.6, outer_radius=0.75,
// positioned at (0, 0.05, 0). StandardMaterial3D albedo green:
//   Color { r: 0.20, g: 0.85, b: 0.30, a: 1.0 }
// Store the MeshInstance3D as a field on UnitNode and toggle its visible flag.
//
// Then expose:
#[func] pub fn set_selected(&mut self, selected: bool) {
    // ring.set_visible(selected)
}
```

```rust
// sim_bridge.rs — append
#[func]
pub fn get_unit_at(&self, world_x: f32, world_z: f32, radius: f32) -> i64 {
    // Iterate living units, find closest within radius, return its id as i64.
    // Return -1 if none.
}
```

## Implementation hints

- `TorusMesh` is in `godot::classes::TorusMesh`. Set `inner_radius` and `outer_radius`.
- Keep `selected: HashSet<u32>` as the source of truth. `get_all()` should sort
  for deterministic GDScript iteration.
- The ring child should default to **invisible** (`set_visible(false)` in `ready`).
- `Array<i64>` push: `arr.push(uid as i64);`

## Acceptance criteria

Run from `spikes/godot-gdext/rust/`:

```bash
cargo build
```

Must succeed cleanly. Additionally:

- [ ] `selection_manager.rs` exists, all 6 `#[func]` methods present and compile
- [ ] `unit_node.rs` builds a child torus ring AND exposes `set_selected(bool)`
- [ ] `sim_bridge.rs` has the new `get_unit_at` `#[func]` method
- [ ] `lib.rs` adds exactly `mod selection_manager;`
- [ ] No other files modified
- [ ] `git diff --stat` shows at most 4 files changed

## Out of scope

- Box-drag selection rectangle rendering — orchestrator will do this in GDScript
- Sending the multi-unit right-click orders — done in `main.gd` integration
- Click priority (unit vs terrain) — done in `main.gd` integration

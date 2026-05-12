# PRD-19 — Tree + Stone Outcrop Models

## Goal

Replace the brown/grey boxes in `ResourceNode` (the visual one in
`resource_node_visual.rs`) with:
- `res://assets/models/tree.glb` for Wood
- `res://assets/models/stone.glb` for Stone

If a model is missing, **keep the box fallback** for that kind.

Bonus: when a wood node depletes, smoothly rotate the tree mesh to
90° on the X axis over 1 second to simulate falling, then queue_free.
Sim already removes depleted nodes; we just want a visible tip-over
before disappearance. This requires polling depletion from the bridge.

## Context

`resource_node_visual.rs` has a `kind` field (1=Wood, 2=Stone).

## Files you MAY modify

- `spikes/godot-gdext/rust/src/resource_node_visual.rs`
- `spikes/godot-gdext/rust/src/sim_bridge.rs` — add ONE `#[func]`:
  `get_resource_amount(&self, node_id: u32) -> i64` returning current
  amount (or -1 if not found / depleted).

## Files you MUST NOT touch

- `main.gd`, `Main.tscn`
- `game-core/**`
- Other Rust source

## Interface contract

```rust
// resource_node_visual.rs — in ready():
// - Pick model path by kind. Try ResourceLoader. If success, instantiate as child.
// - Fall back to BoxMesh on failure (current code).
//
// Add a process() that polls sim_bridge.get_resource_amount(self.node_id):
// - If amount was > 0 last frame and is now 0 (kind==Wood only), start tip-over.
// - Tip-over: rotate child mesh X by +90° linearly over 1.0s, then queue_free.

// sim_bridge.rs:
#[func]
pub fn get_resource_amount(&self, node_id: u32) -> i64 {
    self.sim.as_ref()
        .and_then(|s| s.iter_resources().find(|n| n.id == node_id))
        .map(|n| n.amount as i64)
        .unwrap_or(-1)
}
```

## Acceptance criteria

```bash
cd spikes/godot-gdext/rust && cargo build
```

- [ ] Clean build
- [ ] BoxMesh fallback preserved
- [ ] `sim_bridge.rs` has `get_resource_amount`
- [ ] ≤ 2 files modified

## Out of scope

- Stone-specific death animation (mining flash, etc.)
- Stumps left behind

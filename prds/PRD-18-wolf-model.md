# PRD-18 — Animated Wolf Model

## Goal

Replace the red capsule in `GaiaNode` with the wolf glb model at
`res://assets/models/wolf.glb`. Animate idle/walk/attack based on
behavior — but `GaiaNode` doesn't currently know its gaia behavior, so
also expose that.

## Context

Same pattern as PRD-17 but for `GaiaNode` and `CGaiaEntity`.

## Files you MAY modify

- `spikes/godot-gdext/rust/src/gaia_node.rs`
- `spikes/godot-gdext/rust/src/sim_bridge.rs` — add ONE `#[func]`:
  `get_gaia_behavior(&self, gaia_idx: u32) -> i64`
  returning: 0 = Roaming, 1 = Chasing, 2 = Attacking, 3 = Returning.
  (Index into `sim.gaia[]`, matching `get_gaia_positions` order.)

## Files you MUST NOT touch

- `main.gd`, `Main.tscn`
- `game-core/**`
- Other Rust source

## Interface contract

Mirror PRD-17 exactly:

```rust
// gaia_node.rs — in ready(): load wolf.glb, fall back to capsule on failure.
// In a new process(): poll sim_bridge.get_gaia_behavior(self.gaia_id) and switch animations.

// sim_bridge.rs:
#[func]
pub fn get_gaia_behavior(&self, gaia_idx: u32) -> i64 {
    use game_core::gaia::GaiaBehavior;
    self.sim.as_ref()
        .and_then(|s| s.gaia.get(gaia_idx as usize))
        .map(|g| match &g.behavior {
            GaiaBehavior::Roaming { .. } => 0,
            GaiaBehavior::Chasing { .. } => 1,
            GaiaBehavior::Attacking { .. } => 2,
            GaiaBehavior::Returning => 3,
        })
        .unwrap_or(0)
}
```

## Acceptance criteria

```bash
cd spikes/godot-gdext/rust && cargo build
```

- [ ] Clean build
- [ ] Capsule fallback kept for missing glb
- [ ] `sim_bridge.rs` has `get_gaia_behavior`
- [ ] ≤ 2 files modified

## Out of scope

- Wolf-pack behaviors
- Howling animations

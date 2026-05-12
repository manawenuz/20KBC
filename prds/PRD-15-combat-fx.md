# PRD-15 — Combat Feedback FX

## Goal

Add three visual feedback mechanisms when combat happens:

1. **Hit flash**: when a unit takes damage, briefly tint its mesh red.
2. **Damage number**: spawn a floating yellow text label that drifts up
   and fades out, showing the damage amount.
3. **Death fade**: when a unit dies, fade its mesh to transparent over
   ~1 second, then despawn (instead of instantly disappearing).

These run client-side only. The simulation already has authoritative
HP. The Rust nodes observe HP deltas across frames and trigger effects.

## Context

`UnitNode` already exists. We need to track HP per visual node, compare
to sim HP each frame, and react to deltas.

Bridge needs to expose HP. Currently it doesn't.

## Files you MAY create

- `spikes/godot-gdext/rust/src/damage_number.rs`

## Files you MAY modify

- `spikes/godot-gdext/rust/src/lib.rs` — add `mod damage_number;`
- `spikes/godot-gdext/rust/src/sim_bridge.rs` — add:
  - `get_unit_hp(&self, unit_id: u32) -> f32` — returns hp, -1.0 if not found
- `spikes/godot-gdext/rust/src/unit_node.rs` — track `prev_hp: f32` field, compare in `process`, trigger flash + emit signal for damage number, on hp <= 0 start fade.

## Files you MUST NOT touch

- `game-core/**`
- `main.gd`, `Main.tscn`, `project.godot`
- Other Rust source

## Interface contract

```rust
// damage_number.rs — a self-deleting floating Label3D
use godot::prelude::*;
use godot::classes::{INode3D, Label3D, Node3D};

#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct DamageNumber {
    elapsed: f32,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for DamageNumber {
    fn init(base: Base<Node3D>) -> Self { Self { elapsed: 0.0, base } }
    fn ready(&mut self) {
        // Build a Label3D child showing self.text (set by spawner before add).
        // billboard = ENABLED, font_size big, modulate yellow.
    }
    fn process(&mut self, delta: f64) {
        self.elapsed += delta as f32;
        let t = self.elapsed;
        // Drift up at 1.5 wu/s, fade out alpha = 1 - t/1.0
        let mut pos = self.base().get_position();
        pos.y += 1.5 * delta as f32;
        self.base_mut().set_position(pos);
        // Set Label3D modulate alpha to 1 - t.
        if t >= 1.0 { self.base_mut().queue_free(); }
    }
}

#[godot_api]
impl DamageNumber {
    #[func] pub fn set_amount(&mut self, amount: i64) {
        // Set Label3D text to format!("{}", amount), store; ready() reads it.
    }
}
```

```rust
// sim_bridge.rs — append
#[func]
pub fn get_unit_hp(&self, unit_id: u32) -> f32 {
    self.sim.as_ref()
        .and_then(|s| s.iter_units().find(|u| u.id == unit_id))
        .map(|u| u.hp).unwrap_or(-1.0)
}
```

```rust
// unit_node.rs — add a process() that polls hp from sim_bridge sibling.
// On hp drop: spawn DamageNumber as a child of Main.Units (or world), tint mesh red 0.15s, restore.
// On hp <= 0: skip mesh removal (sim does that), but if you can find a way to play a 1s fade — do it. Otherwise it's fine to just rely on sim's unit removal which already culls the node.
//
// HP is fetched via a node lookup: self.base().get_tree() etc — but cross-node access in gdext is fragile.
// SIMPLEST: expose a #[var] hp: f32 on UnitNode, and let main.gd push hp values in via set_hp(uid, hp) — OR let main.gd call set_hp() each tick. The orchestrator will wire this.
//
// For this PRD: just implement set_hp(&mut self, hp: f32) #[func] that updates prev_hp and triggers
// the visual feedback when hp < prev_hp.
```

## Acceptance criteria

```bash
cd spikes/godot-gdext/rust && cargo build
```

- [ ] Clean build
- [ ] `damage_number.rs` exists
- [ ] `unit_node.rs` exposes `set_hp(hp: f32)`
- [ ] `sim_bridge.rs` has `get_unit_hp`
- [ ] ≤ 4 files modified

## Out of scope

- Particle blood/sparks
- Sound (separate PRD)
- Hit decals on terrain

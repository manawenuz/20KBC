# PRD-17 — Animated Peasant Model

## Goal

Replace the brown capsule in `UnitNode` with a 3D model loaded from
`res://assets/models/peasant.glb`. If the model has animations, play
"idle" by default and switch to "walk" when the unit's behavior is
MovingTo or Gathering (returning), "attack" when Attacking.

## Context

`UnitNode` in `spikes/godot-gdext/rust/src/unit_node.rs` currently
spawns a capsule mesh as a child. We replace that subtree with a
PackedScene instantiation of the glb.

PRD-09 or PRD-10 will have written `assets/models/peasant.glb`. If
neither produced a file, **keep the capsule fallback** and emit a
godot_warn.

## Files you MAY create

(none)

## Files you MAY modify

- `spikes/godot-gdext/rust/src/unit_node.rs`
- `spikes/godot-gdext/rust/src/sim_bridge.rs` — add ONE `#[func]`:
  `get_unit_behavior(&self, unit_id: u32) -> i64` returning:
  - 0 = Idle/Dead
  - 1 = MovingTo
  - 2 = Gathering
  - 3 = Attacking

## Files you MUST NOT touch

- `main.gd`, `Main.tscn`, `project.godot`
- `game-core/**` (CUnit::behavior is already public)
- Other Rust source

## Interface contract

```rust
// unit_node.rs — modify ready() and add a process() that polls behavior
// and switches animations.

fn ready(&mut self) {
    let mut loader = godot::classes::ResourceLoader::singleton();
    let model: Option<Gd<PackedScene>> = loader
        .load("res://assets/models/peasant.glb")
        .and_then(|r| r.try_cast::<PackedScene>().ok());

    if let Some(scene) = model {
        if let Some(instance) = scene.instantiate() {
            // Add as child. Scale to ~1.8 m tall if needed; depends on glb.
            self.base_mut().add_child(&instance);
            // Cache the AnimationPlayer if present.
        }
    } else {
        // Fallback to capsule (existing code).
        godot_warn!("peasant.glb missing — falling back to capsule");
        // ... existing capsule path ...
    }

    // Keep the selection ring code from the existing implementation.
}

fn process(&mut self, _delta: f64) {
    // Poll sim_bridge.get_unit_behavior(self.unit_id) once every ~5 frames.
    // If changed, switch AnimationPlayer to "idle"/"walk"/"attack"/"die".
    // If the glb has no AnimationPlayer, skip silently.
}
```

```rust
// sim_bridge.rs
#[func]
pub fn get_unit_behavior(&self, unit_id: u32) -> i64 {
    use game_core::BehaviorState;
    self.sim.as_ref()
        .and_then(|s| s.iter_units().find(|u| u.id == unit_id))
        .map(|u| match &u.behavior {
            BehaviorState::Idle | BehaviorState::Dead => 0,
            BehaviorState::MovingTo { .. } => 1,
            BehaviorState::Gathering { .. } => 2,
            BehaviorState::Attacking { .. } => 3,
        })
        .unwrap_or(0)
}
```

Notes for the agent:
- Use `try_cast` not `cast` so a wrong type returns Option, never panics.
- The fallback path must keep working — the orchestrator may run this
  before glb assets land.
- Selection ring (torus) code stays — don't lose it.

## Acceptance criteria

```bash
cd spikes/godot-gdext/rust && cargo build
```

- [ ] Clean build
- [ ] Capsule fallback path kept intact (search for `CapsuleMesh` in the diff)
- [ ] `sim_bridge.rs` has `get_unit_behavior`
- [ ] ≤ 2 files modified

## Out of scope

- Animation blending
- Per-frame foot IK
- Skin-tone or clothing variation

# PRD-08 — Stress-Test Mode + Frame-Time HUD

## Goal

Add a debug method on `SimBridge` to spawn N extra workers on demand,
and a `FrameTimeLabel` Rust HUD widget that displays current FPS and
frame time. Used to validate the "50 units at 60fps" stress goal from
the MVP plan.

## Context

The existing `GameHud` (Control + Labels) pattern in `hud.rs` shows
the format. We want a sibling Control that displays FPS.

## Files you MAY create

- `spikes/godot-gdext/rust/src/frame_time_label.rs`

## Files you MAY modify

- `spikes/godot-gdext/rust/src/lib.rs` — add `mod frame_time_label;` only
- `spikes/godot-gdext/rust/src/sim_bridge.rs` — add ONE new #[func]:
  `spawn_workers(&mut self, count: u32)`

## Files you MUST NOT touch

- `spikes/godot-gdext/scenes/Main.tscn`
- `spikes/godot-gdext/scripts/main.gd`
- `spikes/godot-gdext/project.godot`
- Any other Rust source
- `game-core/**` (CSimulation::spawn_unit is already public)

## Interface contract

```rust
// frame_time_label.rs
use godot::prelude::*;
use godot::classes::{Control, IControl, Label, Engine};

#[derive(GodotClass)]
#[class(base = Control)]
pub struct FrameTimeLabel {
    /// Update every N frames to avoid distracting flicker.
    #[export] pub refresh_frames: u32,
    counter: u32,
    base: Base<Control>,
}

#[godot_api]
impl IControl for FrameTimeLabel {
    fn init(base: Base<Control>) -> Self {
        Self { refresh_frames: 15, counter: 0, base }
    }

    fn process(&mut self, _delta: f64) {
        self.counter = self.counter.wrapping_add(1);
        if self.counter % self.refresh_frames.max(1) != 0 { return; }
        let fps = Engine::singleton().get_frames_per_second() as i64;
        let ms = if fps > 0 { 1000.0 / fps as f32 } else { 0.0 };
        if let Some(mut label) = self.base().get_node_or_null("Label")
            .and_then(|n| n.try_cast::<Label>().ok())
        {
            label.set_text(&format!("FPS: {}  ({:.1} ms)", fps, ms));
        }
    }
}
```

```rust
// sim_bridge.rs — append

/// Spawn `count` extra workers near the depot for stress testing.
/// Used from a debug binding (orchestrator will wire to a hotkey).
#[func]
pub fn spawn_workers(&mut self, count: u32) {
    if let Some(sim) = &mut self.sim {
        let depot = sim.players.get(0).and_then(|p| p.supply_depot)
            .unwrap_or(game_core::Vec2::ZERO);
        for i in 0..count {
            let angle = i as f32 * 0.6;
            let r = 5.0 + (i as f32 * 0.15);
            let offset = game_core::Vec2::new(angle.cos() * r, angle.sin() * r);
            sim.spawn_unit(0, depot + offset);
        }
    }
}
```

## Acceptance criteria

```bash
cd spikes/godot-gdext/rust && cargo build
```

Must succeed cleanly. Additionally:

- [ ] `frame_time_label.rs` exists and compiles
- [ ] `lib.rs` has `mod frame_time_label;`
- [ ] `sim_bridge.rs` has the new `spawn_workers` `#[func]`
- [ ] No other files modified
- [ ] `git diff --stat` shows ≤ 3 files changed

## Out of scope

- Wiring the FrameTimeLabel into Main.tscn — orchestrator
- Hotkey binding for spawn_workers — orchestrator
- Per-frame metrics, render-time vs physics-time split — out of scope

# PRD-06 — Day/Night Cycle

## Goal

Add a 10-minute (real-time) day/night cycle: the directional light's
energy and colour lerp through dawn → noon → dusk → night. Add a Rust
GDExtension node `DayNightController` that drives this. The orchestrator
will wire it into `Main.tscn` afterwards.

## Context

The scene currently has a `DirectionalLight3D` named `Sun` with fixed
`light_energy = 1.2`. We need a node that finds the Sun via NodePath
property and modulates it every frame.

## Files you MAY create

- `spikes/godot-gdext/rust/src/day_night.rs`

## Files you MAY modify

- `spikes/godot-gdext/rust/src/lib.rs` — add `mod day_night;` only

## Files you MUST NOT touch

- `spikes/godot-gdext/scenes/Main.tscn` (orchestrator will wire later)
- `spikes/godot-gdext/scripts/main.gd`
- `spikes/godot-gdext/project.godot`
- `spikes/godot-gdext/rust/src/sim_bridge.rs` and any other Rust file
- `game-core/**`

## Interface contract

```rust
// day_night.rs
use godot::prelude::*;
use godot::classes::{DirectionalLight3D, INode, Node};

#[derive(GodotClass)]
#[class(base = Node)]
pub struct DayNightController {
    /// NodePath (set in editor) pointing at the DirectionalLight3D to drive.
    #[export] pub sun_path: NodePath,
    /// Cycle length in seconds (default 600 = 10 minutes).
    #[export] pub cycle_seconds: f32,
    /// Current time-of-day in [0, 1). 0.25 = noon, 0.75 = midnight.
    #[var] pub time_of_day: f32,
    base: Base<Node>,
}

#[godot_api]
impl INode for DayNightController {
    fn init(base: Base<Node>) -> Self {
        Self {
            sun_path: NodePath::default(),
            cycle_seconds: 600.0,
            time_of_day: 0.30, // start a little after dawn
            base,
        }
    }

    fn process(&mut self, delta: f64) {
        // Advance time_of_day, wrap to [0, 1).
        // Look up Sun via sun_path. If found:
        //   energy = smoothstep curve based on time_of_day:
        //     dawn ~0.2: 0.2  → noon 0.25: 1.4  → dusk ~0.75: 0.2  → night: 0.05
        //   color: warm orange at dawn/dusk (1.0, 0.6, 0.3), bright white at noon (1.0, 0.95, 0.85),
        //          cool blue at night (0.30, 0.40, 0.60).
        // Apply via sun.set_light_energy() and sun.set_light_color().
    }
}

#[godot_api]
impl DayNightController {
    /// Force-set time_of_day (useful for tests or skip-to-night debug).
    #[func]
    pub fn set_time_of_day(&mut self, t: f32) {
        self.time_of_day = t.rem_euclid(1.0);
    }
}
```

Implementation tips:

- `self.base().get_node_or_null(&self.sun_path)` to look up the Sun
- `try_cast::<DirectionalLight3D>().ok()` to convert
- `delta` is in seconds; `time_of_day += delta as f32 / self.cycle_seconds`
- Use a simple piecewise lerp for both energy and color. Don't over-engineer.

Suggested keyframes (time → energy, color):

| t    | phase  | energy | color (r,g,b)        |
|------|--------|--------|----------------------|
| 0.00 | night  | 0.10   | (0.30, 0.40, 0.60)   |
| 0.20 | dawn   | 0.30   | (1.00, 0.60, 0.30)   |
| 0.25 | noon-r | 1.40   | (1.00, 0.95, 0.85)   |
| 0.70 | dusk   | 0.30   | (1.00, 0.55, 0.25)   |
| 0.80 | night  | 0.10   | (0.30, 0.40, 0.60)   |

Lerp linearly between adjacent keyframes; wrap from 0.80→1.00→0.00.

## Acceptance criteria

```bash
cd spikes/godot-gdext/rust && cargo build
```

Must succeed cleanly. Additionally:

- [ ] `day_night.rs` exists, exports `DayNightController` via #[derive(GodotClass)]
- [ ] `lib.rs` has the new `mod day_night;` line, kept alphabetical
- [ ] No other files modified
- [ ] `git diff --stat` shows ≤ 2 files changed

## Out of scope

- Wiring into `Main.tscn` — orchestrator handles
- Vision-radius reduction at night (separate PRD)
- Skybox or fog modulation

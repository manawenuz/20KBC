# PRD-07 — Replay Log Save-on-Quit

## Goal

Surface `CSimulation::write_replay` to GDScript so `main.gd` can save
the input log to `replay.bin` when the game closes. The function
already exists in game-core — we just need the bridge method.

## Context

`game-core/src/simulation.rs::write_replay(&self, path: &str)`
serializes `input_log` to a binary file. MVP criterion #9.

## Files you MAY create

(none)

## Files you MAY modify

- `spikes/godot-gdext/rust/src/sim_bridge.rs` — add ONE new #[func]:
  `save_replay(&self, path: GString)`

## Files you MUST NOT touch

- Anything else under `spikes/**`
- `game-core/**`
- `spikes/godot-gdext/scripts/main.gd` (orchestrator will hook the quit signal)

## Interface contract

```rust
// sim_bridge.rs — append to the existing #[godot_api] impl SimBridge

/// Serialize the input log to `path`. Called from main.gd on quit so
/// every match leaves a replay.bin alongside the executable.
#[func]
pub fn save_replay(&self, path: GString) {
    if let Some(sim) = &self.sim {
        let p = path.to_string();
        sim.write_replay(&p);
    }
}
```

That's literally all. `GString` is gdext's string type — `to_string()`
gives a `String` which `write_replay` accepts via `&str`.

## Acceptance criteria

```bash
cd spikes/godot-gdext/rust && cargo build
```

Must succeed cleanly. Additionally:

- [ ] `sim_bridge.rs` has the new `save_replay` `#[func]`
- [ ] No other files modified
- [ ] `git diff --stat` shows exactly 1 file changed

## Out of scope

- Loading replays (separate PRD if/when we play them back)
- Compressing the replay
- Anything in `main.gd` — orchestrator's job

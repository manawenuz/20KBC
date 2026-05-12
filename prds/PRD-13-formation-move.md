# PRD-13 — Formation Move Planner

## Goal

When multiple units receive a move order to the same point, they
shouldn't all path to the exact same coordinate (clumping). Add a pure
function in `game-core` that distributes N units in a ring around the
target, and expose a bridge method that takes a list of unit IDs and a
target, dispatching one move order per unit to its formation slot.

## Context

`game-core/src/orders.rs` defines `Order`. The current flow in
`main.gd::_handle_right_click` calls `sim.issue_move_order(uid, x, z)`
in a loop — every unit gets the same target. Replace this with a single
bridge call that lays out the formation.

## Files you MAY create

- `game-core/src/formation.rs`

## Files you MAY modify

- `game-core/src/lib.rs` — add `pub mod formation;` and `pub use formation::formation_positions;` only
- `spikes/godot-gdext/rust/src/sim_bridge.rs` — add ONE `#[func]`:
  `issue_formation_move(&mut self, unit_ids: Array<i64>, x: f32, z: f32)`

## Files you MUST NOT touch

- `simulation.rs`, `unit.rs`, `pathfinding.rs`, etc.
- `main.gd`, `Main.tscn`

## Interface contract

```rust
// game-core/src/formation.rs
use glam::Vec2;

/// Place `count` units in concentric rings around `center`.
/// Returns positions ordered such that index 0 ≈ closest to center.
///
/// Layout: 1 unit at center, then 6 in first ring at radius `spacing`,
/// then 12 in second ring at radius 2*spacing, etc. Spacing default 2.0.
pub fn formation_positions(center: Vec2, count: usize, spacing: f32) -> Vec<Vec2> {
    // ring 0: 1 slot at center
    // ring k>=1: 6*k slots at radius k * spacing, angles evenly spaced.
    // Fill outward until count slots filled.
}
```

```rust
// sim_bridge.rs — append
#[func]
pub fn issue_formation_move(&mut self, unit_ids: Array<i64>, x: f32, z: f32) {
    use game_core::formation::formation_positions;
    let center = game_core::Vec2::new(x, z);
    let n = unit_ids.len();
    let slots = formation_positions(center, n, 2.0);
    if let Some(sim) = &mut self.sim {
        for (i, raw) in unit_ids.iter_shared().enumerate() {
            let uid = raw as u32;
            let pos = slots.get(i).copied().unwrap_or(center);
            sim.issue_order(uid, game_core::Order::Move { target: pos });
        }
    }
}
```

## Acceptance criteria

```bash
cd game-core && cargo build && cargo test
cd ../spikes/godot-gdext/rust && cargo build
```

Both clean. Plus a unit test in `formation.rs`:

```rust
#[test]
fn formation_centered_first() {
    let p = formation_positions(Vec2::new(10.0, 10.0), 1, 2.0);
    assert_eq!(p, vec![Vec2::new(10.0, 10.0)]);
}

#[test]
fn formation_rings_outward() {
    let p = formation_positions(Vec2::new(0.0, 0.0), 7, 2.0);
    assert_eq!(p.len(), 7);
    assert_eq!(p[0], Vec2::ZERO);
    for q in &p[1..] {
        let d = q.length();
        assert!((d - 2.0).abs() < 0.01);
    }
}
```

- [ ] Tests pass
- [ ] Build clean
- [ ] ≤ 3 files modified

## Out of scope

- Adjusting for unit's individual radius
- Movement leader/follower hierarchy
- Stance preservation across formation shuffle

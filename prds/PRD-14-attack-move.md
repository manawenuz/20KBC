# PRD-14 — Attack-Move + Hostile Target Detection

## Goal

Right-clicking on a hostile entity (currently: the wolf) issues an
Attack order to all selected workers. Workers will path to attack
range, then attack until target dies or escapes leash range. Already
implemented in `game-core`; we just need bridge plumbing and a hostile
lookup.

## Context

`game-core` already supports `Order::Attack { target: UnitId }` and the
behavior `BehaviorState::Attacking`. The current right-click handler
issues only Move/Gather. We need:
1. A bridge method to issue an Attack order
2. A bridge query to find a hostile entity at a world position

Note: the wolf is a `CGaiaEntity`, not a `CUnit`, so attacking it
requires either (a) treating GAIA entities as attackable in the
simulation (game-core change, larger scope), or (b) for MVP, attacking
hostile units only and treating the wolf as a future expansion.

For this PRD, scope to **inter-player combat** (player 0 unit attacking
a hypothetical player 1 unit), with `get_hostile_at()` returning -1 for
GAIA. The orchestrator will follow up separately for GAIA-as-target.

## Files you MAY create

(none)

## Files you MAY modify

- `spikes/godot-gdext/rust/src/sim_bridge.rs` — add TWO `#[func]`s:
  - `issue_attack_order(&mut self, attacker: u32, target: u32)`
  - `get_hostile_at(&self, world_x: f32, world_z: f32, player: u8, radius: f32) -> i64`
    Returns the UnitId of the closest living unit whose `owner != player` within `radius`, or -1.

## Files you MUST NOT touch

- `game-core/**`
- Anything else

## Interface contract

```rust
// sim_bridge.rs — append
#[func]
pub fn issue_attack_order(&mut self, attacker: u32, target: u32) {
    if let Some(sim) = &mut self.sim {
        sim.issue_order(attacker, game_core::Order::Attack { target });
    }
}

#[func]
pub fn get_hostile_at(&self, world_x: f32, world_z: f32, player: u8, radius: f32) -> i64 {
    let pos = game_core::Vec2::new(world_x, world_z);
    let r2 = radius * radius;
    self.sim.as_ref().and_then(|s| {
        s.iter_units()
            .filter(|u| !u.is_dead && u.owner != player)
            .filter(|u| u.pos.distance_squared(pos) <= r2)
            .min_by(|a, b| a.pos.distance_squared(pos)
                .partial_cmp(&b.pos.distance_squared(pos))
                .unwrap_or(std::cmp::Ordering::Equal))
            .map(|u| u.id as i64)
    }).unwrap_or(-1)
}
```

## Acceptance criteria

```bash
cd spikes/godot-gdext/rust && cargo build
```

- [ ] Clean build
- [ ] Both `#[func]`s present
- [ ] Only `sim_bridge.rs` modified

## Out of scope

- Attacking GAIA entities (separate PRD; needs game-core changes)
- Auto-acquire targets while attack-moving (sim already moves but won't
  re-acquire — fine for MVP)

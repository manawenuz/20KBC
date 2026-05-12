# PRD-24 — Godot Animation Hookup

## Goal

Make the existing UnitNode/GaiaNode behavior-switching code actually
play the right WC3 animation names. PRDs 21-23 produce a glb with
animations named "Stand", "Walk", "Attack - 1", "Death", etc. The
current Rust code uses generic placeholders like "idle"/"walk". Map
behavior → real animation name.

## Context

`UnitNode::process()` polls `SimBridge::get_unit_behavior(uid)`
(returns 0-3) and currently tries to switch animations on a cached
`AnimationPlayer` reference but uses placeholder strings.

WC3 sequence naming convention (verbatim from typical MDX):
- `Stand` — idle
- `Walk` — locomotion
- `Attack` or `Attack - 1` — primary attack
- `Death` — death sequence
- `Stand Work` — work idle (gathering)
- `Stand Ready` — alert idle
- `Birth` — spawn anim

## Files you MAY create

(none)

## Files you MAY modify

- `spikes/godot-gdext/rust/src/unit_node.rs`
- `spikes/godot-gdext/rust/src/gaia_node.rs`

## Files you MUST NOT touch

- `scripts/asset-extract/**`
- `main.gd`, `Main.tscn`, `project.godot`
- `game-core/**`
- Other Rust source

## Plan

For each of UnitNode and GaiaNode:

1. After loading the glb, find an `AnimationPlayer` child (recurse the
   tree if it's not at the top level — WC3 glbs may nest it).
2. Get the player's animation list. Build a per-behavior name lookup
   that tries variants and falls back gracefully:
   ```
   IDLE: try "Stand", "stand", "idle", "Idle" → first match
   WALK: try "Walk", "walk", "Run"
   ATTACK: try "Attack - 1", "Attack", "attack"
   GATHER: try "Stand Work", "Stand", "Walk"
   DEATH: try "Death", "death"
   ```
3. In `process()`, when behavior changes:
   - Look up the target anim name from the table.
   - If it differs from currently-playing: `player.play(name)`.
   - If lookup is None (no anim found): leave whatever was playing.
4. On `ready()`, autoplay the IDLE animation if found.

## Acceptance criteria

```bash
cd spikes/godot-gdext/rust && cargo build
godot --path /Users/manwe/CascadeProjects/20KBC/spikes/godot-gdext/ --quit-after 100
```

- [ ] Build clean
- [ ] Run the game: peasants should be playing the "Stand" idle
      animation immediately. When you right-click to move them, they
      switch to "Walk".
- [ ] No errors about missing animations even if a specific anim name
      isn't present (graceful fallback)
- [ ] Only `unit_node.rs` and `gaia_node.rs` modified

## Out of scope

- Animation blending / cross-fade
- Animation event hooks (footstep sound on specific frame, etc.)
- Direction-aware animations (turn left vs right)

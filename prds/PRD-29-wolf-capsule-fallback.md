# PRD-29 — Wolf: Revert to Red Capsule (TimberWolf Texture Broken)

## Goal

The user reports the WC3 TimberWolf glb renders without texture (all
white/grey). Until we sort out why the wolf's material texture isn't
applying (separate investigation), explicitly **force the capsule
fallback path** in `gaia_node.rs` so the wolf is visibly red and
distinct again instead of a colorless blob.

## Files you MAY modify

- `spikes/godot-gdext/rust/src/gaia_node.rs`

## Files you MUST NOT touch

- `scripts/asset-extract/**` — keep the wolf.glb extracted, just don't load it
- `main.gd`, `Main.tscn`, `project.godot`
- Other Rust source

## Plan

In `gaia_node.rs::ready()`, locate the `loader.load("res://assets/models/wolf.glb")`
call. Wrap the success path so it ALWAYS falls back to the capsule path
for now — easiest: just delete or short-circuit the model-loading branch
and call the capsule-spawning block unconditionally.

You may keep the model-loading code structure but make it inert (e.g.,
set `let model: Option<Gd<PackedScene>> = None;`) so the file diff is
small and easy to revert when the texture issue is fixed.

Add a brief comment explaining the temporary revert:
```rust
// TODO(20kbc): TimberWolf.glb extracted via PRD-21 has no applied
// texture — renders white. Capsule fallback until material wiring
// is fixed (probably needs material index alignment in mdx_to_gltf).
```

## Acceptance criteria

```bash
cd spikes/godot-gdext/rust && cargo build
godot --path /Users/manwe/CascadeProjects/20KBC/spikes/godot-gdext/ --quit-after 100
```

- [ ] Clean build
- [ ] At runtime the wolf is rendered as the red capsule (not a white blob)
- [ ] `git diff --stat` shows exactly 1 file modified

## Out of scope

- Fixing the actual texture-application bug
- Choosing a different model
- Re-extracting the wolf

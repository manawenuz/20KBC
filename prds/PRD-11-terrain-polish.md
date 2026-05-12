# PRD-11 — Terrain Texture Polish

## Goal

The current terrain reuses one tiny sub-rect of the WC3 grass atlas
(`uv 0.55..0.70, 0.05..0.45`) on every tile, which makes the entire
ground look like a uniform fuzzy carpet. Improve this:

1. Use **multiple sub-rects** of the atlas chosen pseudo-randomly per
   tile (deterministic on tile coords) for visual variation.
2. Add a procedural dirt path or patch of dirt tiles around the depot
   (use the extracted `ground_dirt.png` for these tiles).
3. Add a subtle dark border around the map edge so the playing field
   reads clearly.

## Context

Current implementation: `spikes/godot-gdext/rust/src/terrain_node.rs`.
Both `ground_grass.png` and `ground_dirt.png` are at
`spikes/godot-gdext/assets/textures/`. They are 512×256 atlases with
multiple tile variants (look at them to see — the right half has
mostly-clean grass/dirt tiles).

## Files you MAY create

(none)

## Files you MAY modify

- `spikes/godot-gdext/rust/src/terrain_node.rs`

## Files you MUST NOT touch

- Anything else

## Implementation hints

- Inspect the PNG files visually before writing UV code. The atlas
  layout is roughly: left 50% = decorative/edge tiles, right 50% = solid
  centre tiles you want to sample.
- A deterministic per-tile RNG: `let r = ((x.wrapping_mul(2654435761)) ^ z.wrapping_mul(40503)) % 4` and pick one of 4 UV rects.
- Dirt around depot: depot is at world center = `(64.0, 64.0)`. Mark
  any tile within ~8 world units (4 tiles) of depot as "dirt". Map tile
  (x, z): tile center = `(x as f32 + 0.5) * TILE`.
- For the dirt tiles you'll need a second surface (different texture).
  Easiest: build a second sub-mesh with its own material in the same
  ArrayMesh (`add_surface_from_arrays` twice).
- Border: a single dark quad around the perimeter (4 thin strips) using
  `Color { 0.05, 0.04, 0.02, 1 }` material.

## Acceptance criteria

```bash
cd spikes/godot-gdext/rust && cargo build
# then visually
godot --path /Users/manwe/CascadeProjects/20KBC/spikes/godot-gdext/ --quit-after 100
```

- [ ] `cargo build` clean
- [ ] No `godot_warn!` at startup for missing textures
- [ ] `git diff --stat` shows exactly 1 file changed

## Out of scope

- Heightmap / non-flat terrain
- Real splatmap blending — discrete tile-by-tile is fine
- Cliff edges

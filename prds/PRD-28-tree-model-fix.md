# PRD-28 — Real WC3 Tree Model in ResourceNode

## Goal

Replace the placeholder procedural `tree.glb` (small 4KB shape from PRD-10)
with a real WC3 tree model. The agent should:

1. Add a real WC3 tree to the extract list (use `LordaeronTree0.mdx` —
   the standard recognizable conifer).
2. Modify `resource_node_visual.rs` so `kind == 1` (Wood) loads the new
   WC3 tree glb. Keep the BoxMesh fallback for safety.

## Files you MAY modify

- `scripts/asset-extract/extract.py` — append one new mapping for
  `LordaeronTree0.mdx → lordaerontree.glb`.
- `spikes/godot-gdext/rust/src/resource_node_visual.rs` — change the
  model path for `kind == 1` from `tree.glb` to `lordaerontree.glb`
  AND ensure the scale-up applied to the loaded model is reasonable
  for the WC3 unit scale (try 0.04–0.06 — WC3 trees are larger in MDX
  units than peasants).

## Files you MUST NOT touch

- `scripts/asset-extract/mdx_to_gltf.py` — the parser/writer is fine
- `game-core/**`
- Other Rust source
- `main.gd`, `Main.tscn`, `project.godot`

## Plan

1. Add the mapping to extract.py:
   ```python
   "doodads/terrain/lordaerontree/lordaerontree0.mdx:lordaerontree.glb",
   ```
2. Run extraction once:
   ```bash
   python3 scripts/asset-extract/extract.py --mdx \
       --mpq /Volumes/samGames/WC3/War3.mpq \
       --out spikes/godot-gdext/assets/models/
   ```
3. In `resource_node_visual.rs::ready()`, find the line that picks the
   model path for wood and change it to point at `lordaerontree.glb`.
   Adjust the upscale factor:
   ```rust
   let s = if self.kind == 1 { 0.05 } else { 1.8 };
   ```
   (The previous `2.5` was for the tiny procedural tree.glb. WC3 trees
   are ~80 MDX units tall — same scale family as peasants.)

## Acceptance criteria

```bash
ls -la spikes/godot-gdext/assets/models/lordaerontree.glb   # exists, > 5KB
godot --headless --path spikes/godot-gdext/ --import        # exits 0
cd spikes/godot-gdext/rust && cargo build                   # clean
```

- [ ] `lordaerontree.glb` extracted
- [ ] `resource_node_visual.rs` references the new glb for kind==1
- [ ] BoxMesh fallback preserved
- [ ] ≤ 3 files changed

## Out of scope

- Tree variation by ResourceNode instance (all use same model)
- Animated tree wind/sway
- Multiple tree species

# PRD-10 — OSS CC0 Fallback Model Pack

## Goal

Ensure the project has working 3D models for peasant, wolf, tree, and
stone outcrop in `.glb` format, regardless of whether MDX extraction
(PRD-09) succeeds. Source from CC0 / public-domain packs so we can ship.

## Context

Quaternius (https://quaternius.com) and Kenney (https://kenney.nl)
publish CC0 glTF asset packs. We need 4 specific models:
1. A humanoid peasant or villager (Quaternius "Ultimate RPG Pack" or
   Kenney "Mini Characters") with at least an idle pose.
2. A wolf or generic quadruped (Quaternius "Ultimate Animated Animals"
   or any CC0 wolf).
3. A pine/oak/ashen tree (Quaternius "Stylized Nature").
4. A rock cluster / stone outcrop (Quaternius "Stylized Nature").

Static models are fine for the MVP — animations are added later.

## Approach

Either:
- **Download a known CC0 pack zip via curl** and extract just the four
  models we need. Common sources:
  - Quaternius packs: https://quaternius.com (CC0, no API but consistent URLs)
  - Kenney's CC0 assets: https://kenney.nl (zip distributions)
- **Or commit hand-picked CC0 models from another source.** Anything CC0
  is acceptable.

If a download fails (network restricted), fall back to programmatically
generating simple `.glb` files using Python `pygltflib` or by writing
the binary glTF format directly (the format is small — header + JSON
chunk + binary chunk).

Final committed files must be valid `.glb` parseable by Godot.

## Files you MAY create

- `spikes/godot-gdext/assets/models/peasant.glb`
- `spikes/godot-gdext/assets/models/wolf.glb`
- `spikes/godot-gdext/assets/models/tree.glb`
- `spikes/godot-gdext/assets/models/stone.glb`
- `spikes/godot-gdext/assets/models/CREDITS.md` (attribution + source URLs)
- Anything under `scripts/asset-fallback/` if you need a script

## Files you MUST NOT touch

- `scripts/asset-extract/` (PRD-09's territory)
- Anything else

## Acceptance criteria

```bash
ls -la spikes/godot-gdext/assets/models/
# Must show: peasant.glb wolf.glb tree.glb stone.glb CREDITS.md
file spikes/godot-gdext/assets/models/*.glb
# All must say "data" or be recognized as binary; non-zero size
```

And:

- [ ] Each `.glb` is < 5 MB
- [ ] `CREDITS.md` lists source URL + license for every model
- [ ] Godot import succeeds (`godot --headless --import` exits 0 from
      `spikes/godot-gdext/`)

## Out of scope

- Animation rigs (static mesh is fine; PRD-17/18 layer animation later)
- High-fidelity PBR materials
- LOD levels

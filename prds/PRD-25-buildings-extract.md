# PRD-25 — Extract WC3 Buildings (TownHall / Keep / Castle)

## Goal

Add 3 WC3 building MDX models to the asset pipeline so the orchestrator
can place them as decorative structures on the map. WC3 doesn't ship
separate Keep/Castle .mdx files (they're upgrades of TownHall with
geoset swaps), so we substitute:
- TownHall.mdx → "townhall.glb"
- HumanBarracks.mdx → "keep.glb"
- AltarOfKings.mdx → "castle.glb"

## Files you MAY modify

- `scripts/asset-extract/extract.py` — append the 3 mappings to the
  default `--mdx` list (around line 243).
- Output `.glb` files under `spikes/godot-gdext/assets/models/`
  (will be created by the extract script).

## Files you MUST NOT touch

- `scripts/asset-extract/mdx_to_gltf.py` (already does the right
  thing — don't change extraction logic)
- `spikes/godot-gdext/rust/**`
- `spikes/godot-gdext/scenes/`, `scripts/`, `project.godot`
- `game-core/**`

## Plan

In `scripts/asset-extract/extract.py`, extend the default mapping list:

```python
mappings = args.mdx if args.mdx else [
    "units/human/peasant/peasant.mdx:peasant.glb",
    "units/creeps/timberwolf/timberwolf.mdx:wolf.glb",
    "doodads/terrain/ashentree/ashentree0.mdx:ashentree.glb",
    "doodads/terrain/rockchunks/rockchunks0.mdx:rockchunks.glb",
    "buildings/human/townhall/townhall.mdx:townhall.glb",
    "buildings/human/humanbarracks/humanbarracks.mdx:keep.glb",
    "buildings/human/altarofkings/altarofkings.mdx:castle.glb",
]
```

Then run:
```bash
python3 scripts/asset-extract/extract.py --mdx \
    --mpq /Volumes/samGames/WC3/War3.mpq \
    --out spikes/godot-gdext/assets/models/
```

## Acceptance criteria

```bash
ls -la spikes/godot-gdext/assets/models/townhall.glb spikes/godot-gdext/assets/models/keep.glb spikes/godot-gdext/assets/models/castle.glb
file spikes/godot-gdext/assets/models/townhall.glb   # must say "glTF binary model"
godot --headless --path spikes/godot-gdext/ --import   # exits 0
```

- [ ] All 3 .glb files exist, > 5KB each, valid glTF
- [ ] Godot import succeeds with no errors
- [ ] `git diff --stat` shows only extract.py modified + the 3 new glbs

## Out of scope

- Spawning these in the sim (PRD-26 handles)
- Rendering them in Godot (PRD-27 handles)
- Team-color rendering on building roofs (separate concern)

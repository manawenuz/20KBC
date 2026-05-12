# PRD-21 — MDX Texture Extraction + glTF Material References

## Goal

Make the WC3 peasant and wolf show with real colors. Currently
`peasant.glb` and `wolf.glb` are untextured (PRD-09 shipped geometry
only). Extract the BLP textures these MDX files reference, convert to
PNG, and patch `mdx_to_gltf.py` to emit glTF materials that point at
those textures.

## Context

The MDX format includes a `TEXS` chunk listing texture file paths
relative to the MPQ root (typical: `Textures/Peasant.blp` or
`Units/Human/Peasant/Peasant.blp`). The existing extractor at
`scripts/asset-extract/mdx_to_gltf.py` already opens the MPQ and parses
MDX chunks — it just doesn't currently read `TEXS` or emit material
references.

BLP→PNG conversion already works elsewhere in the tool (PRD-04 extracted
`ground_grass.png` and `ground_dirt.png`). Reuse that decode path.

glTF 2.0 material support is trivial: a `materials` entry with
`pbrMetallicRoughness.baseColorTexture.index` referencing an `images`
entry that names the PNG file (external reference, not embedded — keeps
the glb size reasonable).

## Files you MAY create

- `spikes/godot-gdext/assets/models/textures/*.png` (any BLP-derived PNGs needed)

## Files you MAY modify

- `scripts/asset-extract/mdx_to_gltf.py`
- `scripts/asset-extract/extract.py` (only if you need a CLI wrapper change)
- `spikes/godot-gdext/assets/models/peasant.glb` (regenerated output)
- `spikes/godot-gdext/assets/models/wolf.glb` (regenerated output)
- `spikes/godot-gdext/assets/models/ashentree.glb` (regenerated output, optional)
- `spikes/godot-gdext/assets/models/rockchunks.glb` (regenerated output, optional)

## Files you MUST NOT touch

- `spikes/godot-gdext/rust/**`
- `spikes/godot-gdext/scenes/`, `scripts/`, `project.godot`
- `game-core/**`
- Other PRDs' files

## Plan

1. Extend `mdx_to_gltf.py` to parse the `TEXS` chunk: a flat list of
   file path strings (256 bytes each, null-terminated, plus a 4-byte
   flags field per entry).
2. For each unique texture path:
   - Read the BLP bytes from the MPQ.
   - Decode BLP → RGBA8 (JPEG or paletted variant — same decode you
     already have for ground textures).
   - Write as `spikes/godot-gdext/assets/models/textures/<name>.png`.
3. Extend the glTF emitter:
   - Build a `samplers` entry (default linear filtering).
   - Build an `images` entry per PNG with `uri` = relative path
     (e.g. `"textures/Peasant.png"`).
   - Build a `textures` entry linking `source` (image index) and `sampler`.
   - Build a `materials` entry with
     `pbrMetallicRoughness.baseColorTexture.index` = texture index,
     `pbrMetallicRoughness.metallicFactor = 0`, roughness ~0.9.
   - For each `meshes[i].primitives[j]`, set
     `material: <index>` to the matching texture for that geoset.
4. The MDX `GEOS` chunk has a per-geoset `materialId` byte index — use
   it to pick which `MTLS` material entry the geoset uses, then map the
   MTLS layer's `textureId` to the TEXS index. (If this is too hairy,
   fall back to: every geoset uses TEXS[0]. The peasant will look
   approximately right.)

## Acceptance criteria

```bash
# Re-extract from MPQ:
python scripts/asset-extract/extract.py --mdx \
    --mpq /Volumes/samGames/WC3/War3.mpq \
    --out spikes/godot-gdext/assets/models/

# Verify outputs:
ls spikes/godot-gdext/assets/models/textures/   # must contain PNGs
file spikes/godot-gdext/assets/models/peasant.glb   # still valid glTF
godot --headless --path spikes/godot-gdext/ --import   # imports cleanly
```

- [ ] `textures/` directory contains at least 1 PNG (Peasant.blp converted)
- [ ] peasant.glb is regenerated and references the PNG (use a glb
      inspector or just diff size — it should grow slightly)
- [ ] Godot import succeeds with no errors about missing textures
- [ ] `git diff --stat` shows changes only under the whitelist
- [ ] If TEXS parsing fails or BLP decode fails for a specific texture,
      log it but keep going — partial textures are better than nothing.

## Out of scope

- Bone hierarchy / skinning (PRD-22)
- Animation (PRD-23)
- Re-emitting tree/rock glbs with textures (allowed but optional)
- Embedded textures inside the glb (external `.png` refs are simpler)
- Team-color shader (WC3 uses TC_Glow textures — skip those)

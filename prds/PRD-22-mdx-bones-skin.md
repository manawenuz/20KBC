# PRD-22 — MDX Bones + Skinning → glTF Skin

## Goal

Eliminate the "floating arms / detached parts" look. Parse the MDX
`BONE`/`PIVT` chunks into a proper joint hierarchy and the geoset
vertex-matrix indices into glTF skin joints, so the mesh deforms as a
unified body. This is the second wave of the MDX-extension path —
PRD-21 textures are assumed merged.

## Context

In MDX800 (WC3 1.27):
- `BONE` chunk: a flat list of bones, each with a parent index and a
  reference to a node in the model's node tree.
- `PIVT` chunk: pivot points (one Vec3 per node) — these are the
  bone rest-pose positions in model space.
- `GEOS.MTGC` (per-geoset): a "matrix groups counts" array — number of
  vertices in each weight group.
- `GEOS.MATS` (per-geoset): a flat "matrix" array of bone indices per
  vertex group.
- `GEOS.VRTX[i].matrixId`: index into the geoset's groups, indirectly
  picking which bones influence vertex `i`.

WC3 vertices are **rigidly bound** — each vertex belongs to exactly one
or two bones (averaged transform), not arbitrary 4-weight skinning. For
glTF, emit weights summing to 1.0 distributed across the bones in that
vertex's group.

## Files you MAY create

(none, beyond what mdx_to_gltf.py emits as glb output)

## Files you MAY modify

- `scripts/asset-extract/mdx_to_gltf.py`
- `spikes/godot-gdext/assets/models/peasant.glb` (regenerated)
- `spikes/godot-gdext/assets/models/wolf.glb` (regenerated)

## Files you MUST NOT touch

- `spikes/godot-gdext/rust/**`
- Everything else outside the whitelist

## Plan

1. Parse `BONE` chunk → list of `{name, parent, pivot_node_id}`.
2. Parse the model's node tree (KGTR/KGRT/KGSC chunks have node
   references; the `NODE_TRANSLATION/ROTATION/SCALE` static transforms
   live in the node's `OBJ` header in MDX terms). For static rest pose,
   the pivot translation is enough.
3. Build a glTF `skins[0]` entry:
   - `joints`: indices into the glTF node list, one per BONE
   - `inverseBindMatrices`: for each joint, the inverse of the rest-pose
     world transform (= negative of pivot translation if no rotation/scale)
4. Build the glTF node hierarchy: one node per bone, with `translation`
   = pivot position relative to parent, `name` = bone name.
5. For each geoset emit:
   - `JOINTS_0` (uvec4 per vertex): up to 4 joint indices touching this vertex
     (usually just 1 or 2 in WC3)
   - `WEIGHTS_0` (vec4 per vertex): weights summing to 1.0
6. Reference the skin in each mesh node:
   `nodes[i] = { mesh: M, skin: 0 }`.

## Acceptance criteria

```bash
python scripts/asset-extract/extract.py --mdx \
    --mpq /Volumes/samGames/WC3/War3.mpq \
    --out spikes/godot-gdext/assets/models/

godot --headless --path spikes/godot-gdext/ --import
```

- [ ] Both peasant.glb and wolf.glb regenerate without error
- [ ] Inspecting the glb (e.g., via `python -m gltf-validator` if you have
      it, or by `unzip`/`hexdump` for the JSON chunk) shows a `skins` array
      with > 0 joints and `meshes[].primitives[].attributes.JOINTS_0`
- [ ] Godot import succeeds with no errors
- [ ] Run the game: peasants should now appear as one connected body in
      a T-pose (or whatever the MDX rest pose is) — NOT floating chunks
- [ ] `git diff --stat` shows changes only under the whitelist

## Out of scope

- Animations (PRD-23) — rest pose only
- Inverse kinematics
- Material mixing across geosets
- Per-vertex 4-weight blend (WC3 uses 1-2 bones — that's fine)
- Wolf or other models if they hit format edge cases — peasant is the
  priority; ship what works and document the gap

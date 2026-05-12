# PRD-23 — MDX Animations → glTF Animation Tracks

## Goal

Add playable animation clips to peasant.glb and wolf.glb. After this
PRD lands, Godot's AnimationPlayer should expose "Stand", "Walk",
"Attack", "Death" (and any others present) clips that drive the bones
from PRD-22.

## Context

MDX animation is keyframe-based per-bone. Relevant chunks:
- `SEQS`: list of named animation sequences with start/end frame
  ranges, looping flag, movement speed.
- Per-bone `KGTR`/`KGRT`/`KGSC`: translation, rotation, scale
  keyframe tracks. Each is a list of `(time_ms, value)` pairs plus
  an interpolation type (`NONE`, `LINEAR`, `HERMITE`, `BEZIER`).

Map MDX sequences to glTF animations one-to-one:
- glTF `animations[k].name` = MDX sequence name (e.g. `"Stand"`,
  `"Walk"`, `"Attack - 1"`, `"Death"`).
- For each bone, slice the bone's KGTR/KGRT/KGSC keyframes to those
  whose time falls inside `[start_ms, end_ms]` of the sequence, then
  rebase times to start at 0 (subtract `start_ms`).
- Emit glTF `animations[k].channels` and `samplers`:
  - One sampler per (bone, channel-type) holding `input` (times in
    seconds — divide ms by 1000) and `output` (translation/rotation/scale)
  - Channels target `node = <bone-node-index>` and
    `path = "translation" | "rotation" | "scale"`
  - Interpolation: MDX `HERMITE`/`BEZIER` → glTF `"CUBICSPLINE"`,
    MDX `LINEAR` → glTF `"LINEAR"`, MDX `NONE` → glTF `"STEP"`.

The CUBICSPLINE conversion can be tricky (glTF expects in/out tangents
in the output buffer). For first pass, **flatten Hermite/Bezier to
LINEAR** — sample at every keyframe time without tangent computation.
Animation will lose some smoothness but will look correct.

## Files you MAY create

(none)

## Files you MAY modify

- `scripts/asset-extract/mdx_to_gltf.py`
- `spikes/godot-gdext/assets/models/peasant.glb` (regenerated)
- `spikes/godot-gdext/assets/models/wolf.glb` (regenerated)

## Files you MUST NOT touch

- `spikes/godot-gdext/rust/**`
- Everything else outside the whitelist

## Acceptance criteria

```bash
python scripts/asset-extract/extract.py --mdx \
    --mpq /Volumes/samGames/WC3/War3.mpq \
    --out spikes/godot-gdext/assets/models/

godot --headless --path spikes/godot-gdext/ --import
```

- [ ] Both glbs regenerate
- [ ] Open peasant.glb in Godot (Import dock or just inspect generated
      `.scn`): there should be ≥ 3 animations named after MDX sequences
- [ ] Run the game (no behavior changes needed): if you set the
      UnitNode's AnimationPlayer to autoplay `"Stand"` (or whatever
      idle is called), peasants should idle-animate. This may require
      a small Rust tweak that PRD-24 handles — for this PRD, just
      verify the animations exist.
- [ ] `git diff --stat` shows changes only under the whitelist

## Out of scope

- Animation blending / smoothing (linear is fine)
- Particle/event tracks in MDX
- Sound tracks in MDX
- Wolf animations if format differs — best effort; document gaps

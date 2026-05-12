# PRD-37 — Warsmash-style Per-Frame Geoset Alpha

## Goal

Replace the **build-time geoset filter** in `mdx/builder.rs` with
**runtime per-frame alpha sampling**, mirroring how WC3 and the
WarsmashModEngine renderer drive geoset visibility.

Today we pick which geosets to keep at mesh-build time by sampling
GEOA at the Stand sequence midpoint. This is wrong — Blizzard's
peasants (and most units) use GEOA to *animate* alpha between idle /
lumber-carrying / gold-carrying / work variants. The basic Stand
sequence has the lumber bundle at alpha=0 and the bare-handed arms at
alpha=1; Stand Work flips that. A static build-time pick can only
match *one* of those poses — currently it loses the arms.

## Reference

`core/src/com/etheller/warsmash/viewer5/handlers/mdx/MdxComplexInstance.java`
lines 423–470 — read this before implementing. The relevant loop:

```java
for (int i = 0, l = geosets.size(); i < l; i++) {
    final Geoset geoset = geosets.get(i);
    final GeosetAnimation geosetAnimation = geoset.geosetAnimation;
    final float[] geosetColor = geosetColors[i];
    if (geosetAnimation != null) {
        if (forced || geosetAnimation.variants.get("alpha")[sequence] != 0) {
            geosetAnimation.getAlpha(alphaHeap, sequence, frame, counter);
            geosetColor[3] = alphaHeap[0];
        }
    } else {
        geosetColor[3] = 1;
    }
}
```

Per-instance, per-frame: sample each geoset's alpha curve at the
current `(sequence, frame)`, store in `geosetColor[3]`. Shader
multiplies vertex alpha by this. Geosets with alpha=0 are transparent.

## Files you MAY create

- `spikes/godot-gdext/rust/src/mdx_instance.rs` — new component that
  owns per-spawn animation/alpha state and runs the per-frame update.

## Files you MAY modify

- `spikes/godot-gdext/rust/src/lib.rs` — register the new module.
- `spikes/godot-gdext/rust/src/mdx/builder.rs` — **remove** the
  `kept_geosets` filter. Emit **all** geosets as separate surfaces.
  Return a new `BuiltMesh.geoset_indices: Vec<usize>` so the caller
  can map `surface_index → original geoset_index` (needed to look up
  the right GEOA curve at runtime).
- `spikes/godot-gdext/rust/src/asset_registry.rs` — extend
  `ResolvedModel` with the geoset→surface mapping and a reference
  (Arc) to the parsed `MdxModel` so the per-frame sampler can read
  GEOA curves. Materials should be configured for transparency
  (`StandardMaterial3D::set_transparency(Transparency::ALPHA)` and
  `set_blend_mode(BlendMode::MIX)`).
- `spikes/godot-gdext/rust/src/unit_node.rs`,
  `gaia_node.rs`, `building_node.rs`, `resource_node_visual.rs` —
  on `try_spawn_from_registry`, attach an `MdxInstance` child node
  that takes the `ResolvedModel` and the `MeshInstance3D`. Wire the
  current animation: when the existing behavior-driven animation
  switch fires, call `mdx_instance.set_sequence(name)` so the alpha
  sampler tracks the same sequence the `AnimationPlayer` is playing.

## Files you MUST NOT touch

- `mdx/parser.rs` — parsing is correct; do not change.
- `mdx/animation.rs` — bone tracks are correct.
- `mdx/skin.rs` — bind matrices are correct.
- `blp/**`, `datasource/**`, `main.gd`, `Main.tscn`, `game-core/**`.

## Interface contract

```rust
// mdx_instance.rs
use godot::prelude::*;
use godot::classes::{INode, MeshInstance3D, Node, StandardMaterial3D};
use std::sync::Arc;
use crate::mdx::types::MdxModel;

#[derive(GodotClass)]
#[class(base = Node)]
pub struct MdxInstance {
    base: Base<Node>,
    model: Option<Arc<MdxModel>>,
    /// `surface_index → original geoset index` (mirror of BuiltMesh.geoset_indices)
    surface_to_geoset: Vec<usize>,
    /// Materials we created in asset_registry, kept here so we can mutate albedo.a per frame.
    materials: Vec<Gd<StandardMaterial3D>>,
    /// Current sequence start/end in ms; None = no sequence playing.
    current_seq: Option<(u32, u32)>,
    /// Time within the current sequence, in ms.
    elapsed_ms: f32,
}

#[godot_api]
impl INode for MdxInstance {
    fn process(&mut self, delta: f64) {
        let Some((start, end)) = self.current_seq else { return };
        let Some(ref model) = self.model else { return };
        let dur = (end - start) as f32;
        if dur > 0.0 {
            self.elapsed_ms = (self.elapsed_ms + delta as f32 * 1000.0) % dur;
        }
        let t = start + self.elapsed_ms as u32;
        for (surf_idx, &geoset_idx) in self.surface_to_geoset.iter().enumerate() {
            let alpha = match model.geoset_alpha.get(geoset_idx).and_then(|o| o.as_ref()) {
                Some(entry) => sample_alpha_at(entry, t),
                None => 1.0,
            };
            if let Some(mat) = self.materials.get_mut(surf_idx) {
                let mut c = mat.get_albedo();
                c.a = alpha;
                mat.set_albedo(c);
            }
        }
    }
}

#[godot_api]
impl MdxInstance {
    pub fn configure(
        &mut self,
        model: Arc<MdxModel>,
        surface_to_geoset: Vec<usize>,
        materials: Vec<Gd<StandardMaterial3D>>,
    );

    /// Switch to the named sequence (e.g. "Stand", "Walk", "Stand Work").
    /// No-op if the name isn't found.
    #[func]
    pub fn set_sequence(&mut self, name: GString);
}
```

`sample_alpha_at` already exists in `mdx/builder.rs` — move it to
`mdx/mod.rs` as a `pub fn` so `MdxInstance` can reuse it. Same
interpolation semantics (clamp at curve ends, no key → static_alpha).

## Acceptance criteria

```bash
cd spikes/godot-gdext/rust && cargo build
godot --path /Users/manwe/CascadeProjects/20KBC/spikes/godot-gdext/ --quit-after 300
```

- [ ] Build clean, no new warnings.
- [ ] Peasant spawns with **both arms visible** during Stand.
- [ ] When a peasant gets a Gather order and starts chopping a tree,
      the lumber bundle appears on its shoulder smoothly (alpha fades
      in over the chop animation, doesn't pop).
- [ ] Static geosets (no GEOA entry at all) render at full alpha.
- [ ] No alpha-test "halo" around peasant edges (use Transparency::ALPHA
      + sorted draw order, not alpha-scissor).
- [ ] Tree, wolf, buildings still render correctly (they have GEOA
      tracks for death fade and damage variants — those should
      activate but not on the default Stand).
- [ ] ≤ 6 files modified.

## Out of scope

- Per-vertex color tracks (GEOA's RGB component; we only care about
  alpha for now).
- Layer alpha tracks (MdlxLayer's own KMTA — separate from GEOA).
  This drives shader-level effects like the moving water on docks and
  the team-color glow; defer to a later PRD.
- Replaceable texture swaps (team color, lordaeron variants).
- Node visibility propagation (`node.visible` flag in MDX). Add only
  if visibly needed.
- Cross-sequence alpha blending (Warsmash interpolates between
  outgoing and incoming sequence for a few hundred ms). Snap is fine
  for v1.

## Why this fixes "missing arms" and "stacked variants"

Both bugs share the same root cause: a build-time filter has to make
a single binary choice per geoset, but the asset is designed for
animated alpha. With per-frame sampling, *all* geosets exist in the
mesh, but the GPU sees:

- Stand: arms alpha=1, lumber alpha=0, gold alpha=0 → only arms render.
- Stand Work: arms alpha=0, lumber alpha=1 → only lumber renders.

No stacking, no missing geometry, just the correct mesh per frame.

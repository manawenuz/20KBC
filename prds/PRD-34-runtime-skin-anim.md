# PRD-34 — Runtime Skin + Animation (depends on PRD-32)

## Goal

Extend the runtime MDX → Godot pipeline with skeleton skinning + animated
playback. After PRD-32 lands the static mesh, this PRD adds the
`Skeleton3D` + `Skin` + `AnimationPlayer` so peasants actually walk.

## Files you MAY create

- `spikes/godot-gdext/rust/src/mdx/skin.rs`
- `spikes/godot-gdext/rust/src/mdx/animation.rs`

## Files you MAY modify

- `spikes/godot-gdext/rust/src/mdx/mod.rs` — re-export new builders only
- `spikes/godot-gdext/rust/src/mdx/builder.rs` — add `build_skeleton()`
  + `build_animation_library()` next to existing `build_mesh()`. Don't
  touch the geometry path.
- `spikes/godot-gdext/rust/src/mdx/parser.rs` — extend Bone struct to
  include `translations`, `rotations`, `scales` keyframe vectors
  (parsed from KGTR / KGRT / KGSC inside BONE chunk entries)

## Files you MUST NOT touch

- `spikes/godot-gdext/rust/src/datasource/**`
- `spikes/godot-gdext/rust/src/blp/**`
- `spikes/godot-gdext/rust/src/wc3_material/**`
- Other Rust source

## Interface contract

```rust
// mdx/skin.rs
use godot::prelude::*;
use godot::classes::{Skeleton3D, Skin};

pub fn build_skeleton(model: &MdxModel) -> Gd<Skeleton3D>;
pub fn build_skin(model: &MdxModel, skeleton: &Gd<Skeleton3D>) -> Gd<Skin>;
```

```rust
// mdx/animation.rs
use godot::prelude::*;
use godot::classes::{Animation, AnimationLibrary};

/// Build one Animation per MDX sequence, named after the sequence
/// (e.g., "Stand", "Walk", "Attack - 1"). Tracks target node paths
/// relative to a Skeleton3D root.
pub fn build_animation_library(model: &MdxModel, skeleton_path: NodePath) -> Gd<AnimationLibrary>;
```

Loop mode: sequences with `loop = true` (MDX SEQS no_loop == 0) get
`LoopMode::LINEAR`; others stay one-shot.

Interpolation:
- MDX `NONE` (0) → `Animation::INTERPOLATION_NEAREST`
- MDX `LINEAR` (1) → `Animation::INTERPOLATION_LINEAR`
- MDX `HERMITE` / `BEZIER` (2/3) → flatten to LINEAR (PRD-32 era choice)

## Acceptance criteria

```rust
#[test]
fn peasant_skeleton_22_bones() {
    let model = parse_mdx(&std::fs::read("tests/data/peasant.mdx").unwrap()).unwrap();
    let skel = build_skeleton(&model);
    assert!(skel.get_bone_count() >= 22);  // bones + helpers
}

#[test]
fn peasant_animation_library_has_stand_walk() {
    let model = parse_mdx(&std::fs::read("tests/data/peasant.mdx").unwrap()).unwrap();
    let lib = build_animation_library(&model, NodePath::from("Skeleton3D"));
    let names: Vec<_> = lib.get_animation_list().to_vec();
    assert!(names.iter().any(|n| n.to_string() == "Stand"));
    assert!(names.iter().any(|n| n.to_string() == "Walk"));
}
```

- [ ] Both tests pass
- [ ] `cargo build` clean
- [ ] No #[class] exposed yet
- [ ] ≤ 4 files modified

## Out of scope

- Particle / event / sound tracks inside MDX
- Animation blending across sequences
- Inverse kinematics
- Multi-skin meshes (LOD)

# PRD-32 — Runtime MDX Parser → Godot ArrayMesh

## Goal

Port the MDX800 parser from `scripts/asset-extract/mdx_to_gltf.py` to
Rust as a runtime parser that produces Godot `ArrayMesh` resources
directly. No more offline glTF intermediate step.

This is the biggest piece. **Aim for parity with the offline parser's
geometry + texture-name extraction.** Skinning and animations are
separate PRDs (PRD-34); for this PRD we just need the bind-pose mesh
+ material/texture references emitted.

## Files you MAY create

- `spikes/godot-gdext/rust/src/mdx/mod.rs`
- `spikes/godot-gdext/rust/src/mdx/parser.rs` — chunk walking (VERS,
  MODL, GEOS, TEXS, MTLS, BONE, HELP, PIVT, SEQS, GEOA)
- `spikes/godot-gdext/rust/src/mdx/types.rs` — `MdxModel`, `Geoset`,
  `Material`, `Texture`, `Bone`, `Sequence`, `GeosetAlpha` structs
- `spikes/godot-gdext/rust/src/mdx/builder.rs` — `MdxModel → Gd<ArrayMesh>`
  (one surface per kept geoset, GEOA filter as in current Python writer)

## Files you MAY modify

- `spikes/godot-gdext/rust/src/lib.rs` — add `mod mdx;` only

## Files you MUST NOT touch

- `spikes/godot-gdext/rust/src/datasource/**` (PRD-30 territory)
- `spikes/godot-gdext/rust/src/blp/**` (PRD-31 territory)
- `scripts/asset-extract/**`
- `game-core/**`

## Interface contract

```rust
// mdx/types.rs
pub struct MdxModel {
    pub textures: Vec<TextureRef>,        // path strings
    pub materials: Vec<MaterialDef>,
    pub geosets: Vec<Geoset>,
    pub bones: Vec<Bone>,
    pub helpers: Vec<Bone>,                // HELP chunk
    pub pivots: Vec<[f32; 3]>,
    pub sequences: Vec<Sequence>,
    pub geoset_alpha: Vec<Option<GeosetAlpha>>,
}
```

```rust
// mdx/parser.rs
pub fn parse_mdx(bytes: &[u8]) -> Result<MdxModel, String>;
```

```rust
// mdx/builder.rs
use godot::prelude::*;
use godot::classes::ArrayMesh;

pub struct BuiltMesh {
    pub mesh: Gd<ArrayMesh>,
    /// Texture path for surface i, from MTLS→TEXS resolution.
    /// `None` if the material layer doesn't reference a texture.
    pub surface_textures: Vec<Option<String>>,
}

pub fn build_mesh(model: &MdxModel) -> BuiltMesh;
```

Use exactly the same geometry conventions as the Python writer:
- Per-tile triangulation from GEOS.PVTX + UVBS + GNDX + FACS
- WC3 → glTF axis conversion (Z-up → Y-up; for runtime Godot is also
  Y-up so the same conversion applies)
- GEOA Stand-window filter for alt geosets (the strict rule from the
  most recent commit, NOT the loose "max anywhere" variant)
- Apply +1 node offset for children (avoid the self-cycle bug from
  earlier — `parent_id != object_id` defensive check)

## Acceptance criteria

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn parse_peasant() {
        let bytes = std::fs::read("tests/data/peasant.mdx").unwrap();
        let model = super::parser::parse_mdx(&bytes).unwrap();
        assert!(model.geosets.len() >= 6);
        assert!(model.bones.len() > 0);
        assert!(model.sequences.iter().any(|s| s.name.contains("Stand")));
    }

    #[test]
    fn build_peasant_mesh_kept_2_surfaces() {
        let bytes = std::fs::read("tests/data/peasant.mdx").unwrap();
        let model = super::parser::parse_mdx(&bytes).unwrap();
        let built = super::builder::build_mesh(&model);
        // Strict filter keeps geosets 0 and 1 (head + body) = 2 surfaces
        assert_eq!(built.surface_textures.len(), 2);
    }
}
```

Place a `tests/data/peasant.mdx` fixture (commit the ~140KB file —
acceptable cost for stable tests).

- [ ] Tests pass
- [ ] `cargo build` clean
- [ ] No GDExtension `#[class]` exposed yet — PRD-36 (orchestrator
  integration, not in this batch) will wire `UnitNode` to use this
- [ ] ≤ 6 files modified

## Out of scope

- Animation track parsing (PRD-34 — bones-only here)
- Skin weights → `Skin` resource (PRD-34)
- Texture loading from MPQ (PRD-30 + PRD-31 wire up; this returns names)
- WC3 layered materials (PRD-33)
- GDExtension class for runtime loading (later integration PRD)

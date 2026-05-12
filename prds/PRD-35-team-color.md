# PRD-35 — Team-Color Texture Substitution

## Goal

Implement WC3's runtime team-color mechanic. Materials whose texture
references the `ReplaceableId` for team color (`ReplaceableId = 1`) get
their texture swapped at render time to `ReplaceableTextures/TeamColor00.blp`
… `TeamColor11.blp` based on the unit's owning player.

## Files you MAY create

- `spikes/godot-gdext/rust/src/team_color.rs`

## Files you MAY modify

- `spikes/godot-gdext/rust/src/lib.rs` — add `mod team_color;` only
- `spikes/godot-gdext/rust/src/wc3_material/mod.rs` — extend
  `LayerSpec` with an optional `team_color_slot: Option<u8>` field
  (None = use texture as-is; Some(slot) = substitute at material build
  time). Don't touch the shader code itself.

## Files you MUST NOT touch

- `spikes/godot-gdext/rust/src/datasource/**`
- `spikes/godot-gdext/rust/src/blp/**`
- `spikes/godot-gdext/rust/src/mdx/**`
- `main.gd`, `Main.tscn`, `project.godot`

## Interface contract

```rust
// team_color.rs
use godot::classes::Texture2D;
use godot::obj::Gd;

/// The 12 official WC3 team colors (Red, Blue, Teal, Purple, Yellow,
/// Orange, Green, Pink, Grey, LightBlue, DarkGreen, Brown).
#[derive(Copy, Clone)]
pub enum TeamColor {
    Red, Blue, Teal, Purple, Yellow, Orange,
    Green, Pink, Grey, LightBlue, DarkGreen, Brown,
}

impl TeamColor {
    pub fn slot(self) -> u8;
    pub fn mpq_path(self) -> &'static str;  // "ReplaceableTextures/TeamColor/TeamColor00.blp"
}

/// Lazy-loaded cache of team-color Texture2Ds, one per slot. Looks up
/// the BLP via PRD-30 DataSource and decodes via PRD-31 BLP decoder
/// on first access.
pub struct TeamColorCache {
    /* internals */
}

impl TeamColorCache {
    pub fn new(ds: std::sync::Arc<dyn crate::datasource::DataSource>) -> Self;
    pub fn texture(&mut self, color: TeamColor) -> Option<Gd<Texture2D>>;
}
```

In `wc3_material/mod.rs` extend `LayerSpec`:

```rust
pub struct LayerSpec {
    pub texture: Gd<Texture2D>,
    pub filter:  FilterMode,
    pub unshaded: bool,
    pub two_sided: bool,
    pub team_color_slot: Option<u8>,   // NEW
}
```

`build_material(layers)` checks `team_color_slot` and, if present, the
caller is responsible for having pre-resolved the substituted texture
into `layer.texture`. (No global state in the shader.)

## Acceptance criteria

```rust
#[test]
fn all_12_team_color_paths_well_formed() {
    use TeamColor::*;
    let all = [Red, Blue, Teal, Purple, Yellow, Orange, Green, Pink, Grey, LightBlue, DarkGreen, Brown];
    for c in all {
        assert!(c.mpq_path().starts_with("ReplaceableTextures"));
        assert!(c.mpq_path().ends_with(".blp"));
        assert!(c.slot() < 12);
    }
}
```

- [ ] Test passes
- [ ] `cargo build` clean
- [ ] `LayerSpec` gets the new field WITHOUT breaking PRD-33's
      existing constructor — if needed, add `LayerSpec::new(...)` that
      defaults `team_color_slot` to `None`
- [ ] ≤ 3 files modified

## Out of scope

- ReplaceableId values other than 1 (TeamColor). Skip TeamGlow (2),
  Cliff (11), etc. for now.
- Per-unit color override (always-team-of-owner is fine)
- Animated team color tints (e.g. flashing when low HP)

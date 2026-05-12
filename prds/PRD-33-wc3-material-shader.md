# PRD-33 — WC3 Multi-Layer Material Shader

## Goal

Implement a Godot shader resource that reproduces WC3's multi-layer
material rendering. WC3 materials have 1–4 layers; each layer has a
filter mode (`NONE`, `TRANSPARENT`, `BLEND`, `ADDITIVE`, `ADD_ALPHA`,
`MODULATE`, `MODULATE2X`) and a texture. Standard PBR can't express
these; we need a custom shader.

This PRD ships **only the shader + a Rust factory that builds a
ShaderMaterial** from a layer description. PRD-32 emits layer info;
PRD-35 (team color) substitutes textures; this is the rendering core.

## Files you MAY create

- `spikes/godot-gdext/assets/shaders/wc3_layered.gdshader`
- `spikes/godot-gdext/rust/src/wc3_material/mod.rs`

## Files you MAY modify

- `spikes/godot-gdext/rust/src/lib.rs` — add `mod wc3_material;` only

## Files you MUST NOT touch

- Other Rust source
- `main.gd`, `Main.tscn`, `project.godot`
- `game-core/**`
- `scripts/asset-extract/**`

## Interface contract

```rust
// wc3_material/mod.rs
use godot::prelude::*;
use godot::classes::{ShaderMaterial, Texture2D};

pub enum FilterMode {
    None_       = 0,
    Transparent = 1,
    Blend       = 2,
    Additive    = 3,
    AddAlpha    = 4,
    Modulate    = 5,
    Modulate2x  = 6,
}

pub struct LayerSpec {
    pub texture: Gd<Texture2D>,
    pub filter:  FilterMode,
    pub unshaded: bool,           // MDL "Unshaded" flag
    pub two_sided: bool,          // MDL "TwoSided" flag
}

/// Build a ShaderMaterial parameterised with up to N layers.
pub fn build_material(layers: &[LayerSpec]) -> Gd<ShaderMaterial>;
```

```gdshader
// wc3_layered.gdshader
shader_type spatial;
render_mode unshaded;  // emulated; switch via uniform

uniform sampler2D layer0;
uniform sampler2D layer1;
uniform sampler2D layer2;
uniform sampler2D layer3;
uniform int filter0;
uniform int filter1;
uniform int filter2;
uniform int filter3;
uniform int layer_count;

void fragment() {
    vec4 c = texture(layer0, UV);
    if (layer_count > 1) {
        vec4 next = texture(layer1, UV);
        c = apply_filter(c, next, filter1);
    }
    // ... up to layer3
    ALBEDO = c.rgb;
    ALPHA = c.a;
}
```

`apply_filter()` covers the WC3 filter modes. Reference table for
implementation:

| Mode | GL blend |
|---|---|
| None_ | base = next |
| Transparent | discard if alpha < 0.75 |
| Blend | mix(base, next.rgb, next.a) |
| Additive | base + next.rgb |
| AddAlpha | base + next.rgb * next.a |
| Modulate | base * next.rgb |
| Modulate2x | base * next.rgb * 2.0 |

## Acceptance criteria

```bash
cd spikes/godot-gdext/rust && cargo build
godot --headless --path /Users/manwe/CascadeProjects/20KBC/spikes/godot-gdext/ --import
```

- [ ] `cargo build` clean
- [ ] `wc3_layered.gdshader` parses (no Godot parse errors at import)
- [ ] `build_material(&[layer_spec])` returns a valid `Gd<ShaderMaterial>`
- [ ] Single-layer material renders the texture as-is (regression-equivalent
      to current StandardMaterial3D behaviour)
- [ ] ≤ 3 files modified

## Out of scope

- TeamColor substitution (PRD-35)
- Vertex animation
- Particle / ribbon materials
- Light influence (use `unshaded` mode for parity with WC3's flat look)

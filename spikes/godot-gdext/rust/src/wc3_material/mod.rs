//! WC3 multi-layer material factory.
//!
//! Builds a `ShaderMaterial` from up to 4 texture layers with WC3 filter
//! modes (`NONE`, `TRANSPARENT`, `BLEND`, `ADDITIVE`, `ADD_ALPHA`,
//! `MODULATE`, `MODULATE2X`).

use godot::prelude::*;
use godot::classes::{ResourceLoader, Shader, ShaderMaterial, Texture2D};

/// WC3 texture-layer filter modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FilterMode {
    None_       = 0,
    Transparent = 1,
    Blend       = 2,
    Additive    = 3,
    AddAlpha    = 4,
    Modulate    = 5,
    Modulate2x  = 6,
}

/// Description of one material layer.
pub struct LayerSpec {
    pub texture: Gd<Texture2D>,
    pub filter:  FilterMode,
    pub unshaded: bool,           // MDL "Unshaded" flag
    pub two_sided: bool,          // MDL "TwoSided" flag
    pub team_color_slot: Option<u8>, // None = use texture as-is; Some(slot) = substitute at material build time
}

impl LayerSpec {
    pub fn new(texture: Gd<Texture2D>, filter: FilterMode) -> Self {
        Self {
            texture,
            filter,
            unshaded: false,
            two_sided: false,
            team_color_slot: None,
        }
    }
}

/// Build a `ShaderMaterial` parameterised with up to 4 layers.
///
/// Loads the shared `wc3_layered.gdshader` resource and binds textures /
/// filter-mode uniforms according to `layers`.
pub fn build_material(layers: &[LayerSpec]) -> Gd<ShaderMaterial> {
    assert!(!layers.is_empty() && layers.len() <= 4,
            "wc3 material requires 1–4 layers, got {}", layers.len());

    let mut mat = ShaderMaterial::new_gd();

    // Load shader resource (cached by Godot after first load).
    let shader: Gd<Shader> = ResourceLoader::singleton()
        .load("res://assets/shaders/wc3_layered.gdshader")
        .and_then(|r| r.try_cast::<Shader>().ok())
        .expect("wc3_layered.gdshader not found");
    mat.set_shader(&shader);

    // Bind layer_count uniform.
    mat.set_shader_parameter(
        "layer_count",
        &Variant::from(layers.len() as i32),
    );

    for (i, layer) in layers.iter().enumerate() {
        let tex_name = format!("layer{}", i);
        mat.set_shader_parameter(
            &tex_name,
            &Variant::from(layer.texture.clone()),
        );

        // Filter for layer 0 is not used by the shader (base colour),
        // but we set it for completeness.
        let filter_name = format!("filter{}", i);
        mat.set_shader_parameter(
            &filter_name,
            &Variant::from(layer.filter as i32),
        );
    }

    // For unused layers, bind a default white texture so the shader
    // doesn't sample an unbound sampler.
    for i in layers.len()..4 {
        let tex_name = format!("layer{}", i);
        // null / nil texture — Godot will use the sampler's hint_default_white
        mat.set_shader_parameter(&tex_name, &Variant::nil());

        let filter_name = format!("filter{}", i);
        mat.set_shader_parameter(&filter_name, &Variant::from(0i32));
    }

    mat
}

//! Per-spawn MDX animation state: drives per-frame geoset alpha from the
//! current sequence's GEOA curves (Warsmash-style). Attach as a child of
//! any visual node that mounted a runtime MDX mesh; call `set_sequence`
//! when the underlying `AnimationPlayer` switches sequence so the alpha
//! sampler tracks the same window.

use std::rc::Rc;

use godot::classes::{INode, Node, StandardMaterial3D};
use godot::prelude::*;

use crate::mdx::sample_alpha_at;
use crate::mdx::types::MdxModel;

#[derive(GodotClass)]
#[class(base = Node)]
pub struct MdxInstance {
    base: Base<Node>,
    model: Option<Rc<MdxModel>>,
    surface_to_geoset: Vec<usize>,
    materials: Vec<Gd<StandardMaterial3D>>,
    /// Current sequence window in absolute MDX time (ms). `None` = paused
    /// (no alpha mutation per frame; surfaces stay at their last alpha).
    current_seq: Option<(u32, u32)>,
    elapsed_ms: f32,
}

#[godot_api]
impl INode for MdxInstance {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            model: None,
            surface_to_geoset: Vec::new(),
            materials: Vec::new(),
            current_seq: None,
            elapsed_ms: 0.0,
        }
    }

    fn process(&mut self, delta: f64) {
        let Some((start, end)) = self.current_seq else { return };
        let Some(ref model) = self.model else { return };
        let dur = end.saturating_sub(start) as f32;
        if dur <= 0.0 {
            return;
        }
        self.elapsed_ms = (self.elapsed_ms + delta as f32 * 1000.0).rem_euclid(dur);
        let t = start + self.elapsed_ms as u32;
        for (surf_idx, &geoset_idx) in self.surface_to_geoset.iter().enumerate() {
            let alpha = match model.geoset_alpha.get(geoset_idx).and_then(|o| o.as_ref()) {
                Some(entry) => sample_alpha_at(entry, t),
                None => 1.0,
            };
            if let Some(mat) = self.materials.get_mut(surf_idx) {
                let mut c = mat.get_albedo();
                c.a = alpha.clamp(0.0, 1.0);
                mat.set_albedo(c);
            }
        }
    }
}

impl MdxInstance {
    /// Wire up the model + materials. Call right after the parent node
    /// has built its `MeshInstance3D` and surface override materials.
    pub fn configure(
        &mut self,
        model: Rc<MdxModel>,
        surface_to_geoset: Vec<usize>,
        materials: Vec<Gd<StandardMaterial3D>>,
    ) {
        self.model = Some(model);
        self.surface_to_geoset = surface_to_geoset;
        self.materials = materials;
        self.elapsed_ms = 0.0;
    }
}

#[godot_api]
impl MdxInstance {
    /// Switch to the named sequence. No-op if the model isn't configured
    /// or the name doesn't match. Matches are case-insensitive prefix.
    #[func]
    pub fn set_sequence(&mut self, name: GString) {
        let Some(ref model) = self.model else { return };
        let target = name.to_string().to_lowercase();
        if let Some(seq) = model
            .sequences
            .iter()
            .find(|s| s.name.to_lowercase() == target)
            .or_else(|| {
                model
                    .sequences
                    .iter()
                    .find(|s| s.name.to_lowercase().starts_with(&target))
            })
        {
            self.current_seq = Some((seq.start_ms, seq.end_ms));
            self.elapsed_ms = 0.0;
        }
    }
}

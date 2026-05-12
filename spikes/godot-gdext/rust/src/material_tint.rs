//! Helper for tinting untextured glb surfaces.
//!
//! Some WC3 MDX materials reference no texture (filter / team-color
//! layers stripped during conversion). Without a texture they render
//! as pure white in Godot. Walking the loaded glb subtree and
//! overriding only those untextured surfaces with a solid colour
//! preserves the WC3 textures we DO have while killing the white
//! blobs (peasant scraps, altar banner, etc.).

use godot::prelude::*;
use godot::classes::{Material, MeshInstance3D, Node, StandardMaterial3D};
use godot::classes::base_material_3d::ShadingMode;

/// Recursively walk `node` and, for every `MeshInstance3D` surface whose
/// material lacks a `albedo_texture`, install a solid-colour
/// `StandardMaterial3D` override using `fallback`.
pub fn paint_untextured(node: &Gd<Node>, fallback: Color) {
    for i in 0..node.get_child_count() {
        let Some(child) = node.get_child(i) else { continue };
        if let Ok(mut mi) = child.clone().try_cast::<MeshInstance3D>() {
            if let Some(mesh) = mi.get_mesh() {
                let surf_count = mesh.get_surface_count();
                for s in 0..surf_count {
                    let has_texture = mi
                        .get_active_material(s)
                        .and_then(|m| m.try_cast::<StandardMaterial3D>().ok())
                        .and_then(|sm| {
                            sm.get_texture(
                                godot::classes::base_material_3d::TextureParam::ALBEDO,
                            )
                        })
                        .is_some();
                    if has_texture {
                        continue;
                    }
                    let mut mat = StandardMaterial3D::new_gd();
                    mat.set_albedo(fallback);
                    mat.set_shading_mode(ShadingMode::PER_PIXEL);
                    mi.set_surface_override_material(s, &mat.upcast::<Material>());
                }
            }
        }
        paint_untextured(&child, fallback);
    }
}

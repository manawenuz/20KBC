use godot::prelude::*;
use godot::classes::ArrayMesh;
use godot::classes::mesh::PrimitiveType;

use super::types::*;

pub use super::skin::{build_skeleton, build_skin};
pub use super::animation::build_animation_library;

pub struct BuiltMesh {
    pub mesh: Gd<ArrayMesh>,
    pub surface_textures: Vec<Option<String>>,
}

pub fn build_mesh(model: &MdxModel) -> BuiltMesh {
    // GEOA Stand-window filter
    let stand = model
        .sequences
        .iter()
        .find(|s| s.name.to_lowercase().starts_with("stand"))
        .or_else(|| model.sequences.first());

    let kept_geosets: Vec<(usize, &Geoset)> = model
        .geosets
        .iter()
        .enumerate()
        .filter(|(gidx, _)| {
            if let Some(ref stand_seq) = stand {
                if let Some(Some(ref entry)) = model.geoset_alpha.get(*gidx) {
                    let alpha = alpha_in_window(entry, stand_seq.start_ms, stand_seq.end_ms);
                    alpha >= 0.5
                } else {
                    true
                }
            } else {
                true
            }
        })
        .collect();

    let mut mesh = ArrayMesh::new_gd();
    let mut surface_textures = Vec::new();

    for (_, g) in kept_geosets {
        if g.faces.is_empty() {
            continue;
        }

        let mut positions = PackedVector3Array::new();
        let mut normals = PackedVector3Array::new();
        let mut uvs = PackedVector2Array::new();
        let mut indices = PackedInt32Array::new();

        for v_idx in 0..g.vertex_positions.len() {
            let pos = g.vertex_positions[v_idx];
            let nrm = g.vertex_normals.get(v_idx).copied().unwrap_or([0.0, 0.0, 1.0]);

            // WC3 → Y-up (same as glTF writer)
            let p = wc3_to_godot_pos(pos);
            let n = wc3_to_godot_normal(nrm);

            positions.push(Vector3::new(p[0], p[1], p[2]));
            normals.push(Vector3::new(n[0], n[1], n[2]));

            let uv = if v_idx < g.uvs.len() {
                g.uvs[v_idx]
            } else {
                [0.0, 0.0]
            };
            // Flip V for Godot
            uvs.push(Vector2::new(uv[0], 1.0 - uv[1]));
        }

        // Reverse winding: the (x, y, z) → (x, z, -y) WC3→Godot transform
        // flips handedness, turning CCW triangles into CW. Without this
        // swap, Godot's default backface culling hides every front face
        // — visible only as thin "blade" silhouettes from edge-on angles.
        for (i0, i1, i2) in &g.faces {
            indices.push(*i0 as i32);
            indices.push(*i2 as i32);
            indices.push(*i1 as i32);
        }

        let mut arrays = VarArray::new();
        arrays.resize(13, &Variant::nil());
        arrays.set(0, &positions.to_variant());
        arrays.set(1, &normals.to_variant());
        arrays.set(4, &uvs.to_variant());
        arrays.set(12, &indices.to_variant());

        mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);

        // Resolve texture name from material layer 0
        let tex_name = resolve_texture_name(model, g.material_id);
        surface_textures.push(tex_name);
    }

    BuiltMesh { mesh, surface_textures }
}

fn alpha_in_window(entry: &GeosetAlpha, win_start_ms: u32, win_end_ms: u32) -> f32 {
    let in_window: Vec<f32> = entry
        .keys
        .iter()
        .filter(|(t, _)| *t >= win_start_ms && *t <= win_end_ms)
        .map(|(_, a)| *a)
        .collect();
    if !in_window.is_empty() {
        in_window.into_iter().fold(f32::NEG_INFINITY, f32::max)
    } else {
        entry.static_alpha
    }
}

fn wc3_to_godot_pos(pos: [f32; 3]) -> [f32; 3] {
    let [x, y, z] = pos;
    [x, z, -y]
}

fn wc3_to_godot_normal(nrm: [f32; 3]) -> [f32; 3] {
    let [x, y, z] = nrm;
    let nx = x;
    let ny = z;
    let nz = -y;
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len > 0.0 {
        [nx / len, ny / len, nz / len]
    } else {
        [0.0, 1.0, 0.0]
    }
}

fn resolve_texture_name(model: &MdxModel, material_id: u32) -> Option<String> {
    let mat = model.materials.get(material_id as usize)?;
    // WC3 materials can have multiple layers — first is often team-color
    // (replaceable_id=1, empty path). Walk every layer and pick the first
    // one that points at a real file.
    for layer in &mat.layers {
        if let Some(tex) = model.textures.get(layer.texture_id as usize) {
            if tex.replaceable_id == 0 && !tex.file_name.is_empty() {
                return Some(tex.file_name.clone());
            }
        }
    }
    None
}

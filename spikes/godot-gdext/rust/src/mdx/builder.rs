use std::collections::HashMap;

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

/// Build an object_id → skeleton-bone-index map matching the layout used
/// by `build_skeleton` (bones first, then helpers, in order).
fn object_id_to_bone_idx(model: &MdxModel) -> HashMap<u32, i32> {
    let mut map = HashMap::new();
    let all_nodes = model.bones.iter().chain(model.helpers.iter());
    for (i, n) in all_nodes.enumerate() {
        map.insert(n.object_id, i as i32);
    }
    map
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
            let keep = match (stand.as_ref(), model.geoset_alpha.get(*gidx)) {
                (Some(stand_seq), Some(Some(entry))) => {
                    // Some peasant geosets have NO alpha keys within the
                    // Stand window — those should fall back to static_alpha
                    // (the default-visibility flag) instead of being treated
                    // as alpha=0. Without this fallback, the arm geoset (its
                    // keys mostly live in Stand Ready / Stand Work) gets
                    // dropped from the basic Stand build.
                    let in_window_max = entry
                        .keys
                        .iter()
                        .filter(|(t, _)| *t >= stand_seq.start_ms && *t <= stand_seq.end_ms)
                        .map(|(_, a)| *a)
                        .fold(f32::NEG_INFINITY, f32::max);
                    let alpha = if in_window_max > f32::NEG_INFINITY {
                        in_window_max.max(entry.static_alpha)
                    } else {
                        entry.static_alpha
                    };
                    alpha >= 0.5
                }
                _ => true,
            };
            godot_print!("MDX geoset {} kept={}", gidx, keep);
            keep
        })
        .collect();

    let mut mesh = ArrayMesh::new_gd();
    let mut surface_textures = Vec::new();

    let obj_to_bone = object_id_to_bone_idx(model);

    for (_, g) in kept_geosets {
        if g.faces.is_empty() {
            continue;
        }

        let mut positions = PackedVector3Array::new();
        let mut normals = PackedVector3Array::new();
        let mut uvs = PackedVector2Array::new();
        let mut indices = PackedInt32Array::new();
        // Bones + weights per vertex — 4 ints/floats per vertex.
        let mut bones = PackedInt32Array::new();
        let mut weights = PackedFloat32Array::new();

        // Precompute the matrix-group cumulative starts so vertex_group i
        // resolves to indices [start..start+group_count] inside matrix_indices.
        let mut group_starts: Vec<u32> = Vec::with_capacity(g.matrix_group_counts.len());
        let mut acc: u32 = 0;
        for &c in &g.matrix_group_counts {
            group_starts.push(acc);
            acc += c;
        }

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

            // Skin weights: vertex_groups[v_idx] is an index into matrix
            // groups; each group lists 1–4 bone object_ids that influence
            // this vertex with equal weight (WC3 rigid-multi-bone skinning).
            let mut vb = [0i32; 4];
            let mut vw = [0.0f32; 4];
            let vg_idx = g.vertex_groups.get(v_idx).copied().unwrap_or(0) as usize;
            if vg_idx < g.matrix_group_counts.len() {
                let count = g.matrix_group_counts[vg_idx] as usize;
                let start = group_starts[vg_idx] as usize;
                let w = if count > 0 { 1.0 / count as f32 } else { 0.0 };
                for k in 0..count.min(4) {
                    let obj_id = g.matrix_indices.get(start + k).copied().unwrap_or(0);
                    vb[k] = obj_to_bone.get(&obj_id).copied().unwrap_or(0);
                    vw[k] = w;
                }
            }
            for j in 0..4 {
                bones.push(vb[j]);
                weights.push(vw[j]);
            }
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
        arrays.set(0, &positions.to_variant());   // ARRAY_VERTEX
        arrays.set(1, &normals.to_variant());     // ARRAY_NORMAL
        arrays.set(4, &uvs.to_variant());         // ARRAY_TEX_UV
        arrays.set(10, &bones.to_variant());      // ARRAY_BONES
        arrays.set(11, &weights.to_variant());    // ARRAY_WEIGHTS
        arrays.set(12, &indices.to_variant());    // ARRAY_INDEX

        mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);

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

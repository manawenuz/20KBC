use godot::prelude::*;
use godot::classes::{Skeleton3D, Skin};

use super::types::*;

fn wc3_to_godot_pos(pos: [f32; 3]) -> [f32; 3] {
    let [x, y, z] = pos;
    [x, z, -y]
}

pub fn build_skeleton(model: &MdxModel) -> Gd<Skeleton3D> {
    let mut skeleton = Skeleton3D::new_alloc();

    // Combine bones and helpers; preserve object_id for parent mapping.
    let all_nodes: Vec<&Bone> = model.bones.iter().chain(model.helpers.iter()).collect();

    // object_id -> skeleton bone index
    let mut obj_to_idx = std::collections::HashMap::new();

    for node in &all_nodes {
        skeleton.add_bone(&node.name);
        let idx = skeleton.get_bone_count() - 1;
        obj_to_idx.insert(node.object_id, idx);
    }

    // Second pass: set parents and rest transforms.
    //
    // Godot stores bone rest as PARENT-RELATIVE. WC3 pivots are stored in
    // ABSOLUTE model space. So for each child bone, the rest position is
    // (own pivot) - (parent pivot). Defensive: ignore self-references and
    // missing parents (same fix we applied in the offline glTF writer).
    for node in &all_nodes {
        let idx = obj_to_idx[&node.object_id];

        let parent_idx = if node.parent_id == 0xFFFFFFFF || node.parent_id == node.object_id {
            -1
        } else {
            obj_to_idx.get(&node.parent_id).copied().unwrap_or(-1)
        };
        skeleton.set_bone_parent(idx, parent_idx);

        let pivot_abs = model
            .pivots
            .get(node.object_id as usize)
            .copied()
            .unwrap_or([0.0, 0.0, 0.0]);
        let pos_abs = wc3_to_godot_pos(pivot_abs);

        let pos_rel = if parent_idx >= 0 && node.parent_id != 0xFFFFFFFF {
            let parent_pivot = model
                .pivots
                .get(node.parent_id as usize)
                .copied()
                .unwrap_or([0.0, 0.0, 0.0]);
            let p_godot = wc3_to_godot_pos(parent_pivot);
            [pos_abs[0] - p_godot[0], pos_abs[1] - p_godot[1], pos_abs[2] - p_godot[2]]
        } else {
            pos_abs
        };

        let rest = Transform3D::new(
            Basis::IDENTITY,
            Vector3::new(pos_rel[0], pos_rel[1], pos_rel[2]),
        );
        skeleton.set_bone_rest(idx, rest);
        skeleton.set_bone_pose_position(idx, Vector3::new(pos_rel[0], pos_rel[1], pos_rel[2]));
    }

    skeleton
}

pub fn build_skin(model: &MdxModel, _skeleton: &Gd<Skeleton3D>) -> Gd<Skin> {
    let mut skin = Skin::new_gd();
    // Inverse-bind for each bone = inverse of its absolute (world-space)
    // bind-pose transform. WC3 vertices live in absolute model space at
    // bind pose, so the world-rest is just the absolute pivot translation.
    let all_nodes: Vec<&Bone> = model.bones.iter().chain(model.helpers.iter()).collect();
    for (idx, node) in all_nodes.iter().enumerate() {
        let pivot = model
            .pivots
            .get(node.object_id as usize)
            .copied()
            .unwrap_or([0.0, 0.0, 0.0]);
        let p = wc3_to_godot_pos(pivot);
        let world_rest = Transform3D::new(Basis::IDENTITY, Vector3::new(p[0], p[1], p[2]));
        let bind_pose = world_rest.affine_inverse();
        skin.add_bind(idx as i32, bind_pose);
    }
    skin
}

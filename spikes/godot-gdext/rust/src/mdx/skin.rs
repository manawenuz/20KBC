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
    for node in &all_nodes {
        let idx = obj_to_idx[&node.object_id];
        let parent_idx = if node.parent_id == 0xFFFFFFFF {
            -1
        } else {
            obj_to_idx.get(&node.parent_id).copied().unwrap_or(-1)
        };
        skeleton.set_bone_parent(idx, parent_idx);

        let pivot = model
            .pivots
            .get(node.object_id as usize)
            .copied()
            .unwrap_or([0.0, 0.0, 0.0]);
        let pos = wc3_to_godot_pos(pivot);
        let rest = Transform3D::new(
            Basis::IDENTITY,
            Vector3::new(pos[0], pos[1], pos[2]),
        );
        skeleton.set_bone_rest(idx, rest);
        skeleton.set_bone_pose(idx, rest);
    }

    skeleton
}

pub fn build_skin(_model: &MdxModel, skeleton: &Gd<Skeleton3D>) -> Gd<Skin> {
    let mut skin = Skin::new_gd();
    let bone_count = skeleton.get_bone_count();

    for bone_idx in 0..bone_count {
        let rest = skeleton.get_bone_rest(bone_idx);
        let bind_pose = rest.affine_inverse();
        skin.add_bind(bone_idx, bind_pose);
    }

    skin
}

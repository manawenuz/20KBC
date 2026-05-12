use godot::prelude::*;
use godot::classes::{Animation, AnimationLibrary};

use super::types::*;

fn wc3_to_godot_pos(pos: [f32; 3]) -> [f32; 3] {
    let [x, y, z] = pos;
    [x, z, -y]
}

fn wc3_to_godot_quat(q: [f32; 4]) -> [f32; 4] {
    let [x, y, z, w] = q;
    [x, z, -y, w]
}

fn mdx_interp_to_godot(interp: u32) -> godot::classes::animation::InterpolationType {
    use godot::classes::animation::InterpolationType;
    match interp {
        0 => InterpolationType::NEAREST,
        1 => InterpolationType::LINEAR,
        _ => InterpolationType::LINEAR,
    }
}

pub fn build_animation_library(model: &MdxModel, skeleton_path: NodePath) -> Gd<AnimationLibrary> {
    let mut lib = AnimationLibrary::new_gd();
    let base_path = skeleton_path.to_string();

    let all_nodes: Vec<&Bone> = model.bones.iter().chain(model.helpers.iter()).collect();

    for seq in &model.sequences {
        let duration_sec = (seq.end_ms.saturating_sub(seq.start_ms)) as f32 / 1000.0;
        if duration_sec <= 0.0 {
            continue;
        }

        let mut anim = Animation::new_gd();
        anim.set_length(duration_sec);
        if seq.loop_ {
            anim.set_loop_mode(godot::classes::animation::LoopMode::LINEAR);
        }

        for node in &all_nodes {
            let node_path = NodePath::from(format!("{}:{}", base_path, node.name).as_str());

            // Bone rest position (parent-relative, same convention as skin.rs).
            // MDX KGTR keyframes are deltas FROM rest in parent space, but
            // Godot POSITION_3D tracks REPLACE the pose position, so we add
            // rest to each keyframe value.
            let pivot_abs = model.pivots.get(node.object_id as usize).copied().unwrap_or([0.0; 3]);
            let parent_pivot_abs = if node.parent_id == 0xFFFFFFFF || node.parent_id == node.object_id {
                [0.0; 3]
            } else {
                model.pivots.get(node.parent_id as usize).copied().unwrap_or([0.0; 3])
            };
            let pa = wc3_to_godot_pos(pivot_abs);
            let pp = wc3_to_godot_pos(parent_pivot_abs);
            let rest = [pa[0] - pp[0], pa[1] - pp[1], pa[2] - pp[2]];

            // Translation track
            let pos_kfs: Vec<_> = node
                .translations
                .iter()
                .filter(|kf| kf.time_ms >= seq.start_ms && kf.time_ms <= seq.end_ms)
                .collect();
            if !pos_kfs.is_empty() {
                let t_idx = anim.add_track(godot::classes::animation::TrackType::POSITION_3D);
                anim.track_set_path(t_idx, &node_path);
                for kf in &pos_kfs {
                    let t = (kf.time_ms - seq.start_ms) as f64 / 1000.0;
                    let d = wc3_to_godot_pos(kf.value);
                    anim.position_track_insert_key(
                        t_idx,
                        t,
                        Vector3::new(rest[0] + d[0], rest[1] + d[1], rest[2] + d[2]),
                    );
                }
                anim.track_set_interpolation_type(t_idx, mdx_interp_to_godot(pos_kfs[0].interpolation));
            }

            // Rotation track
            let rot_kfs: Vec<_> = node
                .rotations
                .iter()
                .filter(|kf| kf.time_ms >= seq.start_ms && kf.time_ms <= seq.end_ms)
                .collect();
            if !rot_kfs.is_empty() {
                let t_idx = anim.add_track(godot::classes::animation::TrackType::ROTATION_3D);
                anim.track_set_path(t_idx, &node_path);
                for kf in &rot_kfs {
                    let t = (kf.time_ms - seq.start_ms) as f64 / 1000.0;
                    let q = wc3_to_godot_quat(kf.value);
                    anim.rotation_track_insert_key(t_idx, t, Quaternion::new(q[0], q[1], q[2], q[3]));
                }
                anim.track_set_interpolation_type(t_idx, mdx_interp_to_godot(rot_kfs[0].interpolation));
            }

            // Scale track
            let scl_kfs: Vec<_> = node
                .scales
                .iter()
                .filter(|kf| kf.time_ms >= seq.start_ms && kf.time_ms <= seq.end_ms)
                .collect();
            if !scl_kfs.is_empty() {
                let t_idx = anim.add_track(godot::classes::animation::TrackType::SCALE_3D);
                anim.track_set_path(t_idx, &node_path);
                for kf in &scl_kfs {
                    let t = (kf.time_ms - seq.start_ms) as f64 / 1000.0;
                    let s = kf.value;
                    anim.scale_track_insert_key(t_idx, t, Vector3::new(s[0], s[2], s[1]));
                }
                anim.track_set_interpolation_type(t_idx, mdx_interp_to_godot(scl_kfs[0].interpolation));
            }
        }

        lib.add_animation(&seq.name, &anim);
    }

    lib
}

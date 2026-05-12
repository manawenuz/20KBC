use godot::prelude::*;
use godot::classes::{
    AnimationPlayer, CapsuleMesh, MeshInstance3D, INode3D, Node, Node3D, PackedScene,
    ResourceLoader, StandardMaterial3D,
};
use godot::classes::base_material_3d::ShadingMode;

use crate::sim_bridge::SimBridge;

/// Red colour for the GAIA wolf.
const COLOR_GAIA: Color = Color {
    r: 0.75,
    g: 0.18,
    b: 0.15,
    a: 1.0,
};

/// A single GAIA entity's visual representation.
///
/// `GaiaNode` is spawned by GDScript (`main.gd`) and its world position is
/// updated every physics tick to mirror `CGaiaEntity::pos`.
#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct GaiaNode {
    /// Matches `CGaiaEntity::id` — used by GDScript to correlate position arrays.
    #[var]
    pub gaia_id: u32,
    base: Base<Node3D>,
    anim_player: Option<Gd<AnimationPlayer>>,
    prev_behavior: i64,
    frame_counter: u32,
    anim_map: [Option<String>; 4],
}

#[godot_api]
impl INode3D for GaiaNode {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            gaia_id: 0,
            base,
            anim_player: None,
            prev_behavior: -1,
            frame_counter: 0,
            anim_map: [None, None, None, None],
        }
    }

    fn ready(&mut self) {
        // TODO(20kbc): TimberWolf.glb extracted via PRD-21 has no applied
        // texture — renders white. Capsule fallback until material wiring
        // is fixed (probably needs material index alignment in mdx_to_gltf).
        let _loader = ResourceLoader::singleton();
        let model: Option<Gd<PackedScene>> = None;

        if let Some(scene) = model {
            if let Some(instance) = scene.instantiate() {
                // WC3 wolf.glb is in native WC3 units (~80 tall).
                // Normalize to ~1.2 m.
                if let Ok(mut node3d) = instance.clone().try_cast::<Node3D>() {
                    node3d.set_scale(Vector3::new(0.02, 0.02, 0.02));
                }
                self.base_mut().add_child(&instance);
                if let Some(mut anim) = Self::find_anim_player(&instance) {
                    self.anim_map = Self::build_anim_map(&anim);
                    if let Some(ref idle) = self.anim_map[0] {
                        anim.play_ex().name(idle.as_str()).done();
                    }
                    self.anim_player = Some(anim);
                }
                return;
            }
        }

        // Fallback to capsule (existing code).
        godot_warn!("wolf.glb missing — falling back to capsule");

        let mut capsule = CapsuleMesh::new_gd();
        capsule.set_radius(0.5);
        capsule.set_height(1.2);

        let mut mat = StandardMaterial3D::new_gd();
        mat.set_albedo(COLOR_GAIA);
        mat.set_shading_mode(ShadingMode::PER_PIXEL);
        capsule.surface_set_material(0, &mat);

        let mut mesh_inst = MeshInstance3D::new_alloc();
        mesh_inst.set_mesh(&capsule);
        mesh_inst.set_position(Vector3::new(0.0, 0.6, 0.0));

        self.base_mut().add_child(&mesh_inst);
    }

    fn process(&mut self, _delta: f64) {
        self.frame_counter += 1;
        if self.frame_counter % 5 != 0 {
            return;
        }

        let behavior = self
            .base()
            .get_node_or_null("../../SimBridge")
            .and_then(|n| n.try_cast::<SimBridge>().ok())
            .map(|bridge| bridge.bind().get_gaia_behavior(self.gaia_id))
            .unwrap_or(0);

        if behavior == self.prev_behavior {
            return;
        }
        self.prev_behavior = behavior;

        if let Some(ref mut anim) = self.anim_player {
            let idx = behavior as usize;
            if idx < self.anim_map.len() {
                if let Some(ref name) = self.anim_map[idx] {
                    anim.play_ex().name(name.as_str()).done();
                }
            }
        }
    }
}

#[godot_api]
impl GaiaNode {
    fn find_anim_player(node: &Gd<Node>) -> Option<Gd<AnimationPlayer>> {
        for i in 0..node.get_child_count() {
            let child = node.get_child(i)?;
            if let Ok(anim) = child.clone().try_cast::<AnimationPlayer>() {
                return Some(anim);
            }
            if let Some(found) = Self::find_anim_player(&child) {
                return Some(found);
            }
        }
        None
    }

    fn resolve_anim(anim: &AnimationPlayer, candidates: &[&str]) -> Option<String> {
        for candidate in candidates {
            if anim.has_animation(*candidate) {
                return Some(candidate.to_string());
            }
        }
        None
    }

    fn build_anim_map(anim: &AnimationPlayer) -> [Option<String>; 4] {
        [
            Self::resolve_anim(anim, &["Stand", "stand", "idle", "Idle"]),
            Self::resolve_anim(anim, &["Walk", "walk", "Run"]),
            Self::resolve_anim(anim, &["Attack - 1", "Attack", "attack"]),
            Self::resolve_anim(anim, &["Walk", "walk", "Run"]),
        ]
    }
}

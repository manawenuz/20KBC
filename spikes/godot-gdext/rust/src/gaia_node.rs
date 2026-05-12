use godot::prelude::*;
use godot::classes::{
    AnimationPlayer, CapsuleMesh, MeshInstance3D, INode3D, Node3D, PackedScene,
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
        }
    }

    fn ready(&mut self) {
        let mut loader = ResourceLoader::singleton();
        let model: Option<Gd<PackedScene>> = loader
            .load("res://assets/models/wolf.glb")
            .and_then(|r| r.try_cast::<PackedScene>().ok());

        if let Some(scene) = model {
            if let Some(instance) = scene.instantiate() {
                self.base_mut().add_child(&instance);
                // Cache AnimationPlayer if present.
                if let Some(anim) = instance
                    .get_node_or_null("AnimationPlayer")
                    .and_then(|n| n.try_cast::<AnimationPlayer>().ok())
                {
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

        let anim_name = match behavior {
            0 => "idle",
            1 => "walk",
            2 => "attack",
            3 => "walk", // Returning — use walk animation
            _ => "idle",
        };

        if let Some(ref mut anim) = self.anim_player {
            if anim.has_animation(anim_name) {
                anim.play_ex().name(anim_name).done();
            }
        }
    }
}

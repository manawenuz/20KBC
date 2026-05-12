use godot::prelude::*;
use godot::classes::{
    AnimationPlayer, CapsuleMesh, MeshInstance3D, INode3D, Node, Node3D, PackedScene,
    ResourceLoader, StandardMaterial3D, TorusMesh,
};
use godot::classes::base_material_3d::ShadingMode;
use crate::damage_number::DamageNumber;

/// Sandy-brown colour matching a prehistoric human unit.
const COLOR_UNIT: Color = Color {
    r: 0.76,
    g: 0.60,
    b: 0.42,
    a: 1.0,
};

/// Green selection ring colour.
const COLOR_RING: Color = Color {
    r: 0.20,
    g: 0.85,
    b: 0.30,
    a: 1.0,
};

/// Flash colour when the unit takes damage.
const COLOR_FLASH: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

/// A single unit's visual representation: a 3D capsule placed in the world.
///
/// `UnitNode` is spawned by GDScript (`main.gd`) whenever `sim.get_unit_count()`
/// exceeds the number of existing `UnitNode` children.  Its world position is
/// updated every physics tick to mirror `CUnit::pos`.
///
/// Keeping the visual node thin (no game logic) and driven entirely by
/// `SimBridge` preserves the clean sim/renderer separation from the spec.
#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct UnitNode {
    /// Matches `CUnit::id` — used by GDScript to correlate position arrays.
    #[var]
    pub unit_id: u32,
    base: Base<Node3D>,
    ring: Option<Gd<MeshInstance3D>>,
    mesh: Option<Gd<MeshInstance3D>>,
    material: Option<Gd<StandardMaterial3D>>,
    prev_hp: f32,
    flash_timer: f32,
    dying: bool,
    death_elapsed: f32,
    anim_player: Option<Gd<AnimationPlayer>>,
    prev_behavior: i64,
    behavior_poll: u32,
    anim_map: [Option<String>; 4],
}

#[godot_api]
impl INode3D for UnitNode {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            unit_id: 0,
            base,
            ring: None,
            mesh: None,
            material: None,
            prev_hp: -1.0,
            flash_timer: 0.0,
            dying: false,
            death_elapsed: 0.0,
            anim_player: None,
            prev_behavior: -1,
            behavior_poll: 0,
            anim_map: [None, None, None, None],
        }
    }

    fn ready(&mut self) {
        let mut loader = ResourceLoader::singleton();
        let model: Option<Gd<PackedScene>> = loader
            .load("res://assets/models/peasant.glb")
            .and_then(|r| r.try_cast::<PackedScene>().ok());

        if let Some(scene) = model {
            if let Some(instance) = scene.instantiate() {
                // WC3 peasant.glb is in native WC3 units (~80–100 tall).
                // Normalize to roughly 1.8 m by scaling 0.02 and offset Y so
                // feet sit on ground plane.
                if let Ok(mut node3d) = instance.clone().try_cast::<Node3D>() {
                    node3d.set_scale(Vector3::new(0.02, 0.02, 0.02));
                    node3d.set_position(Vector3::new(0.0, 0.0, 0.0));
                }
                self.base_mut().add_child(&instance);
                if let Some(mut anim) = Self::find_anim_player(&instance) {
                    self.anim_map = Self::build_anim_map(&anim);
                    if let Some(ref idle) = self.anim_map[0] {
                        anim.play_ex().name(idle.as_str()).done();
                    }
                    self.anim_player = Some(anim);
                }
            } else {
                self.spawn_capsule();
            }
        } else {
            godot_warn!("peasant.glb missing — falling back to capsule");
            self.spawn_capsule();
        }

        // Build green selection ring (torus) at feet.
        let mut torus = TorusMesh::new_gd();
        torus.set_inner_radius(0.6);
        torus.set_outer_radius(0.75);

        let mut ring_mat = StandardMaterial3D::new_gd();
        ring_mat.set_albedo(COLOR_RING);
        ring_mat.set_shading_mode(ShadingMode::PER_PIXEL);
        torus.surface_set_material(0, &ring_mat);

        let mut ring_inst = MeshInstance3D::new_alloc();
        ring_inst.set_mesh(&torus);
        ring_inst.set_position(Vector3::new(0.0, 0.05, 0.0));
        ring_inst.set_visible(false);

        self.base_mut().add_child(&ring_inst);
        self.ring = Some(ring_inst);
    }

    fn process(&mut self, delta: f64) {
        let dt = delta as f32;

        // Hit-flash restore.
        if self.flash_timer > 0.0 {
            self.flash_timer -= dt;
            if self.flash_timer <= 0.0 {
                if let Some(ref mut mat) = self.material {
                    mat.set_albedo(COLOR_UNIT);
                }
            }
        }

        // Death fade-out over ~1 second.
        if self.dying {
            self.death_elapsed += dt;
            let t = self.death_elapsed;
            if t >= 1.0 {
                self.base_mut().queue_free();
            } else if let Some(ref mut mat) = self.material {
                let mut color = COLOR_UNIT;
                color.a = 1.0 - t;
                mat.set_albedo(color);
                mat.set_transparency(godot::classes::base_material_3d::Transparency::ALPHA);
            }
        }

        // Poll behavior and switch animation ~ every 5 frames.
        self.behavior_poll += 1;
        if self.behavior_poll >= 5 {
            self.behavior_poll = 0;
            let behavior = self
                .find_sim_bridge()
                .map(|sim| sim.bind().get_unit_behavior(self.unit_id));
            if let (Some(behavior), Some(ref mut anim)) = (behavior, &mut self.anim_player) {
                if behavior != self.prev_behavior {
                    self.prev_behavior = behavior;
                    let idx = behavior as usize;
                    if idx < self.anim_map.len() {
                        if let Some(ref name) = self.anim_map[idx] {
                            anim.play_ex().name(name.as_str()).done();
                        }
                    }
                }
            }
        }
    }
}

#[godot_api]
impl UnitNode {
    #[func]
    pub fn set_selected(&mut self, selected: bool) {
        if let Some(ring) = &mut self.ring {
            ring.set_visible(selected);
        }
    }

    /// Called by GDScript each tick (or on HP change) to update visual feedback.
    #[func]
    pub fn set_hp(&mut self, hp: f32) {
        if self.prev_hp > 0.0 && hp < self.prev_hp {
            let damage = (self.prev_hp - hp).ceil() as i64;
            self.trigger_hit_flash();
            self.spawn_damage_number(damage);
        }

        if hp <= 0.0 && self.prev_hp > 0.0 {
            self.dying = true;
            self.death_elapsed = 0.0;
        }

        self.prev_hp = hp;
    }

    fn trigger_hit_flash(&mut self) {
        self.flash_timer = 0.15;
        if let Some(ref mut mat) = self.material {
            mat.set_albedo(COLOR_FLASH);
        }
    }

    fn spawn_damage_number(&mut self, amount: i64) {
        let mut damage_number: Gd<DamageNumber> =
            Gd::from_init_fn(|base| DamageNumber::init(base));
        damage_number.bind_mut().set_amount(amount);

        // Position it slightly above the unit.
        let pos = self.base().get_position();
        damage_number.set_position(Vector3::new(pos.x, pos.y + 1.5, pos.z));

        // Add to the world (parent of this unit node) so it persists after the unit dies.
        if let Some(mut parent) = self.base().get_parent() {
            parent.add_child(&damage_number);
        } else {
            self.base_mut().add_child(&damage_number);
        }
    }

    fn spawn_capsule(&mut self) {
        // Build capsule mesh (radius 0.4, height 1.8 — matches spec).
        let mut capsule = CapsuleMesh::new_gd();
        capsule.set_radius(0.4);
        capsule.set_height(1.8);

        // Sandy-brown material — no textures needed for this spike.
        let mut mat = StandardMaterial3D::new_gd();
        mat.set_albedo(COLOR_UNIT);
        mat.set_shading_mode(ShadingMode::PER_PIXEL);
        capsule.surface_set_material(0, &mat);

        // Attach as a child MeshInstance3D so the unit node's transform
        // controls world position while the mesh stays at local origin.
        let mut mesh_inst = MeshInstance3D::new_alloc();
        mesh_inst.set_mesh(&capsule);
        // Offset upward by half height so the capsule sits on y=0.
        mesh_inst.set_position(Vector3::new(0.0, 0.9, 0.0));

        self.base_mut().add_child(&mesh_inst);
        self.mesh = Some(mesh_inst);
        self.material = Some(mat);
    }

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
            Self::resolve_anim(anim, &["Stand Work", "Stand", "Walk"]),
        ]
    }

    fn find_sim_bridge(&self) -> Option<Gd<crate::sim_bridge::SimBridge>> {
        let parent = self.base().get_parent()?;
        let grandparent = parent.get_parent()?;
        grandparent
            .get_node_or_null("SimBridge")
            .and_then(|n| n.try_cast::<crate::sim_bridge::SimBridge>().ok())
    }
}

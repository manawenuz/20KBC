use godot::prelude::*;
use godot::classes::{
    CapsuleMesh, MeshInstance3D, INode3D, Node3D, StandardMaterial3D, TorusMesh,
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
        }
    }

    fn ready(&mut self) {
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
        self.mesh = Some(mesh_inst);
        self.material = Some(mat);
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
}

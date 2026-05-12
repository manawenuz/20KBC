use godot::prelude::*;
use godot::classes::{
    CapsuleMesh, MeshInstance3D, INode3D, Node3D, StandardMaterial3D,
};
use godot::classes::base_material_3d::ShadingMode;

/// Sandy-brown colour matching a prehistoric human unit.
const COLOR_UNIT: Color = Color {
    r: 0.76,
    g: 0.60,
    b: 0.42,
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
}

#[godot_api]
impl INode3D for UnitNode {
    fn init(base: Base<Node3D>) -> Self {
        Self { unit_id: 0, base }
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
    }
}

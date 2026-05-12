use godot::prelude::*;
use godot::classes::{
    CapsuleMesh, MeshInstance3D, INode3D, Node3D, StandardMaterial3D,
};
use godot::classes::base_material_3d::ShadingMode;

/// Red colour for the GAIA wolf.
const COLOR_GAIA: Color = Color {
    r: 0.75,
    g: 0.18,
    b: 0.15,
    a: 1.0,
};

/// A single GAIA entity's visual representation: a 3D capsule placed in the world.
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
}

#[godot_api]
impl INode3D for GaiaNode {
    fn init(base: Base<Node3D>) -> Self {
        Self { gaia_id: 0, base }
    }

    fn ready(&mut self) {
        // Build capsule mesh (radius 0.5, height 1.2 — matches spec).
        let mut capsule = CapsuleMesh::new_gd();
        capsule.set_radius(0.5);
        capsule.set_height(1.2);

        // Red material — no textures needed for this spike.
        let mut mat = StandardMaterial3D::new_gd();
        mat.set_albedo(COLOR_GAIA);
        mat.set_shading_mode(ShadingMode::PER_PIXEL);
        capsule.surface_set_material(0, &mat);

        // Attach as a child MeshInstance3D so the gaia node's transform
        // controls world position while the mesh stays at local origin.
        let mut mesh_inst = MeshInstance3D::new_alloc();
        mesh_inst.set_mesh(&capsule);
        // Offset upward by half height so the capsule sits on y=0.
        mesh_inst.set_position(Vector3::new(0.0, 0.6, 0.0));

        self.base_mut().add_child(&mesh_inst);
    }
}

use godot::prelude::*;
use godot::classes::{BoxMesh, MeshInstance3D, INode3D, Node3D, StandardMaterial3D};
use godot::classes::base_material_3d::ShadingMode;

/// Wood colour — brown.
const COLOR_WOOD: Color = Color {
    r: 0.42,
    g: 0.27,
    b: 0.13,
    a: 1.0,
};

/// Stone colour — grey.
const COLOR_STONE: Color = Color {
    r: 0.55,
    g: 0.55,
    b: 0.58,
    a: 1.0,
};

/// Fallback colour for unknown kinds — magenta.
const COLOR_UNKNOWN: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};

/// A single resource node's visual representation: a 3D box placed in the world.
///
/// `ResourceNode` is spawned by GDScript whenever the sim reports resource nodes.
/// Its world position is set by GDScript to mirror `CResourceNode::pos`.
#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct ResourceNode {
    /// 1 = Wood (brown), 2 = Stone (grey). Set BEFORE adding to scene tree
    /// so ready() picks the right color.
    #[var]
    pub kind: u32,
    /// Sim-side ResourceNodeId — used to correlate right-click → gather order.
    #[var]
    pub node_id: u32,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for ResourceNode {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            kind: 0,
            node_id: 0,
            base,
        }
    }

    fn ready(&mut self) {
        // Build box mesh (1.5 × 1.5 × 1.5).
        let mut box_mesh = BoxMesh::new_gd();
        box_mesh.set_size(Vector3::new(1.5, 1.5, 1.5));

        // Choose colour by kind.
        let color = match self.kind {
            1 => COLOR_WOOD,
            2 => COLOR_STONE,
            _ => COLOR_UNKNOWN,
        };

        let mut mat = StandardMaterial3D::new_gd();
        mat.set_albedo(color);
        mat.set_shading_mode(ShadingMode::PER_PIXEL);
        box_mesh.surface_set_material(0, &mat);

        // Attach as a child MeshInstance3D so the resource node's transform
        // controls world position while the mesh stays at local origin.
        let mut mesh_inst = MeshInstance3D::new_alloc();
        mesh_inst.set_mesh(&box_mesh);
        // Offset upward by half height so the box sits on y=0.
        mesh_inst.set_position(Vector3::new(0.0, 0.75, 0.0));

        self.base_mut().add_child(&mesh_inst);
    }
}

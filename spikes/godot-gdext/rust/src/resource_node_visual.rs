use godot::prelude::*;
use godot::classes::{
    BoxMesh, MeshInstance3D, INode3D, Node3D, StandardMaterial3D, ResourceLoader, PackedScene,
};
use godot::classes::base_material_3d::ShadingMode;
use crate::sim_bridge::SimBridge;

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

/// A single resource node's visual representation: a 3D model placed in the world.
///
/// `ResourceNode` is spawned by GDScript whenever the sim reports resource nodes.
/// Its world position is set by GDScript to mirror `CResourceNode::pos`.
///
/// - Wood uses `res://assets/models/tree.glb` with a tip-over animation on depletion.
/// - Stone uses `res://assets/models/stone.glb`.
/// - If a model is missing, a coloured BoxMesh is used as fallback.
#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct ResourceNode {
    /// 1 = Wood (tree), 2 = Stone (outcrop). Set BEFORE adding to scene tree
    /// so ready() picks the right model.
    #[var]
    pub kind: u32,
    /// Sim-side ResourceNodeId — used to correlate right-click → gather order.
    #[var]
    pub node_id: u32,
    base: Base<Node3D>,
    last_amount: i64,
    falling: bool,
    fall_timer: f32,
    mesh: Option<Gd<Node3D>>,
    sim_bridge: Option<Gd<SimBridge>>,
}

#[godot_api]
impl INode3D for ResourceNode {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            kind: 0,
            node_id: 0,
            base,
            last_amount: -1,
            falling: false,
            fall_timer: 0.0,
            mesh: None,
            sim_bridge: None,
        }
    }

    fn ready(&mut self) {
        // Try to load a GLB model based on kind.
        let model_path = match self.kind {
            1 => "res://assets/models/lordaerontree.glb",
            2 => "res://assets/models/stone.glb",
            _ => {
                self.build_fallback_mesh();
                return;
            }
        };

        let mut loader = ResourceLoader::singleton();
        let packed = loader
            .load(model_path)
            .and_then(|r| r.try_cast::<PackedScene>().ok());

        if let Some(ps) = packed {
            let instance = ps.instantiate();
            if let Some(node) = instance {
                if let Ok(mut node3d) = node.try_cast::<Node3D>() {
                    // PRD-10 procedural tree/stone glbs are ~1m authored size,
                    // hard to see from the RTS camera distance. Scale up so
                    // they read clearly on the map.
                    let s = if self.kind == 1 { 0.05 } else { 1.8 };
                    node3d.set_scale(Vector3::new(s, s, s));
                    self.base_mut().add_child(&node3d);
                    self.mesh = Some(node3d);
                    self.sim_bridge = self.find_sim_bridge();
                    return;
                }
            }
        }

        // Model missing or failed to load — use the coloured box fallback.
        self.build_fallback_mesh();
    }

    fn process(&mut self, delta: f64) {
        let dt = delta as f32;

        // If already tipping over, advance the animation.
        if self.falling {
            self.fall_timer += dt;
            let t = self.fall_timer.min(1.0);
            let angle = std::f32::consts::FRAC_PI_2 * t; // 90° in radians
            if let Some(ref mut mesh) = self.mesh {
                mesh.set_rotation(Vector3::new(angle, 0.0, 0.0));
            }
            if self.fall_timer >= 1.0 {
                self.base_mut().queue_free();
            }
            return;
        }

        // Poll sim bridge for depletion.
        let amount = self
            .sim_bridge
            .as_ref()
            .map(|b| b.bind().get_resource_amount(self.node_id))
            .unwrap_or(-1);

        if self.last_amount > 0 && amount <= 0 && self.kind == 1 {
            self.falling = true;
            self.fall_timer = 0.0;
        }

        self.last_amount = amount;
    }
}

impl ResourceNode {
    /// Build the legacy coloured BoxMesh fallback.
    fn build_fallback_mesh(&mut self) {
        let mut box_mesh = BoxMesh::new_gd();
        box_mesh.set_size(Vector3::new(1.5, 1.5, 1.5));

        let color = match self.kind {
            1 => COLOR_WOOD,
            2 => COLOR_STONE,
            _ => COLOR_UNKNOWN,
        };

        let mut mat = StandardMaterial3D::new_gd();
        mat.set_albedo(color);
        mat.set_shading_mode(ShadingMode::PER_PIXEL);
        box_mesh.surface_set_material(0, &mat);

        let mut mesh_inst = MeshInstance3D::new_alloc();
        mesh_inst.set_mesh(&box_mesh);
        // Offset upward by half height so the box sits on y=0.
        mesh_inst.set_position(Vector3::new(0.0, 0.75, 0.0));

        self.base_mut().add_child(&mesh_inst);
        self.mesh = Some(mesh_inst.upcast::<Node3D>());
        self.sim_bridge = self.find_sim_bridge();
    }

    /// Walk up the tree to find the SimBridge node (sibling of our grandparent).
    fn find_sim_bridge(&self) -> Option<Gd<SimBridge>> {
        let parent = self.base().get_parent()?;
        let grandparent = parent.get_parent()?;
        grandparent
            .get_node_or_null("SimBridge")?
            .try_cast::<SimBridge>()
            .ok()
    }
}

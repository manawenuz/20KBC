use godot::prelude::*;
use godot::classes::{INode3D, Node3D, PackedScene, ResourceLoader, StandardMaterial3D, BoxMesh, MeshInstance3D};
use godot::classes::base_material_3d::ShadingMode;

#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct BuildingNode {
    /// 1 = TownHall, 2 = Keep, 3 = Castle. Set BEFORE adding to scene
    /// tree so ready() picks the right model.
    #[var] pub kind: u32,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for BuildingNode {
    fn init(base: Base<Node3D>) -> Self {
        Self { kind: 0, base }
    }
    fn ready(&mut self) {
        // Try runtime MDX from MPQ first. Mapping mirrors the visual swap
        // we applied in extract.py: townhall.glb→AltarOfKings, etc.
        let mdx_path = match self.kind {
            1 => "Buildings/Human/AltarOfKings/AltarOfKings.mdx",
            2 => "Buildings/Human/HumanBarracks/HumanBarracks.mdx",
            3 => "Buildings/Human/TownHall/TownHall.mdx",
            _ => "",
        };
        if !mdx_path.is_empty() && self.try_spawn_from_registry(mdx_path) {
            return;
        }

        let path = match self.kind {
            1 => "res://assets/models/townhall.glb",
            2 => "res://assets/models/keep.glb",
            3 => "res://assets/models/castle.glb",
            _ => "",
        };
        let scene: Option<Gd<PackedScene>> = ResourceLoader::singleton()
            .load(path)
            .and_then(|r| r.try_cast::<PackedScene>().ok());
        if let Some(s) = scene {
            if let Some(inst) = s.instantiate() {
                if let Ok(mut n) = inst.clone().try_cast::<Node3D>() {
                    // WC3 buildings use the same ~1cm-per-unit scale.
                    n.set_scale(Vector3::new(0.02, 0.02, 0.02));
                }
                // Paint untextured submeshes with a sensible fallback
                // per building. The altar's giant white banner gets a
                // royal blue here; the townhall's untextured roof
                // pieces get a slate grey.
                let fallback = match self.kind {
                    1 => Color { r: 0.18, g: 0.28, b: 0.65, a: 1.0 }, // altar banner blue
                    2 => Color { r: 0.55, g: 0.45, b: 0.30, a: 1.0 }, // barracks brown
                    3 => Color { r: 0.40, g: 0.42, b: 0.45, a: 1.0 }, // townhall slate
                    _ => Color { r: 0.55, g: 0.55, b: 0.55, a: 1.0 },
                };
                crate::material_tint::paint_untextured(&inst, fallback);
                self.base_mut().add_child(&inst);
                return;
            }
        }
        // Fallback: a coloured cube so missing models are obviously placeholders.
        let color = match self.kind {
            1 => Color { r: 0.65, g: 0.55, b: 0.35, a: 1.0 },
            2 => Color { r: 0.55, g: 0.55, b: 0.60, a: 1.0 },
            3 => Color { r: 0.70, g: 0.65, b: 0.55, a: 1.0 },
            _ => Color { r: 1.0, g: 0.0, b: 1.0, a: 1.0 },
        };
        let mut mesh = BoxMesh::new_gd();
        mesh.set_size(Vector3::new(6.0, 6.0, 6.0));
        let mut mat = StandardMaterial3D::new_gd();
        mat.set_albedo(color);
        mat.set_shading_mode(ShadingMode::PER_PIXEL);
        mesh.surface_set_material(0, &mat);
        let mut inst = MeshInstance3D::new_alloc();
        inst.set_mesh(&mesh);
        inst.set_position(Vector3::new(0.0, 3.0, 0.0));
        self.base_mut().add_child(&inst);
    }
}

impl BuildingNode {
    /// Try to spawn this building's visual from the runtime AssetRegistry.
    /// Returns true on success.
    fn try_spawn_from_registry(&mut self, mdx_path: &str) -> bool {
        use godot::classes::base_material_3d::TextureParam;
        use godot::classes::Material;
        let resolved = crate::asset_registry::with(|reg| reg.load(mdx_path)).flatten();
        let Some(r) = resolved else { return false };
        let mut mi = MeshInstance3D::new_alloc();
        mi.set_mesh(&r.mesh);
        for (i, tex) in r.textures.iter().enumerate() {
            if let Some(t) = tex {
                let mut mat = StandardMaterial3D::new_gd();
                mat.set_texture(TextureParam::ALBEDO, t);
                mat.set_shading_mode(ShadingMode::PER_PIXEL);
                mi.set_surface_override_material(i as i32, &mat.upcast::<Material>());
            }
        }
        mi.set_scale(Vector3::new(0.02, 0.02, 0.02));
        self.base_mut().add_child(&mi);
        true
    }
}

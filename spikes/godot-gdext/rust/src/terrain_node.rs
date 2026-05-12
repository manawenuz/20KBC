use godot::prelude::*;
use godot::classes::{
    ArrayMesh, IMeshInstance3D, ImageTexture, MeshInstance3D, ResourceLoader,
    StandardMaterial3D, Texture2D,
};
use godot::classes::base_material_3d::ShadingMode;
use godot::classes::mesh::PrimitiveType;

/// Flat 64×64 terrain rendered as a single `ArrayMesh` with the WC3 grass
/// texture tiled across it. Each tile is 2.0 world-units wide, matching the
/// pathfinder cell size from `SimConfig::default()` (`tile_size = 2.0`).
///
/// The terrain mesh carries UVs so the WC3 grass atlas tiles cleanly. The
/// previous vertex-color checkerboard is gone — the texture now provides
/// visual feedback.
#[derive(GodotClass)]
#[class(base = MeshInstance3D)]
pub struct TerrainNode {
    base: Base<MeshInstance3D>,
}

#[godot_api]
impl IMeshInstance3D for TerrainNode {
    fn init(base: Base<MeshInstance3D>) -> Self {
        Self { base }
    }

    fn ready(&mut self) {
        let mesh = self.build_terrain_mesh();
        self.base_mut().set_mesh(&mesh);

        // Load the extracted WC3 grass texture and apply as albedo.
        let mut loader = ResourceLoader::singleton();
        let tex: Option<Gd<Texture2D>> = loader
            .load("res://assets/textures/ground_grass.png")
            .and_then(|r| r.try_cast::<Texture2D>().ok());

        let mut mat = StandardMaterial3D::new_gd();
        mat.set_shading_mode(ShadingMode::PER_PIXEL);
        if let Some(t) = tex {
            mat.set_texture(godot::classes::base_material_3d::TextureParam::ALBEDO, &t);
        } else {
            // Fallback: flat green.
            mat.set_albedo(Color { r: 0.29, g: 0.56, b: 0.20, a: 1.0 });
            godot_warn!("ground_grass.png not found — using flat green fallback");
        }
        self.base_mut().set_surface_override_material(0, &mat);
        // Silence unused-import warning when no texture loaded.
        let _ = ImageTexture::new_gd;
    }
}

#[godot_api]
impl TerrainNode {
    fn build_terrain_mesh(&self) -> Gd<ArrayMesh> {
        const GRID: u32 = 64;
        const TILE: f32 = 2.0;

        let mut positions = PackedVector3Array::new();
        let mut normals = PackedVector3Array::new();
        let mut uvs = PackedVector2Array::new();

        let up = Vector3::UP;

        for z in 0..GRID {
            for x in 0..GRID {
                let x0 = x as f32 * TILE;
                let z0 = z as f32 * TILE;
                let x1 = x0 + TILE;
                let z1 = z0 + TILE;

                // Each tile maps UV 0-1 onto the grass texture. Texture content
                // is a 4x2-cell atlas, so picking u in [0.5, 1.0] and v in [0, 0.5]
                // selects the central solid grass cell without seams.
                let u0 = 0.55;
                let u1 = 0.70;
                let v0 = 0.05;
                let v1 = 0.45;

                let verts = [
                    (Vector3::new(x0, 0.0, z0), Vector2::new(u0, v0)),
                    (Vector3::new(x1, 0.0, z0), Vector2::new(u1, v0)),
                    (Vector3::new(x0, 0.0, z1), Vector2::new(u0, v1)),
                    (Vector3::new(x1, 0.0, z0), Vector2::new(u1, v0)),
                    (Vector3::new(x1, 0.0, z1), Vector2::new(u1, v1)),
                    (Vector3::new(x0, 0.0, z1), Vector2::new(u0, v1)),
                ];

                for (p, uv) in &verts {
                    positions.push(*p);
                    normals.push(up);
                    uvs.push(*uv);
                }
            }
        }

        let mut arrays = VarArray::new();
        arrays.resize(13, &Variant::nil());
        arrays.set(0, &positions.to_variant()); // ARRAY_VERTEX
        arrays.set(1, &normals.to_variant());   // ARRAY_NORMAL
        arrays.set(4, &uvs.to_variant());       // ARRAY_TEX_UV

        let mut mesh = ArrayMesh::new_gd();
        mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);
        mesh
    }
}

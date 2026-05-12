use godot::prelude::*;
use godot::classes::{
    ArrayMesh, IMeshInstance3D, ImageTexture, MeshInstance3D, ResourceLoader,
    StandardMaterial3D, Texture2D,
};
use godot::classes::base_material_3d::ShadingMode;
use godot::classes::mesh::PrimitiveType;

/// Flat 64×64 terrain rendered as an `ArrayMesh` with the WC3 grass and dirt
/// textures tiled across it. Each tile is 2.0 world-units wide, matching the
/// pathfinder cell size from `SimConfig::default()` (`tile_size = 2.0`).
///
/// Visual features:
/// - Grass tiles use one of 4 deterministic UV sub-rects from the grass atlas.
/// - Dirt tiles form a patch around the depot (world centre) using the dirt atlas.
/// - A dark border quad surrounds the map edge.
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

        let mut loader = ResourceLoader::singleton();

        let grass_tex: Option<Gd<Texture2D>> = loader
            .load("res://assets/textures/ground_grass.png")
            .and_then(|r| r.try_cast::<Texture2D>().ok());

        let dirt_tex: Option<Gd<Texture2D>> = loader
            .load("res://assets/textures/ground_dirt.png")
            .and_then(|r| r.try_cast::<Texture2D>().ok());

        // Surface 0 — grass
        let mut grass_mat = StandardMaterial3D::new_gd();
        grass_mat.set_shading_mode(ShadingMode::PER_PIXEL);
        if let Some(t) = grass_tex {
            grass_mat.set_texture(godot::classes::base_material_3d::TextureParam::ALBEDO, &t);
        } else {
            grass_mat.set_albedo(Color { r: 0.29, g: 0.56, b: 0.20, a: 1.0 });
            godot_warn!("ground_grass.png not found — using flat green fallback");
        }
        self.base_mut().set_surface_override_material(0, &grass_mat);

        // Surface 1 — dirt
        let mut dirt_mat = StandardMaterial3D::new_gd();
        dirt_mat.set_shading_mode(ShadingMode::PER_PIXEL);
        if let Some(t) = dirt_tex {
            dirt_mat.set_texture(godot::classes::base_material_3d::TextureParam::ALBEDO, &t);
        } else {
            dirt_mat.set_albedo(Color { r: 0.55, g: 0.40, b: 0.25, a: 1.0 });
            godot_warn!("ground_dirt.png not found — using flat brown fallback");
        }
        self.base_mut().set_surface_override_material(1, &dirt_mat);

        // Surface 2 — border
        let mut border_mat = StandardMaterial3D::new_gd();
        border_mat.set_shading_mode(ShadingMode::PER_PIXEL);
        border_mat.set_albedo(Color { r: 0.05, g: 0.04, b: 0.02, a: 1.0 });
        self.base_mut().set_surface_override_material(2, &border_mat);

        // Silence unused-import warning when no texture loaded.
        let _ = ImageTexture::new_gd;
    }
}

#[godot_api]
impl TerrainNode {
    fn build_terrain_mesh(&self) -> Gd<ArrayMesh> {
        const GRID: u32 = 64;
        const TILE: f32 = 2.0;
        const DEPOT_X: f32 = 64.0;
        const DEPOT_Z: f32 = 64.0;
        const DIRT_RADIUS: f32 = 8.0;

        // Four clean-ish sub-rects from the right half of each atlas.
        // Each rect is roughly one cell in a 4×2 grid (U 0.5..1.0, V 0..1).
        let grass_rects: [(f32, f32, f32, f32); 4] = [
            (0.51, 0.62, 0.02, 0.48), // col 0, top
            (0.63, 0.74, 0.02, 0.48), // col 1, top
            (0.51, 0.62, 0.52, 0.98), // col 0, bottom
            (0.63, 0.74, 0.52, 0.98), // col 1, bottom
        ];
        let dirt_rects: [(f32, f32, f32, f32); 4] = [
            (0.51, 0.62, 0.02, 0.48),
            (0.63, 0.74, 0.02, 0.48),
            (0.51, 0.62, 0.52, 0.98),
            (0.63, 0.74, 0.52, 0.98),
        ];

        let mut grass_positions = PackedVector3Array::new();
        let mut grass_normals = PackedVector3Array::new();
        let mut grass_uvs = PackedVector2Array::new();

        let mut dirt_positions = PackedVector3Array::new();
        let mut dirt_normals = PackedVector3Array::new();
        let mut dirt_uvs = PackedVector2Array::new();

        let mut border_positions = PackedVector3Array::new();
        let mut border_normals = PackedVector3Array::new();

        for z in 0..GRID {
            for x in 0..GRID {
                let x0 = x as f32 * TILE;
                let z0 = z as f32 * TILE;
                let x1 = x0 + TILE;
                let z1 = z0 + TILE;

                let tile_cx = x0 + TILE * 0.5;
                let tile_cz = z0 + TILE * 0.5;
                let dx = tile_cx - DEPOT_X;
                let dz = tile_cz - DEPOT_Z;
                let dist_sq = dx * dx + dz * dz;

                // Deterministic per-tile RNG — same seed always yields same pattern.
                let r = ((x.wrapping_mul(2654435761)) ^ z.wrapping_mul(40503)) % 4;
                let rect_idx = r as usize;

                if dist_sq < DIRT_RADIUS * DIRT_RADIUS {
                    let (u0, u1, v0, v1) = dirt_rects[rect_idx];
                    Self::push_tile(
                        &mut dirt_positions,
                        &mut dirt_normals,
                        &mut dirt_uvs,
                        x0, z0, x1, z1,
                        u0, u1, v0, v1,
                    );
                } else {
                    let (u0, u1, v0, v1) = grass_rects[rect_idx];
                    Self::push_tile(
                        &mut grass_positions,
                        &mut grass_normals,
                        &mut grass_uvs,
                        x0, z0, x1, z1,
                        u0, u1, v0, v1,
                    );
                }
            }
        }

        // Dark border — 4 thin strips just below the terrain to avoid z-fighting.
        let border_y = -0.02;
        let w = 1.0f32;
        let s = GRID as f32 * TILE;

        // Front
        Self::push_border_quad(&mut border_positions, &mut border_normals,
                               -w, -w, s + w, 0.0, border_y);
        // Back
        Self::push_border_quad(&mut border_positions, &mut border_normals,
                               -w, s, s + w, s + w, border_y);
        // Left
        Self::push_border_quad(&mut border_positions, &mut border_normals,
                               -w, 0.0, 0.0, s, border_y);
        // Right
        Self::push_border_quad(&mut border_positions, &mut border_normals,
                               s, 0.0, s + w, s, border_y);

        let mut mesh = ArrayMesh::new_gd();

        // Surface 0 — grass
        {
            let mut arrays = VarArray::new();
            arrays.resize(13, &Variant::nil());
            arrays.set(0, &grass_positions.to_variant()); // ARRAY_VERTEX
            arrays.set(1, &grass_normals.to_variant());   // ARRAY_NORMAL
            arrays.set(4, &grass_uvs.to_variant());       // ARRAY_TEX_UV
            mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);
        }

        // Surface 1 — dirt
        {
            let mut arrays = VarArray::new();
            arrays.resize(13, &Variant::nil());
            arrays.set(0, &dirt_positions.to_variant());
            arrays.set(1, &dirt_normals.to_variant());
            arrays.set(4, &dirt_uvs.to_variant());
            mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);
        }

        // Surface 2 — border
        {
            let mut arrays = VarArray::new();
            arrays.resize(13, &Variant::nil());
            arrays.set(0, &border_positions.to_variant());
            arrays.set(1, &border_normals.to_variant());
            mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);
        }

        mesh
    }
}

impl TerrainNode {
    fn push_tile(
        positions: &mut PackedVector3Array,
        normals: &mut PackedVector3Array,
        uvs: &mut PackedVector2Array,
        x0: f32,
        z0: f32,
        x1: f32,
        z1: f32,
        u0: f32,
        u1: f32,
        v0: f32,
        v1: f32,
    ) {
        let up = Vector3::UP;
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

    fn push_border_quad(
        positions: &mut PackedVector3Array,
        normals: &mut PackedVector3Array,
        x0: f32,
        z0: f32,
        x1: f32,
        z1: f32,
        y: f32,
    ) {
        let up = Vector3::UP;
        let verts = [
            Vector3::new(x0, y, z0),
            Vector3::new(x1, y, z0),
            Vector3::new(x0, y, z1),
            Vector3::new(x1, y, z0),
            Vector3::new(x1, y, z1),
            Vector3::new(x0, y, z1),
        ];
        for p in &verts {
            positions.push(*p);
            normals.push(up);
        }
    }
}

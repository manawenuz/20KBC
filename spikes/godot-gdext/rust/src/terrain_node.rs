use godot::prelude::*;
use godot::classes::{ArrayMesh, MeshInstance3D, IMeshInstance3D};
use godot::classes::mesh::PrimitiveType;

/// Grass green (even tiles).
const COLOR_GRASS: Color = Color {
    r: 0.29,
    g: 0.56,
    b: 0.20,
    a: 1.0,
};

/// Dirt brown (odd tiles).
const COLOR_DIRT: Color = Color {
    r: 0.55,
    g: 0.40,
    b: 0.24,
    a: 1.0,
};

/// Flat 64×64 terrain rendered as a single `ArrayMesh` using vertex colours.
/// Each tile is 2.0 world-units wide, matching the pathfinder cell size from
/// `SimConfig::default()` (`tile_size = 2.0`).
///
/// We build one quad (two triangles) per tile and colour it alternately
/// grass/dirt so the checkerboard gives immediate spatial feedback without
/// needing any texture assets.
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
    }
}

#[godot_api]
impl TerrainNode {
    fn build_terrain_mesh(&self) -> Gd<ArrayMesh> {
        const GRID: u32 = 64;
        const TILE: f32 = 2.0;

        // 64×64 tiles × 2 triangles × 3 verts = 24 576 vertices total.
        let mut positions = PackedVector3Array::new();
        let mut colors = PackedColorArray::new();
        let mut normals = PackedVector3Array::new();

        let up = Vector3::UP;

        for z in 0..GRID {
            for x in 0..GRID {
                let x0 = x as f32 * TILE;
                let z0 = z as f32 * TILE;
                let x1 = x0 + TILE;
                let z1 = z0 + TILE;

                let color = if (x + z) % 2 == 0 { COLOR_GRASS } else { COLOR_DIRT };

                // Two CCW triangles (Godot front-face winding for Y-up looking down)
                // Triangle 1
                let verts = [
                    Vector3::new(x0, 0.0, z0),
                    Vector3::new(x1, 0.0, z0),
                    Vector3::new(x0, 0.0, z1),
                    // Triangle 2
                    Vector3::new(x1, 0.0, z0),
                    Vector3::new(x1, 0.0, z1),
                    Vector3::new(x0, 0.0, z1),
                ];

                for v in &verts {
                    positions.push(*v);
                    colors.push(color);
                    normals.push(up);
                }
            }
        }

        // Build the surface array. Godot expects exactly Mesh::ARRAY_MAX (13) slots.
        // Unfilled slots must be Nil variants (VariantArray default).
        let mut arrays = VariantArray::new();
        arrays.resize(13);
        // Slot 0 = ARRAY_VERTEX
        arrays.set(0, positions.to_variant());
        // Slot 1 = ARRAY_NORMAL
        arrays.set(1, normals.to_variant());
        // Slot 3 = ARRAY_COLOR
        arrays.set(3, colors.to_variant());

        let mut mesh = ArrayMesh::new_gd();
        mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);
        mesh
    }
}

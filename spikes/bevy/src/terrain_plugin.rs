use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_terrain);
    }
}

fn spawn_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    const GRID: u32 = 64;
    const TILE: f32 = 2.0;

    let mesh = build_terrain_mesh(GRID, GRID, TILE);

    // Vertex-colored material — no texture, uses per-vertex color.
    // Bevy 0.15 picks up ATTRIBUTE_COLOR from the mesh automatically — no flag needed.
    let material = materials.add(StandardMaterial {
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));
}

/// Build a flat grid mesh with per-vertex colors.
/// Tiles alternate grass / dirt based on `(x + y) % 2`.
fn build_terrain_mesh(cols: u32, rows: u32, tile_size: f32) -> Mesh {
    let vertex_count = ((cols + 1) * (rows + 1)) as usize;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(vertex_count);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(vertex_count);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(vertex_count);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(vertex_count);

    // Grass: (0.3, 0.6, 0.2)   Dirt: (0.5, 0.35, 0.2)
    let grass = [0.3_f32, 0.6, 0.2, 1.0];
    let dirt = [0.5_f32, 0.35, 0.2, 1.0];

    for y in 0..=rows {
        for x in 0..=cols {
            positions.push([x as f32 * tile_size, 0.0, y as f32 * tile_size]);
            normals.push([0.0, 1.0, 0.0]);
            // Color corner by the tile to the upper-left (x, y) for x,y < grid size.
            let tile_x = x.min(cols.saturating_sub(1));
            let tile_y = y.min(rows.saturating_sub(1));
            colors.push(if (tile_x + tile_y) % 2 == 0 { grass } else { dirt });
            uvs.push([x as f32 / cols as f32, y as f32 / rows as f32]);
        }
    }

    // Two triangles per tile.
    let index_count = (cols * rows * 6) as usize;
    let mut indices: Vec<u32> = Vec::with_capacity(index_count);
    let stride = cols + 1;
    for y in 0..rows {
        for x in 0..cols {
            let tl = y * stride + x;
            let tr = tl + 1;
            let bl = tl + stride;
            let br = bl + 1;
            // Triangle 1
            indices.push(tl);
            indices.push(bl);
            indices.push(tr);
            // Triangle 2
            indices.push(tr);
            indices.push(bl);
            indices.push(br);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

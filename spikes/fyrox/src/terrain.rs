/// Procedural terrain — uses SurfaceData::make_quad for simplicity.
/// In the full game this will be replaced with a proper heightmap mesh.
/// For the spike, a large flat quad is sufficient to validate the pipeline.
use fyrox::{
    core::{
        algebra::{Matrix4, Vector3},
        pool::Handle,
    },
    scene::{
        base::BaseBuilder,
        mesh::{
            surface::{SurfaceBuilder, SurfaceData, SurfaceSharedData},
            MeshBuilder,
        },
        node::Node,
        Scene,
    },
};

/// Spawn a flat terrain quad 128×128 world-units centred at (64, 0, 64).
pub fn create_terrain_mesh(scene: &mut Scene) -> Handle<Node> {
    // Scale a unit quad to cover the 128×128 map area.
    let transform = Matrix4::new_nonuniform_scaling(&Vector3::new(128.0, 1.0, 128.0));
    let surface_data = SurfaceData::make_quad(&transform);
    let shared = SurfaceSharedData::new(surface_data);
    let surface = SurfaceBuilder::new(shared).build();

    MeshBuilder::new(
        BaseBuilder::new().with_local_transform(
            fyrox::scene::transform::TransformBuilder::new()
                .with_local_position(Vector3::new(64.0, 0.0, 64.0))
                .build(),
        ),
    )
    .with_surfaces(vec![surface])
    .build(&mut scene.graph)
}

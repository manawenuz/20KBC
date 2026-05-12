/// Unit rendering sync — maps `UnitId → Handle<Node>` and keeps scene nodes
/// in sync with `CSimulation` state each frame.
use std::collections::HashMap;

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
use game_core::{CUnit, UnitId};

/// Synchronize Fyrox scene nodes with the current simulation unit list.
pub fn sync_units(
    scene: &mut Scene,
    sim_units: impl Iterator<Item = CUnit>,
    unit_handles: &mut HashMap<UnitId, Handle<Node>>,
) {
    for unit in sim_units {
        if unit.is_dead {
            if let Some(h) = unit_handles.remove(&unit.id) {
                scene.graph.remove_node(h);
            }
            continue;
        }

        let handle = unit_handles.entry(unit.id).or_insert_with(|| {
            spawn_unit_box(scene)
        });

        if let Some(node) = scene.graph.try_get_mut(*handle) {
            node.local_transform_mut()
                .set_position(Vector3::new(unit.pos.x, 0.9, unit.pos.y));
        }
    }
}

fn spawn_unit_box(scene: &mut Scene) -> Handle<Node> {
    // Unit box: 0.8×1.8×0.8. Scale the unit cube (1×1×1) via transform.
    let scale = Matrix4::new_nonuniform_scaling(&Vector3::new(0.8, 1.8, 0.8));
    let surface_data = SurfaceData::make_cube(scale);
    let surface = SurfaceBuilder::new(SurfaceSharedData::new(surface_data)).build();

    MeshBuilder::new(BaseBuilder::new())
        .with_surfaces(vec![surface])
        .build(&mut scene.graph)
}

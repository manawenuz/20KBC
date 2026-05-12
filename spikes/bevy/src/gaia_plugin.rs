use bevy::prelude::*;
use std::collections::HashSet;

use crate::sim_plugin::GameSim;

/// Marks a Bevy entity as a GAIA (wolf) entity.
#[derive(Component)]
pub struct GaiaTag(pub u32);

/// Tracks which GAIA ids have been spawned.
#[derive(Resource, Default)]
pub struct SpawnedGaia(pub HashSet<u32>);

pub struct GaiaPlugin;

impl Plugin for GaiaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnedGaia>()
            .add_systems(Update, sync_gaia);
    }
}

fn sync_gaia(
    sim: Res<GameSim>,
    mut spawned: ResMut<SpawnedGaia>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&GaiaTag, &mut Transform)>,
) {
    // Spawn entities for unseen GAIA entities.
    for entity in &sim.0.gaia {
        if !spawned.0.contains(&entity.id) {
            let mesh = meshes.add(Capsule3d::new(0.5, 1.0));
            let mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.1, 0.1), // red / wolf
                ..default()
            });
            commands.spawn((
                GaiaTag(entity.id),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_xyz(entity.pos.x, 0.5, entity.pos.y),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
            spawned.0.insert(entity.id);
        }
    }

    // Sync transforms.
    for (tag, mut transform) in &mut query {
        if let Some(e) = sim.0.gaia.iter().find(|g| g.id == tag.0) {
            transform.translation = Vec3::new(e.pos.x, 0.5, e.pos.y);
        }
    }
}

use bevy::prelude::*;
use game_core::UnitId;
use std::collections::HashSet;

use crate::sim_plugin::GameSim;

/// Marks a Bevy entity as representing a specific simulation unit.
#[derive(Component)]
pub struct SimUnitId(pub UnitId);

/// Marks the entity as currently selected.
#[derive(Component)]
pub struct Selected;

/// Tracks which simulation unit IDs already have Bevy entities.
#[derive(Resource, Default)]
pub struct SpawnedUnits(pub HashSet<UnitId>);

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpawnedUnits>()
            .add_systems(Update, (sync_units, handle_unit_death));
    }
}

fn sync_units(
    sim: Res<GameSim>,
    mut spawned: ResMut<SpawnedUnits>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&SimUnitId, &mut Transform)>,
) {
    // Spawn entities for units that don't yet have one.
    for unit in sim.0.iter_units() {
        if !spawned.0.contains(&unit.id) {
            let mesh = meshes.add(Capsule3d::new(0.4, 1.4));
            let mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.82, 0.55, 0.3), // sandy brown
                ..default()
            });
            commands.spawn((
                SimUnitId(unit.id),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_xyz(unit.pos.x, 0.7, unit.pos.y),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
            spawned.0.insert(unit.id);
        }
    }

    // Sync transforms every frame.
    for (sim_id, mut transform) in &mut query {
        if let Some(unit) = sim.0.get_unit(sim_id.0) {
            transform.translation = Vec3::new(unit.pos.x, 0.7, unit.pos.y);
            transform.rotation = Quat::from_rotation_y(-unit.facing);
        }
    }
}

fn handle_unit_death(
    sim: Res<GameSim>,
    query: Query<(Entity, &SimUnitId)>,
    mut commands: Commands,
    mut spawned: ResMut<SpawnedUnits>,
) {
    for (entity, sim_id) in &query {
        if sim.0.get_unit(sim_id.0).is_none() {
            // Unit removed from sim — despawn the Bevy entity.
            commands.entity(entity).despawn_recursive();
            spawned.0.remove(&sim_id.0);
        }
    }
}

use bevy::prelude::*;
use game_core::{Order, UnitId};

use crate::sim_plugin::GameSim;
use crate::unit_plugin::{Selected, SimUnitId};

/// Currently selected unit IDs.
#[derive(Resource, Default)]
pub struct Selection(pub Vec<UnitId>);

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Selection>()
            .add_systems(Update, (handle_selection, handle_orders).chain());
    }
}

/// Unproject mouse position to a ray and find the nearest unit within 1.5 units.
fn handle_selection(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    units_q: Query<(Entity, &SimUnitId, &Transform)>,
    mut commands: Commands,
    mut selection: ResMut<Selection>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.get_single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera_q.get_single() else { return };

    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) else { return };

    // Intersect ray with y=0 plane.
    let Some(world_pos) = ray_plane_y0(ray) else { return };

    // Clear old selection.
    for (entity, _, _) in &units_q {
        commands.entity(entity).remove::<Selected>();
    }
    selection.0.clear();

    // Find nearest unit within 1.5 world-units.
    let mut nearest: Option<(Entity, UnitId, f32)> = None;
    for (entity, sim_id, transform) in &units_q {
        let unit_xz = Vec2::new(transform.translation.x, transform.translation.z);
        let dist = unit_xz.distance(world_pos);
        if dist < 1.5 {
            if nearest.as_ref().map_or(true, |n| dist < n.2) {
                nearest = Some((entity, sim_id.0, dist));
            }
        }
    }

    if let Some((entity, uid, _)) = nearest {
        commands.entity(entity).insert(Selected);
        selection.0.push(uid);
    }
}

/// Right-click: intersect ray with y=0 plane, issue Move order to selected units.
fn handle_orders(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut sim: ResMut<GameSim>,
    selection: Res<Selection>,
) {
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }
    if selection.0.is_empty() {
        return;
    }

    let Ok(window) = windows.get_single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera_q.get_single() else { return };

    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) else { return };
    let Some(world_pos) = ray_plane_y0(ray) else { return };

    let target = Vec2::new(world_pos.x, world_pos.y);
    for &uid in &selection.0 {
        sim.0.issue_order(uid, Order::Move { target });
    }
}

/// Intersect a Bevy ray with the y=0 plane. Returns XZ position as Vec2.
fn ray_plane_y0(ray: Ray3d) -> Option<Vec2> {
    let denom = ray.direction.y;
    if denom.abs() < 1e-6 {
        return None;
    }
    let t = -ray.origin.y / denom;
    if t < 0.0 {
        return None;
    }
    let hit = ray.origin + *ray.direction * t;
    Some(Vec2::new(hit.x, hit.z))
}

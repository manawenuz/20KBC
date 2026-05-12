use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;

pub struct RtsCameraPlugin;

impl Plugin for RtsCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, (pan_camera, zoom_camera));
    }
}

fn spawn_camera(mut commands: Commands) {
    // Positioned at (64, 50, 80) looking toward map center (64, 0, 64).
    let eye = Vec3::new(64.0, 50.0, 80.0);
    let target = Vec3::new(64.0, 0.0, 64.0);
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(eye).looking_at(target, Vec3::Y),
    ));
}

fn pan_camera(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    const SPEED: f32 = 20.0;
    let dt = time.delta_secs();
    let mut dir = Vec3::ZERO;

    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        dir.z -= 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        dir.z += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }

    if dir != Vec3::ZERO {
        for mut transform in &mut query {
            transform.translation += dir.normalize() * SPEED * dt;
        }
    }
}

fn zoom_camera(
    mut scroll_events: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    let mut scroll = 0.0_f32;
    for ev in scroll_events.read() {
        scroll += ev.y;
    }
    if scroll.abs() < f32::EPSILON {
        return;
    }
    for mut transform in &mut query {
        transform.translation.y -= scroll * 3.0;
        transform.translation.y = transform.translation.y.clamp(10.0, 80.0);
    }
}

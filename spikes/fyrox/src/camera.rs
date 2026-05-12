/// RTS camera controller.
///
/// Position: (64, 50, 80) looking at (64, 0, 64) on init.
/// WASD pans the camera on the XZ plane.
/// Mouse scroll zooms (adjusts Y height), clamped to [10, 80].
///
/// Fyrox API notes (0.34):
///   - Cameras are created via `CameraBuilder` with a `BaseBuilder`.
///   - Transform is set via `node.local_transform_mut().set_position(...)`.
///   - Keyboard state is read from `fyrox::event::Event` / input buffer in PluginContext.
///   - There is no built-in `KeyboardState` struct; input is event-driven.
///     We track pressed keys manually in `GamePlugin` and pass booleans here.
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        pool::Handle,
    },
    scene::{
        base::BaseBuilder,
        camera::{CameraBuilder, OrthographicProjection, Projection},
        node::Node,
        Scene,
    },
};

pub struct RtsCamera {
    pub node: Handle<Node>,
    pan_speed: f32,
    zoom_speed: f32,
}

/// Key booleans passed from the event handler each frame.
pub struct KeyInput {
    pub w: bool,
    pub a: bool,
    pub s: bool,
    pub d: bool,
}

impl RtsCamera {
    /// Create camera node and add it to the scene.
    pub fn new(scene: &mut Scene) -> Self {
        let node = CameraBuilder::new(
            BaseBuilder::new()
                .with_local_transform(
                    fyrox::scene::transform::TransformBuilder::new()
                        .with_local_position(Vector3::new(64.0, 50.0, 80.0))
                        .build(),
                ),
        )
        .build(&mut scene.graph);

        // Point the camera toward the center of the map.
        look_at(&mut scene.graph[node], Vector3::new(64.0, 50.0, 80.0), Vector3::new(64.0, 0.0, 64.0));

        Self {
            node,
            pan_speed: 20.0,
            zoom_speed: 15.0,
        }
    }

    /// Call every frame from `GamePlugin::update`.
    pub fn update(&mut self, scene: &mut Scene, dt: f32, keys: &KeyInput, scroll: f32) {
        let node = &mut scene.graph[self.node];
        let pos = **node.local_transform().position();

        let mut dx = 0.0_f32;
        let mut dz = 0.0_f32;
        if keys.w { dz -= 1.0; }
        if keys.s { dz += 1.0; }
        if keys.a { dx -= 1.0; }
        if keys.d { dx += 1.0; }

        let new_x = pos.x + dx * self.pan_speed * dt;
        let new_z = pos.z + dz * self.pan_speed * dt;
        let new_y = (pos.y - scroll * self.zoom_speed * dt).clamp(10.0, 80.0);

        node.local_transform_mut()
            .set_position(Vector3::new(new_x, new_y, new_z));
    }
}

/// Rotate a node to look from `eye` toward `target` with Y-up.
fn look_at(node: &mut Node, eye: Vector3<f32>, target: Vector3<f32>) {
    let dir = (target - eye).normalize();
    // Build quaternion from direction vector (Fyrox convention: -Z is forward).
    let rot = UnitQuaternion::face_towards(&(-dir), &Vector3::y());
    node.local_transform_mut().set_rotation(rot);
}

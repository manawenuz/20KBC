/// Day/night cycle — rotates a directional light through a 10-minute period.
///
/// t=0.25 → noon  (illuminance 15 000 lux)
/// t=0.75 → midnight (illuminance 500 lux)
///
/// Fyrox API notes (0.34):
///   - `DirectionalLight` is accessed via the scene graph as a `Node`.
///   - The directional light node exposes `base_light_mut()` which has `set_color()`.
///   - There is no direct "illuminance" float in Fyrox 0.34; brightness is encoded in
///     the light's `color` (RGB intensity). We scale the color channels instead.
///   - The light direction is controlled by rotating the node's transform.
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        color::Color,
        pool::Handle,
    },
    scene::{
        base::BaseBuilder,
        light::{
            directional::DirectionalLightBuilder,
            BaseLightBuilder,
        },
        node::Node,
        Scene,
    },
};

pub struct DayNightCycle {
    /// Elapsed time within the current period (seconds).
    pub time: f32,
    /// Full day/night period in seconds (default: 600 s = 10 min).
    pub period: f32,
    light_handle: Handle<Node>,
}

impl DayNightCycle {
    /// Create a directional light and register the day/night controller.
    pub fn new(scene: &mut Scene) -> Self {
        let light_handle = DirectionalLightBuilder::new(BaseLightBuilder::new(
            BaseBuilder::new().with_local_transform(
                fyrox::scene::transform::TransformBuilder::new()
                    .with_local_rotation(
                        // Start at a 45° angle — mid-morning.
                        UnitQuaternion::from_euler_angles(-std::f32::consts::FRAC_PI_4, 0.0, 0.0),
                    )
                    .build(),
            ),
        ))
        .build(&mut scene.graph);

        Self {
            time: 0.0,
            period: 600.0,
            light_handle,
        }
    }

    /// Advance the cycle by `dt` seconds and update the light.
    pub fn update(&mut self, scene: &mut Scene, dt: f32) {
        self.time = (self.time + dt) % self.period;
        let t = self.time / self.period; // 0..1

        // Illuminance in lux: noon=15000, midnight=500.
        // Map to a [0,1] brightness factor for the color channels.
        let illuminance = lerp_illuminance(t);
        // Clamp to [0,1] for Color channels (lux → relative float).
        let brightness = (illuminance / 15_000.0).clamp(0.0, 1.0);

        // Daylight is warm (slight yellow tint), moonlight is cool blue.
        let (r, g, b) = if t < 0.5 {
            // Day
            let w = 1.0 - (t - 0.25).abs() / 0.25; // peaks at noon
            let warmth = w * 0.05;
            (brightness + warmth, brightness, brightness - warmth * 0.5)
        } else {
            // Night
            let coolness = brightness * 0.05;
            (brightness - coolness, brightness - coolness, brightness + coolness)
        };

        let color = Color::from_rgba(
            (r.clamp(0.0, 1.0) * 255.0) as u8,
            (g.clamp(0.0, 1.0) * 255.0) as u8,
            (b.clamp(0.0, 1.0) * 255.0) as u8,
            255,
        );

        // Update light color.
        if let Some(light_node) = scene.graph.try_get_mut(self.light_handle) {
            if let Some(light) = light_node.cast_mut::<fyrox::scene::light::directional::DirectionalLight>() {
                light.base_light_mut().set_color(color);
            }
        }

        // Rotate the light direction to simulate sun/moon arc.
        // At t=0.25 (noon) the sun is directly overhead; at t=0.75 it's below horizon.
        let angle = (t - 0.25) * std::f32::consts::TAU; // full rotation per period
        if let Some(node) = scene.graph.try_get_mut(self.light_handle) {
            node.local_transform_mut().set_rotation(UnitQuaternion::from_euler_angles(
                -std::f32::consts::FRAC_PI_4 - angle * 0.5,
                0.0,
                0.0,
            ));
        }
    }
}

/// Map normalized day time `t ∈ [0,1]` to an illuminance value in lux.
/// Smooth sinusoidal transition: noon=15000, midnight=500.
fn lerp_illuminance(t: f32) -> f32 {
    // sin peaks at t=0.25 (noon) and troughs at t=0.75 (midnight).
    let sin_val = (t * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2).sin();
    // Map [-1, 1] → [500, 15000]
    let min_lux = 500.0_f32;
    let max_lux = 15_000.0_f32;
    min_lux + (max_lux - min_lux) * (sin_val + 1.0) * 0.5
}
